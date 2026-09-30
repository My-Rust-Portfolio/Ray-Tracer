use winit::keyboard::KeyCode;

use crate::camera::Camera;
use crate::controller::CameraController;
use crate::scene::Scene;

/// Shared render inputs and camera state consumed by all rendering backends.
pub struct RenderWorld {
    camera: Camera,
    scene: Scene,
    controller: CameraController,
    random_state: u64,
    spawned_spheres: usize,
}

impl RenderWorld {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(1, 1),
            scene: Scene::new(),
            controller: CameraController::default(),
            random_state: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(1, |duration| duration.as_nanos() as u64 | 1),
            spawned_spheres: 0,
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

    pub fn spawn_random_sphere_nearby(&mut self) {
        use crate::{material::Material, objects::Sphere};

        let radius = 0.25 + self.random_unit() * 0.35;
        let forward_distance = 2.0 + self.random_unit() * 2.0;
        let side_offset = (self.random_unit() * 2.0 - 1.0) * 1.8;
        let vertical_offset = (self.random_unit() * 2.0 - 1.0) * 0.25;
        let camera = &self.camera;
        let mut center = [
            camera.origin[0] + camera.forward[0] * forward_distance + camera.right[0] * side_offset,
            camera.origin[1] + camera.forward[1] * forward_distance + camera.right[1] * side_offset,
            camera.origin[2] + camera.forward[2] * forward_distance + camera.right[2] * side_offset,
        ];
        center[1] += vertical_offset;
        center[1] = center[1].max(self.scene.plane.point[1] + radius + 0.02);

        let mut material = Material::sphere_default();
        material.base_color = [self.random_unit(), self.random_unit(), self.random_unit()];
        self.scene
            .add_sphere(Sphere::with_material(center, radius, material));
        self.spawned_spheres += 1;
    }

    pub fn delete_last_spawned_sphere(&mut self) -> bool {
        if self.spawned_spheres == 0 {
            return false;
        }
        self.scene.spheres.pop();
        self.spawned_spheres -= 1;
        true
    }

    pub fn spawned_sphere_count(&self) -> usize {
        self.spawned_spheres
    }

    fn random_unit(&mut self) -> f32 {
        // xorshift64*: compact per-world PRNG for interactive scene spawning.
        let mut value = self.random_state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.random_state = value;
        let bits = value.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40;
        bits as f32 / ((1u32 << 24) as f32)
    }
}
