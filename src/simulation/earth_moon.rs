use crate::simulation::trait_def::Simulation;
use crate::simulation::types::Body;
use std::f32::consts::FRAC_PI_2;

// Earth-Moon system data using realistic parameters
// Source: NASA data and standard astronomical measurements
pub struct EarthMoonSimulation;

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

        // // Earth's initial orbit angle
        // let earth_orbit_angle = 0.0;

        // // Create Earth and get its position/velocity for Moon calculation
        // let (earth_body, earth_pos, earth_vel) = self.create_earth(earth_orbit_angle);
        // bodies.push(earth_body);

        // // Create Moon orbiting Earth
        // bodies.push(self.create_moon(earth_pos, earth_vel, earth_orbit_angle));

        // Fill remaining slots with empty bodies if needed
        if count > bodies.len() as u32 {
            let remaining = count as usize - bodies.len();
            bodies.extend(self.create_empty_bodies(remaining));
        }

        bodies
    }

    fn camera_position(&self) -> [f32; 3] {
        [0.0, 300.0, 20.0] // Positioned directly above to view the 2D simulation from top-down
    }

    fn camera_target(&self) -> [f32; 3] {
        [0.0, 0.0, 0.0] // Looking at the sun
    }
}

impl EarthMoonSimulation {
    // Constants for the simulation
    const DISTANCE_SCALE: f32 = 20.0; // Scale factor for distances (smaller = more compact)
    const SIZE_SCALE: f32 = 0.5; // Scale factor for visual sizes
    const MIN_SIZE: f32 = 0.2; // Minimum visual size

    // Creates the Sun at the center of the system
    fn create_sun(&self) -> Body {
        let sun_mass = 333000.0; // Mass in Earth masses
        let sun_radius = 109.0 * Self::SIZE_SCALE * 0.7; // Visual radius

        Body {
            position: [0.0, 0.0, 0.0, sun_mass],
            velocity: [0.0, 0.0, 0.0, sun_radius],
            color: [1.0, 0.9, 0.1, 1.0], // Yellow
        }
    }

    // Creates Earth with proper orbital parameters
    fn create_earth(&self, orbit_angle: f32) -> (Body, [f32; 2], [f32; 2]) {
        // Distance: 1 AU (149.6 million km)
        let earth_distance = 1.0 * Self::DISTANCE_SCALE;
        let earth_mass = 1.0; // Earth mass
        let earth_radius = 1.0 * Self::SIZE_SCALE.max(Self::MIN_SIZE);

        // Calculate position in 2D (X-Z plane)
        let earth_x = earth_distance * orbit_angle.cos();
        let earth_z = earth_distance * orbit_angle.sin();

        // Calculate orbital velocity (circular orbit approximation)
        let sun_mass = 333000.0; // Same as in create_sun
        let earth_speed = (sun_mass / earth_distance).sqrt() * 0.12;

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

    // Creates Moon orbiting around Earth
    fn create_moon(&self, earth_pos: [f32; 2], earth_vel: [f32; 2], orbit_angle: f32) -> Body {
        // Distance from Earth: 384,400 km (0.00257 AU)
        let moon_earth_distance = 0.00257 * Self::DISTANCE_SCALE * 5.0; // Scaled up for visibility
        let moon_mass = 0.0123; // Moon mass in Earth masses
        let moon_radius = 0.273 * Self::SIZE_SCALE.max(Self::MIN_SIZE);

        // Position moon relative to Earth with a phase angle
        let moon_orbit_angle = orbit_angle + FRAC_PI_2; // 90 degrees offset from Earth

        // Calculate moon position relative to Earth
        let moon_relative_x = moon_earth_distance * moon_orbit_angle.cos();
        let moon_relative_z = moon_earth_distance * moon_orbit_angle.sin();

        // Absolute moon position
        let moon_x = earth_pos[0] + moon_relative_x;
        let moon_z = earth_pos[1] + moon_relative_z;

        // Calculate orbital velocity around Earth
        let earth_mass = 1.0; // Same as in create_earth
        let moon_orbital_speed = ((earth_mass + moon_mass) / moon_earth_distance).sqrt() * 0.5;

        // Orbital velocity components
        let moon_orbital_vx = -moon_orbital_speed * moon_orbit_angle.sin();
        let moon_orbital_vz = moon_orbital_speed * moon_orbit_angle.cos();

        // Moon's total velocity is Earth's velocity plus its orbital velocity
        let moon_vx = earth_vel[0] + moon_orbital_vx;
        let moon_vz = earth_vel[1] + moon_orbital_vz;

        Body {
            position: [moon_x, 0.0, moon_z, moon_mass],
            velocity: [moon_vx, 0.0, moon_vz, moon_radius],
            color: [0.8, 0.8, 0.8, 1.0], // Gray
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
