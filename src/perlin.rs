use crate::rand_utils;
use glam::{DVec3, UVec3};
use itertools::Itertools;

/// Number of points in the perlin noise lattice
const PERLIN_POINTS: usize = 256;

/// Scale of points in perlin. bigger = more noticeable effect
const PERLIN_SCALE: f64 = 4.0;

/// Frequency of noise in perlin noise. bigger = more frequent
const PERLIN_FREQ: f64 = 60.0;

/// Amplitude of the noise pattern in perlin noise
const PERLIN_AMP: f64 = 20.0;

/// Recursion depth in perlin turbulence
const PERLIN_OCTAVES: i32 = 6;

/// Scale of each term in turbulence. should be less than 1.0
const PERLIN_GAIN: f64 = 0.5;

/// Helper struct to store permutation vectors for each dimension.
struct PermutationXyz {
    x: Vec<usize>,
    y: Vec<usize>,
    z: Vec<usize>,
}

/// Perlin noise generator.
pub struct Perlin {
    /// a.k.a the underlying colour. use texture instead?
    albedo: DVec3,
    /// Random normals of the Perlin lattice
    lattice: Vec<DVec3>,
    /// Permutation directions
    perm: PermutationXyz,
}

impl Default for Perlin {
    /// Perlin generator with underlying color as rgb(1,1,1)
    fn default() -> Self {
        Self::new(DVec3::ONE)
    }
}

impl Perlin {
    /// Constructs new Perlin generator with the given underlying colour.
    /// Pass texture instead?
    pub fn new(albedo: DVec3) -> Self {
        Self {
            albedo,
            lattice: rand_utils::rand_vec_dvec3(PERLIN_POINTS),
            perm: PermutationXyz {
                x: rand_utils::perm_n(PERLIN_POINTS),
                y: rand_utils::perm_n(PERLIN_POINTS),
                z: rand_utils::perm_n(PERLIN_POINTS),
            },
        }
    }

    /// Computes color of the noise at point `p`. Perlin noise
    /// with turbulence.
    pub fn albedo_at(&self, p: DVec3) -> DVec3 {
        self.albedo * self._scale_turb(p.x, self.turbulence(0.0, PERLIN_SCALE * p.abs(), 0))
    }

    fn _scale_turb(&self, px: f64, t: f64) -> f64 {
        1.0 - (0.5 + 0.5 * (PERLIN_FREQ * px + PERLIN_AMP * t).sin()).powi(6)
    }

    /// Computes the turbulence for the noise. I.e. absolute values of the
    /// noise at different octaves are summed together.
    fn turbulence(&self, acc: f64, p: DVec3, depth: i32) -> f64 {
        if depth >= PERLIN_OCTAVES {
            return acc;
        }
        let w = PERLIN_GAIN.powi(depth);

        self.turbulence(acc + w * self.noise_at(p).abs(), 2.0 * p, depth + 1)
    }

    /// Computes traditional perlin noise at point `p`
    fn noise_at(&self, p: DVec3) -> f64 {
        let weight = p.fract();
        let floor = p.floor();

        let normals = (0..2)
            .cartesian_product(0..2)
            .cartesian_product(0..2)
            .map(|((i, j), k)| {
                self.lattice[self._hash(
                    floor.x as usize + i,
                    floor.y as usize + j,
                    floor.z as usize + k,
                )]
            })
            .collect();

        self.interp(normals, self._smootherstep(weight))
    }

    /// Hash utility function to get normals in the lattice
    fn _hash(&self, x: usize, y: usize, z: usize) -> usize {
        self.perm.x[x % PERLIN_POINTS]
            ^ self.perm.y[y % PERLIN_POINTS]
            ^ self.perm.z[z % PERLIN_POINTS]
    }

    /// Smoothing for weights
    fn _hermite_cubic(&self, x: DVec3) -> DVec3 {
        (3.0 - 2.0 * x) * x * x
    }

    /// Smoothing for weights
    fn _smootherstep(&self, x: DVec3) -> DVec3 {
        ((6.0 * x - 15.0) * x + 10.0) * x * x * x
    }

    /// Trilinear interpolation
    ///
    /// # Arguments
    /// * `normals` - Normals to perform interpolation with
    /// * `w` - Fractional part of the point. Gives distances to each normal.
    fn interp(&self, normals: Vec<DVec3>, w: DVec3) -> f64 {
        (0..2)
            .cartesian_product(0..2)
            .cartesian_product(0..2)
            .zip(normals)
            .map(|(((x, y), z), norm)| {
                let idx = UVec3::new(x, y, z).as_dvec3();
                let widx = 2.0 * w * idx + DVec3::ONE - w - idx;

                widx.x * widx.y * widx.z * norm.dot(w - idx)
            })
            .fold(0.0, |acc, v| acc + v)
    }
}
