use glam::f64::DVec3;
use rayon::iter::{ParallelIterator, IntoParallelIterator};

mod image;
mod tracer;

const EPSILON: f64 = 0.001;
const WIDTH: usize = 3840;
const HEIGHT: usize = 2160;
const DEBUG_R: f64 = 0.005;

fn main() {
    let scene = tracer::scene::Scene::default();
    let cam = tracer::camera::Camera::new(
        WIDTH as f64 / HEIGHT as f64,
        DVec3::new(0.0, 0.0, 0.0), // origin
        DVec3::new(0.0, 0.0, -100.0), // towards
        DVec3::new(0.0, 1.0, 0.0) // up
    );

    let start_img = std::time::SystemTime::now();
    let image_buffer = (0..HEIGHT).into_par_iter().flat_map(|y| {
        (0..WIDTH).map(|x| {
            let u = x as f64
                / (WIDTH-1) as f64;
            let v = (HEIGHT - 1 - y) as f64
                / (HEIGHT-1) as f64;
            let r = cam.ray_at(u, v);
            r.color(&scene, 0)
        }).collect::<Vec<DVec3>>()
    }).collect::<Vec<DVec3>>();
    match start_img.elapsed() {
        Ok(v) => println!("rendering done in {v:?}"),
        Err(e) => println!("rendering done, error measuring duration {e:?}"),
    }

    let image = image::Image {
        buffer: image_buffer,
        width: WIDTH,
        height: HEIGHT,
        fname: String::from("cover.png"),
    };

    let start_png = std::time::SystemTime::now();
    image.save();
    match start_png.elapsed() {
        Ok(v) => println!("created png in {v:?}"),
        Err(e) => println!("png done, error measuring duration {e:?}"),
    }
}
