use std::sync::Arc;
use winit::window::Window;

use crate::camera::Camera;
use crate::controller::CameraController;
use crate::cpu_renderer::CpuRenderer;
use crate::scene::Scene;
use rayon::prelude::*;
use std::num::NonZeroU32;

pub struct CpuView {
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    width: u32,
    height: u32,
    camera: Camera,
    scene: Scene,
    renderer: CpuRenderer,

    controller: CameraController,
    frame: Vec<u32>,
}

impl CpuView {
    pub fn new(window: Arc<Window>, width: u32, height: u32) -> Self {
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        let camera = Camera::new(width, height);
        let scene = Scene::new();
        let renderer = CpuRenderer::new();

        Self {
            surface,
            width,
            height,
            camera,
            scene,
            renderer,
            controller: CameraController::default(),
            frame: vec![0; width as usize * height as usize],
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.width = width;
        self.height = height;
        self.frame.resize(width as usize * height as usize, 0);

        self.surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .unwrap();

        self.camera = Camera::new(width, height);
    }

    pub fn render_frame(&mut self, dt: f32) {
        self.controller.update(&mut self.camera, dt);
        let width = self.width;

        // Render CPU frame into a flat u32 buffer (ABGR for softbuffer).
        let buffer = &mut self.frame;

        // Parallel over rows.
        buffer
            .par_chunks_mut(width as usize)
            .enumerate()
            .for_each(|(y, row)| {
                let y = y as u32;
                for (x, pixel) in row.iter_mut().enumerate() {
                    let x = x as u32;
                    let ray = self.camera.ray_for_pixel(x, y);
                    let [r, g, b] = self.renderer.shade_ray(&self.scene, &ray, 0);
                    // softbuffer expects 0xBBGGRR00 layout (little-endian u32).
                    *pixel = (b as u32) | ((g as u32) << 8) | ((r as u32) << 16);
                }
            });

        // Present to window.
        let mut surface_buffer = self.surface.buffer_mut().unwrap();
        surface_buffer.copy_from_slice(&buffer);
        surface_buffer.present().unwrap();
    }

    pub fn handle_key(&mut self, key_code: winit::keyboard::KeyCode, pressed: bool) {
        self.controller.set_key(key_code, pressed);
    }

    pub fn handle_mouse_delta(&mut self, delta_x: f32, delta_y: f32) {
        self.controller.mouse_delta(delta_x, delta_y);
    }
}
