use app::App;
use sdf::scenes::Scene1;
use winit::dpi::LogicalSize;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod fps_counter;

#[derive(Debug)]
pub enum AppEvent {
    PixelsReady(pixels::Pixels<'static>),
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::<AppEvent>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let size_logical = LogicalSize::<u32>::new(640, 400);
    let mut app = App::new(size_logical, Scene1, event_loop.create_proxy());

    event_loop.run_app(&mut app)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();

    use winit::platform::web::EventLoopExtWebSys;

    let event_loop = EventLoop::<AppEvent>::with_user_event()
        .build()
        .map_err(|err| wasm_bindgen::JsValue::from_str(&err.to_string()))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let size_logical = LogicalSize::<u32>::new(640, 400);
    let app = App::new(size_logical, Scene1, event_loop.create_proxy());

    event_loop.spawn_app(app);

    Ok(())
}
