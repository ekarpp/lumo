use glam::f64::DVec3;
use rayon::iter::{ParallelIterator, IntoParallelIterator};

mod image;
mod tracer;

const EPSILON: f64 = 0.001;
const WIDTH: usize = 3840;
const HEIGHT: usize = 2160;
const DEBUG_R: f64 = 0.005;

#[derive(argh::FromArgs)]
/// Just a ray tracer :)
struct TracerCli {
    /// use anti-aliasing (not implemented!)
    #[argh(switch, short='a')]
    _alias: bool,

    /// number of threads used (defaults to all)
    #[argh(option, short='t')]
    threads: Option<usize>,

    /// width of the rendered image (defaults to 3840)
    #[argh(option, short='w')]
    width: Option<usize>,

    /// height of the rendered image (defaults to 2160)
    #[argh(option, short='h')]
    height: Option<usize>,
}

fn main() {
    let cli_args: TracerCli = argh::from_env();

    let img_width = match cli_args.width {
        Some(w) => w,
        None => WIDTH,
    };

    let img_height = match cli_args.height {
        Some(h) => h,
        None => HEIGHT,
    };

    match cli_args.threads {
        Some(t) => rayon::ThreadPoolBuilder::new().num_threads(t)
            .build_global().unwrap(),
        None => (),
    };

    let scene = tracer::scene::Scene::default();
    let cam = tracer::camera::Camera::new(
        img_width as f64 / img_height as f64,
        DVec3::new(0.0, 0.0, 0.0), // origin
        DVec3::new(0.0, 0.0, -100.0), // towards
        DVec3::new(0.0, 1.0, 0.0) // up
    );

    let start_img = std::time::SystemTime::now();
    let image_buffer = (0..img_height).into_par_iter().flat_map(|y| {
        (0..img_width).map(|x| {
            let u = x as f64
                / (img_width-1) as f64;
            let v = (img_height - 1 - y) as f64
                / (img_height-1) as f64;
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
        width: img_width,
        height: img_height,
        fname: String::from("cover.png"),
    };

    let start_png = std::time::SystemTime::now();
    image.save();
    match start_png.elapsed() {
        Ok(v) => println!("created png in {v:?}"),
        Err(e) => println!("png done, error measuring duration {e:?}"),
    }
}
