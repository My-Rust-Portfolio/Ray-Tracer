mod camera;
mod math;
mod objects;
mod renderer;
mod scene;

use camera::Camera;
use image::{Rgb, RgbImage};
use renderer::CpuRenderer;
use scene::Scene;

fn main() -> image::ImageResult<()> {
    let width = 256;
    let height = 256;
    let camera = Camera::new(width, height);
    let scene = Scene::new();
    let renderer = CpuRenderer::new();
    let mut image = RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let ray = camera.ray_for_pixel(x, y);
            let [r, g, b] = renderer.colour_for_ray(&scene, &ray);
            image.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    image.save("output.png")?;
    Ok(())
}
