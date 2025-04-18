use crate::simulation::types::Body;

/// A trait that defines the interface for all simulation types
pub trait Simulation {
    /// Name of the simulation
    fn name(&self) -> &str;

    /// Description of the simulation
    fn description(&self) -> &str;

    /// Initialize bodies for this simulation
    fn initialize_bodies(&self, count: u32) -> Vec<Body>;

    /// Get the recommended camera position for this simulation
    fn camera_position(&self) -> [f32; 3];

    /// Get the recommended camera target for this simulation
    fn camera_target(&self) -> [f32; 3];
}
