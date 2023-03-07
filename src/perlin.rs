use crate::{DVec3, UVec3};
use crate::rand_utils;
use itertools::Itertools;

const POINTS: usize = 256;
const FREQ: f64 = 3.0;
const AMP: f64 = 15.0;

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
    pub fn new(c: DVec3) -> Perlin {
        Perlin {
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
        self.color * 0.5 *
            (1.0 + (FREQ * (p.y + AMP *
                             self.turbulence(0.0, p.abs(), 0))).sin())

    }

    fn turbulence(&self, acc: f64, p: DVec3, depth: usize) -> f64 {
        if depth > 10 {
            return acc.abs();
        }
        let w = 0.5_f64.powf(depth as f64);

        self.turbulence(acc + w*self.noise_at(p), 2.0 * p, depth + 1)
    }

    fn noise_at(&self, p: DVec3) -> f64 {
        let weight = p.fract();
        let x = p.x.floor() as usize;
        let y = p.y.floor() as usize;
        let z = p.z.floor() as usize;

        let gradients = (0..2).cartesian_product(0..2)
            .cartesian_product(0..2).map(|((i,j),k)| {
                self.rng_dvec3[
                    self.perm.x[(x + i) % POINTS]
                        ^ self.perm.y[(y + j) % POINTS]
                        ^ self.perm.z[(z + k) % POINTS]
                ]
        }).collect();

        self.interp(gradients, self._smootherstep(weight))
    }

    fn _hermite_cubic(&self, x: DVec3) -> DVec3 {
        (3.0 - 2.0*x)*x*x
    }

    fn _smootherstep(&self, x: DVec3) -> DVec3 {
        ((6.0*x - 15.0)*x + 10.0)*x*x*x
    }

    fn interp(&self, gradients: Vec<DVec3>, w: DVec3) -> f64 {
        (0..2).cartesian_product(0..2)
            .cartesian_product(0..2).zip(gradients).map(|(((x,y),z), g)| {
                let idx = UVec3::new(x, y, z).as_dvec3();
                let widx = 2.0*w*idx + DVec3::ONE - w - idx;

                widx.x * widx.y * widx.z * g.dot(w - idx)
            }).fold(0.0, |acc, v| acc + v)
    }
}
