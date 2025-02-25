use crate::{Float, Vec2, rand_utils};
use glam::UVec2;
use std::fmt;

mod sobol_seq;

/// Determines the type of a 2D sampler
pub enum SamplerType {
    /// Sample uniformly within `[0,1]x[0,1]`
    Uniform,
    /// Divide `[0,1]x[0,1]` are to `NxN` strata and return one sample from each
    Jittered,
    /// Correlated multi-jittered sampler, Kensler 2013
    MultiJittered,
    /// Samples generated from the Sobol sequence
    Sobol,
}

impl SamplerType {
    /// Returns a sampler with `samples` corresponding to `self`
    #[allow(clippy::new_ret_no_self)]
    pub fn new(&self, samples: u32) -> Box<dyn Sampler> {
        match self {
            Self::Uniform => Box::new(UniformSampler::new(samples)),
            Self::Jittered => Box::new(JitteredSampler::new(samples)),
            Self::MultiJittered => Box::new(MultiJitteredSampler::new(samples)),
            Self::Sobol => Box::new(SobolSampler::new(samples)),
        }
    }
}

impl fmt::Display for SamplerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uniform => write!(f, "uniform"),
            Self::Jittered => write!(f, "jittered"),
            Self::MultiJittered => write!(f, "multi-jittered"),
            Self::Sobol => write!(f, "Sobol"),
        }
    }
}

pub trait Sampler: Iterator<Item=Vec2> { }
impl Sampler for UniformSampler { }
impl Sampler for JitteredSampler { }
impl Sampler for MultiJitteredSampler { }
impl Sampler for SobolSampler { }

/// Choose each sample point uniformly at random
pub struct UniformSampler {
    /// How many samples have been given?
    state: u32,
    /// How many samples was asked?
    samples: u32,
}

impl UniformSampler {
    fn new(samples: u32) -> Self {
        assert!(samples > 0);
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
struct JitteredSampler {
    /// Width of one strata
    scale: Vec2,
    /// How many samples have been given?
    state: u32,
    /// How many strata per dimension?
    strata_dim: u32,
    /// How many samples have been asked for? Should be a square,
    /// otherwise gets rounded down to the nearest square.
    samples: u32,
}

impl JitteredSampler {
    /// Constructs a jittered sampler
    fn new(samples: u32) -> Self {
        let dim = (samples as Float).sqrt().ceil() as u32;
        let scale = Vec2::new(
            1.0 / (dim as Float),
            (dim as Float) / (samples as Float),
        );
        Self {
            scale,
            samples,
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

struct MultiJitteredSampler {
    samples: u32,
    state: u32,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    scale0: Vec2,
    scale1: Vec2,
    strata_dim: u32,
}

impl MultiJitteredSampler {
    fn new(samples: u32) -> Self {
        let dim = (samples as Float).sqrt().ceil() as u32;
        let scale = Vec2::new(
            1.0 / (dim as Float),
            (dim as Float) / (samples as Float),
        );

        Self {
            samples,
            state: 0,
            perm_x: rand_utils::perm_n(dim as usize),
            perm_y: rand_utils::perm_n(dim as usize),
            scale0: scale,
            scale1: scale / dim as Float,
            strata_dim: dim,
        }
    }
}

impl Iterator for MultiJitteredSampler {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.samples {
            None
        } else {
            let x0 = self.state % self.strata_dim;
            let y0 = self.state / self.strata_dim;
            let offset0 = self.scale0 * Vec2::new(x0 as Float, y0 as Float);

            let x1 = self.perm_x[y0 as usize];
            let y1 = self.perm_y[x0 as usize];
            let offset1 = self.scale1 * Vec2::new(x1 as Float, y1 as Float);

            let rand_sq = self.scale1 * rand_utils::unit_square();

            self.state += 1;
            Some( offset0 + offset1 + rand_sq )
        }
    }
}


struct SobolSampler {
    samples: u32,
    state: u32,
    seed: u32,
    prev: UVec2,
}

impl SobolSampler {
    fn new(samples: u32) -> Self {
        assert!(samples > 0);
        let samples = samples as usize;
        assert!(samples <= sobol_seq::SOBOL_MAX_LEN);

        Self {
            samples: samples as u32,
            state: 0,
            seed: (rand_utils::rand_float() * Float::powi(2.0, 32)) as u32,
            prev: UVec2::ZERO,
        }
    }

    fn step(&mut self) {
        self.state += 1;
        self.prev = UVec2::new(
            self.prev.x ^ sobol_seq::VS1[self.state.trailing_zeros() as usize],
            self.prev.y ^ sobol_seq::VS2[self.state.trailing_zeros() as usize],
        );
    }

    fn shuffle(&self, v: u32) -> u32 {
        v ^ self.seed
    }
}

impl Iterator for SobolSampler {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.samples {
            None
        } else {
            self.step();

            let xy = Vec2::new(
                Float::from(self.shuffle(self.prev.x)) * Float::powi(2.0, -32),
                Float::from(self.shuffle(self.prev.y)) * Float::powi(2.0, -32),
            );

            Some( xy )
        }
    }
}
