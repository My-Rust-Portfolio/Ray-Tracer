use crate::math::Ray;

pub struct Camera {
    pub origin: [f32; 3],
    pub forward: [f32; 3], // normalized
    pub right: [f32; 3],   // normalized
    pub up: [f32; 3],      // normalized
    width: u32,
    height: u32,
    pub yaw: f32,   // radians, around Y axis
    pub pitch: f32, // radians, up/down
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        let yaw = 0.0;
        let pitch = 0.0;
        let forward = [0.0, 0.0, -1.0];
        let right = [1.0, 0.0, 0.0];
        let up = [0.0, 1.0, 0.0];

        Self {
            origin: [0.0, 0.0, 0.0],
            forward,
            right,
            up,
            width,
            height,
            yaw,
            pitch,
        }
    }

    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.origin = pos;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
    }

    pub fn set_orientation(&mut self, yaw: f32, pitch: f32) {
        use std::f32::consts::PI;

        self.yaw = yaw;
        self.pitch = pitch.clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01);

        let sy = self.yaw.sin();
        let cy = self.yaw.cos();
        let sp = self.pitch.sin();
        let cp = self.pitch.cos();

        // Forward vector from yaw/pitch
        let mut forward = [cp * sy, sp, cp * cy];
        let len =
            f32::sqrt(forward[0] * forward[0] + forward[1] * forward[1] + forward[2] * forward[2]);
        forward[0] /= len;
        forward[1] /= len;
        forward[2] /= len;

        // Right = normalize(cross(forward, world_up))
        let world_up = [0.0, 1.0, 0.0];
        let mut right = [
            forward[1] * world_up[2] - forward[2] * world_up[1],
            forward[2] * world_up[0] - forward[0] * world_up[2],
            forward[0] * world_up[1] - forward[1] * world_up[0],
        ];
        let len = f32::sqrt(right[0] * right[0] + right[1] * right[1] + right[2] * right[2]);
        right[0] /= len;
        right[1] /= len;
        right[2] /= len;

        // Up = cross(right, forward)
        let up = [
            right[1] * forward[2] - right[2] * forward[1],
            right[2] * forward[0] - right[0] * forward[2],
            right[0] * forward[1] - right[1] * forward[0],
        ];

        self.forward = forward;
        self.right = right;
        self.up = up;
    }

    /// Creates a ray through a subpixel location, where each offset is in [0, 1].
    pub fn ray_for_sample(&self, x: u32, y: u32, sample_offset: [f32; 2]) -> Ray {
        let viewport_x = 2.0 * (x as f32 + sample_offset[0]) / self.width as f32 - 1.0;
        let viewport_y = 1.0 - 2.0 * (y as f32 + sample_offset[1]) / self.height as f32;

        // Build direction in camera space, then transform to world space.
        let mut dir = [
            viewport_x * self.right[0] + viewport_y * self.up[0] + self.forward[0],
            viewport_x * self.right[1] + viewport_y * self.up[1] + self.forward[1],
            viewport_x * self.right[2] + viewport_y * self.up[2] + self.forward[2],
        ];

        let len = f32::sqrt(dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]);
        dir[0] /= len;
        dir[1] /= len;
        dir[2] /= len;

        Ray {
            origin: self.origin,
            direction: dir,
        }
    }
}
