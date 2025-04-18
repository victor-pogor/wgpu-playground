use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
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
            WindowEvent::ModifiersChanged(modifiers) => {
                // Track modifier key states (Ctrl, Shift)
                state.handle_key_state(
                    modifiers.state().control_key(),
                    modifiers.state().shift_key(),
                );
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Handle mouse wheel for zoom
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        // Regular mouse wheel - use value directly
                        state.handle_mouse_wheel(y);
                    }
                    MouseScrollDelta::PixelDelta(position) => {
                        // Touchpad gesture - needs smaller scaling factor to feel natural
                        // Multiply by 0.003 for touchpad sensitivity
                        state.handle_mouse_wheel(position.y as f32 * 0.003);
                    }
                }
            }
            WindowEvent::MouseInput {
                state: button_state,
                button: MouseButton::Left,
                ..
            } => {
                match button_state {
                    ElementState::Pressed => {
                        // We'll initiate the position in the MouseMove handler
                    }
                    ElementState::Released => {
                        state.handle_mouse_release();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                // Convert position to normalized coordinates for easier handling
                let x = position.x as f32;
                let y = position.y as f32;

                if state.mouse_pressed {
                    // If mouse is already pressed, process movement
                    state.handle_mouse_move([x, y]);
                } else if state.ctrl_pressed || state.shift_pressed {
                    // Start tracking if ctrl or shift is pressed, and we get movement
                    state.handle_mouse_press([x, y], state.ctrl_pressed, state.shift_pressed);
                }
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
                        // Number keys for direct simulation selection
                        KeyCode::Digit1 => state.switch_simulation(0),

                        // Next/previous simulation
                        KeyCode::ArrowRight | KeyCode::KeyN => state.next_simulation(),
                        KeyCode::ArrowLeft | KeyCode::KeyP => state.previous_simulation(),

                        // Toggle info display
                        KeyCode::KeyI => state.toggle_info(),

                        // Reset camera
                        KeyCode::KeyR => {
                            state.camera_offset = [0.0, 0.0];
                            state.camera_zoom = 1.0;
                            state.camera_rotation = 0.0;
                            state.update_camera_view();
                        }

                        _ => (),
                    }
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
