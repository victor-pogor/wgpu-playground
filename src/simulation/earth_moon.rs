use crate::simulation::trait_def::Simulation;
use crate::simulation::types::Body;

// Earth-Moon system data using realistic parameters
// Source: NASA data and standard astronomical measurements
pub(crate) struct EarthMoonSimulation;

impl Simulation for EarthMoonSimulation {
    fn name(&self) -> &str {
        "Earth-Moon System"
    }

    fn description(&self) -> &str {
        "Accurate 2D simulation of the Sun, Earth, and Moon with correct masses, distances, and velocities"
    }

    fn initialize_bodies(&self, count: u32) -> Vec<Body> {
        let mut bodies = Vec::with_capacity(count as usize);

        // Create the celestial bodies
        bodies.push(self.create_sun());

        // Fill remaining slots with empty bodies if needed
        if count > bodies.len() as u32 {
            let remaining = count as usize - bodies.len();
            bodies.extend(self.create_empty_bodies(remaining));
        }

        bodies
    }
}

impl EarthMoonSimulation {
    // Creates the Sun at the center of the system
    fn create_sun(&self) -> Body {
        let sun_mass = 333000.0; // Mass in Earth masses
        let sun_radius = 3.0; // Visual radius

        Body {
            position: [0.0, 0.0, 0.0, sun_mass],
            velocity: [0.0, 0.0, 0.0, sun_radius],
            color: [1.0, 0.9, 0.1, 1.0], // Yellow
        }
    }

    // Create empty placeholder bodies to fill the required count
    fn create_empty_bodies(&self, count: usize) -> Vec<Body> {
        (0..count)
            .map(|_| Body {
                position: [0.0, 0.0, 0.0, 0.0], // Zero mass
                velocity: [0.0, 0.0, 0.0, 0.0], // Zero radius (invisible)
                color: [0.0, 0.0, 0.0, 0.0],    // Transparent
            })
            .collect()
    }
}
