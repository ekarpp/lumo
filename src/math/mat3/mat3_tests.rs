use super::*;
use crate::rng::Xorshift;

const NUM_SAMPLES: usize = 10_000;

#[test]
fn transpose() {
    let mut rng = Xorshift::default();

    for _ in 0..NUM_SAMPLES {
        let a = Mat3::new(rng.gen_vec3(), rng.gen_vec3(), rng.gen_vec3());
        let t = a.transpose();

        assert!(a == t.transpose());
    }
}

#[test]
fn det() {
    let mut rng = Xorshift::default();

    assert!(Mat3::ID.det() == 1.0);

    for _ in 0..NUM_SAMPLES {
        let a = Mat3::new(rng.gen_vec3(), rng.gen_vec3(), rng.gen_vec3());
        let ad = a.det();
        let at = a.transpose();
        let atd = at.det();

        let b = Mat3::new(rng.gen_vec3(), rng.gen_vec3(), rng.gen_vec3());
        let bd = b.det();
        let bt = b.transpose();
        let btd = bt.det();

        let c = a * b;
        let cd = c.det();
        let ct = c.transpose();
        let ctd = ct.det();

        assert!((ad - atd).abs() < crate::EPSILON);
        assert!((bd - btd).abs() < crate::EPSILON);
        assert!((cd - ctd).abs() < crate::EPSILON);
        assert!((ad * bd - cd).abs() < crate::EPSILON);
    }
}

#[test]
fn inv() {
    let mut rng = Xorshift::default();

    for _ in 0..NUM_SAMPLES {
        let a = Mat3::new(rng.gen_vec3(), rng.gen_vec3(), rng.gen_vec3());
        let ad = a.det();
        let b = a.inv();
        let c = a * b;

        if ad == 0.0 {
            // only check the diagonal
            assert!(c.y0.x.is_nan()
                    && c.y1.y.is_nan()
                    && c.y2.z.is_nan());
        } else {
            assert!((1.0 - c.det()).abs() < 1e-5);
        }
    }
}
