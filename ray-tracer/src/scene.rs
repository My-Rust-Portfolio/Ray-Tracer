use crate::math::Ray;
use crate::objects::Sphere;

pub struct Scene {
    pub sphere: Sphere,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            sphere: Sphere {
                center: [0.0, 0.0, -3.0],
                radius: 1.0,
            },
        }
    }

    pub fn closest_hit(&self, ray: &Ray) -> Option<f32> {
        self.sphere.intersect(ray.origin, ray.direction)
    }
}
