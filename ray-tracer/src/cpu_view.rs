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

    // Input state
    keys_pressed: [bool; 256], // indexed by winit::KeyCode as usize
    yaw: f32,
    pitch: f32,
    position: [f32; 3],
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
            keys_pressed: [false; 256],
            yaw: 0.0,
            pitch: 0.0,
            position: [0.0, 0.0, 0.0],
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

    pub fn render_frame(&mut self, dt: f32) {
        self.update_camera_from_input(dt);
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

    pub fn handle_key(&mut self, key_code: winit::keyboard::KeyCode, pressed: bool) {
        if let Some(idx) = Self::key_code_to_index(key_code) {
            self.keys_pressed[idx] = pressed;
        }
    }

    pub fn handle_mouse_delta(&mut self, delta_x: f32, delta_y: f32, sensitivity: f32) {
        self.yaw -= delta_x * sensitivity;
        self.pitch -= delta_y * sensitivity;
    }

    fn key_code_to_index(key_code: winit::keyboard::KeyCode) -> Option<usize> {
        Some(key_code as usize)
    }

    fn update_camera_from_input(&mut self, dt: f32) {
        use winit::keyboard::KeyCode;

        let move_speed = 3.0; // units per second
        let mut move_dir = [0.0f32; 3];

        // Forward/back
        if self.keys_pressed[KeyCode::KeyW as usize] {
            move_dir[0] += self.camera.forward[0];
            move_dir[1] += self.camera.forward[1];
            move_dir[2] += self.camera.forward[2];
        }
        if self.keys_pressed[KeyCode::KeyS as usize] {
            move_dir[0] -= self.camera.forward[0];
            move_dir[1] -= self.camera.forward[1];
            move_dir[2] -= self.camera.forward[2];
        }
        // Left/right
        if self.keys_pressed[KeyCode::KeyA as usize] {
            move_dir[0] -= self.camera.right[0];
            move_dir[1] -= self.camera.right[1];
            move_dir[2] -= self.camera.right[2];
        }
        if self.keys_pressed[KeyCode::KeyD as usize] {
            move_dir[0] += self.camera.right[0];
            move_dir[1] += self.camera.right[1];
            move_dir[2] += self.camera.right[2];
        }
        // Up/down
        if self.keys_pressed[KeyCode::ShiftLeft as usize]
            || self.keys_pressed[KeyCode::ShiftRight as usize]
        {
            move_dir[1] -= 1.0;
        }
        if self.keys_pressed[KeyCode::Space as usize] {
            move_dir[1] += 1.0;
        }

        // Normalize horizontal movement
        let hx = move_dir[0];
        let hy = 0.0;
        let hz = move_dir[2];
        let hlen = f32::sqrt(hx * hx + hy * hy + hz * hz);
        let mut move_dir = move_dir;
        if hlen > 0.0 {
            move_dir[0] /= hlen;
            move_dir[2] /= hlen;
        }

        self.position[0] += move_dir[0] * move_speed * dt;
        self.position[1] += move_dir[1] * move_speed * dt;
        self.position[2] += move_dir[2] * move_speed * dt;

        self.camera.set_position(self.position);
        self.camera.set_orientation(self.yaw, self.pitch);
    }
}
