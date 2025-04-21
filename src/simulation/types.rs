use bytemuck::{Pod, Zeroable};

// Constants for simulation
pub(crate) const NUM_BODIES: u32 = 2;
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
    pub delta_time: f32,    // 4 bytes
    pub _padding: [f32; 3], // 12 bytes of padding to align with mat4, see https://stackoverflow.com/a/75525055
}

// Debug buffer structure to match the shader's DebugData
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub(crate) struct DebugData {
    pub iterations: u32,         // 4 bytes
    pub _padding: [u32; 3],      // 12 bytes of padding to align with mat4, see https://stackoverflow.com/a/75525055
    pub particle_info: [f32; 4], // 16 bytes
}
