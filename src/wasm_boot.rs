use crate::viewer::Viewer;
use sdf::scenes::{Scene, Scene1, Scene2, SimplexNoise, SimplexNoise3d, SmoothUnion};
use winit::dpi::LogicalSize;
use winit::event_loop::{ControlFlow, EventLoop};

#[allow(unused_imports)]
#[cfg(feature = "wasm_threads")]
pub use wasm_bindgen_rayon::init_thread_pool;

#[derive(Debug)]
pub enum AppEvent {
    PixelsReady(pixels::Pixels<'static>),
}

const DEFAULT_SCENE_SLUG: &str = "scene-1";
struct SceneEntry {
    slug: &'static str,
    create: fn() -> Box<dyn Scene>,
    markdown: Option<&'static str>,
}

const AVAILABLE_SCENES: &[SceneEntry] = &[
    SceneEntry {
        slug: "scene-1",
        create: || Box::new(Scene1),
        markdown: None,
    },
    SceneEntry {
        slug: "scene-2",
        create: || Box::new(Scene2),
        markdown: None,
    },
    SceneEntry {
        slug: "smooth-union",
        create: || Box::new(SmoothUnion),
        markdown: Some(include_str!("scenes/smooth_union.md")),
    },
    SceneEntry {
        slug: "simplex-noise",
        create: || Box::new(SimplexNoise),
        markdown: Some(include_str!("scenes/simplex_noise.md")),
    },
    SceneEntry {
        slug: "simplex-noise-3d",
        create: || Box::new(SimplexNoise3d),
        markdown: Some(include_str!("scenes/simplex_noise_3d.md")),
    },
];

fn available_scene_slugs() -> impl Iterator<Item = &'static str> {
    AVAILABLE_SCENES.iter().map(|scene| scene.slug)
}

fn create_scene(slug: &str) -> Box<dyn Scene> {
    AVAILABLE_SCENES
        .iter()
        .find(|scene| scene.slug == slug)
        .map(|scene| (scene.create)())
        .unwrap_or_else(|| Box::new(Scene1))
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn available_scene_slugs_csv() -> String {
    available_scene_slugs().collect::<Vec<_>>().join(",")
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn scene_markdown(scene_slug: &str) -> String {
    AVAILABLE_SCENES
        .iter()
        .find(|scene| scene.slug == scene_slug)
        .and_then(|scene| scene.markdown)
        .unwrap_or_default()
        .to_owned()
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
