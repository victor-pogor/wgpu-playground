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

        // Earth's initial orbit angle
        let earth_orbit_angle = 0.0;

        // Create Earth and get its position/velocity for Moon calculation
        let (earth_body, earth_pos, earth_vel) = self.create_earth(earth_orbit_angle);
        bodies.push(earth_body);

        // Fill remaining slots with empty bodies if needed
        if count > bodies.len() as u32 {
            let remaining = count as usize - bodies.len();
            bodies.extend(self.create_empty_bodies(remaining));
        }

        bodies
    }
}

impl EarthMoonSimulation {
    // Constants for the simulation
    const DISTANCE_SCALE: f32 = 20.0; // Scale factor for distances (smaller = more compact)
    const SIZE_SCALE: f32 = 0.0275; // Scale factor for visual sizes
    const MIN_SIZE: f32 = 0.2; // Minimum visual size

    // Creates the Sun at the center of the system
    fn create_sun(&self) -> Body {
        let sun_mass = 1.98847e30; // Mass in kg (real Sun mass)
        let sun_radius = 6.9634e8; // Sun radius in meters (for visual scale)

        Body {
            position: [0.0, 0.0, 0.0, sun_mass],
            velocity: [0.0, 0.0, 0.0, sun_radius],
            color: [1.0, 0.9, 0.1, 1.0], // Yellow
        }
    }

    // Creates Earth with proper orbital parameters
    fn create_earth(&self, orbit_angle: f32) -> (Body, [f32; 2], [f32; 2]) {
        // Distance: 1 AU (149.6 million km)
        let earth_distance = 1.496e11; // meters
        let earth_mass = 5.9722e24; // kg
        let earth_radius = 6.371e6; // meters (for visual scale)

        // Calculate position in 2D (X-Z plane)
        let earth_x = earth_distance * orbit_angle.cos();
        let earth_z = earth_distance * orbit_angle.sin();

        // Calculate orbital velocity (circular orbit approximation)
        let sun_mass = 1.98847e30; // kg
        let g = 6.67430e-11; // m^3 kg^-1 s^-2
        let earth_speed = (g * sun_mass / earth_distance).sqrt(); // m/s

        // Velocity vector perpendicular to position vector
        let earth_vx = -earth_speed * orbit_angle.sin();
        let earth_vz = earth_speed * orbit_angle.cos();

        let earth_body = Body {
            position: [earth_x, 0.0, earth_z, earth_mass],
            velocity: [earth_vx, 0.0, earth_vz, earth_radius],
            color: [0.2, 0.4, 0.8, 1.0], // Blue
        };

        // Return the body and its position/velocity for use with the moon
        (earth_body, [earth_x, earth_z], [earth_vx, earth_vz])
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
