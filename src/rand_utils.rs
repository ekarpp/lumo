use crate::DVec3;
use std::f64::consts::PI;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::prelude::SliceRandom;

type MyRng = ThreadRng;
fn get_rng() -> MyRng {
    rand::thread_rng()
}

/* should figure better way to rng creation??
 * (thread_rng() always creates new?) */

pub fn rand_f64() -> f64 {
    get_rng().gen()
}

/* return n normalized random dvec3 in a vector */
pub fn rand_vec_dvec3(n: usize) -> Vec<DVec3> {
    let mut rng = get_rng();
    (0..n).map(|_| rand_unit_sphere(&mut rng)).collect()
}

/* uniform random DVec3 in unit sphere */
fn rand_unit_sphere(rng: &mut MyRng) -> DVec3 {
    let r1: f64 = rng.gen();
    let r2: f64 = rng.gen();

    DVec3::new(
        (2.0*PI*r1).cos()*2.0*(r2*(1.0 - r2)).sqrt(),
        (2.0*PI*r1).sin()*2.0*(r2*(1.0 - r2)).sqrt(),
        1.0 - 2.0*r2,
    )
}

/* random permutation of 0..n */
pub fn perm_n(n: usize) -> Vec<usize> {
    let mut v: Vec<usize> = (0..n).collect();
    v.shuffle(&mut get_rng());
    v
}
