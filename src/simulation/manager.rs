use std::sync::{Arc, Mutex};

use crate::simulation::solar_system::SolarSystemSimulation;
use crate::simulation::trait_def::Simulation;
use crate::simulation::types::{Body, SimulationState};

pub struct SimulationManager {
    simulations: Vec<Arc<dyn Simulation + Send + Sync>>,
    current_simulation_index: usize,
    bodies: Mutex<Vec<Body>>,
}

impl SimulationManager {
    pub fn new() -> Self {
        // Create available simulations
        let simulations: Vec<Arc<dyn Simulation + Send + Sync>> =
            vec![Arc::new(SolarSystemSimulation)];

        // Initialize with the first simulation
        let bodies = simulations[0].initialize_bodies(crate::simulation::types::NUM_BODIES);

        Self {
            simulations,
            current_simulation_index: 0,
            bodies: Mutex::new(bodies),
        }
    }

    pub fn get_current_simulation(&self) -> Arc<dyn Simulation + Send + Sync> {
        self.simulations[self.current_simulation_index].clone()
    }

    pub fn get_simulation_count(&self) -> usize {
        self.simulations.len()
    }

    pub fn get_bodies(&self) -> Vec<Body> {
        self.bodies.lock().unwrap().clone()
    }

    pub fn next_simulation(&mut self) -> bool {
        if self.current_simulation_index < self.simulations.len() - 1 {
            self.current_simulation_index += 1;
            self.reinitialize_bodies();
            true
        } else {
            false
        }
    }

    pub fn previous_simulation(&mut self) -> bool {
        if self.current_simulation_index > 0 {
            self.current_simulation_index -= 1;
            self.reinitialize_bodies();
            true
        } else {
            false
        }
    }

    pub fn switch_to_simulation(&mut self, index: usize) -> bool {
        if index < self.simulations.len() {
            self.current_simulation_index = index;
            self.reinitialize_bodies();
            true
        } else {
            false
        }
    }

    fn reinitialize_bodies(&mut self) {
        let current_sim = &self.simulations[self.current_simulation_index];
        let new_bodies = current_sim.initialize_bodies(crate::simulation::types::NUM_BODIES);
        *self.bodies.lock().unwrap() = new_bodies;
    }

    pub fn update_simulation_state(&self, state: &mut SimulationState) {
        let current_sim = &self.simulations[self.current_simulation_index];
        let camera_pos = current_sim.camera_position();
        let camera_target = current_sim.camera_target();

        // Update view matrix based on the current simulation's camera settings
        state.view_matrix = glam::Mat4::look_at_rh(
            glam::Vec3::new(camera_pos[0], camera_pos[1], camera_pos[2]),
            glam::Vec3::new(camera_target[0], camera_target[1], camera_target[2]),
            glam::Vec3::new(0.0, 1.0, 0.0),
        )
        .to_cols_array();
    }
}
