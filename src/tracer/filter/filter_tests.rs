use super::*;
use crate::simpson_integration;

const MAX_R: Float = 5.0;
const STEPS: usize = 1_000;
const TOLERANCE: Float = 1e-3;

#[test]
fn square_zero_radius() {
    let f = |r: Float| -> PixelFilter {
        PixelFilter::square(r)
    };
    test_radius(f);
}

#[test]
fn triangle_zero_radius() {
    let f = |r: Float| -> PixelFilter {
        PixelFilter::triangle(r)
    };
    test_radius(f);
}

#[test]
fn gaussian_05_zero_radius() {
    let f = |r: Float| -> PixelFilter {
        PixelFilter::gaussian(r, 0.5)
    };
    test_radius(f);
}

#[test]
fn gaussian_20_zero_radius() {
    let f = |r: Float| -> PixelFilter {
        PixelFilter::gaussian(r, 2.0)
    };
    test_radius(f);
}

#[test]
fn mitchell_03_zero_radius() {
    let f = |r: Float| -> PixelFilter {
        PixelFilter::mitchell(r, 1.0 / 3.0)
    };
    test_radius(f);
}

#[test]
fn square_integral() {
    let f = |r: Float| -> PixelFilter {
        PixelFilter::square(r)
    };
    test_integral(f);
}

#[test]
fn triangle_integral() {
    let f = |r: Float| -> PixelFilter {
        PixelFilter::triangle(r)
    };
    test_integral(f);
}

#[test]
fn gaussian_05_integral() {
    let f = |r: Float| -> PixelFilter {
        PixelFilter::gaussian(r, 0.5)
    };
    test_integral(f);
}

#[test]
fn gaussian_20_integral() {
    let f = |r: Float| -> PixelFilter {
        PixelFilter::gaussian(r, 2.0)
    };
    test_integral(f);
}

#[test]
fn mitchell_03_integral() {
    let f = |r: Float| -> PixelFilter {
        PixelFilter::mitchell(r, 1.0 / 3.0)
    };
    test_integral(f);
}

fn test_radius<F>(gen: F) where F: Fn(Float) -> PixelFilter {
    let rs = vec!(0.5, 1.0, 1.5, 2.0, 2.5, 1.3333, 0.4213);

    for r in rs {
        let f = gen(r);
        let rr = r + 1e-10;
        assert!(f.eval(Vec2::splat(2.0 * r)) == 0.0);
        assert!(f.eval(Vec2::X * rr) == 0.0);
        assert!(f.eval(Vec2::Y * rr) == 0.0);
    }
}

fn test_integral<F>(gen: F) where F: Fn(Float) -> PixelFilter {
    let rs = vec!(0.5, 1.0, 1.5, 2.0, 2.5, 1.3333);

    for r in rs {
        assert!(r < MAX_R);
        let f = gen(r);
        let ig = f.integral();
        let igs = integrate(&f);

        println!("r = {}: {} {}", r, ig, igs);
        assert!((ig - igs).abs() < TOLERANCE);
    }
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
