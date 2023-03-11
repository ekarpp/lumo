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

pub fn rand_f64() -> f64 {
    _get_rng().gen()
}

/* return n normalized random dvec3 in a vector */
pub fn rand_vec_dvec3(n: usize) -> Vec<DVec3> {
    (0..n).map(|_| rand_unit_sphere()).collect()
}

/* uniform random DVec2 in unit square */
pub fn rand_unit_square() -> DVec2 {
    DVec2::new(rand_f64(), rand_f64())
}

/* concentric map of the unit square to unit disk */
pub fn unit_square_to_unit_disk(rand_sq: DVec2) -> DVec2 {
    /* map [0,1]^2 to [-1,1]^2 */
    let offset = 2.0*rand_sq - DVec2::ONE;

    if offset.x == 0.0 && offset.y == 0.0 {
        DVec2::ZERO
    } else {
        let r = if offset.x.abs() > offset.y.abs() { offset.x } else { offset.y };
        let theta = if offset.x.abs() > offset.y.abs() {
            PI * (offset.y / offset.x ) / 4.0
        } else {
            2.0 * PI  - PI * (offset.x / offset.y) / 4.0
        };

        r * DVec2::new(theta.cos(), theta.sin())
    }
}

/* uniform random DVec3 in unit sphere */
pub fn rand_unit_sphere() -> DVec3 {
    let r1 = rand_f64();
    let r2 = rand_f64();

    DVec3::new(
        (2.0*PI*r1).cos()*2.0*(r2*(1.0 - r2)).sqrt(),
        (2.0*PI*r1).sin()*2.0*(r2*(1.0 - r2)).sqrt(),
        1.0 - 2.0*r2,
    )
}

/* random permutation of 0..n */
pub fn perm_n(n: usize) -> Vec<usize> {
    let mut v: Vec<usize> = (0..n).collect();
    v.shuffle(&mut _get_rng());
    v
}
