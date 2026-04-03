#[cfg(not(target_arch = "wasm32"))]
use sdf::scenes::SimplexNoise;
#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
use viewer::Viewer;
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::LogicalSize;
#[cfg(not(target_arch = "wasm32"))]
use winit::event_loop::{ControlFlow, EventLoop};

mod fps_counter;
mod viewer;
#[cfg(target_arch = "wasm32")]
mod wasm_boot;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let size_logical = LogicalSize::<u32>::new(640, 400);
    let mut viewer = Viewer::new(size_logical, Box::new(SimplexNoise));

    event_loop.run_app(&mut viewer)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
