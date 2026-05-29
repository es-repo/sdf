/// Implements `Scene` persistence methods for a type that already implements
/// `StatefulScene`.
///
/// Use this inside an `impl Scene for MyScene` block to forward `save_state`
/// and `load_state` to the reusable `StatefulScene` serialization logic.
#[macro_export]
macro_rules! scene_state {
    () => {
        fn save_state(&self) -> Option<serde_json::Value> {
            <Self as $crate::scene::StatefulScene>::save_scene_state(self)
        }

        fn load_state(&mut self, state: &serde_json::Value) {
            <Self as $crate::scene::StatefulScene>::load_scene_state(self, state);
        }
    };
}
