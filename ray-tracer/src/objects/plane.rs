use crate::material::Material;

pub struct Plane {
    pub point: [f32; 3],  // a point on the plane
    pub normal: [f32; 3], // normalized
    pub material: Material,
}

impl Plane {
    pub fn intersect(&self, ray_origin: [f32; 3], ray_dir: [f32; 3]) -> Option<f32> {
        // Plane equation: (p - point) · normal = 0
        // Ray: p = origin + t * dir
        // Solve for t: t = (point - origin) · normal / (dir · normal)
        let denom =
            ray_dir[0] * self.normal[0] + ray_dir[1] * self.normal[1] + ray_dir[2] * self.normal[2];

        if denom.abs() < 1e-6 {
            return None; // Ray parallel to plane
        }

        let t = ((self.point[0] - ray_origin[0]) * self.normal[0]
            + (self.point[1] - ray_origin[1]) * self.normal[1]
            + (self.point[2] - ray_origin[2]) * self.normal[2])
            / denom;

        if t > 0.0 { Some(t) } else { None }
    }

    pub fn normal_at(&self, _point: [f32; 3]) -> [f32; 3] {
        self.normal
    }
}
