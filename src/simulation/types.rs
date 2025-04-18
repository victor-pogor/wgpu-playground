use bytemuck::{Pod, Zeroable};

// Constants for simulation
pub const NUM_BODIES: u32 = 1024;
pub const COMPUTE_WORKGROUP_SIZE: u32 = 64;

// Represents a single body in our n-body simulation
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Body {
    pub position: [f32; 4], // xyz = position, w = mass
    pub velocity: [f32; 4], // xyz = velocity, w = visual radius
    pub color: [f32; 4],    // rgba color
}

// Runtime state for the simulation
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SimulationState {
    pub delta_time: f32,
    pub _padding: [f32; 3], // Padding to align with mat4
    pub view_matrix: [f32; 16],
    pub projection_matrix: [f32; 16],
}
