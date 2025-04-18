// Constants for N-body simulation
const NUM_BODIES: u32 = 1024;
const G: f32 = 6.67430e-11;  // Gravitational constant
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
    // For view/camera handling
    viewMatrix: mat4x4<f32>,
    projectionMatrix: mat4x4<f32>,
}

// Body data in, body data out
@group(0) @binding(0) var<storage, read> bodies_in: array<Body>;
@group(0) @binding(1) var<storage, read_write> bodies_out: array<Body>;
@group(0) @binding(2) var<uniform> sim: SimulationState;

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
    
    // Compute interactions with all other bodies
    for (var i: u32 = 0u; i < NUM_BODIES; i = i + 1u) {
        if (i == index) { continue; } // Skip self-interaction
        
        let other = bodies_in[i];
        let other_pos = other.position.xyz;
        let other_mass = other.position.w; // Other mass from position.w
        
        // Calculate direction and distance
        let dir = other_pos - pos;
        let dist_squared = dot(dir, dir) + SOFTENING;
        let dist = sqrt(dist_squared);
        
        // Newton's law of gravitation: F = G * m1 * m2 / r^2
        // a = F/m = G * m2 / r^2
        let force_mag = G * other_mass / dist_squared;
        
        // Accumulate acceleration
        acceleration = acceleration + force_mag * normalize(dir);
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
    @location(1) visual_radius: f32,
    @location(2) point_coord: vec2<f32>, // For calculating circle in fragment shader
    @location(3) clip_position: vec4<f32>, // Clip position for scaling
};

@vertex
fn vertex_main(@builtin(instance_index) instance_idx: u32, 
               @builtin(vertex_index) vertex_idx: u32) -> VertexOutput {
    let body = bodies_in[instance_idx];
    
    // Define a quad (2 triangles) for each particle
    // We'll use these to draw a circle in the fragment shader
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
    
    // Transform position by view and projection matrices
    let world_pos = vec4<f32>(body.position.xyz, 1.0);
    let view_pos = sim.viewMatrix * world_pos;
    let clip_pos = sim.projectionMatrix * view_pos;
    
    // Get the visual radius and scale it based on distance from camera
    // The scaling factor adjusts how the size changes with distance
    let visual_radius = body.velocity.w;
    
    // Scale the quad based on visual radius
    // For orthographic projection, we don't need to scale based on distance
    var scaled_quad_pos = clip_pos;
    let offset = corner * visual_radius;
    scaled_quad_pos.x = scaled_quad_pos.x + offset.x;
    scaled_quad_pos.y = scaled_quad_pos.y + offset.y;
    
    var output: VertexOutput;
    output.position = scaled_quad_pos;
    output.color = body.color;
    output.visual_radius = visual_radius;
    output.point_coord = corner; // Pass corner coordinate to fragment shader
    output.clip_position = clip_pos;
    
    return output;
}

// Fragment shader for rendering particles as circles
@fragment
fn fragment_main(
    @location(0) color: vec4<f32>,
    @location(1) visual_radius: f32,
    @location(2) point_coord: vec2<f32>,
    @location(3) clip_pos: vec4<f32>
) -> @location(0) vec4<f32> {
    // Calculate distance from center of quad
    let distance_from_center = length(point_coord);
    
    // Discard fragments outside the circle
    if (distance_from_center > 1.0) {
        discard;
    }

    // Create a smoother edge with configurable width
    let edge_smoothness = 0.1; // Increased for more gradual fade
    let alpha_edge = smoothstep(1.0, 1.0 - edge_smoothness, distance_from_center);
    
    // Start with the original color (including its alpha)
    var final_color = color.rgb;
    
    // Base alpha on the original color's alpha and edge smoothing
    // All objects will be opaque (except for edge smoothing)
    var final_alpha = color.a * alpha_edge;
    
    // Return the color with computed alpha
    return vec4<f32>(final_color, final_alpha);
}