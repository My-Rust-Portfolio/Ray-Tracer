pub struct Ray {
    pub direction: [f32; 3],
}

pub struct Camera {
    width: u32,
    height: u32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn ray_for_pixel(&self, x: u32, y: u32) -> Ray {
        let viewport_x = 2.0 * (x as f32 + 0.5) / self.width as f32 - 1.0;
        let viewport_y = 1.0 - 2.0 * (y as f32 + 0.5) / self.height as f32;

        Ray {
            direction: [viewport_x, viewport_y, -1.0],
        }
    }
}
