// Export the renderer components
pub mod camera;
pub mod render_config;
pub mod renderer;
pub mod simulation_resources;

// Re-export the main components for easier imports
pub use camera::Camera;
pub use render_config::RenderConfig;
pub use renderer::Renderer;
pub use simulation_resources::SimulationResources;
