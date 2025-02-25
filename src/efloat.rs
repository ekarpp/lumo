use std::ops::{Add, Div, Mul, Neg, Sub};
use crate::Float;

/// Used for error estimation in manually propagated floating point errors
pub fn gamma(n: i32) -> Float {
    let n = n as Float;
    (n * Float::EPSILON) / (1.0 - n * Float::EPSILON)
}

/// Makes the smallest increment possible to `v`
pub fn next_float(v: Float) -> Float {
    if v.is_infinite() && v > 0.0 {
        v
    } else {
        let v = if v == -0.0 { 0.0 } else { v };
        let bits = if v >= 0.0 {
            v.to_bits() + 1
        } else {
            v.to_bits() - 1
        };
        Float::from_bits(bits)
    }
}

/// Makes the smalles decrement possible to `v`
pub fn previous_float(v: Float) -> Float {
    if v.is_infinite() && v < 0.0 {
        v
    } else {
        let v = if v == 0.0 { -0.0 } else { v };
        let bits = if v > 0.0 {
            v.to_bits() - 1
        } else {
            v.to_bits() + 1
        };
        Float::from_bits(bits)
    }
}

/// `Float` with running floating point error tracking
#[derive(Copy, Clone)]
pub struct EFloat {
    /// Actual `Float` value
    pub value: Float,
    /// Lower bound of error interval
    pub low: Float,
    /// Higher bound of error interval
    pub high: Float,
}

impl EFloat {
    fn new(value: Float, low: Float, high: Float) -> Self {
        Self {
            value,
            low,
            high,
        }
    }

    pub fn sqrt(&self) -> Self {
        Self::new(
            self.value.sqrt(),
            previous_float(self.low.sqrt()),
            next_float(self.high.sqrt()),
        )
    }

    pub fn quadratic(a: Self, b: Self, c: Self) -> Option<(Self, Self)> {
        let disc = b.value * b.value - 4.0 * a.value * c.value;
        if disc < 0.0 {
            return None;
        }
        let disc_root = Self::from(disc).sqrt();

        let mut t0 = (-b - disc_root) / (Self::from(2.0) * a);
        let mut t1 = (-b + disc_root) / (Self::from(2.0) * a);

        if t0.value > t1.value {
            std::mem::swap(&mut t0, &mut t1);
        }

        // t0 always lower value
        Some((t0, t1))
    }

    pub fn abs_error(&self) -> Float {
        next_float(
            (self.high - self.value).abs().max((self.value - self.low).abs())
        )
    }
}

impl From<Float> for EFloat {
    fn from(value: Float) -> Self {
        Self::new(
            value,
            value,
            value,
        )
    }
}

impl Neg for EFloat {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(
            -self.value,
            -self.low,
            -self.high,
        )
    }
}

impl Add for EFloat {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(
            self.value + other.value,
            previous_float(self.low + other.low),
            next_float(self.high + other.high),
        )
    }
}

impl Sub for EFloat {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(
            self.value - other.value,
            previous_float(self.low - other.high),
            next_float(self.high - other.low),
        )
    }
}

impl Mul for EFloat {
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
            previous_float(min),
            next_float(max),
        )
    }
}

impl Div for EFloat {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        if other.low < 0.0 && other.high > 0.0 {
            // possible division by zero. just make interval everything..
            Self::new(
                self.value / other.value,
                crate::NEG_INF,
                crate::INF,
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
                previous_float(min),
                next_float(max),
            )
        }
    }
}
