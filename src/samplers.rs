use crate::{Float, Vec2, rand_utils};

/// Choose each sample point uniformly at random
pub struct UniformSampler {
    /// How many samples have been given?
    state: u32,
    /// How many samples was asked?
    samples: u32,
}

impl UniformSampler {
    /// Constructs an uniform sampler with `samples` samples
    #[allow(dead_code)]
    pub fn new(samples: u32) -> Self {
        Self { samples, state: 0 }
    }
}

impl Iterator for UniformSampler {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.samples {
            None
        } else {
            self.state += 1;
            Some(rand_utils::unit_square())
        }
    }
}

/// Divide unit square to `n`x`n` strata and provide one sample from each strata.
pub struct JitteredSampler {
    /// Width of one strata
    scale: Float,
    /// How many samples have been given?
    state: u32,
    /// How many strata per dimension?
    strata_dim: u32,
    /// How many samples have been asked for? Should be a square,
    /// otherwise gets rounded down to the nearest square.
    samples: u32,
}

impl JitteredSampler {
    /// Constructs a jittered sampler with `floor(sqrt(samples))^2` samples
    pub fn new(samples: u32) -> Self {
        let dim = (samples as Float).sqrt() as u32;
        Self {
            scale: (dim as Float).recip(),
            samples: dim * dim,
            strata_dim: dim,
            state: 0,
        }
    }
}

impl Iterator for JitteredSampler {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.samples {
            None
        } else {
            let offset = self.scale
                * Vec2::new(
                    (self.state % self.strata_dim) as Float,
                    (self.state / self.strata_dim) as Float,
                );
            self.state += 1;
            Some(self.scale * rand_utils::unit_square() + offset)
        }
    }
}
