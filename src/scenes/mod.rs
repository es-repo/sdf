mod domain_warping;
mod scene_1;
mod scene_2;
mod simplex_noise;
mod simplex_noise_3d;
mod smooth_union;

use crate::Vec2;
pub use domain_warping::DomainWarping;
use pixels::wgpu::Color;
pub use scene_1::Scene1;
pub use scene_2::Scene2;
pub use simplex_noise::SimplexNoise;
pub use simplex_noise_3d::SimplexNoise3d;
pub use smooth_union::SmoothUnion;

pub trait SceneFrame: Send + Sync {
    fn get_pixel_color(&self, coord: Vec2<f32>, time: f32) -> Color;
}

pub trait Scene: Send + Sync {
    fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame>;
}

pub trait ParameterizedScene: Scene {
    fn parameters_ui(&mut self, ui: &mut egui::Ui);
}

pub enum SceneInstance {
    Plain(Box<dyn Scene>),
    Parameterized(Box<dyn ParameterizedScene>),
}

impl SceneInstance {
    pub fn plain<T>(scene: T) -> Self
    where
        T: Scene + 'static,
    {
        Self::Plain(Box::new(scene))
    }

    pub fn parameterized<T>(scene: T) -> Self
    where
        T: ParameterizedScene + 'static,
    {
        Self::Parameterized(Box::new(scene))
    }

    pub fn prepare_frame(&self, time: f32) -> Box<dyn SceneFrame> {
        match self {
            Self::Plain(scene) => scene.prepare_frame(time),
            Self::Parameterized(scene) => scene.prepare_frame(time),
        }
    }

    pub fn parameterized_scene(&self) -> Option<&dyn ParameterizedScene> {
        match self {
            Self::Plain(_) => None,
            Self::Parameterized(scene) => Some(scene.as_ref()),
        }
    }

    pub fn parameterized_scene_mut(&mut self) -> Option<&mut dyn ParameterizedScene> {
        match self {
            Self::Plain(_) => None,
            Self::Parameterized(scene) => Some(scene.as_mut()),
        }
    }
}
