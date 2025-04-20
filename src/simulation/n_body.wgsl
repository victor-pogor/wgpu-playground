// Constants for N-body simulation
const NUM_BODIES: u32 = 1024;
const G: f32 = 6.67430e-11;  // Gravitational constant
const MAX_FORCE_MAG: f32 = 1.0e10; // Maximum force magnitude
const MIN_DISTANCE_SQUARED: f32 = 0.0001; // Minimum distance to avoid singularities
const SOFTENING: f32 = 0.01; // Softening parameter to avoid numerical instability
const DT: f32 = 0.001;       // Time step for integration

// Structure for a body
struct Body {
    position: vec4<f32>, // xyz = position, w = mass
    velocity: vec4<f32>, // xyz = velocity, w = visual radius
    color: vec4<f32>,    // rgba color
}

// Particle system state
struct SimulationState {
    // Runtime parameters that can be adjusted
    deltaTime: f32,
}

// Debug buffer structure - can store any values you want to inspect
struct DebugData {
    // Example debug values
    iterations: u32,
    max_force: f32,
    min_distance: f32,
    particle_info: vec4<f32>, // Can store position or other per-particle info
}

// Body data in, body data out
@group(0) @binding(0) var<storage, read> bodies_in: array<Body>;
@group(0) @binding(1) var<storage, read_write> bodies_out: array<Body>;
@group(0) @binding(2) var<uniform> sim: SimulationState;
@group(0) @binding(3) var<storage, read_write> debug_buffer: DebugData;

// Compute shader for n-body simulation using Verlet integration
@compute @workgroup_size(64)
fn compute_step(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    
    // Early return if index is out of bounds
    if (index >= NUM_BODIES) {
        return;
    }
    
    let body = bodies_in[index];
    var new_body = body;
    
    // Current position and velocity
    let pos = body.position.xyz;
    let vel = body.velocity.xyz;
    let mass = body.position.w; // Mass from position.w
    let visual_radius = body.velocity.w; // Visual radius from velocity.w
    
    // Calculate acceleration at current position
    var acceleration = vec3<f32>(0.0, 0.0, 0.0);
    
    // DEBUG variables
    var max_force: f32 = 0.0;
    var min_distance: f32 = 1000000.0; // Start with a large value
    
    // Compute interactions with all other bodies
    for (var i: u32 = 0u; i < NUM_BODIES; i = i + 1u) {
        // Skip self-interaction
        if (i == index) { 
            continue; 
        }
        
        let other = bodies_in[i];
        let other_pos = other.position.xyz;
        let other_mass = other.position.w; // Other mass from position.w
        
         // Calculate direction and distance
        let dir = other_pos - pos;
        let raw_dist_squared = dot(dir, dir);
        
        // Only process if distance is reasonable (avoids garbage data)
        if (raw_dist_squared <= MIN_DISTANCE_SQUARED) {
            continue; // Skip if distance is zero or negative
        }

        // Calculate distance and force
        let dist_squared = max(raw_dist_squared + SOFTENING, MIN_DISTANCE_SQUARED);
        let dist = sqrt(dist_squared);
        
        // Newton's law of gravitation with clamped maximum force
        let force_mag = min(G * other_mass / dist_squared, MAX_FORCE_MAG);
        
        // Only apply force if significant
        if (force_mag > 0.000001) {
            acceleration = acceleration + force_mag * normalize(dir);
        }
    }
    
    // Write debug data - only from one thread to avoid race conditions
    if (index == 0u) {
        debug_buffer.iterations += 1u;
        debug_buffer.max_force = max_force;
        debug_buffer.min_distance = min_distance;
    }
    
    // If this is a specific particle we're interested in, log its data
    if (index == 0u) { // Track particle #0 as an example
        debug_buffer.particle_info = vec4<f32>(pos, length(acceleration));
    }
    
    // Verlet integration
    // x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt^2
    // v(t+dt) = v(t) + 0.5*(a(t) + a(t+dt))*dt
    
    // First half of Verlet: update position based on current velocity and acceleration
    let new_pos = pos + vel * sim.deltaTime + 0.5 * acceleration * sim.deltaTime * sim.deltaTime;
    
    // Store new position (keep mass unchanged)
    new_body.position = vec4<f32>(new_pos, mass);
    
    // Second half of Verlet: update velocity
    // This is an approximation as we're not calculating the new acceleration at the new position
    // For more accuracy, you could add a second pass in another shader
    let new_vel = vel + acceleration * sim.deltaTime;
    
    // Store new velocity (keep visual radius unchanged)
    new_body.velocity = vec4<f32>(new_vel, visual_radius);
    
    // Store the updated body
    bodies_out[index] = new_body;
}

// Vertex shader for rendering particles
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) point_coord: vec2<f32>, // For calculating circle in fragment shader
};

@vertex
fn vertex_main(@builtin(instance_index) instance_idx: u32, 
               @builtin(vertex_index) vertex_idx: u32) -> VertexOutput {
    let body = bodies_in[instance_idx];
    
    // Define a quad (2 triangles) for each particle
    let vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0)
    );

    // Get the quad corner for this vertex
    let corner = vertices[vertex_idx];
    
    // Simplified positioning without view and projection matrices
    // Just place the particles in clip space directly
    let world_pos = body.position.xyz;
    
    // Get the visual radius from body data
    let visual_radius = body.velocity.w;
    
    // Simplified positioning - just use world position scaled to clip space
    // This is a temporary solution until view/projection matrices are re-added
    let clip_pos = vec4<f32>(world_pos * 0.01, 1.0); // Scale down positions to fit in clip space
    
    // Apply a fixed scale factor
    let base_size = visual_radius * 0.05; // Adjust scale for direct clip space
    
    // Apply the scale to the vertex position in clip space
    var scaled_pos = clip_pos;
    scaled_pos.x += corner.x * base_size;
    scaled_pos.y += corner.y * base_size;
    
    var output: VertexOutput;
    output.position = scaled_pos;
    output.color = body.color;
    output.point_coord = corner;
    
    return output;
}

// Fragment shader for rendering particles as circles
@fragment
fn fragment_main(
    @location(0) color: vec4<f32>,
    @location(1) point_coord: vec2<f32>
) -> @location(0) vec4<f32> {
    // Calculate distance from center of quad
    let distance_from_center = length(point_coord);
    
    // Discard fragments outside the circle
    if (distance_from_center > 1.0) {
        discard;
    }

    // Create a smoother edge with configurable width
    let edge_width = 0.05; // Increase edge width for better blending
    let alpha_factor = 1.0 - smoothstep(1.0 - edge_width, 1.0, distance_from_center);
    
    // Apply a radial gradient to make the center brighter
    let brightness = mix(1.0, 0.7, distance_from_center * distance_from_center);
    let final_color = color.rgb * brightness;
    
    // Apply the original alpha from color, but modulate by our edge factor
    let final_alpha = color.a * alpha_factor;
    
    return vec4<f32>(final_color, final_alpha);
}