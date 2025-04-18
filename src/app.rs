use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::rendering::Renderer;

// Store modifier states at the App level
#[derive(Default)]
pub struct App {
    state: Option<Renderer>,
    ctrl_pressed: bool,
    shift_pressed: bool,
    last_cursor_x: f32,
    last_cursor_y: f32,
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
                // Track modifier key states (Ctrl, Shift) at the App level
                self.ctrl_pressed = modifiers.state().control_key();
                self.shift_pressed = modifiers.state().shift_key();

                // Also pass them to the renderer for its internal tracking
                state.handle_key_state(self.ctrl_pressed, self.shift_pressed);
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
                        // Rather than trying to query the position now, we'll use
                        // the last known position from CursorMoved events
                        state.handle_mouse_press(
                            [self.last_cursor_x, self.last_cursor_y],
                            self.ctrl_pressed,
                            self.shift_pressed,
                        );
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

                // Store the last known cursor position
                self.last_cursor_x = x;
                self.last_cursor_y = y;

                // The camera now handles all the logic for tracking mouse state internally
                state.handle_mouse_move([x, y]);
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
                        KeyCode::Digit2 => state.switch_simulation(1),

                        // Next/previous simulation
                        KeyCode::ArrowRight | KeyCode::KeyN => state.next_simulation(),
                        KeyCode::ArrowLeft | KeyCode::KeyP => state.previous_simulation(),

                        // Toggle info display
                        KeyCode::KeyI => state.toggle_info(),

                        // Reset camera
                        KeyCode::KeyR => {
                            // Reset camera to default state
                            state.reset_camera();
                        }

                        // Camera controls using keyboard
                        KeyCode::KeyW => state.pan_camera(0.0, 10.0), // Pan up
                        KeyCode::KeyS => state.pan_camera(0.0, -10.0), // Pan down
                        KeyCode::KeyA => state.pan_camera(10.0, 0.0), // Pan left
                        KeyCode::KeyD => state.pan_camera(-10.0, 0.0), // Pan right

                        KeyCode::KeyQ => state.rotate_camera(-1.0), // Rotate counter-clockwise
                        KeyCode::KeyE => state.rotate_camera(1.0),  // Rotate clockwise

                        KeyCode::Equal | KeyCode::NumpadAdd => state.zoom_camera(0.5), // Zoom in
                        KeyCode::Minus | KeyCode::NumpadSubtract => state.zoom_camera(-0.5), // Zoom out

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
