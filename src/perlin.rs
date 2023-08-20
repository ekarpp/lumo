use crate::{Float, Vec3, rand_utils};
use itertools::Itertools;

/// Number of points in the perlin noise lattice
const PERLIN_POINTS: usize = 256;

/// Helper struct to store permutation vectors for each dimension.
struct PermutationXyz {
    x: Vec<usize>,
    y: Vec<usize>,
    z: Vec<usize>,
}

/// Perlin noise generator.
pub struct Perlin {
    /// Random normals of the Perlin lattice
    lattice: Vec<Vec3>,
    /// Permutation directions
    perm: PermutationXyz,
}

impl Default for Perlin {
    fn default() -> Self {
        Self {
            lattice: rand_utils::rand_vec_vec3(PERLIN_POINTS),
            perm: PermutationXyz {
                x: rand_utils::perm_n(PERLIN_POINTS),
                y: rand_utils::perm_n(PERLIN_POINTS),
                z: rand_utils::perm_n(PERLIN_POINTS),
            },
        }
    }
}

impl Perlin {
    /// Computes Perlin noise at point `p`
    pub fn noise_at(&self, p: Vec3) -> Float {
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
    fn _hermite_cubic(&self, x: Vec3) -> Vec3 {
        (3.0 - 2.0 * x) * x * x
    }

    /// Smoothing for weights
    fn _smootherstep(&self, x: Vec3) -> Vec3 {
        ((6.0 * x - 15.0) * x + 10.0) * x * x * x
    }

    /// Trilinear interpolation
    ///
    /// # Arguments
    /// * `normals` - Normals to perform interpolation with
    /// * `w` - Fractional part of the point. Gives distances to each normal.
    fn interp(&self, normals: Vec<Vec3>, w: Vec3) -> Float {
        (0..2)
            .cartesian_product(0..2)
            .cartesian_product(0..2)
            .zip(normals)
            .map(|(((x, y), z), norm)| {
                let idx = Vec3::new(
                    x as Float,
                    y as Float,
                    z as Float,
                );
                let widx = 2.0 * w * idx + Vec3::ONE - w - idx;

                widx.x * widx.y * widx.z * norm.dot(w - idx)
            })
            .fold(0.0, |acc, v| acc + v)
    }
}
