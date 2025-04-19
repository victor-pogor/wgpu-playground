use log;
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::rendering::Renderer;

#[derive(Default)]
pub(crate) struct App {
    state: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let (_monitor_width, monitor_height) = compute_screen_size(event_loop);

        // Calculate window size as percentage of monitor size (80% width, 60% height)
        // let window_width = (monitor_width as f32 * 0.8) as u32;
        let window_height = (monitor_height as f32 * 0.6) as u32;
        let window_width = window_height as u32;

        // Create the actual window with the calculated size
        let window_attributes = Window::default_attributes()
            .with_title("WebGPU Playground")
            .with_inner_size(winit::dpi::PhysicalSize::new(window_width, window_height));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let state = pollster::block_on(Renderer::new(window.clone()));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _window_id: winit::window::WindowId, event: winit::event::WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // This tells winit that we want another frame after this one
                state.get_window().request_redraw();

                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.get_size()),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                        log::error!("OutOfMemory");
                        event_loop.exit();
                    }

                    // This happens when the a frame takes too long to present
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout")
                    }
                }
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                if key_state == ElementState::Pressed {
                    match key_code {
                        KeyCode::Digit1 => state.switch_simulation(0),
                        KeyCode::Digit2 => state.switch_simulation(1),
                        KeyCode::ArrowRight => state.next_simulation(),
                        KeyCode::ArrowLeft => state.previous_simulation(),
                        KeyCode::KeyI => state.toggle_info(),
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
}

fn compute_screen_size(event_loop: &winit::event_loop::ActiveEventLoop) -> (u32, u32) {
    // Get the primary monitor
    let monitor = event_loop.primary_monitor().expect("No primary monitor found");

    // Get the size of the monitor
    let size = monitor.size();
    // Return the width and height
    (size.width, size.height)
}
