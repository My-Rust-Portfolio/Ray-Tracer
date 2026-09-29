mod camera;
mod math;
mod objects;
mod scene;

use camera::Camera;
use image::{Rgb, RgbImage};
use scene::Scene;

fn main() -> image::ImageResult<()> {
    let width = 256;
    let height = 256;
    let camera = Camera::new(width, height);
    let scene = Scene::new();
    let mut image = RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let ray = camera.ray_for_pixel(x, y);

            let colour = if scene.closest_hit(&ray).is_some() {
                Rgb([200, 50, 50])
            } else {
                let red = ((ray.direction[0] + 1.0) * 127.5) as u8;
                let green = ((ray.direction[1] + 1.0) * 127.5) as u8;
                Rgb([red, green, 64])
            };

            image.put_pixel(x, y, colour);
        }
    }

    image.save("output.png")?;
    Ok(())
}
