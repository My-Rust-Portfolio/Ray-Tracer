use winit::keyboard::KeyCode;

use crate::camera::Camera;
use crate::controller::CameraController;
use crate::scene::Scene;

/// Shared render inputs and camera state consumed by all rendering backends.
pub struct RenderWorld {
    camera: Camera,
    scene: Scene,
    controller: CameraController,
}

impl RenderWorld {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(1, 1),
            scene: Scene::new(),
            controller: CameraController::default(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    pub fn update(&mut self, dt: f32) {
        self.controller.update(&mut self.camera, dt);
    }

    pub fn set_key(&mut self, key: KeyCode, pressed: bool) {
        self.controller.set_key(key, pressed);
    }

    pub fn mouse_delta(&mut self, dx: f32, dy: f32) {
        self.controller.mouse_delta(dx, dy);
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }
}
