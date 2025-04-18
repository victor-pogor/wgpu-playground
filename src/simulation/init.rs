use rand::Rng;

use crate::simulation::types::Body;
use crate::utils::color::hsl_to_rgb;

pub fn create_random_bodies(count: u32) -> Vec<Body> {
    let mut rng = rand::thread_rng();
    let mut bodies = Vec::with_capacity(count as usize);

    // Create a central "sun" with high mass
    bodies.push(Body {
        position: [0.0, 0.0, 0.0, 5000.0], // Central mass
        velocity: [0.0, 0.0, 0.0, 0.0],
        color: [1.0, 0.9, 0.1, 1.0], // Yellow
    });

    // Create bodies in a disc formation
    for _ in 1..count {
        // Random distance from center (distributed more around the edges)
        let distance = 20.0 + 50.0 * rng.gen_range(0.0f32..1.0f32).powf(0.5_f32);

        // Random angle
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);

        // Calculate position
        let x = distance * angle.cos();
        let z = distance * angle.sin();
        let y = (rng.gen_range(0.0..1.0) - 0.5) * 5.0; // Small vertical variation

        // Calculate orbital velocity (perpendicular to radial direction)
        let speed = (5.0 / distance.sqrt()).min(1.0); // Orbital velocity
        let vx = -speed * angle.sin();
        let vz = speed * angle.cos();

        // Random small mass
        let mass = 0.1 + rng.gen_range(0.0..1.0) * 2.0;

        // Generate a color based on distance
        let hue = distance / 100.0; // Normalize to 0-1 range
        let (r, g, b) = hsl_to_rgb(hue, 0.8, 0.6);

        bodies.push(Body {
            position: [x, y, z, mass],
            velocity: [vx, 0.0, vz, 0.0],
            color: [r, g, b, 1.0],
        });
    }

    bodies
}
