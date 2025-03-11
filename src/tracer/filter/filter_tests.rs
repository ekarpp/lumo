use super::*;
use crate::math::simpson_integration;

const MAX_R: Float = 3.0;
const STEPS: usize = 1_000;
const TOLERANCE: Float = 1e-2;

macro_rules! test_filter {
    ( $( $name:ident, $f_gen:expr ),* ) => {
        $(
            mod $name {
                use super::*;

                #[test]
                fn zero_radius() {
                    let rs = vec!(0.5, 1.0, 1.5, 2.0, 2.5, 1.3333, 0.4213);

                    for r in rs {
                        let f = $f_gen(r);
                        let rr = r + 1e-10;
                        assert!(f.eval(Vec2::splat(2.0 * r)) == 0.0);
                        assert!(f.eval(Vec2::X * rr) == 0.0);
                        assert!(f.eval(Vec2::Y * rr) == 0.0);
                    }
                }

                #[test]
                fn integral() {
                    let rs = vec!(0.5, 1.0, 1.5, 2.0, 2.5, 1.3333);

                    for r in rs {
                        assert!(r < MAX_R);
                        let f = $f_gen(r);
                        let ig = f.integral();
                        let igs = integrate(&f);

                        println!("r = {}: {} {}", r, ig, igs);
                        assert!((ig - igs).abs() < TOLERANCE);
                    }
                }
            }
        )*
    }
}

test_filter!{
    square, |r: Float| -> PixelFilter { PixelFilter::square(r) },
    triangle, |r: Float| -> PixelFilter { PixelFilter::triangle(r) },
    gaussian_05, |r: Float| -> PixelFilter { PixelFilter::gaussian(r, 0.5) },
    gaussian_15, |r: Float| -> PixelFilter { PixelFilter::gaussian(r, 1.5) },
    gaussian_25, |r: Float| -> PixelFilter { PixelFilter::gaussian(r, 2.5) },
    mitchell_03, |r: Float| -> PixelFilter { PixelFilter::mitchell(r, 1.0 / 3.0) }
}

fn integrate(filter: &PixelFilter) -> Float {
    let f = |x: Float, y: Float| {
        filter.eval(Vec2::new(x, y))
    };

    let step = (2.0 * MAX_R) / STEPS as Float;

    let mut ig = 0.0;
    let mut x0 = -MAX_R;
    let mut y0 = -MAX_R;

    while y0 < MAX_R {
        let y1 = y0 + step;
        while x0 < MAX_R {
            let x1 = x0 + step;
            ig += simpson_integration::simpson2d(f, x0, x1, y0, y1);
            x0 = x1;
        }
        y0 = y1;
        x0 = -MAX_R;
    }

    ig
}
