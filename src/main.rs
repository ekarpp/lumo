mod image;
mod tracer;

const WIDTH: usize = 3840;
const HEIGHT: usize = 2160;

fn main() {
    let mut image = image::Image {
        buffer: Vec::with_capacity(WIDTH*HEIGHT),
        width: WIDTH,
        height: HEIGHT
    };

    let cam = tracer::camera::default();
    let scene = tracer::scene::default();

    let mut start = std::time::SystemTime::now();
    for y in (0..HEIGHT).rev() {
        for x in 0..WIDTH {
            let u = x as f32
                / (WIDTH-1) as f32;
            let v = y as f32
                / (HEIGHT-1) as f32;
            let r = cam.ray_at(u, v);
            image.buffer.push(r.color(&scene, 0));
        }
        let percent = 100.0 * (HEIGHT - 1 - y) as f32 / (HEIGHT - 1) as f32;
        print!("{} % done \r", percent as u32);
    }
    let mut diff = start.elapsed();
    match diff {
        Ok(v) => println!("rendering done in {v:?}"),
        Err(e) => println!("rendering done, error measuring duration {e:?}"),
    }

    start = std::time::SystemTime::now();
    image.save();
    diff = start.elapsed();

    match diff {
        Ok(v) => println!("created png in {v:?}"),
        Err(e) => println!("png done, error measuring duration {e:?}"),
    }
}
