use std::sync::Arc;
use std::time::Instant;

use glam::Mat4;
use winit::window::Window;

use crate::rendering::camera::Camera;
use crate::rendering::render_config::RenderConfig;
use crate::rendering::simulation_resources::SimulationResources;
use crate::simulation::manager::SimulationManager;
use crate::simulation::types::{COMPUTE_WORKGROUP_SIZE, NUM_BODIES, SimulationState};

// Main renderer struct
pub struct Renderer {
    pub window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface<'static>,

    // Rendering configuration
    pub render_config: RenderConfig,

    // Simulation resources
    pub simulation_resources: SimulationResources,

    // Simulation state
    pub last_update: Instant,
    pub simulation_state: SimulationState,
    pub simulation_manager: SimulationManager,

    // Camera
    pub camera: Camera,

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

        // Create render configuration (pipelines and bind group layouts)
        let render_config = RenderConfig::new(&device, surface_format);

        // Create simulation resources (buffers and bind groups)
        let simulation_resources = SimulationResources::new(
            &device,
            &simulation_manager,
            &render_config,
            &simulation_state,
        );

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
            render_config,
            simulation_resources,
            last_update: Instant::now(),
            simulation_state,
            simulation_manager,
            camera: Camera::new(base_camera_height),
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
            format: self.render_config.surface_format,
            // Request compatibility with the sRGB-format texture view we're going to create later.
            view_formats: vec![self.render_config.surface_format.add_srgb_suffix()],
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
        self.simulation_resources
            .update_simulation_state(&self.queue, &self.simulation_state);

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
            self.simulation_resources
                .update_bodies(&self.queue, &bodies);

            self.simulation_changed = false;
        }

        // Update simulation state buffer
        self.simulation_resources
            .update_simulation_state(&self.queue, &self.simulation_state);
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
                format: Some(self.render_config.surface_format.add_srgb_suffix()),
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

            compute_pass.set_pipeline(&self.render_config.compute_pipeline);
            compute_pass.set_bind_group(
                0,
                &self.simulation_resources.bind_groups[self.simulation_resources.current_buffer],
                &[],
            );

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

            render_pass.set_pipeline(&self.render_config.render_pipeline);
            render_pass.set_bind_group(
                0,
                &self.simulation_resources.bind_groups[self.simulation_resources.current_buffer],
                &[],
            );

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
        self.simulation_resources.swap_buffers();
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

        // Update simulation state with new view matrix
        self.simulation_state.view_matrix = self
            .camera
            .calculate_view_matrix(base_position)
            .to_cols_array();
    }

    pub fn pan_camera(&mut self, delta_x: f32, delta_y: f32) {
        self.camera.pan(delta_x, delta_y);
        self.update_camera_view();
    }

    pub fn zoom_camera(&mut self, delta: f32) {
        self.camera.zoom(delta);
        self.update_camera_view();
    }

    pub fn rotate_camera(&mut self, delta: f32) {
        self.camera.rotate(delta);
        self.update_camera_view();
    }

    pub fn reset_camera(&mut self) {
        // Get the base camera position from the current simulation
        let camera_position = self
            .simulation_manager
            .get_current_simulation()
            .camera_position();
        let base_camera_height = camera_position[1];

        // Create a new camera with default settings
        self.camera = Camera::new(base_camera_height);

        // Update the view matrix based on the reset camera
        self.update_camera_view();
    }

    // Input handling methods
    pub fn handle_mouse_press(&mut self, position: [f32; 2], ctrl: bool, shift: bool) {
        self.camera.handle_mouse_press(position, ctrl, shift);
    }

    pub fn handle_mouse_release(&mut self) {
        self.camera.handle_mouse_release();
    }

    pub fn handle_mouse_move(&mut self, position: [f32; 2]) {
        if self.camera.handle_mouse_move(position) {
            // Camera was moved by the mouse, update the view
            self.update_camera_view();
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta: f32) {
        self.camera.handle_mouse_wheel(delta);
        self.update_camera_view();
    }

    pub fn handle_key_state(&mut self, ctrl: bool, shift: bool) {
        self.camera.handle_key_state(ctrl, shift);
    }
}
