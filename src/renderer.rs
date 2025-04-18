use std::sync::Arc;
use std::time::Instant;

use bytemuck;
use glam::Mat4;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::simulation::manager::SimulationManager;
use crate::simulation::types::{COMPUTE_WORKGROUP_SIZE, NUM_BODIES, SimulationState};

pub struct Renderer {
    pub window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface<'static>,
    pub surface_format: wgpu::TextureFormat,

    // Simulation resources
    pub compute_pipeline: wgpu::ComputePipeline,
    pub render_pipeline: wgpu::RenderPipeline,
    pub body_buffers: [wgpu::Buffer; 2], // Ping-pong buffers
    pub simulation_state_buffer: wgpu::Buffer,
    pub bind_groups: [wgpu::BindGroup; 2],

    // Simulation state
    pub current_buffer: usize,
    pub last_update: Instant,
    pub simulation_state: SimulationState,
    pub simulation_manager: SimulationManager,

    // Camera state
    pub camera_offset: [f32; 2], // x, z offsets for panning
    pub camera_zoom: f32,        // zoom factor
    pub camera_rotation: f32,    // rotation in radians
    pub base_camera_height: f32, // base height for the camera

    // Mouse interaction state
    pub mouse_pressed: bool,
    pub last_mouse_position: [f32; 2],
    pub ctrl_pressed: bool,
    pub shift_pressed: bool,

    // UI state
    pub show_info: bool,
    pub simulation_changed: bool,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::VERTEX_WRITABLE_STORAGE,
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        // Create simulation manager
        let simulation_manager = SimulationManager::new();

        // Get bodies from the initial simulation
        let bodies = simulation_manager.get_bodies();

        // Create two buffers for ping-pong rendering
        let body_buffers = [
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bodies Buffer 0"),
                contents: bytemuck::cast_slice(&bodies),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bodies Buffer 1"),
                contents: bytemuck::cast_slice(&bodies),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
        ];

        // Create initial simulation state
        let mut simulation_state = SimulationState {
            delta_time: 0.001,
            _padding: [0.0; 3],
            view_matrix: Mat4::IDENTITY.to_cols_array(),
            // Use orthographic projection for 2D top-down view instead of perspective
            projection_matrix: Mat4::orthographic_rh(
                -500.0, // Left
                500.0,  // Right
                -500.0, // Bottom
                500.0,  // Top
                0.1,    // Near
                1000.0, // Far
            )
            .to_cols_array(),
        };

        // Update view matrix based on the current simulation
        simulation_manager.update_simulation_state(&mut simulation_state);

