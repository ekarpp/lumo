use glam::f64::DVec3;
use rand::Rng;

pub fn rand_f64() -> f64 {
    rand::thread_rng().gen()
}

pub fn rand_dvec3() -> DVec3 {
    let mut rng = rand::thread_rng();
    DVec3::new(
        rng.gen(),
        rng.gen(),
        rng.gen()
    )
}
