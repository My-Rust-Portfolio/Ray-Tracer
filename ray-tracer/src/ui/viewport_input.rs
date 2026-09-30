use eframe::egui;
use winit::keyboard::KeyCode;

use crate::renderer::world::RenderWorld;

/// Applies the shared camera controls for either rendering viewport.
pub fn handle(ui: &egui::Ui, response: &egui::Response, world: &mut RenderWorld) {
    response.request_focus();

    if response.dragged() {
        let delta = ui.input(|input| input.pointer.delta());
        world.mouse_delta(delta.x, delta.y);
    }

    if response.has_focus() || response.hovered() {
        ui.input(|input| {
            for (key, code) in [
                (egui::Key::W, KeyCode::KeyW),
                (egui::Key::A, KeyCode::KeyA),
                (egui::Key::S, KeyCode::KeyS),
                (egui::Key::D, KeyCode::KeyD),
                (egui::Key::Space, KeyCode::Space),
            ] {
                world.set_key(code, input.key_down(key));
            }
            world.set_key(KeyCode::ControlLeft, input.modifiers.ctrl);
        });
    }
}
