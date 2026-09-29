use std::sync::Arc;
use winit::window::Window;

use crate::camera::Camera;
use crate::renderer::CpuRenderer;
use crate::scene::Scene;
use rayon::prelude::*;
use std::num::NonZeroU32;

pub struct CpuView {
    window: Arc<Window>,
    context: softbuffer::Context<Arc<Window>>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    width: u32,
    height: u32,
    camera: Camera,
    scene: Scene,
    renderer: CpuRenderer,
}

impl CpuView {
    pub fn new(window: Arc<Window>, width: u32, height: u32) -> Self {
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        let camera = Camera::new(width, height);
        let scene = Scene::new();
        let renderer = CpuRenderer::new();

        Self {
            window,
            context,
            surface,
            width,
            height,
            camera,
            scene,
            renderer,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.width = width;
        self.height = height;

        self.surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .unwrap();

        self.camera = Camera::new(width, height);
    }

    pub fn render_frame(&mut self) {
        let width = self.width;
        let height = self.height;

        // Render CPU frame into a flat u32 buffer (ABGR for softbuffer).
        let mut buffer: Vec<u32> = vec![0u32; (width * height) as usize];

        // Parallel over rows.
        buffer
            .par_chunks_mut(width as usize)
            .enumerate()
            .for_each(|(y, row)| {
                let y = y as u32;
                for (x, pixel) in row.iter_mut().enumerate() {
                    let x = x as u32;
                    let ray = self.camera.ray_for_pixel(x, y);
                    let [r, g, b] = self.renderer.colour_for_ray(&self.scene, &ray);
                    // softbuffer expects 0xBBGGRR00 layout (little-endian u32).
                    *pixel = (b as u32) | ((g as u32) << 8) | ((r as u32) << 16);
                }
            });

        // Present to window.
        let mut surface_buffer = self.surface.buffer_mut().unwrap();
        surface_buffer.copy_from_slice(&buffer);
        surface_buffer.present().unwrap();
    }
}
