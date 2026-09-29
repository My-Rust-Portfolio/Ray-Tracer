use crate::math::Ray;
use crate::scene::Scene;

pub struct CpuRenderer {
    pub light_dir: [f32; 3], // normalized
}

impl CpuRenderer {
    pub fn new() -> Self {
        let dir = [0.5, 0.8, 0.3];
        let len = f32::sqrt(dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]);
        Self {
            light_dir: [dir[0] / len, dir[1] / len, dir[2] / len],
        }
    }

    pub fn colour_for_ray(&self, scene: &Scene, ray: &Ray) -> [u8; 3] {
        if let Some(t) = scene.closest_hit(ray) {
            let hit_point = [
                ray.origin[0] + t * ray.direction[0],
                ray.origin[1] + t * ray.direction[1],
                ray.origin[2] + t * ray.direction[2],
            ];

            let normal = scene.sphere.normal_at(hit_point);

            // Lambertian term: max(0, N · L)
            let mut nl = normal[0] * self.light_dir[0]
                + normal[1] * self.light_dir[1]
                + normal[2] * self.light_dir[2];
            if nl < 0.0 {
                nl = 0.0;
            }

            // Base sphere colour (reddish) scaled by lighting
            let base = [0.9, 0.3, 0.3];
            let ambient = 0.1;
            let intensity = ambient + (1.0 - ambient) * nl;

            let r = (base[0] * intensity * 255.0) as u8;
            let g = (base[1] * intensity * 255.0) as u8;
            let b = (base[2] * intensity * 255.0) as u8;

            [r, g, b]
        } else {
            // Background gradient
            let red = ((ray.direction[0] + 1.0) * 127.5) as u8;
            let green = ((ray.direction[1] + 1.0) * 127.5) as u8;
            [red, green, 64]
        }
    }
}
