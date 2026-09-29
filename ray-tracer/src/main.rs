mod camera;

use camera::Camera;
use image::{Rgb, RgbImage};

fn main() -> image::ImageResult<()> {
    let width = 256;
    let height = 256;
    let camera = Camera::new(width, height);
    let mut image = RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let ray = camera.ray_for_pixel(x, y);

            let red = ((ray.direction[0] + 1.0) * 127.5) as u8;
            let green = ((ray.direction[1] + 1.0) * 127.5) as u8;

            image.put_pixel(x, y, Rgb([red, green, 64]));
        }
    }

    image.save("output.png")?;
    Ok(())
}
