mod camera;
mod math;
mod objects;
mod renderer;
mod scene;

use camera::Camera;
use image::{Rgb, RgbImage};
use rayon::prelude::*;
use renderer::CpuRenderer;
use scene::Scene;

fn main() -> image::ImageResult<()> {
    let width = 256;
    let height = 256;
    let camera = Camera::new(width, height);
    let scene = Scene::new();
    let renderer = CpuRenderer::new();

    // Pre-allocate a flat buffer of pixels.
    let mut pixels: Vec<Rgb<u8>> = vec![Rgb([0, 0, 0]); (width * height) as usize];

    // Process each row in parallel.
    (0..height).into_par_iter().for_each(|y| {
        let row_start = (y * width) as usize;
        let row_end = row_start + width as usize;
        let row_pixels = &mut pixels[row_start..row_end];

        for (x, pixel) in row_pixels.iter_mut().enumerate() {
            let x = x as u32;
            let ray = camera.ray_for_pixel(x, y);
            let [r, g, b] = renderer.colour_for_ray(&scene, &ray);
            *pixel = Rgb([r, g, b]);
        }
    });

    // Convert the flat buffer into an ImageBuffer.
    let image = RgbImage::from_raw(width, height, pixels).expect("pixel count mismatch");

    image.save("output.png")?;
    Ok(())
}
