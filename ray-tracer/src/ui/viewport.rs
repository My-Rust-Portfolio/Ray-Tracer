use eframe::egui;
use winit::keyboard::KeyCode;

use crate::renderer::{RenderSettings, cpu::CpuView};

#[derive(Default)]
pub struct Viewport {
    texture: Option<egui::TextureHandle>,
}

impl Viewport {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        cpu_view: &mut CpuView,
        settings: RenderSettings,
        dt: f32,
    ) {
        let available = ui.available_size();
        let width = available.x.max(1.0).round() as u32;
        let height = available.y.max(1.0).round() as u32;
        let rgb = cpu_view.render_frame(width, height, settings, dt);
        let image = egui::ColorImage::from_rgb([width as usize, height as usize], rgb);
        if let Some(texture) = &mut self.texture {
            texture.set(image, egui::TextureOptions::LINEAR);
        } else {
            self.texture = Some(ui.ctx().load_texture(
                "ray-tracer",
                image,
                egui::TextureOptions::LINEAR,
            ));
        }

        let texture = self.texture.as_ref().expect("texture initialized");
        let response = ui
            .add(egui::Image::new((texture.id(), available)).sense(egui::Sense::click_and_drag()));
        response.request_focus();
        if response.dragged() {
            let delta = ui.input(|input| input.pointer.delta());
            cpu_view.handle_mouse_delta(delta.x, delta.y);
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
                    cpu_view.handle_key(code, input.key_down(key));
                }
                cpu_view.handle_key(KeyCode::ControlLeft, input.modifiers.ctrl);
            });
        }
    }
}
