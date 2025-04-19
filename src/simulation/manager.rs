use std::sync::{Arc, Mutex};

use crate::simulation::earth_moon::EarthMoonSimulation;
use crate::simulation::trait_def::Simulation;
use crate::simulation::types::Body;

pub(crate) struct SimulationManager {
    simulations: Vec<Arc<dyn Simulation + Send + Sync>>,
    current_simulation_index: usize,
    bodies: Mutex<Vec<Body>>,
}

impl SimulationManager {
    pub(crate) fn new() -> Self {
        // Create available simulations
        let simulations: Vec<Arc<dyn Simulation + Send + Sync>> = vec![Arc::new(EarthMoonSimulation)];

        // Initialize with the first simulation
        let bodies = simulations[0].initialize_bodies(crate::simulation::types::NUM_BODIES);

        Self {
            simulations,
            current_simulation_index: 0,
            bodies: Mutex::new(bodies),
        }
    }

    pub(crate) fn get_current_simulation(&self) -> Arc<dyn Simulation + Send + Sync> {
        self.simulations[self.current_simulation_index].clone()
    }

    pub(crate) fn get_simulation_count(&self) -> usize {
        self.simulations.len()
    }

    pub(crate) fn get_bodies(&self) -> Vec<Body> {
        self.bodies.lock().unwrap().clone()
    }

    pub(crate) fn next_simulation(&mut self) -> bool {
        if self.current_simulation_index < self.simulations.len() - 1 {
            self.current_simulation_index += 1;
            self.reinitialize_bodies();
            true
        } else {
            false
        }
    }

    pub(crate) fn previous_simulation(&mut self) -> bool {
        if self.current_simulation_index > 0 {
            self.current_simulation_index -= 1;
            self.reinitialize_bodies();
            true
        } else {
            false
        }
    }

    pub(crate) fn switch_to_simulation(&mut self, index: usize) -> bool {
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
}
