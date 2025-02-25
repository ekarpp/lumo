use crate::Float;

const EPS: Float = 1e-6;
const MAX_DEPTH: usize = 6;

struct SimpsonIteration {
    a: Float,
    b: Float,
    c: Float,
    fa: Float,
    fb: Float,
    fc: Float,
    integrand: Float,
}

impl SimpsonIteration {
    pub fn new(a: Float, b: Float, c: Float, fa: Float, fb: Float, fc: Float, integrand: Float) -> Self {
        Self { a, b, c, fa, fb, fc, integrand }
    }
}

pub fn simpson2d<F>(f: F, x0: Float, x1: Float, y0: Float, y1: Float) -> Float where
    F: Fn(Float, Float) -> Float {
    assert!(x1 > x0);
    assert!(y1 > y0);

    let g = |y: Float| {
        let h = |x: Float| f(x, y);
        simpson(h, x0, x1)
    };

    simpson(g, y0, y1)
}

pub fn simpson<F>(f: F, x0: Float, x1: Float) -> Float where
    F: Fn(Float) -> Float {

    fn adaptive<G>(f: &G, it: SimpsonIteration, eps: Float, depth: usize) -> Float where
        G: Fn(Float) -> Float {
        let SimpsonIteration { a, b, c, fa, fb, fc, integrand } = it;
        // split to sub intervals
        let (d, e) = (0.5 * (a + b), 0.5 * (b + c));
        let (fd, fe) = (f(d), f(e));

        // integrate subintervals
        let h = c - a;
        let i0 = h * (fa + 4.0 * fd + fb) / 12.0;
        let i1 = h * (fb + 4.0 * fe + fc) / 12.0;
        let i01 = i0 + i1;

        if depth == 0 || (i01 - integrand).abs() < 15.0 * eps {
            i01 + (i01 - integrand) / 15.0
        } else {
            let it1 = SimpsonIteration::new(a, d, b, fa, fd, fb, i0);
            let it2 = SimpsonIteration::new(b, e, c, fb, fe, fc, i1);

            adaptive(f, it1, 0.5 * eps, depth - 1) + adaptive(f, it2, 0.5 * eps, depth - 1)
        }
    }

    let (a, b, c) = (x0, 0.5 * (x0 + x1), x1);
    let (fa, fb, fc) = (f(a), f(b), f(c));
    let integrand = (fa + 4.0 * fb + fc) * (c - a) / 6.0;
    let it = SimpsonIteration { a, b, c, fa, fb, fc, integrand };

    adaptive(&f, it, EPS, MAX_DEPTH)
}
