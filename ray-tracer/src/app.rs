use std::time::{Duration, Instant};

use eframe::egui;

use crate::renderer::{RenderSettings, cpu::CpuView};
use crate::ui::{settings_panel, viewport::Viewport};

pub struct App {
    cpu_view: CpuView,
    viewport: Viewport,
    settings: RenderSettings,
    last_frame: Instant,
    fps_elapsed: f32,
    fps_frames: u32,
    fps: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cpu_view: CpuView::new(),
            viewport: Viewport::default(),
            settings: RenderSettings::default(),
            last_frame: Instant::now(),
            fps_elapsed: 0.0,
            fps_frames: 0,
            fps: 0.0,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let now = Instant::now();
        let frame_seconds = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.fps_elapsed += frame_seconds;
        self.fps_frames += 1;
        if self.fps_elapsed >= 0.5 {
            self.fps = self.fps_frames as f32 / self.fps_elapsed;
            self.fps_elapsed = 0.0;
            self.fps_frames = 0;
        }
        let dt = frame_seconds.min(0.1);

        egui::Panel::left("settings")
            .resizable(false)
            .default_size(230.0)
            .show(root, |ui| {
                settings_panel::show(ui, &mut self.settings, self.fps)
            });

        egui::CentralPanel::default().show(root, |ui| {
            self.viewport
                .show(ui, &mut self.cpu_view, self.settings, dt);
        });
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}
