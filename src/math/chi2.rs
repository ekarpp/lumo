#![allow(clippy::excessive_precision)]
use crate::Float;

const BIG: Float = 4.503599627370496e15;
const BIG_INV: Float = 2.22044604925031308085e-16;
const MAXLOG: Float = 7.097827128933839967322e2;
const EPS: Float = 1.11022302462515654042e-16;

/// Chi^2 distirbution CDF using incomplete gamma functions
pub fn chi2_cdf(dof: usize, stat: Float) -> Float {
    if dof < 1 || stat <= 0.0 {
        0.0
    } else if dof == 2 {
        1.0 - (-0.5 * stat).exp()
    } else {
        gamma_p(dof as Float / 2.0, stat / 2.0)
    }
}

/// Normalized lower incomplete gamma function. Implementation from PBRT/Cepeh.
fn gamma_p(a: Float, x: Float) -> Float {
    #[cfg(debug_assertions)]
    {
        assert!(x > 0.0);
        assert!(a > 0.0 && a != 1.0);
    }

    let ax = a * x.ln() - x - libm::lgamma(a as f64) as Float;
    if ax < -MAXLOG {
        return if x < a { 0.0 } else { 1.0 };
    }

    if x <= 1.0 || x <= a {
        let mut r = a;
        let mut c = 1.0;
        let mut ans = 1.0;

        loop {
            r += 1.0;
            c *= x / r;
            ans += c;
            if c / ans <= EPS {
                break;
            }
        }
        return ax.exp() * ans / a;
    }

    let mut c = 0.0;
    let mut y = 1.0 - a;
    let mut z = x + y + 1.0;
    let mut p3 = 1.0;
    let mut q3 = x;
    let mut p2 = x + 1.0;
    let mut q2 = z * x;
    let mut ans = p2 / q2;
    let mut err;

    loop {
        c += 1.0;
        y += 1.0;
        z += 2.0;
        let yc = y * c;
        let p = p2 * z - p3 * yc;
        let q = q2 * z - q3 * yc;

        if q == 0.0 {
            err = 1.0;
        } else {
            let r = p / q;
            err = ((ans - r) / r).abs();
            ans = r;
        }
        if err <= EPS {
            break;
        }

        p3 = p2;
        p2 = p;
        q3 = q2;
        q2 = q;

        if p.abs() > BIG {
            p3 *= BIG_INV;
            p2 *= BIG_INV;
            q3 *= BIG_INV;
            q2 *= BIG_INV;
        }
    }

    1.0 - ans * ax.exp()
}
