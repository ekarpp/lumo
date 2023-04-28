use std::ops::{Add, Div, Mul, Neg, Sub};

/// Used for error estimation in manually propagated floating point errors
pub fn gamma(n: i32) -> f64 {
    let n = n as f64;
    (n * f64::EPSILON) / (1.0 - n * f64::EPSILON)
}

/// Makes the smallest increment possible to `v`
fn _next_double(v: f64) -> f64 {
    if v.is_infinite() && v > 0.0 {
        v
    } else {
        let v = if v == -0.0 { 0.0 } else { v };
        let bits = if v >= 0.0 {
            v.to_bits() + 1
        } else {
            v.to_bits() - 1
        };
        f64::from_bits(bits)
    }
}

/// Makes the smalles decrement possible to `v`
fn _previous_double(v: f64) -> f64 {
    if v.is_infinite() && v < 0.0 {
        v
    } else {
        let v = if v == 0.0 { -0.0 } else { v };
        let bits = if v > 0.0 {
            v.to_bits() - 1
        } else {
            v.to_bits() + 1
        };
        f64::from_bits(bits)
    }
}

/// `f64` with running floating point error tracking
#[derive(Copy, Clone)]
pub struct EFloat64 {
    /// Actual `f64` value
    value: f64,
    /// Lower bound of error interval
    low: f64,
    /// Higher bound of error interval
    high: f64,
}

impl EFloat64 {
    fn new(value: f64, low: f64, high: f64) -> Self {
        Self {
            value,
            low,
            high,
        }
    }

    pub fn sqrt(&self) -> Self {
        Self::new(
            self.value.sqrt(),
            _previous_double(self.low.sqrt()),
            _next_double(self.high.sqrt()),
        )
    }

    pub fn quadratic(a: EFloat64, b: EFloat64, c: EFloat64) -> Option<(EFloat64, EFloat64)> {
        let disc = b.value * b.value - 4.0 * a.value * c.value;

        if disc < 0.0 {
            return None;
        }
        let disc_root = EFloat64::from(disc).sqrt();

        let t1 = (-b - disc_root) / (EFloat64::from(2.0) * a);
        let t2 = (-b + disc_root) / (EFloat64::from(2.0) * a);

        Some((t1, t2))
    }
}

impl From<f64> for EFloat64 {
    fn from(value: f64) -> Self {
        Self::new(
            value,
            value,
            value,
        )
    }
}

impl Neg for EFloat64 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(
            -self.value,
            -self.low,
            -self.high,
        )
    }
}

impl Add for EFloat64 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(
            self.value + other.value,
            _previous_double(self.low + other.low),
            _next_double(self.high + other.high),
        )
    }
}

impl Sub for EFloat64 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(
            self.value - other.value,
            _previous_double(self.low - other.high),
            _next_double(self.high - other.low),
        )
    }
}

impl Mul for EFloat64 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let prod_bounds = [
            self.low * other.low,
            self.low * other.high,
            self.high * other.low,
            self.high * other.high,
        ];

        let min = prod_bounds[0]
            .min(prod_bounds[1])
            .min(prod_bounds[2])
            .min(prod_bounds[3]);

        let max = prod_bounds[0]
            .max(prod_bounds[1])
            .max(prod_bounds[2])
            .max(prod_bounds[3]);

        Self::new(
            self.value * other.value,
            _previous_double(min),
            _next_double(max),
        )
    }
}

impl Div for EFloat64 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        if other.low < 0.0 && other.high > 0.0 {
            // possible division by zero. just make interval everything..
            Self::new(
                self.value / other.value,
                f64::NEG_INFINITY,
                f64::INFINITY,
            )
        } else {
            let div_bounds = [
                self.low / other.low,
                self.low / other.high,
                self.high / other.low,
                self.high / other.high,
            ];

            let min = div_bounds[0]
                .min(div_bounds[1])
                .min(div_bounds[2])
                .min(div_bounds[3]);

            let max = div_bounds[0]
                .max(div_bounds[1])
                .max(div_bounds[2])
                .max(div_bounds[3]);

            Self::new(
                self.value / other.value,
                _previous_double(min),
                _next_double(max),
            )
        }
    }
}