        let simulation_state_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Simulation State Buffer"),
                contents: bytemuck::cast_slice(&[simulation_state]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // Load shader
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("N-Body Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Create bind groups
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("N-Body Bind Group Layout"),
            entries: &[
                // bodies_in
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // bodies_out
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // simulation_state
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create two bind groups for ping-pong computation
        let bind_groups = [
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("N-Body Bind Group 0"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: body_buffers[0].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: body_buffers[1].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: simulation_state_buffer.as_entire_binding(),
                    },
                ],
            }),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("N-Body Bind Group 1"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: body_buffers[1].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: body_buffers[0].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: simulation_state_buffer.as_entire_binding(),
                    },
                ],
            }),
        ];

        // Create pipeline layouts
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("N-Body Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create compute pipeline
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("N-Body Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("compute_step"),
            compilation_options: Default::default(),
            cache: None,
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("N-Body Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Get base camera position from simulation for initial setup
        let camera_position = simulation_manager
            .get_current_simulation()
            .camera_position();
        let base_camera_height = camera_position[1];

        let renderer = Self {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            compute_pipeline,
            render_pipeline,
            body_buffers,
            simulation_state_buffer,
            bind_groups,
            current_buffer: 0,
            last_update: Instant::now(),
            simulation_state,
            simulation_manager,
            camera_offset: [0.0, 0.0],
            camera_zoom: 1.0,
            camera_rotation: 0.0,
            base_camera_height,
            mouse_pressed: false,
            last_mouse_position: [0.0, 0.0],
            ctrl_pressed: false,
            shift_pressed: false,
            show_info: true,
            simulation_changed: false,
        };

        // Configure surface for the first time
        renderer.configure_surface();

        renderer
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we're going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // Update projection matrix with new aspect ratio
        let aspect = new_size.width as f32 / new_size.height as f32;

        // Maintain orthographic projection when resizing, adjusting for aspect ratio
        let height = 500.0;
        let width = height * aspect;
        self.simulation_state.projection_matrix = Mat4::orthographic_rh(
            -width,  // Left
            width,   // Right
            -height, // Bottom
            height,  // Top
            0.1,     // Near
            1000.0,  // Far
        )
        .to_cols_array();

        // Update simulation state buffer
        self.queue.write_buffer(
            &self.simulation_state_buffer,
            0,
            bytemuck::cast_slice(&[self.simulation_state]),
        );

        // Reconfigure the surface
        self.configure_surface();
    }

    fn update(&mut self) {
        // Calculate time since last update
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // Limit delta time to prevent large jumps in the simulation
        // This is especially important on the first frame when dt can be very large
        let clamped_dt = dt.min(0.016); // Cap at ~60 FPS time step (16ms)

        // Update delta time in simulation state
        self.simulation_state.delta_time = clamped_dt;

        // If simulation has changed, update the view matrix and buffers
        if self.simulation_changed {
            // Update camera position based on the current simulation
            self.simulation_manager
                .update_simulation_state(&mut self.simulation_state);

            // Get new bodies from the simulation manager
            let bodies = self.simulation_manager.get_bodies();

            // Update the current buffer with the new bodies
            self.queue.write_buffer(
                &self.body_buffers[self.current_buffer],
                0,
                bytemuck::cast_slice(&bodies),
            );

            self.simulation_changed = false;
        }

        // Update simulation state buffer
        self.queue.write_buffer(
            &self.simulation_state_buffer,
            0,
            bytemuck::cast_slice(&[self.simulation_state]),
        );
    }

    pub fn render(&mut self) {
        // Update simulation state
        self.update();

        // Create texture view
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("N-Body Command Encoder"),
            });

        // Compute pass - update body positions
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("N-Body Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.bind_groups[self.current_buffer], &[]);

            // Dispatch compute work groups
            let workgroup_count =
                (NUM_BODIES + COMPUTE_WORKGROUP_SIZE - 1) / COMPUTE_WORKGROUP_SIZE;
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        // Render pass - draw the bodies
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("N-Body Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.03,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_groups[self.current_buffer], &[]);

            // Draw 6 vertices (2 triangles) per body instance
            render_pass.draw(0..6, 0..NUM_BODIES);
        }

        // Submit command buffer
        self.queue.submit([encoder.finish()]);

        // If we want to show info, print the current simulation
        if self.show_info {
            let current_sim = self.simulation_manager.get_current_simulation();
            println!(
                "Simulation: {} - {} (Press 1-{} to switch, I to toggle info)",
                current_sim.name(),
                current_sim.description(),
                self.simulation_manager.get_simulation_count()
            );
            self.show_info = false;
        }

        self.window.pre_present_notify();
        surface_texture.present();

        // Swap buffers for ping-pong computation
        self.current_buffer = 1 - self.current_buffer;
    }

    pub fn switch_simulation(&mut self, index: usize) {
        if self.simulation_manager.switch_to_simulation(index) {
            self.simulation_changed = true;
            self.show_info = true;
        }
    }

    pub fn next_simulation(&mut self) {
        if self.simulation_manager.next_simulation() {
            self.simulation_changed = true;
            self.show_info = true;
        }
    }

    pub fn previous_simulation(&mut self) {
        if self.simulation_manager.previous_simulation() {
            self.simulation_changed = true;
            self.show_info = true;
        }
    }

    pub fn toggle_info(&mut self) {
        self.show_info = true;
    }

    // Camera control methods
    pub fn update_camera_view(&mut self) {
        // Get base camera position from simulation
        let base_position = self
            .simulation_manager
            .get_current_simulation()
            .camera_position();

        // Apply camera transformations (pan, zoom, rotate)
        let mut camera_mat = Mat4::IDENTITY;

        // First apply rotation around Y axis
        camera_mat = camera_mat * Mat4::from_rotation_y(self.camera_rotation);

        // Then apply translation (pan)
        camera_mat = camera_mat
            * Mat4::from_translation(glam::Vec3::new(
                self.camera_offset[0],
                0.0,
                self.camera_offset[1],
            ));

        // Calculate zoom-adjusted camera position
        let camera_height = base_position[1] / self.camera_zoom;
        let camera_pos = glam::Vec3::new(base_position[0], camera_height, base_position[2]);

        // Get target position (always looking at the center for now)
        let target_pos = glam::Vec3::new(0.0, 0.0, 0.0);

        let view_matrix = Mat4::look_at_rh(camera_pos, target_pos, glam::Vec3::Y);

        // Apply camera transformations to view matrix
        let final_view_matrix = view_matrix * camera_mat;

        // Update simulation state with new view matrix
        self.simulation_state.view_matrix = final_view_matrix.to_cols_array();
    }

    pub fn pan_camera(&mut self, delta_x: f32, delta_y: f32) {
        // Scale pan amount based on zoom level (faster pan when zoomed out)
        let pan_speed = 1.0 / self.camera_zoom;

        // Apply the rotation to the pan direction
        let sin_rot = self.camera_rotation.sin();
        let cos_rot = self.camera_rotation.cos();

        // Apply rotation to get world-space pan
        self.camera_offset[0] += (delta_x * cos_rot - delta_y * sin_rot) * pan_speed;
        self.camera_offset[1] += (delta_x * sin_rot + delta_y * cos_rot) * pan_speed;

        // Update the view matrix with new camera position
        self.update_camera_view();
    }

    pub fn zoom_camera(&mut self, delta: f32) {
        // Apply zoom (delta is positive for zoom in, negative for zoom out)
        let zoom_speed = 0.1;
        let new_zoom = self.camera_zoom * (1.0 + delta * zoom_speed);

        // Clamp zoom to reasonable limits
        self.camera_zoom = new_zoom.clamp(0.1, 10.0);

        // Update the view matrix with new zoom
        self.update_camera_view();
    }

    pub fn rotate_camera(&mut self, delta: f32) {
        // Apply rotation (in radians)
        self.camera_rotation += delta * 0.01;

        // Keep rotation in 0-2π range for simplicity
        while self.camera_rotation > std::f32::consts::TAU {
            self.camera_rotation -= std::f32::consts::TAU;
        }
        while self.camera_rotation < 0.0 {
            self.camera_rotation += std::f32::consts::TAU;
        }

        // Update the view matrix with new rotation
        self.update_camera_view();
    }

    // Input handling methods
    pub fn handle_mouse_press(&mut self, position: [f32; 2], ctrl: bool, shift: bool) {
        self.mouse_pressed = true;
        self.last_mouse_position = position;
        self.ctrl_pressed = ctrl;
        self.shift_pressed = shift;
    }

    pub fn handle_mouse_release(&mut self) {
        self.mouse_pressed = false;
    }

    pub fn handle_mouse_move(&mut self, position: [f32; 2]) {
        if self.mouse_pressed {
            let delta_x = position[0] - self.last_mouse_position[0];
            let delta_y = position[1] - self.last_mouse_position[1];

            if self.ctrl_pressed {
                // Pan with Ctrl+drag
                self.pan_camera(delta_x, delta_y);
            } else if self.shift_pressed {
                // Rotate with Shift+drag
                self.rotate_camera(delta_x);
            }

            self.last_mouse_position = position;
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta: f32) {
        // Zoom with mouse wheel
        self.zoom_camera(delta);
    }

    pub fn handle_key_state(&mut self, ctrl: bool, shift: bool) {
        self.ctrl_pressed = ctrl;
        self.shift_pressed = shift;
    }
}
