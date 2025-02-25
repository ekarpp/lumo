use crate::Float;
use std::ops::{
    Add, Sub, Mul, Div
};

#[cfg(test)]
mod complex_tests;

#[derive(Clone, Copy)]
#[allow(non_snake_case)]
pub struct Complex {
    Re: Float,
    Im: Float,
}

impl Complex {
    #[allow(non_snake_case)]
    pub fn new(Re: Float, Im: Float) -> Self {
        Self { Re, Im }
    }

    /// Squared norm
    pub fn norm_sqr(&self) -> Float {
        self.Re * self.Re + self.Im * self.Im
    }

    /// Complement: Co(x + iy) = x - iy
    pub fn co(&self) -> Complex {
        Self {
            Re: self.Re,
            Im: -self.Im,
        }
    }

    /// sqrt(re^(ix)) = sqrt(r) e^(ix/2)
    pub fn sqrt(&self) -> Complex {
        Complex {
            Re: self.norm().sqrt() * (self.arg() / 2.0).cos(),
            Im: self.norm().sqrt() * (self.arg() / 2.0).sin(),
        }
    }

    fn norm(&self) -> Float {
        self.norm_sqr().sqrt()
    }

    fn arg(&self) -> Float {
        libm::atan2(self.Im, self.Re)
    }
}

impl From<Float> for Complex {
    fn from(v: Float) -> Self {
        Self {
            Re: v,
            Im: 0.0,
        }
    }
}

impl Add<Complex> for Complex {
    type Output = Self;

    fn add(self, rhs: Complex) -> Self::Output {
        Self {
            Re: self.Re + rhs.Re,
            Im: self.Im + rhs.Im,
        }
    }
}

impl Add<Complex> for Float {
    type Output = Complex;

    fn add(self, rhs: Complex) -> Self::Output {
        Complex {
            Re: self + rhs.Re,
            Im: rhs.Im,
        }
    }
}

impl Sub<Complex> for Complex {
    type Output = Self;

    fn sub(self, rhs: Complex) -> Self::Output {
        Self {
            Re: self.Re - rhs.Re,
            Im: self.Im - rhs.Im,
        }
    }
}

impl Sub<Complex> for Float {
    type Output = Complex;

    fn sub(self, rhs: Complex) -> Self::Output {
        Complex {
            Re: self - rhs.Re,
            Im: -rhs.Im,
        }
    }
}

impl Mul<Complex> for Complex {
    type Output = Self;

    fn mul(self, rhs: Complex) -> Self::Output {
        Self {
            Re: self.Re * rhs.Re - self.Im * rhs.Im,
            Im: self.Re * rhs.Im + self.Im * rhs.Re,
        }
    }
}

impl Mul<Float> for Complex {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self {
            Re: self.Re * rhs,
            Im: self.Im * rhs,
        }
    }
}

impl Mul<Complex> for Float {
    type Output = Complex;

    fn mul(self, rhs: Complex) -> Self::Output {
        Complex {
            Re: self * rhs.Re,
            Im: self * rhs.Im,
        }
    }
}

impl Div<Complex> for Complex {
    type Output = Self;

    fn div(self, rhs: Complex) -> Self::Output {
        if rhs.Re == 0.0 && rhs.Im == 0.0 {
            Self {
                Re: Float::NAN,
                Im: Float::NAN,
            }
        } else {
            self * rhs.co() / rhs.norm_sqr()
        }
    }
}

impl Div<Float> for Complex {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        if rhs == 0.0 {
            Complex {
                Re: Float::NAN,
                Im: Float::NAN,
            }
        } else {
            Complex {
                Re: self.Re / rhs,
                Im: self.Im / rhs,
            }
        }
    }
}

impl Div<Complex> for Float {
    type Output = Complex;

    fn div(self, rhs: Complex) -> Self::Output {
        if rhs.Re == 0.0 && rhs.Im == 0.0 {
            Complex {
                Re: Float::NAN,
                Im: Float::NAN,
            }
        } else {
            self * rhs.co() / rhs.norm_sqr()
        }
    }
}
