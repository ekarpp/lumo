use super::*;
use crate::rng::Xorshift;

const NUM_SAMPLES: usize = 10_000;

fn rand_mat4(rng: &mut Xorshift) -> Mat4 {
    Mat4::new(
        rng.gen_vec4(),
        rng.gen_vec4(),
        rng.gen_vec4(),
        rng.gen_vec4(),
    )
}

#[test]
fn transpose() {
    let mut rng = Xorshift::default();

    for _ in 0..NUM_SAMPLES {
        let a = rand_mat4(&mut rng);
        let t = a.transpose();

        assert!(a == t.transpose());
    }
}

#[test]
fn mul() {
    let mut rng = Xorshift::default();

    for _ in 0..NUM_SAMPLES {
        let a = rand_mat4(&mut rng);
        let at = a.transpose();

        let b = rand_mat4(&mut rng);
        let bt = b.transpose();

        assert!(at.transpose() * Mat4::ID == a);
        assert!(Mat4::ID * at.transpose() == a);

        assert!(bt.transpose() * Mat4::ID == b);
        assert!(Mat4::ID * bt.transpose() == b);

        let aa = at.transpose();
        let bb = bt.transpose();
        assert!((aa * bb).transpose() == bt * at);
        let at = a.transpose();
        let bt = b.transpose();
        assert!((b * a).transpose() == at * bt);
    }
}
