use std::time::{Duration, Instant};

use eframe::egui;

use crate::renderer::{RenderSettings, cpu::CpuView};
use crate::ui::{settings_panel, viewport::Viewport};

pub struct App {
    cpu_view: CpuView,
    viewport: Viewport,
    settings: RenderSettings,
    last_frame: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cpu_view: CpuView::new(),
            viewport: Viewport::default(),
            settings: RenderSettings::default(),
            last_frame: Instant::now(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        egui::Panel::left("settings")
            .resizable(false)
            .default_size(230.0)
            .show(root, |ui| settings_panel::show(ui, &mut self.settings));

        let dt = self.last_frame.elapsed().as_secs_f32().min(0.1);
        self.last_frame = Instant::now();
        egui::CentralPanel::default().show(root, |ui| {
            self.viewport
                .show(ui, &mut self.cpu_view, self.settings, dt);
        });
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}
