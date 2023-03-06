use glam::f64::{DVec3, DMat3, DAffine3, DQuat};
use rayon::iter::{ParallelIterator, IntoParallelIterator};
use crate::tracer::scene::Scene;
use crate::tracer::camera::Camera;

mod image;
mod tracer;
mod rand_utils;

const EPSILON: f64 = 0.001;
const WIDTH: usize = 3840;
const HEIGHT: usize = 2160;

#[derive(argh::FromArgs)]
/// Just a ray tracer :)
struct TracerCli {
    /// toggle anti-aliasing (4xSSAA)
    #[argh(switch, short='a')]
    alias: bool,

    /// render randomly generated scene
    #[argh(switch, short='r')]
    rnd_scene: bool,

    /// filename for rendered image (defaults to render.png)
    #[argh(option, short='o')]
    fname: Option<String>,

    /// vertical field-of-view in degrees (defaults to 90)
    #[argh(option, short='f')]
    vfov: Option<f64>,

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
    /* pixel width, w-1? */
    let pw = 1.0 / img_width as f64;

    let img_height = match cli_args.height {
        Some(h) => h,
        None => HEIGHT,
    };
    /* pixel height, h-1? */
    let ph = 1.0 / img_height as f64;

    match cli_args.threads {
        Some(t) => rayon::ThreadPoolBuilder::new().num_threads(t)
            .build_global().unwrap(),
        None => (),
    };

    let vfov = match cli_args.vfov {
        Some(f) => f,
        None => 90.0,
    };

    // make this a method in the cli struct. try storing def values there too
    println!("rendering {} x {} image using {} thread(s) \
              with anti-aliasing {} and vfov {} deg",
             img_width,
             img_height,
             rayon::current_num_threads(),
             if cli_args.alias { "enabled" } else { "disabled" },
             vfov,
    );

    let scene = if cli_args.rnd_scene {
        Scene::random()
    } else {
        Scene::default()
    };
    let cam = Camera::new(
        img_width as f64 / img_height as f64,
        cli_args.vfov.unwrap_or(90.0),
        DVec3::new(0.0, 0.0, 0.0), // origin
        DVec3::new(0.0, 0.0, -100.0), // towards
        DVec3::new(0.0, 1.0, 0.0), // up
    );

    let start_img = std::time::SystemTime::now();
    let image_buffer: Vec<DVec3> = (0..img_height).into_par_iter().flat_map(|y| {
        (0..img_width).map(|x| {
            let u = x as f64 * pw;
            let v = (img_height - 1 - y) as f64 * ph;
            if cli_args.alias {
                /* must be cleaner way to do this. should avoid Vec */
                cam.ss_rays_at(pw, ph, u, v).iter()
                    .fold(DVec3::ZERO, |acc, r| acc + r.color(&scene, 0)) / 4.0
            } else {
                cam.ray_at(u, v).color(&scene, 0)
            }
        }).collect::<Vec<DVec3>>()
    }).collect();
    match start_img.elapsed() {
        Ok(v) => println!("rendering done in {v:?}"),
        Err(e) => println!("rendering done, error measuring duration {e:?}"),
    }

    let image = image::Image {
        buffer: image_buffer,
        width: img_width,
        height: img_height,
        fname: cli_args.fname.unwrap_or(String::from("render.png")),
    };

    let start_png = std::time::SystemTime::now();
    image.save();
    match start_png.elapsed() {
        Ok(v) => println!("created png in {v:?}"),
        Err(e) => println!("png done, error measuring duration {e:?}"),
    }
}
