use crate::math::Ray;
use crate::scene::{Hit, HitKind, Scene};

pub struct CpuRenderer {
    pub light_dir: [f32; 3],
    sky_image: image::RgbImage,
    sky_width: u32,
    sky_height: u32,

    ground_image: image::RgbImage,
    ground_width: u32,
    ground_height: u32,
}
impl CpuRenderer {
    pub fn new() -> Self {
        let sky_img = image::load_from_memory(include_bytes!("../assets/sky.png"))
            .expect("Failed to decode bundled sky image")
            .to_rgb8();

        let (sky_width, sky_height) = sky_img.dimensions();

        let ground_img = image::load_from_memory(include_bytes!("../assets/ground.jpg"))
            .expect("Failed to decode bundled ground image")
            .to_rgb8();
        let (ground_width, ground_height) = ground_img.dimensions();

        let dir = [0.5, 0.9, 0.3];
        let len = f32::sqrt(dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]);

        Self {
            light_dir: [dir[0] / len, dir[1] / len, dir[2] / len],
            sky_image: sky_img,
            sky_width,
            sky_height,

            ground_image: ground_img,
            ground_width,
            ground_height,
        }
    }

    pub fn shade_ray(&self, scene: &Scene, ray: &Ray, depth: u32) -> [u8; 3] {
        if depth >= 3 {
            return self.sample_sky_dir(ray.direction);
        }

        if let Some(hit) = scene.closest_hit(ray) {
            match hit.kind {
                HitKind::Sphere(_) => self.shade_sphere(scene, &hit, ray, depth),
                HitKind::Plane(_) => self.shade_plane(&hit),
            }
        } else {
            self.sample_sky_dir(ray.direction)
        }
    }

    fn shade_sphere(&self, scene: &Scene, hit: &Hit, ray: &Ray, depth: u32) -> [u8; 3] {
        let n = hit.normal;
        let v = [-ray.direction[0], -ray.direction[1], -ray.direction[2]];

        let ndotl =
            (n[0] * self.light_dir[0] + n[1] * self.light_dir[1] + n[2] * self.light_dir[2])
                .max(0.0);

        let r = [
            2.0 * ndotl * n[0] - self.light_dir[0],
            2.0 * ndotl * n[1] - self.light_dir[1],
            2.0 * ndotl * n[2] - self.light_dir[2],
        ];
        let rdotv = (r[0] * v[0] + r[1] * v[1] + r[2] * v[2]).max(0.0);
        let spec = rdotv.powf(64.0);

        let local = 0.05 * ndotl + 0.6 * spec;

        // Reflect the incoming ray direction, not the direction toward the camera.
        let reflect_dir = self.reflect(ray.direction, n);

        // Move the new ray just outside the sphere to avoid hitting itself
        // immediately due to floating-point rounding.
        let epsilon = 0.001;
        let reflected_ray = Ray {
            origin: [
                hit.point[0] + n[0] * epsilon,
                hit.point[1] + n[1] * epsilon,
                hit.point[2] + n[2] * epsilon,
            ],
            direction: reflect_dir,
        };

        // Unlike sample_sky_dir, this tests the plane (and other scene objects)
        // before falling back to the sky.
        let reflected = self.shade_ray(scene, &reflected_ray, depth + 1);
        let reflect_weight = 0.7;

        let channel = |value: u8| -> u8 {
            let colour = local * (1.0 - reflect_weight) + (value as f32 / 255.0) * reflect_weight;
            (colour.clamp(0.0, 1.0) * 255.0) as u8
        };

        [
            channel(reflected[0]),
            channel(reflected[1]),
            channel(reflected[2]),
        ]
    }

    fn shade_plane(&self, hit: &Hit) -> [u8; 3] {
        let normal = hit.normal;

        // Get brick colour at this hit
        let (u, v) = self.plane_uv_from_hit(hit);
        let brick_col = self.sample_ground(u, v);

        let base_colour = [
            brick_col[0] as f32 / 255.0,
            brick_col[1] as f32 / 255.0,
            brick_col[2] as f32 / 255.0,
        ];

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

    fn sample_ground(&self, u: f32, v: f32) -> [u8; 3] {
        // Wrap UVs to make the texture tile
        let u = (u - u.floor()).clamp(0.0, 1.0);
        let v = (v - v.floor()).clamp(0.0, 1.0);

        let uf = u * (self.ground_width as f32 - 1.0);
        let vf = v * (self.ground_height as f32 - 1.0);

        let x0 = uf as u32;
        let y0 = vf as u32;
        let x1 = (x0 + 1).min(self.ground_width - 1);
        let y1 = (y0 + 1).min(self.ground_height - 1);

        let fx = uf - x0 as f32;
        let fy = vf - y0 as f32;

        let c00 = self.ground_image.get_pixel(x0, y0);
        let c01 = self.ground_image.get_pixel(x0, y1);
        let c10 = self.ground_image.get_pixel(x1, y0);
        let c11 = self.ground_image.get_pixel(x1, y1);

        let r = self.bilerp(c00[0], c10[0], c01[0], c11[0], fx, fy) as u8;
        let g = self.bilerp(c00[1], c10[1], c01[1], c11[1], fx, fy) as u8;
        let b = self.bilerp(c00[2], c10[2], c01[2], c11[2], fx, fy) as u8;

        [r, g, b]
    }

    fn plane_uv_from_hit(&self, hit: &Hit) -> (f32, f32) {
        // Assume plane is roughly Y-up; use x and z as texture coordinates
        let p = hit.point; // [f32; 3], the world-space hit point on the plane

        let scale = 0.2; // adjust to control how big bricks appear
        let u = p[0] * scale;
        let v = p[2] * scale;

        (u, v)
    }
}
