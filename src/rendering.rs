use std::sync::Arc;

use winit::window::Window;

pub struct Renderer {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_caps: wgpu::SurfaceCapabilities,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
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

        let state = Renderer {
            window,
            device,
            queue,
            size,
            surface,
            surface_caps,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {}

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn render(&mut self) {
        // Render the scene
    }

    fn configure_surface(&self) {
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = self
            .surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(self.surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self.size.width,
            height: self.size.height,
            present_mode: self.surface_caps.present_modes[0],
            alpha_mode: self.surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        self.surface.configure(&self.device, &surface_config);
    }
}
