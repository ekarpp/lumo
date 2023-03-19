use crate::{DVec3, DVec2};
use std::f64::consts::PI;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::prelude::SliceRandom;

type MyRng = ThreadRng;
fn _get_rng() -> MyRng {
    rand::thread_rng()
}

/* should figure better way to rng creation??
 * (thread_rng() always creates new?) */

/// Random f64
pub fn rand_f64() -> f64 {
    _get_rng().gen()
}

/// return `n` normalized random DVec3s in a vector
pub fn rand_vec_dvec3(n: usize) -> Vec<DVec3> {
    (0..n).map(|_| square_to_sphere(unit_square())).collect()
}

/// Random permutation of 0..n
pub fn perm_n(n: usize) -> Vec<usize> {
    let mut v: Vec<usize> = (0..n).collect();
    v.shuffle(&mut _get_rng());
    v
}

/// Point uniformly at random in unit square
pub fn unit_square() -> DVec2 {
    DVec2::new(rand_f64(), rand_f64())
}

/// Concentric map of unit square to unit disk. Shirley & Chiu 97
pub fn square_to_disk(rand_sq: DVec2) -> DVec2 {
    /* map [0,1]^2 to [-1,1]^2 */
    let offset = 2.0*rand_sq - DVec2::ONE;

    if offset.x == 0.0 && offset.y == 0.0 {
        DVec2::ZERO
    } else {
        let r = if offset.x.abs() > offset.y.abs() {
            offset.x
        } else {
            offset.y
        };
        let theta = if offset.x.abs() > offset.y.abs() {
            PI * (offset.y / offset.x ) / 4.0
        } else {
            2.0 * PI  - PI * (offset.x / offset.y) / 4.0
        };

        r * DVec2::new(theta.cos(), theta.sin())
    }
}

/// Cosine weighed random point ON hemisphere pointing towards +z.
/// Malley's method i.e. lift unit disk to 3D
pub fn square_to_cos_hemisphere(rand_sq: DVec2) -> DVec3 {
    let rand_disk = square_to_disk(rand_sq);
    let z = (1.0
             - rand_disk.x * rand_disk.x
             - rand_disk.y * rand_disk.y).max(0.0).sqrt();

    rand_disk.extend(z)
}

/// Uniform random point IN unit sphere
pub fn square_to_sphere(rand_sq: DVec2) -> DVec3 {
    let z = 1.0 - 2.0 * rand_sq.y;
    let r = (1.0 - z * z).sqrt();
    let phi = 2.0 * PI * rand_sq.x;

    DVec3::new(
        r * phi.cos(),
        r * phi.sin(),
        z,
    )
}
