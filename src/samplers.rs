use crate::{Float, Vec2, rng::Xorshift};
use std::fmt;

mod sobol_seq;

/// Determines the type of a 2D sampler
#[derive(Clone)]
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

impl Default for SamplerType {
    fn default() -> Self { Self::MultiJittered }
}

impl SamplerType {
    /// Returns a sampler with `samples` corresponding to `self`
    #[allow(clippy::new_ret_no_self)]
    pub fn new(&self, batch: u64, samples: u64, seed: u64) -> Box<dyn Sampler> {
        let rng = || Xorshift::new(seed);
        let s0 = batch * crate::renderer::SAMPLES_INCREMENT;
        let s1 = (batch + 1) * crate::renderer::SAMPLES_INCREMENT;
        let s1 = s1.min(samples);
        match self {
            Self::Uniform => Box::new(UniformSampler::new(s1 - s0, rng())),
            Self::Jittered => Box::new(JitteredSampler::new(s0, s1, samples, rng())),
            Self::MultiJittered => Box::new(MultiJitteredSampler::new(s0, s1, samples, rng())),
            Self::Sobol => Box::new(SobolSampler::new(batch, s0, s1, seed)),
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
struct UniformSampler {
    /// How many samples have been given?
    state: u64,
    /// How many samples was asked?
    samples: u64,
    rng: Xorshift,
}

impl UniformSampler {
    fn new(samples: u64, rng: Xorshift) -> Self {
        assert!(samples > 0);
        Self { samples, rng, state: 0 }
    }
}

impl Iterator for UniformSampler {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.samples {
            None
        } else {
            self.state += 1;
            Some( self.rng.gen_vec2() )
        }
    }
}

/// Divide unit square to `n`x`n` strata and provide one sample from each strata.
struct JitteredSampler {
    /// Width of one strata
    scale: Vec2,
    /// How many samples have been given?
    state: u64,
    /// At which state terminate the current batch
    batch_end: u64,
    /// How many strata per dimension?
    strata_dim: u64,
    rng: Xorshift,
}

impl JitteredSampler {
    /// Constructs a jittered sampler
    fn new(s0: u64, s1: u64, samples: u64, rng: Xorshift) -> Self {
        let dim = (samples as Float).sqrt().ceil() as u64;
        let scale = Vec2::new(
            1.0 / (dim as Float),
            (dim as Float) / (samples as Float),
        );
        Self {
            rng,
            scale,
            strata_dim: dim,
            state: s0,
            batch_end: s1,

        }
    }
}

impl Iterator for JitteredSampler {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.batch_end {
            None
        } else {
            let offset = self.scale
                * Vec2::new(
                    (self.state % self.strata_dim) as Float,
                    (self.state / self.strata_dim) as Float,
                );
            self.state += 1;
            Some(self.scale * self.rng.gen_vec2() + offset)
        }
    }
}

struct MultiJitteredSampler {
    state: u64,
    batch_end: u64,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    scale0: Vec2,
    scale1: Vec2,
    strata_dim: u64,
    rng: Xorshift,
}

impl MultiJitteredSampler {
    fn new(s0: u64, s1: u64, samples: u64, mut rng: Xorshift) -> Self {
        let dim = (samples as Float).sqrt().ceil() as u64;
        let scale = Vec2::new(
            1.0 / (dim as Float),
            (dim as Float) / (samples as Float),
        );

        let perm_x = rng.gen_perm(dim as usize);
        let perm_y = rng.gen_perm(dim as usize);

        Self {
            rng,
            perm_x,
            perm_y,
            state: s0,
            batch_end: s1,
            scale0: scale,
            scale1: scale / dim as Float,
            strata_dim: dim,
        }
    }
}

impl Iterator for MultiJitteredSampler {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.batch_end {
            None
        } else {
            let x0 = self.state % self.strata_dim;
            let y0 = self.state / self.strata_dim;
            let offset0 = self.scale0 * Vec2::new(x0 as Float, y0 as Float);

            let x1 = self.perm_x[y0 as usize];
            let y1 = self.perm_y[x0 as usize];
            let offset1 = self.scale1 * Vec2::new(x1 as Float, y1 as Float);

            let rand_sq = self.scale1 * self.rng.gen_vec2();

            self.state += 1;
            Some( offset0 + offset1 + rand_sq )
        }
    }
}


struct SobolSampler {
    state: u64,
    batch_end: u64,
    seed: u64,
    prev: (u64, u64),
}

impl SobolSampler {
    fn new(batch: u64, s0: u64, s1: u64, seed: u64) -> Self {
        #[cfg(debug_assertions)]
        {
            assert!(s1 > 0);
            assert!(s1 as usize <= sobol_seq::SOBOL_MAX_LEN);
        }

        Self {
            state: s0,
            batch_end: s1,
            seed,
            prev: sobol_seq::BATCH_STATES[batch as usize],
        }
    }

    fn step(&mut self) {
        self.state += 1;
        self.prev = (
            self.prev.0 ^ sobol_seq::VS1[self.state.trailing_zeros() as usize],
            self.prev.1 ^ sobol_seq::VS2[self.state.trailing_zeros() as usize],
        );
    }

    fn shuffle(&self, v: u64) -> u64 {
        v ^ self.seed
    }
}

impl Iterator for SobolSampler {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.batch_end {
            None
        } else {
            self.step();

            let xy = Vec2::new(
                self.shuffle(self.prev.0) as Float * Float::powi(2.0, -64),
                self.shuffle(self.prev.1) as Float * Float::powi(2.0, -64),
            );

            Some( xy )
        }
    }
}
