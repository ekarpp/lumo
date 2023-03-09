use crate::DVec2;
use crate::rand_utils;

pub struct UniformSampler {
    state: usize,
    samples: usize,
}

impl UniformSampler {
    pub fn new(num_rays: usize) -> Self {
        Self {
            state: 0,
            samples: num_rays,
        }
    }
}

impl Iterator for UniformSampler {
    type Item = DVec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.samples {
            None
        } else {
            self.state += 1;
            Some(rand_utils::rand_unit_square())
        }
    }
}

pub struct JitteredSampler {
    scale: f64,
    state: usize,
    strata_dim: usize,
    samples: usize,
}

impl JitteredSampler {
    pub fn new(num_rays: usize) -> Self {
        let dim = (num_rays as f64).sqrt() as usize;
        Self {
            scale: (dim as f64).recip(),
            samples: dim*dim,
            strata_dim: dim,
            state: 0,
        }
    }
}

impl Iterator for JitteredSampler {
    type Item = DVec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.samples {
            None
        } else {
            let offset = self.scale * DVec2::new(
                (self.state % self.strata_dim) as f64,
                (self.state / self.strata_dim) as f64,
            );
            self.state += 1;
            Some(self.scale * rand_utils::rand_unit_square() + offset)
        }
    }
}
