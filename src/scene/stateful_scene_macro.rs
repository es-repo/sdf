/// Implements `StatefulScene` for a scene type that stores persistent data in a
/// state field.
///
/// The two-argument form assumes the field is named `state`:
///
/// ```ignore
/// crate::stateful_scene!(MyScene, MySceneState);
/// ```
///
/// Use the three-argument form when the field has another name:
///
/// ```ignore
/// crate::stateful_scene!(MyScene, MySceneState, persistent_state);
/// ```
#[macro_export]
macro_rules! stateful_scene {
    ($scene:ty, $state:ty) => {
        $crate::stateful_scene!($scene, $state, state);
    };

    ($scene:ty, $state:ty, $field:ident) => {
        impl $crate::scene::StatefulScene for $scene {
            type State = $state;

            fn state(&self) -> &Self::State {
                &self.$field
            }

            fn state_mut(&mut self) -> &mut Self::State {
                &mut self.$field
            }
        }
    };
}
