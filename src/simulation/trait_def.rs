use crate::simulation::types::Body;

/// A trait that defines the interface for all simulation types
pub(crate) trait Simulation {
    /// Name of the simulation
    fn name(&self) -> &str;

    /// Description of the simulation
    fn description(&self) -> &str;

    /// Initialize bodies for this simulation
    fn initialize_bodies(&self, count: u32) -> Vec<Body>;
}
