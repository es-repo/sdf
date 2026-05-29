#[macro_export]
macro_rules! scene_state {
    () => {
        fn save_state(&self) -> Option<serde_json::Value> {
            <Self as $crate::scene::SceneState>::save_scene_state(self)
        }

        fn load_state(&mut self, state: &serde_json::Value) {
            <Self as $crate::scene::SceneState>::load_scene_state(self, state);
        }
    };
}
