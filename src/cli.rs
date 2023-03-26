use crate::tracer::Integrator;

#[derive(argh::FromArgs)]
/// Just a ray tracer :)
pub struct TracerCli {
    /// number of samples per pixel (defaults to 1)
    #[argh(option, short='s', default="1")]
    pub samples: u32,

    /// number of threads used (defaults to all)
    #[argh(option, short='t')]
    pub threads: Option<usize>,

    /// width of the rendered image (defaults to 1000)
    #[argh(option, short='w', default="1000")]
    pub width: i32,

    /// height of the rendered image (defaults to 1000)
    #[argh(option, short='h', default="1000")]
    pub height: i32,

    /* how to handle multiple integrators in the future? */
    /// use direct light integrator instead of path tracing.
    #[argh(switch, short='d', long="direct")]
    pub direct_light: bool,
}



impl TracerCli {
    /// Sets the configured number of threads. Called by renderer on creation.
    pub fn set_threads(&self) {
        if let Some(t) = self.threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(t)
                .build_global()
                .unwrap();
        }
    }

    /// Get the configured integrator.
    pub fn get_integrator(&self) -> Integrator {
        if self.direct_light {
            Integrator::DirectLight
        } else {
            Integrator::PathTrace
        }
    }
}
