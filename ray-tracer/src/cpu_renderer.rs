use crate::math::Ray;
use crate::scene::{Hit, HitKind, Scene};
use std::path::Path;

pub struct CpuRenderer {
    pub light_dir: [f32; 3],
    sky_image: image::RgbImage,
    sky_width: u32,
    sky_height: u32,
}

impl CpuRenderer {
    pub fn new() -> Self {
        let sky_path = Path::new("assets/sky.png");
        let sky_img = image::open(sky_path)
            .expect("Failed to load sky image")
            .to_rgb8();

        let (sky_width, sky_height) = sky_img.dimensions();

        let dir = [0.5, 0.9, 0.3];
        let len = f32::sqrt(dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]);

        Self {
            light_dir: [dir[0] / len, dir[1] / len, dir[2] / len],
            sky_image: sky_img,
            sky_width,
            sky_height,
        }
    }

    pub fn shade_ray(&self, scene: &Scene, ray: &Ray, depth: u32) -> [u8; 3] {
        if depth > 3 {
            // Fallback: just sample sky if we hit max depth
            let (u, v) = self.dir_to_uv(ray.direction);
            return self.sample_sky(u, v);
        }

        if let Some(hit) = scene.closest_hit(ray) {
            match hit.kind {
                HitKind::Sphere(_) => self.shade_sphere(&hit, ray, depth),
                HitKind::Plane(_) => self.shade_plane(&hit, ray, depth),
            }
        } else {
            // Background: skybox
            let (u, v) = self.dir_to_uv(ray.direction);
            self.sample_sky(u, v)
        }
    }

    fn shade_sphere(&self, hit: &Hit, ray: &Ray, depth: u32) -> [u8; 3] {
        let n = hit.normal;
        let v = [-ray.direction[0], -ray.direction[1], -ray.direction[2]];

        // White sun light
        let light_dir = self.light_dir; // already normalized
        let light_colour = [1.0, 1.0, 1.0]; // white

        // Simple Lambert + specular for the sun
        let ndotl = (n[0] * light_dir[0] + n[1] * light_dir[1] + n[2] * light_dir[2]).max(0.0);

        // Reflection vector for specular
        let r = [
            2.0 * ndotl * n[0] - light_dir[0],
            2.0 * ndotl * n[1] - light_dir[1],
            2.0 * ndotl * n[2] - light_dir[2],
        ];
        let rdotv = (r[0] * v[0] + r[1] * v[1] + r[2] * v[2]).max(0.0);
        let shininess = 64.0;
        let spec = rdotv.powf(shininess);

        // Very low diffuse, strong specular, plus environment reflection
        let kd = 0.05; // almost no diffuse
        let ks = 0.6; // strong white specular highlight

        let mut colour = [
            kd * light_colour[0] * ndotl + ks * spec * light_colour[0],
            kd * light_colour[1] * ndotl + ks * spec * light_colour[1],
            kd * light_colour[2] * ndotl + ks * spec * light_colour[2],
        ];

        // Add environment reflection (skybox) for mirror-like look
        let reflect_dir = self.reflect(ray.direction, n);
        let env = self.sample_sky_dir(reflect_dir);

        let reflect_weight = 0.7; // how mirror-like the sphere is
        colour[0] = colour[0] * (1.0 - reflect_weight) + (env[0] as f32 / 255.0) * reflect_weight;
        colour[1] = colour[1] * (1.0 - reflect_weight) + (env[1] as f32 / 255.0) * reflect_weight;
        colour[2] = colour[2] * (1.0 - reflect_weight) + (env[2] as f32 / 255.0) * reflect_weight;

        let r = colour[0].clamp(0.0, 1.0);
        let g = colour[1].clamp(0.0, 1.0);
        let b = colour[2].clamp(0.0, 1.0);

        [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
    }

    fn shade_plane(&self, hit: &Hit, ray: &Ray, depth: u32) -> [u8; 3] {
        let normal = hit.normal;
        let base_colour = [0.7, 0.7, 0.7];
        let ambient = 0.1;

        let mut nl = normal[0] * self.light_dir[0]
            + normal[1] * self.light_dir[1]
            + normal[2] * self.light_dir[2];
        if nl < 0.0 {
            nl = 0.0;
        }

        let intensity = ambient + 0.9 * nl;
        [
            (base_colour[0] * intensity * 255.0) as u8,
            (base_colour[1] * intensity * 255.0) as u8,
            (base_colour[2] * intensity * 255.0) as u8,
        ]
    }

    fn reflect(&self, dir: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
        let dot = dir[0] * normal[0] + dir[1] * normal[1] + dir[2] * normal[2];
        [
            dir[0] - 2.0 * dot * normal[0],
            dir[1] - 2.0 * dot * normal[1],
            dir[2] - 2.0 * dot * normal[2],
        ]
    }

    fn dir_to_uv(&self, dir: [f32; 3]) -> (f32, f32) {
        // Spherical mapping for equirectangular panorama
        let x = dir[0];
        let y = dir[1];
        let z = dir[2];

        let theta = y.atan2((x * x + z * z).sqrt()); // elevation [-pi/2, pi/2]
        let phi = x.atan2(z); // azimuth [-pi, pi]

        let u = 0.5 + phi / (2.0 * std::f32::consts::PI);
        let v = 0.5 - theta / std::f32::consts::PI;

        (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
    }

    fn sample_sky(&self, u: f32, v: f32) -> [u8; 3] {
        let uf = u * (self.sky_width as f32 - 1.0);
        let vf = v * (self.sky_height as f32 - 1.0);

        let x0 = uf as u32;
        let y0 = vf as u32;
        let x1 = (x0 + 1).min(self.sky_width - 1);
        let y1 = (y0 + 1).min(self.sky_height - 1);

        let fx = uf - x0 as f32;
        let fy = vf - y0 as f32;

        let c00 = self.sky_image.get_pixel(x0, y0);
        let c01 = self.sky_image.get_pixel(x0, y1);
        let c10 = self.sky_image.get_pixel(x1, y0);
        let c11 = self.sky_image.get_pixel(x1, y1);

        let r = self.bilerp(c00[0], c10[0], c01[0], c11[0], fx, fy) as u8;
        let g = self.bilerp(c00[1], c10[1], c01[1], c11[1], fx, fy) as u8;
        let b = self.bilerp(c00[2], c10[2], c01[2], c11[2], fx, fy) as u8;

        [r, g, b]
    }

    fn bilerp(&self, c00: u8, c10: u8, c01: u8, c11: u8, fx: f32, fy: f32) -> f32 {
        let c0 = c00 as f32 * (1.0 - fx) + c10 as f32 * fx;
        let c1 = c01 as f32 * (1.0 - fx) + c11 as f32 * fx;
        c0 * (1.0 - fy) + c1 * fy
    }

    fn sample_sky_dir(&self, dir: [f32; 3]) -> [u8; 3] {
        let (u, v) = self.dir_to_uv(dir);
        self.sample_sky(u, v)
    }
}
