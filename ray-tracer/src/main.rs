use image::{Rgb, RgbImage};

fn main() -> image::ImageResult<()> {
    let width = 256;
    let height = 256;
    let mut image = RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let red = x as u8;
            let green = y as u8;
            image.put_pixel(x, y, Rgb([red, green, 64]));
        }
    }

    image.save("output.png")?;
    Ok(())
}
