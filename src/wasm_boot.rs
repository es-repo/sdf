use crate::viewer::Viewer;
use sdf::scene::Scene;
use sdf::scenes::{
    DomainRepetitionScene, DomainWarpingScene, PhongLightingScene, RayMarchingScene, Scene1, Scene2,
    Scene3, Scene4, Scene5, SdfDisplacementScene, SdfScene, SimplexNoise3dScene, SimplexNoiseScene,
    SmoothUnionScene,
};
use std::cell::RefCell;
use std::fmt;
use winit::dpi::LogicalSize;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};

#[allow(unused_imports)]
#[cfg(feature = "wasm_threads")]
pub use wasm_bindgen_rayon::init_thread_pool;

pub enum AppEvent {
    PixelsReady(pixels::Pixels<'static>),
    SwitchScene(Box<dyn Scene>),
    ResizeScene { width: u32, height: u32 },
    SetAudioEnabled(bool),
}

impl fmt::Debug for AppEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PixelsReady(_) => f.write_str("PixelsReady"),
            Self::SwitchScene(_) => f.write_str("SwitchScene"),
            Self::ResizeScene { .. } => f.write_str("ResizeScene"),
            Self::SetAudioEnabled(_) => f.write_str("SetAudioEnabled"),
        }
    }
}

const DEFAULT_SCENE_SLUG: &str = "scene-1";
const DEFAULT_SCENE_WIDTH: u32 = 640;
const DEFAULT_SCENE_HEIGHT: u32 = 400;

thread_local! {
    static EVENT_PROXY: RefCell<Option<EventLoopProxy<AppEvent>>> = const { RefCell::new(None) };
}

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
        slug: "scene-3",
        create: || Box::new(Scene3::default()),
        markdown: None,
    },
    SceneEntry {
        slug: "scene-4",
        create: || Box::new(Scene4::default()),
        markdown: None,
    },
    SceneEntry {
        slug: "scene-5",
        create: || Box::new(Scene5::default()),
        markdown: None,
    },
    SceneEntry {
        slug: "sdf",
        create: || Box::new(SdfScene),
        markdown: Some(include_str!("scenes/sdf/docs.md")),
    },
    SceneEntry {
        slug: "ray-marching",
        create: || Box::new(RayMarchingScene::default()),
        markdown: Some(include_str!("scenes/ray_marching/docs.md")),
    },
    SceneEntry {
        slug: "phong-lighting",
        create: || Box::new(PhongLightingScene::default()),
        markdown: Some(include_str!("scenes/phong_lighting/docs.md")),
    },
    SceneEntry {
        slug: "domain-repetition",
        create: || Box::new(DomainRepetitionScene::default()),
        markdown: Some(include_str!("scenes/domain_repetition/docs.md")),
    },
    SceneEntry {
        slug: "domain-warping",
        create: || Box::new(DomainWarpingScene::default()),
        markdown: Some(include_str!("scenes/domain_warping/docs.md")),
    },
    SceneEntry {
        slug: "sdf-displacement",
        create: || Box::new(SdfDisplacementScene::default()),
        markdown: Some(include_str!("scenes/sdf_displacement/docs.md")),
    },
    SceneEntry {
        slug: "smooth-union",
        create: || Box::new(SmoothUnionScene),
        markdown: Some(include_str!("scenes/smooth_union/docs.md")),
    },
    SceneEntry {
        slug: "simplex-noise",
        create: || Box::new(SimplexNoiseScene),
        markdown: Some(include_str!("scenes/simplex_noise/docs.md")),
    },
    SceneEntry {
        slug: "simplex-noise-3d",
        create: || Box::new(SimplexNoise3dScene),
        markdown: Some(include_str!("scenes/simplex_noise_3d/docs.md")),
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
pub fn demo_scene_slugs_csv() -> String {
    AVAILABLE_SCENES
        .iter()
        .filter(|scene| scene.markdown.is_none())
        .map(|scene| scene.slug)
        .collect::<Vec<_>>()
        .join(",")
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn technique_scene_slugs_csv() -> String {
    AVAILABLE_SCENES
        .iter()
        .filter(|scene| scene.markdown.is_some())
        .map(|scene| scene.slug)
        .collect::<Vec<_>>()
        .join(",")
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
pub fn scene_has_audio(scene_slug: &str) -> bool {
    AVAILABLE_SCENES
        .iter()
        .find(|scene| scene.slug == scene_slug)
        .map(|scene| (scene.create)().audio_track().is_some())
        .unwrap_or(false)
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn wasm_app_cores() -> i32 {
    option_env!("WASM_APP_CORES")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn switch_scene(scene_slug: &str) -> Result<(), wasm_bindgen::JsValue> {
    let selected_slug = if available_scene_slugs().any(|candidate| candidate == scene_slug) {
        scene_slug
    } else {
        DEFAULT_SCENE_SLUG
    };

    EVENT_PROXY.with(|event_proxy| {
        let Some(event_proxy) = event_proxy.borrow().as_ref().cloned() else {
            return Err(wasm_bindgen::JsValue::from_str("Viewer is not running."));
        };

        event_proxy
            .send_event(AppEvent::SwitchScene(create_scene(selected_slug)))
            .map_err(|_| wasm_bindgen::JsValue::from_str("Failed to switch scene."))
    })
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_audio_enabled(enabled: bool) -> Result<(), wasm_bindgen::JsValue> {
    EVENT_PROXY.with(|event_proxy| {
        let Some(event_proxy) = event_proxy.borrow().as_ref().cloned() else {
            return Err(wasm_bindgen::JsValue::from_str("Viewer is not running."));
        };

        event_proxy
            .send_event(AppEvent::SetAudioEnabled(enabled))
            .map_err(|_| wasm_bindgen::JsValue::from_str("Failed to update audio state."))
    })
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn resize_scene(scene_width: u32, scene_height: u32) -> Result<(), wasm_bindgen::JsValue> {
    EVENT_PROXY.with(|event_proxy| {
        let Some(event_proxy) = event_proxy.borrow().as_ref().cloned() else {
            return Err(wasm_bindgen::JsValue::from_str("Viewer is not running."));
        };

        event_proxy
            .send_event(AppEvent::ResizeScene {
                width: scene_dimension(scene_width, DEFAULT_SCENE_WIDTH),
                height: scene_dimension(scene_height, DEFAULT_SCENE_HEIGHT),
            })
            .map_err(|_| wasm_bindgen::JsValue::from_str("Failed to resize scene."))
    })
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start(scene_slug: &str, scene_width: u32, scene_height: u32) -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();

    use winit::platform::web::EventLoopExtWebSys;

    let event_loop = EventLoop::<AppEvent>::with_user_event()
        .build()
        .map_err(|err| wasm_bindgen::JsValue::from_str(&err.to_string()))?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let event_proxy = event_loop.create_proxy();

    EVENT_PROXY.with(|proxy| {
        proxy.replace(Some(event_proxy.clone()));
    });

    let size_logical = LogicalSize::<u32>::new(
        scene_dimension(scene_width, DEFAULT_SCENE_WIDTH),
        scene_dimension(scene_height, DEFAULT_SCENE_HEIGHT),
    );
    let selected_slug = if available_scene_slugs().any(|candidate| candidate == scene_slug) {
        scene_slug
    } else {
        DEFAULT_SCENE_SLUG
    };
    let viewer = Viewer::new(size_logical, create_scene(selected_slug), event_proxy);

    event_loop.spawn_app(viewer);

    Ok(())
}

fn scene_dimension(value: u32, default: u32) -> u32 {
    if value == 0 { default } else { value.min(default) }
}
