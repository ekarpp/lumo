extern crate glam;

mod image;

const WIDTH: usize = 256;
const HEIGHT: usize = WIDTH;

fn main() {
    let mut image = image::Image {
        buffer: Vec::with_capacity(WIDTH*HEIGHT),
        width: WIDTH,
        height: HEIGHT
    };

    for y in 0..HEIGHT
    {
        for x in 0..WIDTH
        {
            image.buffer.push(glam::f32::Vec3::new(
                x as f32 / WIDTH as f32,
                y as f32 / HEIGHT as f32,
                0.5
            ));
        }
    }
    image::saver::save(&image);
}
