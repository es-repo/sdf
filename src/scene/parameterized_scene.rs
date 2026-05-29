use super::Scene;

pub trait ParameterizedScene: Scene {
    fn parameters_ui(&mut self, ui: &mut egui::Ui);
}
