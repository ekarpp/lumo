use glam::{UVec3, f64::{DVec3, DMat3, DVec2, DAffine3}};
use crate::tracer::scene::Scene;
use crate::tracer::camera::Camera;

mod onb;
mod pdfs;
mod image;
mod tracer;
mod consts;
mod perlin;
mod samplers;
mod renderer;
mod rand_utils;

const WIDTH: usize = 3840;
const HEIGHT: usize = 2160;
const NUM_SAMPLES: usize = 1;
const FOV: f64 = 90.0;
const FNAME: &str = "render.png";

#[derive(argh::FromArgs)]
/// Just a ray tracer :)
struct TracerCli {
    /// number of random samples per pixel (defaults to 1)
    #[argh(option, short='s')]
    samples: Option<usize>,

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

    /// render box scene?
    #[argh(switch, short='b')]
    boxx: bool,
}

impl TracerCli {
    pub fn output_cfg(&self) {
        println!("rendering scene to file \"{}\" as a {} x {} image \
                  using {} thread(s) with {} sample(s) per pixel and vfov at {}°",
                 self.fname.as_ref().unwrap_or(&String::from(FNAME)),
                 self.width.unwrap_or(WIDTH),
                 self.height.unwrap_or(HEIGHT),
                 rayon::current_num_threads(),
                 self.samples.unwrap_or(NUM_SAMPLES),
                 self.vfov.unwrap_or(FOV),
        );
    }
}

fn main() {
    let cli_args: TracerCli = argh::from_env();

    let img_width = match cli_args.width {
        Some(w) => w,
        None => WIDTH,
    };
    /* pixel width */
    let px_width = 1.0 / (img_width - 1) as f64;

    let img_height = match cli_args.height {
        Some(h) => h,
        None => HEIGHT,
    };
    /* pixel height */
    let px_height = 1.0 / (img_height - 1) as f64;

    let n_samples = match cli_args.samples {
        Some(n) => n,
        None => NUM_SAMPLES,
    };

    match cli_args.threads {
        Some(t) => rayon::ThreadPoolBuilder::new().num_threads(t)
            .build_global().unwrap(),
        None => (),
    };

    let fl = 1.0;
    let scene = if cli_args.boxx {
        Scene::box_scene(fl)
    } else {
        Scene::default()
    };
    let cam = Camera::new(
        img_width as f64 / img_height as f64,
        cli_args.vfov.unwrap_or(90.0),
        DVec3::new(0.0, 0.0, 0.0), /* origin */
        DVec3::new(0.0, 0.0, -1.0), /* towards */
        DVec3::new(0.0, 1.0, 0.0), /* up */
        fl, /* focal length */
    );

    cli_args.output_cfg();

    let scene_size = scene.size();

    let start_img = std::time::SystemTime::now();
    let image_buffer: Vec<DVec3> = renderer::_render(
        img_height,
        px_height,
        img_width,
        px_width,
        n_samples,
        cam,
        scene,
    );
    match start_img.elapsed() {
        Ok(v) => println!("rendered scene with {} objects in {v:?}",
                          scene_size),
        Err(e) => println!("rendering done, error measuring duration {e:?}"),
    }

    let image = image::Image {
        buffer: image_buffer,
        width: img_width,
        height: img_height,
        fname: cli_args.fname.unwrap_or(String::from(FNAME)),
    };

    let start_png = std::time::SystemTime::now();
    image.save();
    match start_png.elapsed() {
        Ok(v) => println!("created png in {v:?}"),
        Err(e) => println!("png done, error measuring duration {e:?}"),
    }
}
