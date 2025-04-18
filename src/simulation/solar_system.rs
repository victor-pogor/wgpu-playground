use crate::simulation::trait_def::Simulation;
use crate::simulation::types::Body;
use rand::Rng;

// Solar system data - AU (Astronomical Unit) for distances, Earth masses for masses
// Source: NASA data and standard astronomical measurements
pub struct SolarSystemSimulation;

impl Simulation for SolarSystemSimulation {
    fn name(&self) -> &str {
        "Solar System"
    }

    fn description(&self) -> &str {
        "An accurate simulation of our solar system with correct masses and distances"
    }

    fn initialize_bodies(&self, count: u32) -> Vec<Body> {
        // We'll use fixed data for the solar system
        // Only the first 10 bodies are fixed (Sun + 8 planets + Pluto)
        // The rest will be filled with random asteroid/debris

        // Scale factor to make the simulation visually appealing
        // Actual distances in AU would be too spread out for visualization
        let distance_scale = 10.0; // Scale down the distances
        let size_scale = 0.5; // Significantly reduced from 2.5 for more realistic planet sizes

        // Minimum visual size to ensure small planets are still visible
        let min_planet_size = 0.2; // Reduced from 0.8

        let mut bodies = Vec::with_capacity(count as usize);

        // Sun - The sun's radius is about 109 times Earth's radius
        let sun_mass = 333000.0; // Actual sun mass in Earth masses
        let sun_visual_radius = 109.0 * size_scale * 0.7; // Slightly reduce sun size for better display
        bodies.push(Body {
            position: [0.0, 0.0, 0.0, sun_mass],
            velocity: [0.0, 0.0, 0.0, sun_visual_radius],
            color: [1.0, 0.9, 0.1, 1.0], // Yellow
        });

        // Array of planets: [distance in AU, orbital period in Earth years, mass in Earth masses, radius in Earth radii, color]
        // Data source: NASA fact sheets
        let planets = [
            // Mercury (distance, period, mass, radius in Earth units, color)
            (0.39, 0.24, 0.055, 0.38, [0.8, 0.8, 0.8, 1.0]), // Gray
            // Venus
            (0.72, 0.62, 0.815, 0.95, [0.9, 0.7, 0.4, 1.0]), // Yellowish
            // Earth
            (1.0, 1.0, 1.0, 1.0, [0.2, 0.4, 0.8, 1.0]), // Blue
            // Mars
            (1.52, 1.88, 0.107, 0.53, [0.8, 0.3, 0.2, 1.0]), // Red
            // Jupiter
            (5.2, 11.86, 317.8, 11.2, [0.9, 0.75, 0.6, 1.0]), // Orange-ish
            // Saturn
            (9.58, 29.46, 95.2, 9.45, [0.9, 0.8, 0.5, 1.0]), // Yellowish
            // Uranus
            (19.18, 84.01, 14.5, 4.0, [0.5, 0.8, 0.9, 1.0]), // Cyan
            // Neptune
            (30.07, 164.8, 17.1, 3.88, [0.2, 0.4, 0.9, 1.0]), // Blue
            // Pluto (dwarf planet)
            (39.48, 248.59, 0.002, 0.18, [0.7, 0.7, 0.7, 1.0]), // Gray
        ];

        // Add planets
        for (i, planet) in planets.iter().enumerate() {
            let (distance, period, mass, radius, color) = planet;

            // Use distance and period to calculate orbital velocity
            let distance_scaled = distance * distance_scale;

            // Randomize the angle of each planet to spread them out
            let angle = std::f32::consts::TAU * (i as f32 / planets.len() as f32);

            // Calculate position
            let x = distance_scaled * angle.cos();
            let z = distance_scaled * angle.sin();

            // Remove orbital inclination for 2D view
            let y = 0.0; // Set y to 0 for flat 2D representation

            // Calculate orbital velocity
            // For circular orbits, velocity is perpendicular to radius
            let speed = (2.0 * std::f32::consts::PI * distance_scaled / period).sqrt();
            let vx = -speed * angle.sin();
            let vz = speed * angle.cos();

            // Scale the radius for visual purposes
            let visual_radius = (radius * size_scale).max(min_planet_size);

            // Add the planet with proper mass and visual radius
            bodies.push(Body {
                position: [x, y, z, *mass],
                velocity: [vx, 0.0, vz, visual_radius],
                color: *color,
            });
        }

        // Fill the rest with asteroids and other debris if requested
        if count > bodies.len() as u32 {
            let mut rng = rand::thread_rng();

            for _ in bodies.len()..count as usize {
                // Most asteroids are in the asteroid belt between Mars and Jupiter (2.2 to 3.2 AU)
                // and Kuiper belt beyond Neptune (30 to 50 AU)
                let is_kuiper = rng.gen_bool(0.3); // 30% chance for Kuiper belt object

                let distance = if is_kuiper {
                    // Kuiper belt
                    (30.0 + rng.gen_range(0.0..20.0)) * distance_scale
                } else {
                    // Asteroid belt and scattered objects
                    (2.2 + rng.gen_range(0.0..1.0)) * distance_scale
                };

                // Random angle
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);

                // Calculate position
                let x = distance * angle.cos();
                let z = distance * angle.sin();
                let y = 0.0; // Set y to 0 for flat 2D representation

                // Calculated orbital velocity (slower for distant objects)
                let period = distance.powf(1.5); // Kepler's Third Law: T² ∝ r³
                let speed = (1.0 * distance_scale / period.sqrt()).min(0.5);
                let vx = -speed * angle.sin();
                let vz = speed * angle.cos();

                // Mass for physics calculations (small for asteroids)
                let mass = rng.gen_range(0.00001..0.001);

                // Visual size for asteroids - much smaller than planets
                let visual_radius = rng.gen_range(0.01..0.2);

                // Grayish color with some variation
                let color = if is_kuiper {
                    // Icier objects tend to be blueish-gray
                    [
                        0.6 + rng.gen_range(-0.1..0.1),
                        0.6 + rng.gen_range(-0.1..0.1),
                        0.7 + rng.gen_range(-0.1..0.1),
                        1.0,
                    ]
                } else {
                    // Asteroid belt tends to be rocky, brownish
                    [
                        0.5 + rng.gen_range(-0.1..0.1),
                        0.4 + rng.gen_range(-0.1..0.1),
                        0.3 + rng.gen_range(-0.1..0.1),
                        1.0,
                    ]
                };

                bodies.push(Body {
                    position: [x, y, z, mass],
                    velocity: [vx, 0.0, vz, visual_radius],
                    color,
                });
            }
        }

        bodies
    }

    fn camera_position(&self) -> [f32; 3] {
        [0.0, 500.0, 150.0] // Positioned with a slight angle for better visibility
    }

    fn camera_target(&self) -> [f32; 3] {
        [0.0, 0.0, 0.0] // Looking at the sun
    }
}
