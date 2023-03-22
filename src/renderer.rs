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
    img_width: u32,
    img_height: u32,
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

    pub fn set_width(mut self, img_width: u32) {
        self.img_width = img_width;
    }

    pub fn set_height(mut self, img_height: u32) {
        self.img_height = img_height;
    }

    pub fn set_samples(mut self, num_samples: u32) {
        self.num_samples = num_samples;
    }

    pub fn set_integrator(mut self, integrator: Integrator) {
        self.integrator = integrator;
    }

    pub fn render(&self) -> Image {
        Image::new(
            self.compute_buffer(),
            self.img_width,
            self.img_height,
        )
    }

    fn compute_buffer(&self) -> Vec<DVec3> {
        let px_height = 1.0 / (self.img_height - 1) as f64;
        let px_width = 1.0 / (self.img_width - 1) as f64;

        (0..self.img_height).into_par_iter().flat_map(|y: u32| {
            (0..self.img_width).map(|x: u32| {
                let u = x as f64 * px_width;
                let v = (self.img_height - 1 - y) as f64 * px_height;

                PxSampler::new(self.num_samples).map(|rand_sq: DVec2| {
                    self.integrator.integrate(
                        &self.scene,
                        &self.camera.ray_at(
                            u + rand_sq.x*px_width,
                            v + rand_sq.y*px_height
                        ),
                    )
                }).fold(DVec3::ZERO, |acc: DVec3, c: DVec3| acc + c)
                    / self.num_samples as f64
            }).collect::<Vec<DVec3>>()
        }).collect()
    }
}
