use crate::{Float, Vec2, rng::Xorshift};
use glam::UVec2;
use std::{ fmt, cell::RefCell };

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
    pub fn new<'a>(&'a self, samples: u32, rng: &'a RefCell<Xorshift>) -> Box<dyn Sampler + 'a> {
        match self {
            Self::Uniform => Box::new(UniformSampler::new(samples, rng)),
            Self::Jittered => Box::new(JitteredSampler::new(samples, rng)),
            Self::MultiJittered => Box::new(MultiJitteredSampler::new(samples, rng)),
            Self::Sobol => Box::new(SobolSampler::new(samples, rng)),
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
impl Sampler for UniformSampler<'_> { }
impl Sampler for JitteredSampler<'_> { }
impl Sampler for MultiJitteredSampler<'_> { }
impl Sampler for SobolSampler<'_> { }

/// Choose each sample point uniformly at random
struct UniformSampler<'a> {
    /// How many samples have been given?
    state: u32,
    /// How many samples was asked?
    samples: u32,
    rng: &'a RefCell<Xorshift>,
}

impl<'a> UniformSampler<'a> {
    fn new(samples: u32, rng: &'a RefCell<Xorshift>) -> Self {
        assert!(samples > 0);
        Self { samples, rng, state: 0 }
    }
}

impl Iterator for UniformSampler<'_> {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.samples {
            None
        } else {
            self.state += 1;
            Some( self.rng.borrow_mut().gen_vec2() )
        }
    }
}

/// Divide unit square to `n`x`n` strata and provide one sample from each strata.
struct JitteredSampler<'a> {
    /// Width of one strata
    scale: Vec2,
    /// How many samples have been given?
    state: u32,
    /// How many strata per dimension?
    strata_dim: u32,
    /// How many samples have been asked for? Should be a square,
    /// otherwise gets rounded down to the nearest square.
    samples: u32,
    rng: &'a RefCell<Xorshift>,
}

impl<'a> JitteredSampler<'a> {
    /// Constructs a jittered sampler
    fn new(samples: u32, rng: &'a RefCell<Xorshift>) -> Self {
        let dim = (samples as Float).sqrt().ceil() as u32;
        let scale = Vec2::new(
            1.0 / (dim as Float),
            (dim as Float) / (samples as Float),
        );
        Self {
            rng,
            scale,
            samples,
            strata_dim: dim,
            state: 0,
        }
    }
}

impl Iterator for JitteredSampler<'_> {
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
            Some(self.scale * self.rng.borrow_mut().gen_vec2() + offset)
        }
    }
}

struct MultiJitteredSampler<'a> {
    samples: u32,
    state: u32,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    scale0: Vec2,
    scale1: Vec2,
    strata_dim: u32,
    rng: &'a RefCell<Xorshift>,
}

impl<'a> MultiJitteredSampler<'a> {
    fn new(samples: u32, rng: &'a RefCell<Xorshift>) -> Self {
        let dim = (samples as Float).sqrt().ceil() as u32;
        let scale = Vec2::new(
            1.0 / (dim as Float),
            (dim as Float) / (samples as Float),
        );

        let (perm_x, perm_y) = {
            let mut rng_mut = rng.borrow_mut();
            (rng_mut.gen_perm(dim as usize), rng_mut.gen_perm(dim as usize))
        };

        Self {
            rng,
            samples,
            perm_x,
            perm_y,
            state: 0,
            scale0: scale,
            scale1: scale / dim as Float,
            strata_dim: dim,
        }
    }
}

impl Iterator for MultiJitteredSampler<'_> {
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

            let rand_sq = self.scale1 * self.rng.borrow_mut().gen_vec2();

            self.state += 1;
            Some( offset0 + offset1 + rand_sq )
        }
    }
}


struct SobolSampler<'a> {
    _rng: &'a RefCell<Xorshift>,
    samples: u32,
    state: u32,
    seed: u32,
    prev: UVec2,
}

impl<'a> SobolSampler<'a> {
    fn new(samples: u32, rng: &'a RefCell<Xorshift>) -> Self {
        assert!(samples > 0);
        let samples = samples as usize;
        assert!(samples <= sobol_seq::SOBOL_MAX_LEN);

        let seed = {
            rng.borrow_mut().gen_u64()
        };

        Self {
            _rng: rng,
            samples: samples as u32,
            state: 0,
            seed: seed as u32,
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

impl Iterator for SobolSampler<'_> {
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
