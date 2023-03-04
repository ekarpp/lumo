use glam::f64::DVec3;

mod image;
mod tracer;

const EPSILON: f64 = 0.001;
const WIDTH: usize = 3840;
const HEIGHT: usize = 2160;

fn main() {
    let mut image = image::Image {
        buffer: vec![DVec3::ZERO; WIDTH*HEIGHT],
        width: WIDTH,
        height: HEIGHT
    };

    let cam = tracer::camera::Camera::new(
        WIDTH as f64 / HEIGHT as f64,
        DVec3::ZERO, // origin
        DVec3::new(0.0, 0.0, -1.0), // towards (+ focal length)
        DVec3::new(0.0, 1.0, 0.0) // up
    );
    let scene = tracer::scene::Scene::default();

    let mut start = std::time::SystemTime::now();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let u = x as f64
                / (WIDTH-1) as f64;
            let v = (HEIGHT - 1 - y) as f64
                / (HEIGHT-1) as f64;
            let r = cam.ray_at(u, v);
            image.buffer[x + y*WIDTH] = r.color(&scene, 0);
        }
        let percent = 100.0 * y as f64 / (HEIGHT - 1) as f64;
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
