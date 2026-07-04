use crate::rendering::RayMarchSettings;

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(super) struct Scene8Controls {
    pub ray_march_settings: RayMarchSettings,
    #[serde(default = "default_shadow_softness")]
    pub shadow_softness: f32,
    pub animate_ground: bool,
    pub show_convergence_failure_debug: bool,
}

fn default_shadow_softness() -> f32 {
    0.05
}

impl Default for Scene8Controls {
    fn default() -> Self {
        Self {
            ray_march_settings: RayMarchSettings {
                max_steps: 400,
                hit_epsilon: 0.001,
                max_distance: 100.0,
                min_step: 0.005,
                near_clip: 0.05,
            },
            shadow_softness: default_shadow_softness(),
            animate_ground: true,
            show_convergence_failure_debug: false,
        }
    }
}
