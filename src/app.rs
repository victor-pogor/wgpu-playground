use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::renderer::Renderer;

#[derive(Default)]
pub struct App {
    state: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(Renderer::new(window.clone()));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
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
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                match key_code {
                    // Number keys for direct simulation selection
                    KeyCode::Digit1 => state.switch_simulation(0),
                    KeyCode::Digit2 => state.switch_simulation(1),

                    // Next/previous simulation
                    KeyCode::ArrowRight | KeyCode::KeyN => state.next_simulation(),
                    KeyCode::ArrowLeft | KeyCode::KeyP => state.previous_simulation(),

                    // Toggle info display
                    KeyCode::KeyI => state.toggle_info(),

                    _ => (),
                }
            }
            _ => (),
        }
    }
}

pub fn run() {
    // Initialize logger
    env_logger::init();

    // Create event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create app
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
