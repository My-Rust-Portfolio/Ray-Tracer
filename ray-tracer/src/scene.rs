use crate::material::Material;
use crate::math::Ray;
use crate::objects::{Plane, Sphere};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitKind {
    Sphere,
    Plane,
}

pub struct Hit {
    pub t: f32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub kind: HitKind,
    pub material: Material,
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub plane: Plane,
}

impl Scene {
    pub fn new() -> Self {
        let mut scene = Self {
            spheres: Vec::new(),
            plane: Plane {
                point: [0.0, -1.5, 0.0],
                normal: [0.0, 1.0, 0.0],
                material: Material::ground_default(),
            },
        };
        scene.add_sphere(Sphere::new([0.0, -0.5, -3.0], 1.0));
        scene
    }

    pub fn add_sphere(&mut self, sphere: Sphere) -> usize {
        let id = self.spheres.len();
        self.spheres.push(sphere);
        id
    }

    pub fn closest_hit(&self, ray: &Ray) -> Option<Hit> {
        let mut closest_hit: Option<Hit> = None;

        for sphere in &self.spheres {
            if let Some(t) = sphere.intersect(ray.origin, ray.direction)
                && closest_hit.as_ref().is_none_or(|hit| t < hit.t)
            {
                let point = point_on_ray(ray, t);
                closest_hit = Some(Hit {
                    t,
                    point,
                    normal: sphere.normal_at(point),
                    kind: HitKind::Sphere,
                    material: sphere.material,
                });
            }
        }

        if let Some(t) = self.plane.intersect(ray.origin, ray.direction)
            && closest_hit.as_ref().is_none_or(|hit| t < hit.t)
        {
            let point = point_on_ray(ray, t);
            closest_hit = Some(Hit {
                t,
                point,
                normal: self.plane.normal_at(point),
                kind: HitKind::Plane,
                material: self.plane.material,
            });
        }

        closest_hit
    }

    /// Returns whether any scene object blocks a ray before `max_distance`.
    pub fn occluded(&self, ray: &Ray, max_distance: f32) -> bool {
        self.closest_hit(ray)
            .is_some_and(|hit| hit.t < max_distance)
    }
}

fn point_on_ray(ray: &Ray, t: f32) -> [f32; 3] {
    [
        ray.origin[0] + t * ray.direction[0],
        ray.origin[1] + t * ray.direction[1],
        ray.origin[2] + t * ray.direction[2],
    ]
}
