mod shading;

pub use shading::CpuRenderer as CpuShader;

use crate::camera::Camera;
use crate::scene::Scene;
use rayon::prelude::*;

use super::RenderSettings;

/// CPU backend that renders shared camera/scene state into an RGB frame.
pub struct CpuBackend {
    width: u32,
    height: u32,
    shader: CpuShader,
    frame: Vec<u8>,
}

impl CpuBackend {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            shader: CpuShader::new(),
            frame: Vec::new(),
        }
    }

    pub fn render_frame(
        &mut self,
        camera: &Camera,
        scene: &Scene,
        width: u32,
        height: u32,
        settings: RenderSettings,
    ) -> &[u8] {
        let width = width.max(1);
        let height = height.max(1);
        if [width, height] != [self.width, self.height] {
            self.width = width;
            self.height = height;
            self.frame.resize(width as usize * height as usize * 3, 0);
        }

        let samples_per_axis = settings.samples_per_axis.clamp(1, 4);
        if samples_per_axis == 1 {
            self.frame
                .par_chunks_mut(width as usize * 3)
                .enumerate()
                .for_each(|(y, row)| {
                    for (x, pixel) in row.chunks_exact_mut(3).enumerate() {
                        let ray = camera.ray_for_sample(x as u32, y as u32, [0.5, 0.5]);
                        pixel.copy_from_slice(&self.shader.shade_ray(scene, &ray, 0, settings));
                    }
                });
            return &self.frame;
        }

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
                            let ray = camera.ray_for_sample(x as u32, y as u32, offset);
                            let sample = self.shader.shade_ray(scene, &ray, 0, settings);
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
}
