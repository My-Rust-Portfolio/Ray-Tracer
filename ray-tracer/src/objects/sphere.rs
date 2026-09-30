pub struct Sphere {
    pub center: [f32; 3],
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: [f32; 3], radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn intersect(&self, ray_origin: [f32; 3], ray_dir: [f32; 3]) -> Option<f32> {
        let oc = [
            ray_origin[0] - self.center[0],
            ray_origin[1] - self.center[1],
            ray_origin[2] - self.center[2],
        ];

        let a = ray_dir[0] * ray_dir[0] + ray_dir[1] * ray_dir[1] + ray_dir[2] * ray_dir[2];
        let b = 2.0 * (oc[0] * ray_dir[0] + oc[1] * ray_dir[1] + oc[2] * ray_dir[2]);
        let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let t1 = (-b - sqrt_d) / (2.0 * a);
        let t2 = (-b + sqrt_d) / (2.0 * a);

        let t = if t1 > 0.0 {
            t1
        } else if t2 > 0.0 {
            t2
        } else {
            return None;
        };

        Some(t)
    }

    pub fn normal_at(&self, point: [f32; 3]) -> [f32; 3] {
        let nx = point[0] - self.center[0];
        let ny = point[1] - self.center[1];
        let nz = point[2] - self.center[2];

        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        [nx / len, ny / len, nz / len]
    }
}
