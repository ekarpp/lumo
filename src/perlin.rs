use crate::{DVec3, UVec3};
use crate::rand_utils;
use itertools::Itertools;

const POINTS: usize = 256;

const SCALE: f64 = 4.0;
const FREQ: f64 = 60.0;
const AMP: f64 = 20.0;
const OCTAVES: usize = 6;
const GAIN: f64 = 0.5;

struct PermutationXyz {
    x: Vec<usize>,
    y: Vec<usize>,
    z: Vec<usize>,
}

pub struct Perlin {
    color: DVec3,
    rng_dvec3: Vec<DVec3>,
    perm: PermutationXyz,
}

impl Perlin {
    pub fn new(c: DVec3) -> Self {
        Self {
            color: c,
            rng_dvec3: rand_utils::rand_vec_dvec3(POINTS),
            perm: PermutationXyz {
                x: rand_utils::perm_n(POINTS),
                y: rand_utils::perm_n(POINTS),
                z: rand_utils::perm_n(POINTS),
            },
        }
    }

    pub fn color_at(&self, p: DVec3) -> DVec3 {
        self.color
            * self._scale_turb(p.x,self.turbulence(0.0, SCALE*p.abs(), 0))

    }

    fn _scale_turb(&self, px: f64, t: f64) -> f64 {
        1.0 - (0.5 + 0.5*(FREQ * px + AMP * t).sin()).powf(6.0)
    }

    fn turbulence(&self, acc: f64, p: DVec3, depth: usize) -> f64 {
        if depth >= OCTAVES {
            return acc;
        }
        let w = GAIN.powf(depth as f64);

        self.turbulence(acc + w*self.noise_at(p).abs(), 2.0 * p, depth + 1)
    }

    fn noise_at(&self, p: DVec3) -> f64 {
        let weight = p.fract();
        let floor = p.floor();

        let gradients = (0..2).cartesian_product(0..2)
            .cartesian_product(0..2).map(|((i,j),k)| {
                self.rng_dvec3[
                    self._hash(
                        floor.x as usize + i,
                        floor.y as usize + j,
                        floor.z as usize + k
                    )
                ]
        }).collect();

        self.interp(gradients, self._smootherstep(weight))
    }

    fn _hash(&self, x: usize, y: usize, z: usize) -> usize {
        self.perm.x[x % POINTS]
            ^ self.perm.y[y % POINTS]
            ^ self.perm.z[z % POINTS]
    }

    fn _hermite_cubic(&self, x: DVec3) -> DVec3 {
        (3.0 - 2.0*x)*x*x
    }

    fn _smootherstep(&self, x: DVec3) -> DVec3 {
        ((6.0*x - 15.0)*x + 10.0)*x*x*x
    }

    /* trilinear interpolation */
    fn interp(&self, gradients: Vec<DVec3>, w: DVec3) -> f64 {
        (0..2).cartesian_product(0..2)
            .cartesian_product(0..2).zip(gradients).map(|(((x,y),z), g)| {
                let idx = UVec3::new(x, y, z).as_dvec3();
                let widx = 2.0*w*idx + DVec3::ONE - w - idx;

                widx.x * widx.y * widx.z * g.dot(w - idx)
            }).fold(0.0, |acc, v| acc + v)
    }
}
