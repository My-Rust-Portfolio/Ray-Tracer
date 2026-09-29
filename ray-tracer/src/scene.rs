use crate::math::Ray;
use crate::objects::{Plane, Sphere};

pub enum HitKind<'a> {
    Sphere(&'a Sphere),
    Plane(&'a Plane),
}

pub struct Hit<'a> {
    pub t: f32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub kind: HitKind<'a>,
}

pub struct Scene {
    pub sphere: Sphere,
    pub plane: Plane,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            sphere: Sphere {
                center: [0.0, -0.5, -3.0],
                radius: 1.0,
            },
            plane: Plane {
                point: [0.0, -1.5, 0.0],
                normal: [0.0, 1.0, 0.0],
            },
        }
    }

    pub fn closest_hit(&self, ray: &Ray) -> Option<Hit<'_>> {
        let mut closest_t = f32::INFINITY;
        let mut closest_hit = None;

        // Sphere
        if let Some(t) = self.sphere.intersect(ray.origin, ray.direction) {
            if t < closest_t {
                closest_t = t;
                let point = [
                    ray.origin[0] + t * ray.direction[0],
                    ray.origin[1] + t * ray.direction[1],
                    ray.origin[2] + t * ray.direction[2],
                ];
                let normal = self.sphere.normal_at(point);
                closest_hit = Some(Hit {
                    t,
                    point,
                    normal,
                    kind: HitKind::Sphere(&self.sphere),
                });
            }
        }

        // Plane
        if let Some(t) = self.plane.intersect(ray.origin, ray.direction) {
            if t < closest_t {
                closest_t = t;
                let point = [
                    ray.origin[0] + t * ray.direction[0],
                    ray.origin[1] + t * ray.direction[1],
                    ray.origin[2] + t * ray.direction[2],
                ];
                let normal = self.plane.normal_at(point);
                closest_hit = Some(Hit {
                    t,
                    point,
                    normal,
                    kind: HitKind::Plane(&self.plane),
                });
            }
        }

        closest_hit
    }
}
