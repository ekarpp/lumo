use crate::tracer::Integrator;

#[derive(argh::FromArgs)]
/// Optional CLI configuration of renderer. Renderer setter methods have priority.
pub struct TracerCli {
    /// number of samples per pixel (defaults to 1)
    #[argh(option, short = 's', default = "1")]
    pub samples: u32,

    /// number of threads used (defaults to all)
    #[argh(option, short = 't', default = "0")]
    pub threads: usize,

    /// use direct light integrator instead of path tracing
    #[argh(switch, short = 'd', long = "direct")]
    pub direct_light: bool,

    /// use bidirectional path tracing instead of path tracing
    #[argh(switch, short = 'b', long = "bdpt")]
    pub bd_path_trace: bool,
}

impl TracerCli {
    /// Get the configured integrator.
    pub fn get_integrator(&self) -> Integrator {
        if self.direct_light {
            Integrator::DirectLight
        } else if self.bd_path_trace {
            Integrator::BDPathTrace
        } else {
            Integrator::PathTrace
        }
    }
}
