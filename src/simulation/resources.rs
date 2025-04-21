use bytemuck;
use wgpu::util::DeviceExt;

use crate::simulation::manager::SimulationManager;
use crate::simulation::types::{DebugData, SimulationState};

use super::config::RenderConfig;

// Simulation resources management
pub(crate) struct SimulationResources {
    pub body_buffers: [wgpu::Buffer; 2], // Ping-pong buffers
    pub simulation_state_buffer: wgpu::Buffer,
    pub debug_buffer: wgpu::Buffer,
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

        // Create debug buffer with initial zero values
        let initial_debug_data = DebugData {
            iterations: 0,
            max_force: 0.0,
            min_distance: 0.0,
            particle_info: [0.0; 4],
            _padding: 0,
        };

        let debug_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Debug Buffer"),
            contents: bytemuck::cast_slice(&[initial_debug_data]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        });

        // Create bind groups
        let bind_groups = render_config.create_bind_groups(device, &body_buffers, &simulation_state_buffer, &debug_buffer);

        Self {
            body_buffers,
            simulation_state_buffer,
            debug_buffer,
            bind_groups,
            current_buffer: 0,
        }
    }

    pub(crate) fn update_bodies(&mut self, queue: &wgpu::Queue, bodies: &[crate::simulation::types::Body]) {
        // Update the current buffer with the new bodies
        queue.write_buffer(&self.body_buffers[self.current_buffer], 0, bytemuck::cast_slice(bodies));
    }

    pub(crate) fn update_simulation_state(&self, queue: &wgpu::Queue, simulation_state: &SimulationState) {
        // Update simulation state buffer
        queue.write_buffer(&self.simulation_state_buffer, 0, bytemuck::cast_slice(&[*simulation_state]));
    }

    pub(crate) fn swap_buffers(&mut self) {
        // Swap buffers for ping-pong computation
        self.current_buffer = 1 - self.current_buffer;
    }

    pub(crate) fn read_debug_data(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> DebugData {
        // Create a staging buffer to read back debug data
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Debug Staging Buffer"),
            size: std::mem::size_of::<DebugData>() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create a command encoder to copy from the debug buffer to the staging buffer
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Debug Read Encoder"),
        });

        // Copy debug buffer to staging buffer
        encoder.copy_buffer_to_buffer(&self.debug_buffer, 0, &staging_buffer, 0, std::mem::size_of::<DebugData>() as u64);

        // Submit command to the queue
        queue.submit(Some(encoder.finish()));

        // Create a synchronization fence to ensure the data is ready
        let slice = staging_buffer.slice(..);

        // Map the buffer to read it
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        // Poll the device until the buffer is ready
        let _ = device.poll(wgpu::MaintainBase::Wait);

        // This will block until the buffer is mapped
        if let Ok(Ok(_)) = receiver.recv() {
            // Read the mapped buffer data
            let data = slice.get_mapped_range();
            // Cast the buffer data to DebugData
            let debug_data: DebugData = *bytemuck::from_bytes(&data);

            // Unmap the buffer
            drop(data);
            staging_buffer.unmap();

            debug_data
        } else {
            // Return a default value if mapping fails
            DebugData {
                iterations: 0,
                max_force: 0.0,
                min_distance: 0.0,
                particle_info: [0.0; 4],
                _padding: 0,
            }
        }
    }
}
