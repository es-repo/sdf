pub trait StatefulScene {
    type State: serde::Serialize + serde::de::DeserializeOwned;

    fn state(&self) -> &Self::State;

    fn state_mut(&mut self) -> &mut Self::State;

    fn save_scene_state(&self) -> Option<serde_json::Value> {
        serde_json::to_value(self.state()).ok()
    }

    fn load_scene_state(&mut self, state: &serde_json::Value) {
        if let Ok(state) = serde_json::from_value(state.clone()) {
            *self.state_mut() = state;
        }
    }
}
