use std::sync::Arc;

use winit::window::Window;

pub struct Renderer {
    pub window: Arc<Window>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        // Initialize the renderer with the window
        Self { window }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {}

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn render(&mut self) {
        // Render the scene
    }
}
