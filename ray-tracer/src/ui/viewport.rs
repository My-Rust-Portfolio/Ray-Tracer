use eframe::egui;

use crate::renderer::{RenderSettings, cpu::CpuBackend, world::RenderWorld};
use crate::ui::viewport_input;

#[derive(Default)]
pub struct Viewport {
    texture: Option<egui::TextureHandle>,
}

impl Viewport {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut RenderWorld,
        cpu_backend: &mut CpuBackend,
        settings: RenderSettings,
    ) {
        let available = ui.available_size();
        let width = available.x.max(1.0).round() as u32;
        let height = available.y.max(1.0).round() as u32;
        world.resize(width, height);
        let rgb = cpu_backend.render_frame(world.camera(), world.scene(), width, height, settings);
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
        viewport_input::handle(ui, &response, world);
    }
}
