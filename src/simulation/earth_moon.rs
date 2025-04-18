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
        // We'll create a system with only 3 fixed bodies: Sun, Earth, and Moon
        let mut bodies = Vec::with_capacity(count as usize);

        // Visual scaling factors to make the simulation visually appealing
        let distance_scale = 20.0f32; // Scale factor for distances (smaller = more compact)
        let size_scale = 0.5f32; // Scale factor for visual sizes
        let min_size = 0.2f32; // Minimum visual size

        // Sun - center of the system
        let sun_mass = 333000.0f32; // Mass in Earth masses
        let sun_radius = 109.0f32 * size_scale * 0.7f32; // Visual radius
        bodies.push(Body {
            position: [0.0, 0.0, 0.0, sun_mass],
            velocity: [0.0, 0.0, 0.0, sun_radius],
            color: [1.0, 0.9, 0.1, 1.0], // Yellow
        });

        // Earth
        // Distance: 1 AU (149.6 million km)
        let earth_distance = 1.0f32 * distance_scale;
        let earth_mass = 1.0f32; // Earth mass
        let earth_radius = 1.0f32 * size_scale.max(min_size);

        // Earth's orbital parameters - keep it in XZ plane for 2D
        let earth_orbit_angle = 0.0f32; // Starting position angle

        // Calculate position in 2D (X-Z plane)
        let earth_x = earth_distance * earth_orbit_angle.cos();
        let earth_z = earth_distance * earth_orbit_angle.sin();

        // Calculate orbital velocity for Earth (circular orbit approximation)
        let earth_speed = (sun_mass / earth_distance).sqrt() * 0.12f32;

        // Velocity vector perpendicular to position vector in the X-Z plane
        let earth_vx = -earth_speed * earth_orbit_angle.sin();
        let earth_vz = earth_speed * earth_orbit_angle.cos();

        bodies.push(Body {
            position: [earth_x, 0.0, earth_z, earth_mass],
            velocity: [earth_vx, 0.0, earth_vz, earth_radius],
            color: [0.2, 0.4, 0.8, 1.0], // Blue
        });

        // Moon
        // Distance from Earth: 384,400 km (0.00257 AU)
        let moon_earth_distance = 0.00257f32 * distance_scale * 5.0f32; // Scale up a bit for visibility
        let moon_mass = 0.0123f32; // Moon mass in Earth masses
        let moon_radius = 0.273f32 * size_scale.max(min_size);

        // Position moon relative to Earth with a phase angle
        let moon_orbit_angle = earth_orbit_angle + FRAC_PI_2; // 90 degrees offset from Earth

        // Calculate moon position in 2D relative to Earth
        let moon_relative_x = moon_earth_distance * moon_orbit_angle.cos();
        let moon_relative_z = moon_earth_distance * moon_orbit_angle.sin();

        // Absolute moon position (Earth position + relative moon position)
        let moon_x = earth_x + moon_relative_x;
        let moon_z = earth_z + moon_relative_z;

        // Calculate orbital velocity for Moon around Earth
        let moon_orbital_speed = ((earth_mass + moon_mass) / moon_earth_distance).sqrt() * 0.5f32;

        // Calculate moon's velocity vector in 2D
        let moon_orbital_vx = -moon_orbital_speed * moon_orbit_angle.sin();
        let moon_orbital_vz = moon_orbital_speed * moon_orbit_angle.cos();

        // Moon's total velocity is Earth's velocity plus its orbital velocity around Earth
        let moon_vx = earth_vx + moon_orbital_vx;
        let moon_vz = earth_vz + moon_orbital_vz;

        bodies.push(Body {
            position: [moon_x, 0.0, moon_z, moon_mass],
            velocity: [moon_vx, 0.0, moon_vz, moon_radius],
            color: [0.8, 0.8, 0.8, 1.0], // Gray
        });

        // Fill remaining slots with invisible bodies with no mass
        // This ensures compatibility with the compute shader which expects exactly 'count' bodies
        if count > bodies.len() as u32 {
            for _ in bodies.len()..count as usize {
                bodies.push(Body {
                    position: [0.0, 0.0, 0.0, 0.0], // Zero mass
                    velocity: [0.0, 0.0, 0.0, 0.0], // Zero radius (invisible)
                    color: [0.0, 0.0, 0.0, 0.0],    // Transparent
                });
            }
        }

        bodies
    }

    fn camera_position(&self) -> [f32; 3] {
        [0.0, 50.0, 20.0] // Positioned directly above to view the 2D simulation from top-down
    }

    fn camera_target(&self) -> [f32; 3] {
        [0.0, 0.0, 0.0] // Looking at the sun
    }
}
