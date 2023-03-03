extern crate image;

const WIDTH: usize = 256;
const HEIGHT: usize = WIDTH;

fn main() {

    let mut img: [u8; WIDTH*HEIGHT*3] = [0; WIDTH*HEIGHT*3];

    for y in 0..HEIGHT
    {
        for x in 0..WIDTH
        {
            let px = x + y*WIDTH;
            img[3*px + 0] = y as u8;
            img[3*px + 1] = x as u8;
            img[3*px + 2] = 123;
        }
    }
    image::save_buffer("image.png", &img, WIDTH as u32, HEIGHT as u32, image::ColorType::Rgb8);

}
