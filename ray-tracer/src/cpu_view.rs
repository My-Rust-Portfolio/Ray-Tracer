use crate::camera::Camera;
use crate::controller::CameraController;
use crate::cpu_renderer::CpuRenderer;
use crate::scene::Scene;
use rayon::prelude::*;
use winit::keyboard::KeyCode;

pub struct CpuView {
    width: u32,
    height: u32,
    camera: Camera,
    scene: Scene,
    renderer: CpuRenderer,
    controller: CameraController,
    frame: Vec<u8>,
}

impl CpuView {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            camera: Camera::new(1, 1),
            scene: Scene::new(),
            renderer: CpuRenderer::new(),
            controller: CameraController::default(),
            frame: Vec::new(),
        }
    }

    pub fn render_frame(
        &mut self,
        width: u32,
        height: u32,
        samples_per_axis: u32,
        dt: f32,
    ) -> &[u8] {
        let width = width.max(1);
        let height = height.max(1);
        if [width, height] != [self.width, self.height] {
            self.width = width;
            self.height = height;
            self.camera = Camera::new(width, height);
            self.frame.resize(width as usize * height as usize * 3, 0);
        }
        self.controller.update(&mut self.camera, dt);

        let samples_per_axis = samples_per_axis.clamp(1, 4);
        let sample_count = samples_per_axis * samples_per_axis;
        self.frame
            .par_chunks_mut(width as usize * 3)
            .enumerate()
            .for_each(|(y, row)| {
                for (x, pixel) in row.chunks_exact_mut(3).enumerate() {
                    let mut colour = [0u32; 3];
                    for sample_y in 0..samples_per_axis {
                        for sample_x in 0..samples_per_axis {
                            let offset = [
                                (sample_x as f32 + 0.5) / samples_per_axis as f32,
                                (sample_y as f32 + 0.5) / samples_per_axis as f32,
                            ];
                            let ray = self.camera.ray_for_sample(x as u32, y as u32, offset);
                            let sample = self.renderer.shade_ray(&self.scene, &ray, 0);
                            for (channel, value) in colour.iter_mut().zip(sample) {
                                *channel += value as u32;
                            }
                        }
                    }
                    for (out, value) in pixel.iter_mut().zip(colour) {
                        *out = (value / sample_count) as u8;
                    }
                }
            });

        &self.frame
    }

    pub fn handle_key(&mut self, key: KeyCode, pressed: bool) {
        self.controller.set_key(key, pressed);
    }

    pub fn handle_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.controller.mouse_delta(dx, dy);
    }

    pub fn set_shadows_enabled(&mut self, enabled: bool) {
        self.renderer.set_shadows_enabled(enabled);
    }
}
