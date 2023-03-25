use crate::Integrator;

#[derive(argh::FromArgs)]
/// Just a ray tracer :)
pub struct TracerCli {
    /// number of samples per pixel (defaults to 1)
    #[argh(option, short='s')]
    samples: Option<u32>,

    /// number of threads used (defaults to all)
    #[argh(option, short='t')]
    threads: Option<usize>,

    /// width of the rendered image (defaults to 1000)
    #[argh(option, short='w')]
    width: Option<i32>,

    /// height of the rendered image (defaults to 1000)
    #[argh(option, short='h')]
    height: Option<i32>,

    /* how to handle multiple integrators in the future? */
    /// use direct light integrator instead of path tracing.
    #[argh(switch, short='d', long="direct")]
    direct_light: bool,
}



impl TracerCli {
    pub fn output_cfg(&self) {
        println!("Rendering scene as a {} x {} image \
                  with {} thread(s) and {} sample(s) per pixel using {}.",
                 self.get_width(),
                 self.get_height(),
                 rayon::current_num_threads(),
                 self.get_samples(),
                 self.get_integrator(),
        );
    }

    pub fn set_threads(&self) {
        if let Some(t) = self.threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(t)
                .build_global()
                .unwrap()
        }
    }

    pub fn get_integrator(&self) -> Integrator {
        if self.direct_light {
            Integrator::DirectLight
        } else {
            Integrator::PathTrace
        }
    }

    pub fn get_width(&self) -> i32 {
        self.width.unwrap_or(1000)
    }

    pub fn get_samples(&self) -> u32 {
        self.samples.unwrap_or(1)
    }

    pub fn get_height(&self) -> i32 {
        self.height.unwrap_or(1000)
    }
}
