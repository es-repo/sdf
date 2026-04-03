use crate::viewer::Viewer;
use sdf::scenes::{Scene, Scene1, SimplexNoise, SmoothUnion};
use winit::dpi::LogicalSize;
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Debug)]
pub enum AppEvent {
    PixelsReady(pixels::Pixels<'static>),
}

const DEFAULT_SCENE_SLUG: &str = "scene-1";
const AVAILABLE_SCENES: &[(&str, fn() -> Box<dyn Scene>)] = &[
    ("scene-1", || Box::new(Scene1)),
    ("smooth-union", || Box::new(SmoothUnion)),
    ("simplex-noise", || Box::new(SimplexNoise)),
];

fn available_scene_slugs() -> impl Iterator<Item = &'static str> {
    AVAILABLE_SCENES.iter().map(|(slug, _)| *slug)
}

fn create_scene(slug: &str) -> Box<dyn Scene> {
    AVAILABLE_SCENES
        .iter()
        .find(|(candidate, _)| *candidate == slug)
        .map(|(_, create)| create())
        .unwrap_or_else(|| Box::new(Scene1))
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn available_scene_slugs_csv() -> String {
    available_scene_slugs().collect::<Vec<_>>().join(",")
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start(scene_slug: &str) -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();

    use winit::platform::web::EventLoopExtWebSys;

    let event_loop = EventLoop::<AppEvent>::with_user_event()
        .build()
        .map_err(|err| wasm_bindgen::JsValue::from_str(&err.to_string()))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let size_logical = LogicalSize::<u32>::new(640, 400);
    let selected_slug = if available_scene_slugs().any(|candidate| candidate == scene_slug) {
        scene_slug
    } else {
        DEFAULT_SCENE_SLUG
    };
    let viewer = Viewer::new(size_logical, create_scene(selected_slug), event_loop.create_proxy());

    event_loop.spawn_app(viewer);

    Ok(())
}
