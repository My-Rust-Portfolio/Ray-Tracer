use std::time::Duration;
use std::time::Instant;

use eframe::egui;
use winit::keyboard::KeyCode;

use crate::cpu_view::CpuView;

pub struct App {
    cpu_view: CpuView,
    texture: Option<egui::TextureHandle>,
    samples_per_axis: u32,
    shadows_enabled: bool,
    last_frame: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cpu_view: CpuView::new(),
            texture: None,
            samples_per_axis: 1,
            shadows_enabled: true,
            last_frame: Instant::now(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        egui::CentralPanel::default().show(root, |ui| {
            egui::Panel::left("settings")
                .resizable(false)
                .default_size(230.0)
                .show(ui, |ui| {
                    ui.heading("Ray tracer");
                    ui.separator();
                    ui.label("Rendering backend");
                    ui.label("CPU · active");
                    ui.add_enabled(false, egui::Button::new("GPU · coming later"));

                    ui.add_space(16.0);
                    ui.label("Anti-aliasing");
                    egui::ComboBox::from_id_salt("ssaa")
                        .selected_text(match self.samples_per_axis {
                            1 => "Off · 1 sample",
                            2 => "2×2 · 4 samples",
                            3 => "3×3 · 9 samples",
                            _ => "4×4 · 16 samples",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.samples_per_axis, 1, "Off · 1 sample");
                            ui.selectable_value(&mut self.samples_per_axis, 2, "2×2 · 4 samples");
                            ui.selectable_value(&mut self.samples_per_axis, 3, "3×3 · 9 samples");
                            ui.selectable_value(&mut self.samples_per_axis, 4, "4×4 · 16 samples");
                        });
                    ui.label(format!("{} rays per pixel", self.samples_per_axis.pow(2)));
                    ui.add_space(12.0);
                    ui.checkbox(&mut self.shadows_enabled, "Shadows");
                    ui.add_space(16.0);
                    ui.separator();
                    ui.label("Click and drag in the view to look around.");
                    ui.label("WASD to move, Space/Ctrl to move vertically.");
                });

            let available = ui.available_size();
            let width = available.x.max(1.0).round() as u32;
            let height = available.y.max(1.0).round() as u32;
            let dt = self.last_frame.elapsed().as_secs_f32().min(0.1);
            self.last_frame = Instant::now();
            self.cpu_view.set_shadows_enabled(self.shadows_enabled);
            let rgb = self
                .cpu_view
                .render_frame(width, height, self.samples_per_axis, dt);
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
            let response = ui.add(
                egui::Image::new((texture.id(), available)).sense(egui::Sense::click_and_drag()),
            );
            response.request_focus();
            if response.dragged() {
                let delta = ui.input(|input| input.pointer.delta());
                self.cpu_view.handle_mouse_delta(delta.x, delta.y);
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
                        self.cpu_view.handle_key(code, input.key_down(key));
                    }
                    self.cpu_view
                        .handle_key(KeyCode::ControlLeft, input.modifiers.ctrl);
                });
            }
        });

        ctx.request_repaint_after(Duration::from_millis(16));
    }
}
