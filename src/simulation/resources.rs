use bytemuck;
use wgpu::util::DeviceExt;

use crate::simulation::manager::SimulationManager;
use crate::simulation::types::SimulationState;

use super::config::RenderConfig;

// Simulation resources management
pub(crate) struct SimulationResources {
    pub body_buffers: [wgpu::Buffer; 2], // Ping-pong buffers
    pub simulation_state_buffer: wgpu::Buffer,
    pub bind_groups: [wgpu::BindGroup; 2],
    pub current_buffer: usize,
}

impl SimulationResources {
    pub(crate) fn new(device: &wgpu::Device, simulation_manager: &SimulationManager, render_config: &RenderConfig, simulation_state: &SimulationState) -> Self {
        // Get bodies from the initial simulation
        let bodies = simulation_manager.get_bodies();

        // Create two buffers for ping-pong rendering
        let body_buffers = [
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bodies Buffer 0"),
                contents: bytemuck::cast_slice(&bodies),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bodies Buffer 1"),
                contents: bytemuck::cast_slice(&bodies),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
        ];

        // Create simulation state buffer
        let simulation_state_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation State Buffer"),
            contents: bytemuck::cast_slice(&[*simulation_state]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind groups
        let bind_groups = render_config.create_bind_groups(device, &body_buffers, &simulation_state_buffer);

        Self {
            body_buffers,
            simulation_state_buffer,
            bind_groups,
            current_buffer: 0,
        }
    }

    pub fn update_bodies(&mut self, queue: &wgpu::Queue, bodies: &[crate::simulation::types::Body]) {
        // Update the current buffer with the new bodies
        queue.write_buffer(&self.body_buffers[self.current_buffer], 0, bytemuck::cast_slice(bodies));
    }

    pub fn update_simulation_state(&self, queue: &wgpu::Queue, simulation_state: &SimulationState) {
        // Update simulation state buffer
        queue.write_buffer(&self.simulation_state_buffer, 0, bytemuck::cast_slice(&[*simulation_state]));
    }

    pub fn swap_buffers(&mut self) {
        // Swap buffers for ping-pong computation
        self.current_buffer = 1 - self.current_buffer;
    }
}
