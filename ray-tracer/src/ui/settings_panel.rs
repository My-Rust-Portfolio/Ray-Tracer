use eframe::egui;

use crate::renderer::RenderSettings;

pub fn show(ui: &mut egui::Ui, settings: &mut RenderSettings) {
    ui.heading("Ray tracer");
    ui.separator();
    ui.label("Rendering backend");
    ui.label("CPU · active");
    ui.add_enabled(false, egui::Button::new("GPU · coming later"));

    ui.add_space(16.0);
    ui.label("Anti-aliasing");
    egui::ComboBox::from_id_salt("ssaa")
        .selected_text(match settings.samples_per_axis {
            1 => "Off · 1 sample",
            2 => "2×2 · 4 samples",
            3 => "3×3 · 9 samples",
            _ => "4×4 · 16 samples",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut settings.samples_per_axis, 1, "Off · 1 sample");
            ui.selectable_value(&mut settings.samples_per_axis, 2, "2×2 · 4 samples");
            ui.selectable_value(&mut settings.samples_per_axis, 3, "3×3 · 9 samples");
            ui.selectable_value(&mut settings.samples_per_axis, 4, "4×4 · 16 samples");
        });
    ui.label(format!(
        "{} rays per pixel",
        settings.samples_per_axis.pow(2)
    ));
    ui.add_space(12.0);
    ui.checkbox(&mut settings.shadows_enabled, "Shadows");
    ui.add_space(16.0);
    ui.separator();
    ui.label("Click and drag in the view to look around.");
    ui.label("WASD to move, Space/Ctrl to move vertically.");
}
