use crate::{DVec3, DVec2};
use rayon::iter::{ParallelIterator, IntoParallelIterator};
use crate::image::Image;
#[allow(unused_imports)]
use crate::samplers::{JitteredSampler, UniformSampler};
use crate::scene::Scene;
use crate::camera::Camera;
#[allow(unused_imports)]
use crate::tracer::integrator::Integrator;

type PxSampler = JitteredSampler;

pub struct Renderer {
    scene: Scene,
    camera: Camera,
    img_width: i32,
    img_height: i32,
    num_samples: u32,
    integrator: Integrator,
}

impl Renderer {
    pub fn new(scene: Scene, camera: Camera) -> Self {
        Self {
            scene,
            camera,
            img_width: 1000,
            img_height: 1000,
            num_samples: 1,
            integrator: Integrator::PathTrace,
        }
    }

    pub fn set_width(mut self, img_width: i32) {
        self.img_width = img_width;
    }

    pub fn set_height(mut self, img_height: i32) {
        self.img_height = img_height;
    }

    pub fn set_samples(mut self, num_samples: u32) {
        self.num_samples = num_samples;
    }

    pub fn set_integrator(mut self, integrator: Integrator) {
        self.integrator = integrator;
    }

    pub fn render(&self) -> Image {
        let buffer: Vec<DVec3> = (0..self.img_height)
            .into_par_iter()
            .flat_map(|y: i32| {
                (0..self.img_width)
                    .map(|x: i32| self.get_color(x, y))
                    .collect::<Vec<DVec3>>()
            })
            .collect();

        Image::new(
            buffer,
            self.img_width,
            self.img_height,
        )
    }

    fn get_color(&self, x: i32, y: i32) -> DVec3 {
        let max_dim = self.img_height.max(self.img_width) as f64;
        /* (u,v) in [-1, 1]^2 */
        let u = (2*x + 1 - self.img_width) as f64 / max_dim;
        let v = (2 * (self.img_height - y) - 1 - self.img_height) as f64
            / max_dim;

        PxSampler::new(self.num_samples).map(|rand_sq: DVec2| {
            // random offsets
            let ou = rand_sq.x / max_dim;
            let ov = rand_sq.y / max_dim;
            self.integrator.integrate(
                &self.scene,
                &self.camera.ray_at(
                    u + ou,
                    v + ov
                ),
            )
        }).fold(DVec3::ZERO, |acc: DVec3, c: DVec3| acc + c)
            / self.num_samples as f64
    }
}
