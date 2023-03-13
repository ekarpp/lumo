#![allow(dead_code)]
use crate::DVec2;
use crate::rand_utils;

/// Choose each sample point uniformly at random
pub struct UniformSampler {
    /// How many samples have been given?
    state: usize,
    /// How many samples was asked?
    samples: usize,
}

impl UniformSampler {
    pub fn new(samples: usize) -> Self {
        Self {
            state: 0,
            samples: samples,
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

/// Divide unit square to `n`x`n` strata and provide one sample from each strata.
pub struct JitteredSampler {
    /// Width of one strata
    scale: f64,
    /// How many samples have been given?
    state: usize,
    /// How many strata per dimension?
    strata_dim: usize,
    /// How many samples have been asked for? Should be a square,
    /// otherwise gets rounded down to the nearest square.
    samples: usize,
}

impl JitteredSampler {
    pub fn new(samples: usize) -> Self {
        let dim = (samples as f64).sqrt() as usize;
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
