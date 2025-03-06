use super::*;
use crate::rng::Xorshift;

const NUM_SAMPLES: usize = 10_000;
const NUM_TFORM: usize = 8;

fn rand_transform(rng: &mut Xorshift) -> Transform {
    let mut out = Transform::mat3(Mat3::ID);
    let step = 1.0 / 6.0;

    for _ in 0..NUM_TFORM {
        let t = match rng.gen_float() {
            x if x < step => {
                Transform::scale(
                    rng.gen_float(), rng.gen_float(), rng.gen_float()
                )
            }
            x if x < 2.0 * step => {
                Transform::translation(
                    rng.gen_float(), rng.gen_float(), rng.gen_float()
                )
            }
            x if x < 3.0 * step => Transform::rotate_x(rng.gen_float()),
            x if x < 4.0 * step => Transform::rotate_y(rng.gen_float()),
            x if x < 5.0 * step => Transform::rotate_z(rng.gen_float()),
            x if x < 6.0 * step => {
                Transform::perspective(rng.gen_float(), 1e3 * rng.gen_float())
            }
            _ => unreachable!(),
        };

        out = out * t;
    }

    out
}

#[test]
fn inv() {
    let mut rng = Xorshift::default();

    for _ in 0..NUM_SAMPLES {
        let a = rand_transform(&mut rng);

        let p = rng.gen_vec3();

        let ptp = a.transform_pt(p);
        let ptp = a.transform_pt_inv(ptp);
        assert!(p.distance_squared(ptp) < crate::EPSILON);


        if a.row(3).w == 1.0 {
            let ptd = a.transform_dir(p);
            let ptd = a.transform_dir_inv(ptd);
            assert!(p.distance_squared(ptd) < crate::EPSILON);
        }
    }
}
