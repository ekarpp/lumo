use crate::{Vec2, Vec3, Float};
use rand::{
    prelude::SliceRandom,
    rngs::ThreadRng, Rng
};

type MyRng = ThreadRng;
fn _get_rng() -> MyRng {
    rand::thread_rng()
}

/* should figure better way to rng creation??
 * (thread_rng() always creates new?) */

/// Random Float
pub fn rand_float() -> Float {
    _get_rng().gen()
}

/// return `n` normalized random Vec3s in a vector
pub fn rand_vec_vec3(n: usize) -> Vec<Vec3> {
    (0..n).map(|_| square_to_sphere(unit_square())).collect()
}

/// Random permutation of 0..n
pub fn perm_n(n: usize) -> Vec<usize> {
    let mut v: Vec<usize> = (0..n).collect();
    v.shuffle(&mut _get_rng());
    v
}

/// Point uniformly at random in unit square
pub fn unit_square() -> Vec2 {
    Vec2::new(rand_float(), rand_float())
}

/// Concentric map of unit square to unit disk. Shirley & Chiu 97
pub fn square_to_disk(rand_sq: Vec2) -> Vec2 {
    /* map [0,1]^2 to [-1,1]^2 */
    let offset = 2.0 * rand_sq - Vec2::ONE;

    if offset.x == 0.0 && offset.y == 0.0 {
        Vec2::ZERO
    } else {
        let (r, theta) = if offset.x.abs() > offset.y.abs() {
            (
                offset.x,
                crate::PI * (offset.y / offset.x) / 4.0
            )
        } else {
            (
                offset.y,
                crate::PI * (0.5 - (offset.x / offset.y) / 4.0)
            )
        };

        r * Vec2::new(theta.cos(), theta.sin())
    }
}

/// Cosine weighed random point ON hemisphere pointing towards +z.
/// Malley's method i.e. lift unit disk to 3D
pub fn square_to_cos_hemisphere(rand_sq: Vec2) -> Vec3 {
    let rand_disk = square_to_disk(rand_sq);
    let z = (1.0 - rand_disk.x * rand_disk.x - rand_disk.y * rand_disk.y)
        .max(0.0)
        .sqrt();

    rand_disk.extend(z)
}

/// Uniform random point in hemisphere with Z up
#[cfg(test)]
pub fn square_to_hemisphere(rand_sq: Vec2) -> Vec3 {
    let z = rand_sq.x;
    let r = (1.0 - z * z).max(0.0).sqrt();
    let phi = 2.0 * crate::PI * rand_sq.y;

    Vec3::new(r * phi.cos(), r * phi.sin(), z)
}

/// Uniform random point IN unit sphere
pub fn square_to_sphere(rand_sq: Vec2) -> Vec3 {
    let z = 1.0 - 2.0 * rand_sq.y;
    let r = (1.0 - z * z).sqrt();
    let phi = 2.0 * crate::PI * rand_sq.x;

    Vec3::new(r * phi.cos(), r * phi.sin(), z)
}
