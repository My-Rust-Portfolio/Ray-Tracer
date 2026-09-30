use std::collections::HashSet;

use winit::keyboard::KeyCode;

use crate::camera::Camera;

/// Tracks user input and applies it to the camera independently of rendering.
pub struct CameraController {
    pressed_keys: HashSet<KeyCode>,
    position: [f32; 3],
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    mouse_sensitivity: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            position: [0.0; 3],
            yaw: 0.0,
            pitch: 0.0,
            move_speed: 3.0,
            mouse_sensitivity: 0.003,
        }
    }
}

impl CameraController {
    pub fn set_key(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            self.pressed_keys.insert(key);
        } else {
            self.pressed_keys.remove(&key);
        }
    }

    pub fn mouse_delta(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * self.mouse_sensitivity;
        self.pitch -= dy * self.mouse_sensitivity;
    }

    pub fn update(&mut self, camera: &mut Camera, dt: f32) {
        let key_down = |key| self.pressed_keys.contains(&key);
        let mut movement = [0.0; 3];
        for (key, amount) in [(KeyCode::KeyW, 1.0), (KeyCode::KeyS, -1.0)] {
            if key_down(key) {
                for (axis, forward) in movement.iter_mut().zip(camera.forward) {
                    *axis += forward * amount;
                }
            }
        }
        for (key, amount) in [(KeyCode::KeyD, 1.0), (KeyCode::KeyA, -1.0)] {
            if key_down(key) {
                for (axis, right) in movement.iter_mut().zip(camera.right) {
                    *axis += right * amount;
                }
            }
        }
        movement[1] += if key_down(KeyCode::Space) { 1.0 } else { 0.0 };
        movement[1] -= if key_down(KeyCode::ControlLeft) || key_down(KeyCode::ControlRight) {
            1.0
        } else {
            0.0
        };

        let horizontal_length = (movement[0] * movement[0] + movement[2] * movement[2]).sqrt();
        if horizontal_length > 0.0 {
            movement[0] /= horizontal_length;
            movement[2] /= horizontal_length;
        }
        for (position, axis) in self.position.iter_mut().zip(movement) {
            *position += axis * self.move_speed * dt;
        }
        camera.set_position(self.position);
        camera.set_orientation(self.yaw, self.pitch);
    }
}
