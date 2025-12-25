use crate::app::App;
use std::error::Error;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;

const WIDTH: usize = 320;
const HEIGHT: usize = 180;

fn main() -> Result<(), Box<dyn Error>> {
    //tracing_subscriber::fmt()
    //    .with_span_events(FmtSpan::CLOSE)
    //    .init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(WIDTH, HEIGHT);
    event_loop.run_app(&mut app)?;

    Ok(())
}
