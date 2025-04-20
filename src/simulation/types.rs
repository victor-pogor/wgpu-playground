use bytemuck::{Pod, Zeroable};

// Constants for simulation
pub(crate) const NUM_BODIES: u32 = 1024;
pub(crate) const COMPUTE_WORKGROUP_SIZE: u32 = 64;

// Represents a single body in our n-body simulation
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(crate) struct Body {
    pub position: [f32; 4], // xyz = position, w = mass
    pub velocity: [f32; 4], // xyz = velocity, w = visual radius
    pub color: [f32; 4],    // rgba color
}

// Runtime state for the simulation
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(crate) struct SimulationState {
    pub delta_time: f32,
    pub _padding: [f32; 3], // Padding to align with mat4
}

// Debug buffer structure to match the shader's DebugData
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(crate) struct DebugData {
    pub iterations: u32,
    pub max_force: f32,
    pub min_distance: f32,
    pub particle_info: [f32; 4],
    pub _padding: u32,
}
