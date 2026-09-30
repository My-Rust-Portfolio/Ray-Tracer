use std::time::{Duration, Instant};

use eframe::egui;

use crate::renderer::{
    RenderSettings, ViewMode, cpu::CpuBackend, gpu_viewport::GpuViewport, world::RenderWorld,
};
use crate::ui::{settings_panel, settings_panel::SceneAction, viewport::Viewport};

pub struct App {
    world: RenderWorld,
    cpu_backend: CpuBackend,
    viewport: Viewport,
    gpu_viewport: Option<GpuViewport>,
    settings: RenderSettings,
    view_mode: ViewMode,
    last_frame: Instant,
    fps_elapsed: f32,
    fps_frames: u32,
    fps: f32,
}

impl App {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        Self {
            world: RenderWorld::new(),
            cpu_backend: CpuBackend::new(),
            viewport: Viewport::default(),
            gpu_viewport: creation_context
                .wgpu_render_state
                .as_ref()
                .map(GpuViewport::new),
            settings: RenderSettings::default(),
            view_mode: ViewMode::default(),
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
        self.world.update(dt);

        egui::Panel::left("settings")
            .resizable(false)
            .default_size(230.0)
            .show(root, |ui| {
                let scene_action = settings_panel::show(
                    ui,
                    &mut self.settings,
                    &mut self.view_mode,
                    self.fps,
                    self.world.spawned_sphere_count(),
                );
                match scene_action {
                    SceneAction::None => {}
                    SceneAction::SpawnSphere => self.world.spawn_random_sphere_nearby(),
                    SceneAction::DeleteSpawnedSphere => {
                        self.world.delete_last_spawned_sphere();
                    }
                }
            });

        egui::CentralPanel::default().show(root, |ui| match self.view_mode {
            ViewMode::Cpu => {
                self.viewport
                    .show(ui, &mut self.world, &mut self.cpu_backend, self.settings)
            }
            ViewMode::Gpu => {
                if let Some(gpu_viewport) = &mut self.gpu_viewport {
                    gpu_viewport.show(ui, &mut self.world, self.settings);
                } else {
                    ui.label("wgpu is unavailable on this device.");
                }
            }
        });
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}
