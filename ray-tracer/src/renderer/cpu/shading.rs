use crate::math::Ray;
use crate::renderer::RenderSettings;
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
        let sky_img = image::load_from_memory(include_bytes!("../../../assets/sky.png"))
            .expect("Failed to decode bundled sky image")
            .to_rgb8();

        let (sky_width, sky_height) = sky_img.dimensions();

        let ground_img = image::load_from_memory(include_bytes!("../../../assets/ground.jpg"))
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

    pub fn shade_ray(
        &self,
        scene: &Scene,
        ray: &Ray,
        depth: u32,
        settings: RenderSettings,
    ) -> [u8; 3] {
        if depth >= 3 {
            return self.sample_sky_dir(ray.direction);
        }

        if let Some(hit) = scene.closest_hit(ray) {
            match hit.kind {
                HitKind::Sphere => self.shade_sphere(scene, &hit, ray, depth, settings),
                HitKind::Plane => self.shade_plane(scene, &hit, ray, depth, settings),
            }
        } else {
            self.sample_sky_dir(ray.direction)
        }
    }

    fn shade_sphere(
        &self,
        scene: &Scene,
        hit: &Hit,
        ray: &Ray,
        depth: u32,
        settings: RenderSettings,
    ) -> [u8; 3] {
        let n = hit.normal;
        let material = hit.material;
        let v = [-ray.direction[0], -ray.direction[1], -ray.direction[2]];

        let in_shadow = settings.shadows_enabled && self.is_in_shadow(scene, hit);

        let ndotl =
            (n[0] * self.light_dir[0] + n[1] * self.light_dir[1] + n[2] * self.light_dir[2])
                .max(0.0);

        let r = [
            2.0 * ndotl * n[0] - self.light_dir[0],
            2.0 * ndotl * n[1] - self.light_dir[1],
            2.0 * ndotl * n[2] - self.light_dir[2],
        ];
        let rdotv = (r[0] * v[0] + r[1] * v[1] + r[2] * v[2]).max(0.0);
        let specular = if in_shadow {
            0.0
        } else {
            material.specular * rdotv.powf(material.shininess)
        };
        let diffuse = if in_shadow {
            0.0
        } else {
            material.diffuse * ndotl
        };
        let ambient = if in_shadow {
            material.shadow_ambient
        } else {
            material.ambient
        };
        let local = material
            .base_color
            .map(|base| (base * (ambient + diffuse) + specular).clamp(0.0, 1.0));

        let reflected = if material.reflectivity > 0.0 {
            let reflected_ray = Ray {
                origin: offset_point(hit.point, n),
                direction: self.reflect(ray.direction, n),
            };
            Some(self.shade_ray(scene, &reflected_ray, depth + 1, settings))
        } else {
            None
        };
        blend_reflection(local, reflected, material.reflectivity)
    }

    fn shade_plane(
        &self,
        scene: &Scene,
        hit: &Hit,
        ray: &Ray,
        depth: u32,
        settings: RenderSettings,
    ) -> [u8; 3] {
        let normal = hit.normal;
        let material = hit.material;

        // Decode the color texture before filtering and applying lighting.
        let (u, v) = self.plane_uv_from_hit(hit);
        let texture_colour = self.sample_ground_linear(u, v);

        let mut nl = normal[0] * self.light_dir[0]
            + normal[1] * self.light_dir[1]
            + normal[2] * self.light_dir[2];
        if nl < 0.0 {
            nl = 0.0;
        }

        let in_shadow = settings.shadows_enabled && self.is_in_shadow(scene, hit);
        let diffuse = if in_shadow {
            0.0
        } else {
            material.diffuse * nl
        };
        let ambient = if in_shadow {
            material.shadow_ambient
        } else {
            material.ambient
        };
        let view = [-ray.direction[0], -ray.direction[1], -ray.direction[2]];
        let light_reflection = [
            2.0 * nl * normal[0] - self.light_dir[0],
            2.0 * nl * normal[1] - self.light_dir[1],
            2.0 * nl * normal[2] - self.light_dir[2],
        ];
        let specular = if in_shadow {
            0.0
        } else {
            (light_reflection[0] * view[0]
                + light_reflection[1] * view[1]
                + light_reflection[2] * view[2])
                .max(0.0)
                .powf(material.shininess)
                * material.specular
        };
        let local = std::array::from_fn(|channel| {
            (texture_colour[channel] * material.base_color[channel] * (ambient + diffuse)
                + specular)
                .clamp(0.0, 1.0)
        });

        let reflected = if material.reflectivity > 0.0 {
            let reflected_ray = Ray {
                origin: offset_point(hit.point, normal),
                direction: self.reflect(ray.direction, normal),
            };
            Some(self.shade_ray(scene, &reflected_ray, depth + 1, settings))
        } else {
            None
        };
        blend_reflection(local, reflected, material.reflectivity)
    }

    fn is_in_shadow(&self, scene: &Scene, hit: &Hit) -> bool {
        const SHADOW_EPSILON: f32 = 0.001;
        let shadow_ray = Ray {
            origin: [
                hit.point[0] + hit.normal[0] * SHADOW_EPSILON,
                hit.point[1] + hit.normal[1] * SHADOW_EPSILON,
                hit.point[2] + hit.normal[2] * SHADOW_EPSILON,
            ],
            direction: self.light_dir,
        };

        scene.occluded(&shadow_ray, f32::INFINITY)
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

        std::array::from_fn(|channel| {
            let c00 = srgb_to_linear(c00[channel] as f32 / 255.0);
            let c01 = srgb_to_linear(c01[channel] as f32 / 255.0);
            let c10 = srgb_to_linear(c10[channel] as f32 / 255.0);
            let c11 = srgb_to_linear(c11[channel] as f32 / 255.0);
            let c0 = c00 * (1.0 - fx) + c10 * fx;
            let c1 = c01 * (1.0 - fx) + c11 * fx;
            (linear_to_srgb(c0 * (1.0 - fy) + c1 * fy) * 255.0 + 0.5) as u8
        })
    }

    fn sample_sky_dir(&self, dir: [f32; 3]) -> [u8; 3] {
        let (u, v) = self.dir_to_uv(dir);
        self.sample_sky(u, v)
    }

    fn sample_ground_linear(&self, u: f32, v: f32) -> [f32; 3] {
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

        std::array::from_fn(|channel| {
            let c00 = srgb_to_linear(c00[channel] as f32 / 255.0);
            let c01 = srgb_to_linear(c01[channel] as f32 / 255.0);
            let c10 = srgb_to_linear(c10[channel] as f32 / 255.0);
            let c11 = srgb_to_linear(c11[channel] as f32 / 255.0);
            let c0 = c00 * (1.0 - fx) + c10 * fx;
            let c1 = c01 * (1.0 - fx) + c11 * fx;
            c0 * (1.0 - fy) + c1 * fy
        })
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

fn offset_point(point: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    const RAY_EPSILON: f32 = 0.001;
    [
        point[0] + normal[0] * RAY_EPSILON,
        point[1] + normal[1] * RAY_EPSILON,
        point[2] + normal[2] * RAY_EPSILON,
    ]
}

fn blend_reflection(local: [f32; 3], reflected: Option<[u8; 3]>, reflectivity: f32) -> [u8; 3] {
    let reflectivity = reflectivity.clamp(0.0, 1.0);
    std::array::from_fn(|channel| {
        let reflected =
            reflected.map_or(0.0, |color| srgb_to_linear(color[channel] as f32 / 255.0));
        let color = local[channel] * (1.0 - reflectivity) + reflected * reflectivity;
        (linear_to_srgb(color.clamp(0.0, 1.0)) * 255.0 + 0.5) as u8
    })
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}
