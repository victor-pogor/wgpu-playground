use crate::shaders::render_triangle;
use wgpu::Device;

pub(crate) struct RenderPipelines {
    pub render_triangle_pipeline: wgpu::RenderPipeline,
}

impl RenderPipelines {
    pub(crate) fn new(device: &Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        Self {
            render_triangle_pipeline: render_triangle(&render_pipeline_layout, device, surface_config),
        }
    }
}
