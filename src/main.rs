mod app;
mod rendering;
mod simulation;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    // Initialize logger
    env_logger::init();

    // Create event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create app
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
