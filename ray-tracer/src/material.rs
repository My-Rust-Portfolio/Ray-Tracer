/// Surface properties used by CPU and future GPU shading implementations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub base_color: [f32; 3],
    pub ambient: f32,
    pub shadow_ambient: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub shininess: f32,
    /// Fraction of the outgoing surface color supplied by the reflection ray.
    pub reflectivity: f32,
}

impl Material {
    pub const fn sphere_default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0],
            ambient: 0.02,
            shadow_ambient: 0.02,
            diffuse: 0.05,
            specular: 0.6,
            shininess: 64.0,
            reflectivity: 0.7,
        }
    }

    pub const fn ground_default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0],
            ambient: 0.1,
            shadow_ambient: 0.24,
            diffuse: 0.9,
            specular: 0.0,
            shininess: 32.0,
            reflectivity: 0.0,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::sphere_default()
    }
}
