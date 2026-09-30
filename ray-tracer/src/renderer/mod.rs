pub mod cpu;
pub mod gpu_viewport;
pub mod world;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Cpu,
    Gpu,
}

/// Backend-independent controls consumed by every renderer implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderSettings {
    /// Number of stratified samples per pixel axis, clamped to 1..=4.
    pub samples_per_axis: u32,
    pub shadows_enabled: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            samples_per_axis: 1,
            shadows_enabled: true,
        }
    }
}
