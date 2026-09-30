use eframe::egui;

use crate::renderer::{RenderSettings, ViewMode};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneAction {
    None,
    SpawnSphere,
    DeleteSpawnedSphere,
}

pub fn show(
    ui: &mut egui::Ui,
    settings: &mut RenderSettings,
    mode: &mut ViewMode,
    fps: f32,
    spawned_sphere_count: usize,
) -> SceneAction {
    ui.heading("Ray tracer");
    ui.separator();
    ui.label("Rendering path");
    egui::ComboBox::from_id_salt("rendering-path")
        .selected_text(match mode {
            ViewMode::Cpu => "CPU ray tracer",
            ViewMode::Gpu => "GPU ray tracer · basic",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(mode, ViewMode::Cpu, "CPU ray tracer");
            ui.selectable_value(mode, ViewMode::Gpu, "GPU ray tracer · basic");
        });
    if *mode == ViewMode::Gpu {
        ui.small("GPU pass: spheres, ground, textures, lighting, shadows, reflections, and sky.");
    }

    ui.add_space(16.0);
    ui.label("Performance");
    if fps > 0.0 {
        ui.label(format!("{fps:.1} FPS"));
    } else {
        ui.label("Measuring FPS…");
    }

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
    ui.add_space(16.0);
    ui.label("Scene");
    let mut scene_action = SceneAction::None;
    if ui.button("Spawn random sphere nearby").clicked() {
        scene_action = SceneAction::SpawnSphere;
    }
    ui.add_enabled_ui(spawned_sphere_count > 0, |ui| {
        if ui.button("Delete last spawned sphere").clicked() {
            scene_action = SceneAction::DeleteSpawnedSphere;
        }
    });
    ui.small(format!("{spawned_sphere_count} spawned"));
    ui.add_space(12.0);
    ui.checkbox(&mut settings.shadows_enabled, "Shadows");
    ui.add_space(16.0);
    ui.separator();
    ui.label("Click and drag in the view to look around.");
    ui.label("WASD to move, Space/Ctrl to move vertically.");
    scene_action
}
