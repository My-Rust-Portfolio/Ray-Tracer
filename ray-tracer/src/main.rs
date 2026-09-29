use image::{Rgb, RgbImage};

struct Ray {
    origin: [f32; 3],
    direction: [f32; 3],
}

fn main() -> image::ImageResult<()> {
    let width = 256;
    let height = 256;
    let mut image = RgbImage::new(width, height);

    let camera_origin = [0.0, 0.0, 0.0];

    for y in 0..height {
        for x in 0..width {
            // Map pixel centres to a viewport from -1 to +1.
            let viewport_x = 2.0 * (x as f32 + 0.5) / width as f32 - 1.0;
            let viewport_y = 1.0 - 2.0 * (y as f32 + 0.5) / height as f32;

            let ray = Ray {
                origin: camera_origin,
                direction: [viewport_x, viewport_y, -1.0],
            };

            let _ = ray.origin;

            let red = ((ray.direction[0] + 1.0) * 127.5) as u8;
            let green = ((ray.direction[1] + 1.0) * 127.5) as u8;

            image.put_pixel(x, y, Rgb([red, green, 64]));
        }
    }

    image.save("output.png")?;
    Ok(())
}
