mod render_pass;
mod surface;

use std::sync::Arc;
use winit::window::Window;

use render_pass::create_background_render_pass;
use surface::configure_surface;

use crate::shaders::PentagonRenderPipeline;

pub(crate) struct Renderer {
    window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
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

        let state = Renderer {
            window,
            device,
            queue,
            size,
            surface,
            surface_config,
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

        {
            let mut render_pass = create_background_render_pass(
                &mut encoder,
                &texture_view,
                wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                },
            );

            PentagonRenderPipeline::new(self).render_pass(&mut render_pass);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        self.window.pre_present_notify();
        surface_texture.present();

        Ok(())
    }

    pub(crate) fn update(&mut self) {
        // The update method is called once per frame before rendering
        // Currently no state updates are needed, but this will be used
        // for animations, physics simulations, etc.
    }
}
