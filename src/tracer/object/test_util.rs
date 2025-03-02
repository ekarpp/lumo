macro_rules! test_object {
    ($obj:expr) => {
        #[test]
        fn no_self_intersect() {
            let o = $obj;
            let r = Ray::new(Point::X + crate::EPSILON, Direction::Z);
            assert!(o.hit(&r, 0.0, crate::INF).is_none());
        }

        #[test]
        fn no_intersect_behind() {
            let o = $obj;
            let r = Ray::new(2.0 * Point::X, Direction::X);
            assert!(o.hit(&r, 0.0, crate::INF).is_none());
        }

        #[test]
        fn does_intersect() {
            let o = $obj;
            let p = Point::new(1.23, 4.56, 7.89);
            let r = Ray::new(p, -p);
            assert!(o.hit(&r, 0.0, crate::INF).is_some());
        }

        #[test]
        fn shadow_hit_accurate() {
            let o = $obj;
            let mut rng = Xorshift::default();

            for _ in 0..10_000 {
                let xo = 5.0 * rng::maps::square_to_sphere(rng.gen_vec2());

                let r = Ray::new(xo, Point::ZERO - xo);
                let h = o.hit(&r, 0.0, crate::INF);
                let h_t = o.hit_t(&r, 0.0, crate::INF);

                if let Some(h) = h {
                    println!("{} {}", h.t, h_t);
                    assert!((h.t - h_t).abs() < crate::EPSILON);
                } else {
                    println!("{}", h_t);
                    assert!(h_t.is_infinite());
                }
            }
        }
    };
}

macro_rules! test_sampleable {
    ($smp:expr) => {
        test_util::test_object!($smp);

        const NUM_SAMPLES: usize = 10_000;

        #[test]
        fn sampled_rays_hit() {
            let s = $smp;
            let mut rng = Xorshift::default();

            let xo = 5.0 * rng::maps::square_to_sphere(rng.gen_vec2());
            for _ in 0..NUM_SAMPLES {
                let wi = s.sample_towards(xo, rng.gen_vec2());
                let ri = Ray::new(xo, wi);
                let Some(hi) = s.hit(&ri, 0.0, crate::INF) else { panic!() };

                let p = s.sample_towards_pdf(&ri, hi.p, hi.ng);
                assert!(p > 0.0);
            }
        }

        #[test]
        fn samples_on() {
            let s = $smp;
            let mut rng = Xorshift::default();

            for _ in 0..NUM_SAMPLES {
                let h = s.sample_on(rng.gen_vec2());
                let xo = 0.5 * Point::ONE;
                let xi = h.p;
                let ri = Ray::new(xo, xi - xo);

                let Some(hi) = s.hit(&ri, 0.0, crate::INF) else { panic!() };
                assert!(xi.distance(hi.p) < crate::EPSILON);
            }
        }
    };
}

pub(in crate::tracer::object) use test_object;
pub(in crate::tracer::object) use test_sampleable;
