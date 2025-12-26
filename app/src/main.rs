use crate::app::App;
use lemon_sand_core::cell::Cell;
use std::error::Error;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(WIDTH, HEIGHT);
    app.sandbox.place(50, 179, Cell::sand());
    app.sandbox.place(50, 178, Cell::water());
    event_loop.run_app(&mut app)?;

    Ok(())
}
