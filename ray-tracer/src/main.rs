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
    let mut pixels: Vec<u8> = vec![0u8; (width * height * 3) as usize];
    let bytes_per_row = (width * 3) as usize;

    // Parallel iterator over mutable row slices.
    pixels
        .par_chunks_mut(bytes_per_row)
        .enumerate()
        .for_each(|(y, row_bytes)| {
            let y = y as u32;
            for x in 0..width {
                let ray = camera.ray_for_pixel(x, y);
                let [r, g, b] = renderer.colour_for_ray(&scene, &ray);

                let i = (x as usize) * 3;
                row_bytes[i] = r;
                row_bytes[i + 1] = g;
                row_bytes[i + 2] = b;
            }
        });

    // Convert the flat buffer into an ImageBuffer.
    let image = RgbImage::from_raw(width, height, pixels).expect("pixel count mismatch");

    image.save("output.png")?;
    Ok(())
}
