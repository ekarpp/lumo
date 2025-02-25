use super::*;
use std::str::FromStr;

const BINS: usize = 10;
const BIN_STEP: Float = (LAMBDA_MAX - LAMBDA_MIN) / BINS as Float;

#[derive(Clone)]
/// Represents a color spectrum at `BINS` bins of uniform width
/// between `LAMBDA_MIN` and `LAMBDA_MAX` wavelengths.
pub struct Spectrum {
    bins: [Float; BINS],
}

impl Spectrum {
    /// White spectrum, Smits 1999
    pub const WHITE: Self = Self { bins: [1.0, 1.0, 0.9999, 0.9993, 0.9992, 0.9998, 1.0, 1.0, 1.0, 1.0] };
    /// Black spectrum, Smits 1999
    pub const BLACK: Self = Self { bins: [0.0; BINS] };

    /// Cyan spectrum, Smits 1999
    pub const CYAN: Self = Self { bins: [0.971, 0.9426, 1.0007, 1.0007, 1.0007, 1.0007, 0.1564, 0.0, 0.0, 0.0] };
    /// Magenta spectrum, Smits 1999
    pub const MAGENTA: Self = Self { bins: [1.0, 1.0, 0.9685, 0.2229, 0.0, 0.458, 0.8369, 1.0, 1.0, 0.9959] };
    /// Yello spectrum, Smits 1999
    pub const YELLOW: Self = Self { bins: [0.0001, 0.0, 0.1088, 0.6651, 1.0, 1.0, 0.9996, 0.9586, 0.9685, 0.984] };

    /// Red spectrum, Smits 1999
    pub const RED: Self = Self { bins: [0.1012, 0.0515, 0.0, 0.0, 0.0, 0.0, 0.8325, 1.0149, 1.0149, 1.0149] };
    /// Green spectrum, Smits 1999
    pub const GREEN: Self = Self { bins: [0.0, 0.0, 0.0273, 0.7937, 1.0, 0.9418, 0.1719, 0.0, 0.0, 0.0025] };
    /// Blue spectrum, Smits 1999
    pub const BLUE: Self = Self { bins: [1.0, 1.0, 0.8916, 0.3323, 0.0, 0.0, 0.0003, 0.0369, 0.0483, 0.0496] };

    /// Maps rgb vector from `[0,1]^3` to a spectrum as described by Smits 1999
    pub fn from_rgb(rgb: Vec3) -> Self {
        let (r, g, b) = (rgb.x, rgb.y, rgb.z);

        // Smits 1999
        if r <= g && r <= b {
            if g <= b {
                r * Self::WHITE
                    + (g - r) * Self::CYAN
                    + (b - g) * Self::BLUE
            } else {
                r * Self::WHITE
                    + (b - r) * Self::CYAN
                    + (g - b) * Self::GREEN
            }
        } else if g <= r && g <= b {
            if r <= b {
                g * Self::WHITE
                    + (r - g) * Self::MAGENTA
                    + (b - r) * Self::BLUE
            } else {
                g * Self::WHITE
                    + (b - g) * Self::MAGENTA
                    + (r - b) * Self::RED
            }
        } else if r <= g /* && b <= r && b <= g */ {
            b * Self::WHITE
                + (r - b) * Self::YELLOW
                + (g - r) * Self::GREEN
        } else /* if r > g && b <= r && b <= g */ {
            b * Self::WHITE
                + (g - b) * Self::YELLOW
                + (r - g) * Self::RED
        }
    }

    /// Maps gamma encoded sRGB values `r`, `g`, `b` to a spectrum
    pub fn from_srgb(r: u8, g: u8, b: u8) -> Self {
        let dec = |v: u8| -> Float {
            let u = v as Float / 255.0;
            if u <= 0.04045 {
                u / 12.92
            } else {
                ((u + 0.055) / 1.055).powf(2.4)
            }
        };
        Self::from_rgb(Vec3::new(dec(r), dec(g), dec(b)))
    }

    /// Parse string of format `(<wavelength>:<intensity> )*` to a spectrum
    pub fn from_pts(pts: &str) -> Self {
        let mut pairs: Vec<(Float, Float)> = pts.split_whitespace()
            .filter_map(|pt| {
                let (lambda, i) = pt.split_once(":")?;
                let lambda = Float::from_str(lambda).ok()?;
                let i = Float::from_str(i).ok()?;
                Some( (lambda, i) )
            })
            .collect();
        pairs.sort_by(|l, r| l.0.total_cmp(&r.0));

        let bins: [Float; BINS] = (0..BINS)
            .map(|i| {
                let l_min = LAMBDA_MIN + i as Float * BIN_STEP;

                let f = |x: Float| -> Float {
                    let Some(rp) = pairs.iter()
                        .find(|p| p.0 >= x) else { return 0.0; };
                    let Some(lp) = pairs.iter()
                        .filter(|p| p.0 < x).last() else { return 0.0; };

                    let u = (x - lp.0) / (rp.0 - lp.0);

                    (1.0 - u) * lp.1 + u * rp.1
                };

                let simpson_steps = 20;
                let step_size = BIN_STEP / simpson_steps as Float;

                let ig = (0..simpson_steps).fold(0.0, |acc, i| {
                    let x0 = l_min + i as Float * step_size;
                    let x1 = l_min + (i + 1) as Float * step_size;

                    let simpson = (f(x0) + 4.0 * f((x0 + x1) / 2.0) + f(x1))
                        * (x1 - x0) / 6.0;
                    acc + simpson
                });

                ig / BIN_STEP
            })
            .collect::<Vec<Float>>().try_into().unwrap();

        Self { bins }
    }

    /// Samples `self` at `lambda` wavelengths
    pub fn sample(&self, lambda: &ColorWavelength) -> Color {
        let samples: [Float; SPECTRUM_SAMPLES] = lambda.iter()
            .map(|wl| self.sample_one(*wl))
            .collect::<Vec<Float>>().try_into().unwrap();
        Color::from(samples)
    }

    /// Samples self at a single wavelength `lambda`
    pub fn sample_one(&self, lambda: Float) -> Float {
        let bin = ((lambda - LAMBDA_MIN) / BIN_STEP).floor() as usize;
        self.bins[bin]
    }

    /// Are we a constant black spectrum?
    pub fn is_black(&self) -> bool {
        self.bins.iter().all(|v| v == &0.0)
    }
}

impl AddAssign<Spectrum> for Spectrum {
    fn add_assign(&mut self, rhs: Spectrum) {
        for i in 0..BINS {
            self.bins[i] += rhs.bins[i];
        }
    }
}

impl Add for Spectrum {
    type Output = Self;

    fn add(self, rhs: Spectrum) -> Self::Output {
        let bins = self.bins.iter().zip(rhs.bins.iter())
            .map(|(lhs, rhs)| { lhs + rhs })
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { bins }
    }
}

impl Mul<Spectrum> for Float {
    type Output = Spectrum;

    fn mul(self, rhs: Spectrum) -> Spectrum {
        let bins: [Float; BINS] = rhs.bins.iter()
            .map(|v| v * self)
            .collect::<Vec<Float>>().try_into().unwrap();
        Spectrum { bins }
    }
}
