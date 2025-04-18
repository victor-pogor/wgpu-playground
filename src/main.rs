use std::sync::Arc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3}; // Remove unused Vec4 import
use rand::Rng;
use wgpu::util::DeviceExt; // Add this trait for create_buffer_init
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

// Constants for simulation
const NUM_BODIES: u32 = 1024;
const COMPUTE_WORKGROUP_SIZE: u32 = 64;

// Represents a single body in our n-body simulation
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Body {
    position: [f32; 4], // xyz = position, w = mass
    velocity: [f32; 4], // xyz = velocity, w = unused
    color: [f32; 4],    // rgba color
}

// Runtime state for the simulation
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct SimulationState {
    delta_time: f32,
    _padding: [f32; 3], // Padding to align with mat4
    view_matrix: [f32; 16],
    projection_matrix: [f32; 16],
}

struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,

    // Simulation resources
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    body_buffers: [wgpu::Buffer; 2], // Ping-pong buffers
    simulation_state_buffer: wgpu::Buffer,
    bind_groups: [wgpu::BindGroup; 2],

    // Simulation state
    current_buffer: usize,
    last_update: Instant,
    bodies: Vec<Body>,
    simulation_state: SimulationState,
}

impl State {
    async fn new(window: Arc<Window>) -> State {
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
                memory_hints: wgpu::MemoryHints::default(), // Add missing field
                trace: wgpu::Trace::Off,                    // Fix: Use wgpu::Trace::Disabled
            })
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        // Create buffers for simulation
        let bodies = create_random_bodies(NUM_BODIES);

        // Create two buffers for ping-pong rendering
        let body_buffer_size = std::mem::size_of::<Body>() as u64 * NUM_BODIES as u64;
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

        // Create simulation state buffer with view and projection matrices
        let aspect = size.width as f32 / size.height as f32;
        let simulation_state = SimulationState {
            delta_time: 0.001,
            _padding: [0.0; 3],
            view_matrix: Mat4::look_at_rh(
                Vec3::new(0.0, 20.0, 200.0), // Camera position
                Vec3::new(0.0, 0.0, 0.0),    // Look target
                Vec3::new(0.0, 1.0, 0.0),    // Up direction
            )
            .to_cols_array(),
            projection_matrix: Mat4::perspective_rh(
                45.0_f32.to_radians(),
                aspect,
                0.1,    // Near plane
                1000.0, // Far plane
            )
            .to_cols_array(),
        };

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
            entry_point: Some("compute_step"), // Fix entry point to be Option<&str>
            compilation_options: Default::default(),
            cache: None,
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("N-Body Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vertex_main"), // Fix entry point to be Option<&str>
                buffers: &[],
                compilation_options: Default::default(), // Add missing field
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"), // Fix entry point to be Option<&str>
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(), // Add missing field
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
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
            cache: None, // Add missing field
        });

        let state = State {
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
            bodies,
            simulation_state,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
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

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // Update projection matrix with new aspect ratio
        let aspect = new_size.width as f32 / new_size.height as f32;
        self.simulation_state.projection_matrix =
            Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 1000.0).to_cols_array();

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

        // Update delta time in simulation state
        self.simulation_state.delta_time = dt;
        self.queue.write_buffer(
            &self.simulation_state_buffer,
            0,
            bytemuck::cast_slice(&[self.simulation_state]),
        );
    }

    fn render(&mut self) {
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
                            b: 0.05,
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

            // Draw NUM_BODIES instances
            render_pass.draw(0..1, 0..NUM_BODIES);
        }

        // Submit command buffer
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();

        // Swap buffers for ping-pong computation
        self.current_buffer = 1 - self.current_buffer;
    }
}

fn create_random_bodies(count: u32) -> Vec<Body> {
    let mut rng = rand::thread_rng();
    let mut bodies = Vec::with_capacity(count as usize);

    // Create a central "sun" with high mass
    bodies.push(Body {
        position: [0.0, 0.0, 0.0, 5000.0], // Central mass
        velocity: [0.0, 0.0, 0.0, 0.0],
        color: [1.0, 0.9, 0.1, 1.0], // Yellow
    });

    // Create bodies in a disc formation
    for _ in 1..count {
        // Random distance from center (distributed more around the edges)
        let distance = 20.0 + 50.0 * rng.gen_range(0.0f32..1.0f32).powf(0.5_f32);

        // Random angle
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);

        // Calculate position
        let x = distance * angle.cos();
        let z = distance * angle.sin();
        let y = (rng.gen_range(0.0..1.0) - 0.5) * 5.0; // Small vertical variation

        // Calculate orbital velocity (perpendicular to radial direction)
        let speed = (5.0 / distance.sqrt()).min(1.0); // Orbital velocity
        let vx = -speed * angle.sin();
        let vz = speed * angle.cos();

        // Random small mass
        let mass = 0.1 + rng.gen_range(0.0..1.0) * 2.0;

        // Generate a color based on distance
        let hue = distance / 100.0; // Normalize to 0-1 range
        let (r, g, b) = hsl_to_rgb(hue, 0.8, 0.6);

        bodies.push(Body {
            position: [x, y, z, mass],
            velocity: [vx, 0.0, vz, 0.0],
            color: [r, g, b, 1.0],
        });
    }

    bodies
}

// Convert HSL to RGB
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    // Initialize logger
    env_logger::init();

    // Create event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create app
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
