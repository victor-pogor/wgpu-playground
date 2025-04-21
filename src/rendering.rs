mod render_pass;
mod surface;

use render_pass::create_background_render_pass;
use std::{sync::Arc, time::Instant};
use winit::window::Window;

use surface::configure_surface;

use crate::simulation::{
    config::RenderConfig,
    manager::SimulationManager,
    resources::SimulationResources,
    types::{COMPUTE_WORKGROUP_SIZE, NUM_BODIES, SimulationState},
};

pub(crate) struct Renderer {
    window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub render_config: RenderConfig,
    pub simulation_resources: SimulationResources,
    pub last_update: Instant,
    pub simulation_state: SimulationState,
    pub simulation_manager: SimulationManager,
    pub show_info: bool,
    pub simulation_changed: bool,
}

impl Renderer {
    pub(crate) async fn new(window: Arc<Window>) -> Self {
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        // A surface is a platform-specific representation of a window or display
        // where rendering can occur. It is used to present rendered images to the
        // screen. In the context of WebGPU, a surface is created for a specific
        // window or display, allowing the GPU to render directly to that surface.
        let surface = instance.create_surface(window.clone()).unwrap();

        // An adapter is a representation of a physical GPU device. It provides
        // access to the GPU's capabilities and features, allowing you to create
        // logical devices and perform rendering operations.
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // The device is a logical representation of the GPU. It provides access
        // to the GPU's resources and allows you to create command buffers,
        // pipelines, and other objects needed for rendering.
        // The queue is used to submit commands to the GPU for execution.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);

        // Configure surface for the first time
        let surface_config = configure_surface(&device, &size, &surface, &surface_caps);

        // Create simulation manager
        let simulation_manager = SimulationManager::new();

        // Create initial simulation state
        let simulation_state = SimulationState {
            delta_time: 0.001,
            _padding: [0.0; 3],
        };

        // Create render configuration (pipelines and bind group layouts)
        let render_config = RenderConfig::new(&device, surface_config.format);

        // Create simulation resources (buffers and bind groups)
        let simulation_resources = SimulationResources::new(&device, &simulation_manager, &render_config, &simulation_state);

        let state = Renderer {
            window,
            device,
            queue,
            size,
            surface,
            surface_config,
            render_config,
            simulation_resources,
            last_update: Instant::now(),
            simulation_state,
            simulation_manager,
            show_info: true,
            simulation_changed: false,
        };

        state
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub(crate) fn get_window(&self) -> &Window {
        &self.window
    }

    pub(crate) fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub(crate) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Render the scene
        // Create texture view
        let surface_texture = self.surface.get_current_texture().expect("failed to acquire next swapchain texture");

        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("WebGPU Command Encoder"),
        });

        // Compute pass - update body positions
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("N-Body Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.render_config.compute_pipeline);
            compute_pass.set_bind_group(0, &self.simulation_resources.bind_groups[self.simulation_resources.current_buffer], &[]);

            // Dispatch compute work groups
            let workgroup_count = (NUM_BODIES + COMPUTE_WORKGROUP_SIZE - 1) / COMPUTE_WORKGROUP_SIZE;
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        // Render pass - draw the bodies
        {
            let mut render_pass = create_background_render_pass(
                &mut encoder,
                &texture_view,
                wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.05, // Slightly increased blue for better cosmic background
                    a: 1.0,
                },
            );

            render_pass.set_pipeline(&self.render_config.render_pipeline);
            render_pass.set_bind_group(0, &self.simulation_resources.bind_groups[self.simulation_resources.current_buffer], &[]);

            // Draw 6 vertices (2 triangles) per body instance
            render_pass.draw(0..6, 0..NUM_BODIES);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

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

        Ok(())
    }

    pub(crate) fn update(&mut self) {
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
            // Get new bodies from the simulation manager
            let bodies = self.simulation_manager.get_bodies();

            // Update the current buffer with the new bodies
            self.simulation_resources.update_bodies(&self.queue, &bodies);

            self.simulation_changed = false;
        }

        // Update simulation state buffer
        self.simulation_resources.update_simulation_state(&self.queue, &self.simulation_state);
    }

    pub(crate) fn switch_simulation(&mut self, index: usize) {
        if self.simulation_manager.switch_to_simulation(index) {
            self.simulation_changed = true;
            self.show_info = true;
        }
    }

    pub(crate) fn next_simulation(&mut self) {
        if self.simulation_manager.next_simulation() {
            self.simulation_changed = true;
            self.show_info = true;
        }
    }

    pub(crate) fn previous_simulation(&mut self) {
        if self.simulation_manager.previous_simulation() {
            self.simulation_changed = true;
            self.show_info = true;
        }
    }

    pub(crate) fn toggle_info(&mut self) {
        self.show_info = true;
    }

    pub(crate) fn debug_compute_shader(&mut self) {
        // Read the debug data from the debug buffer
        let debug_data = self.simulation_resources.read_debug_data(&self.device, &self.queue);

        // Print the debug information
        println!("===== Compute Shader Debug Data =====");
        println!("Iterations: {}", debug_data.iterations);
        println!(
            "Tracked particle position: ({:.2}, {:.2}, {:.2}), Acceleration: {:.2}",
            debug_data.particle_info[0], debug_data.particle_info[1], debug_data.particle_info[2], debug_data.particle_info[3]
        );
        println!("===================================");
    }
}
