use super::*;

// Integral of 1.0 / cosh^2(0.0072 * (x - 538)) from LAMBDA_MIN to LAMBDA_MAX
const SAMPLE_VISIBLE_INTEGRAL: Float = 253.819;
/*
 * 138.889 * (3.8736 - 0.0072 * LAMBDA_MIN).atan()
 * - 138.889 * (3.8736 - 0.0072 * LAMBDA_MAX).atan()
 */

#[derive(Clone)]
/// Struct to represent the wavelengths a `Color` is sampled at
pub struct ColorWavelength {
    lambda: [Float; SPECTRUM_SAMPLES],
}

impl Default for ColorWavelength {
    fn default() -> Self {
        Self::sample(0.0)
    }
}

impl ColorWavelength {
    /// PDF for the sampled wavelengths
    pub fn pdf(&self) -> Color {
        let samples: [Float; SPECTRUM_SAMPLES] = self.lambda.iter()
            .map(|lambda| Self::pdf_one(*lambda))
            .collect::<Vec<Float>>().try_into().unwrap();

        Color::from(samples)
    }

    /// Sample `SPECTRUM_SAMPLES` wavelengths with the visible spectrum weighed
    pub fn sample(rand_u: Float) -> Self {
        let mut lambda = [0.0; SPECTRUM_SAMPLES];
        for i in 0..SPECTRUM_SAMPLES {
            let rand_v = rand_u + i as Float / SPECTRUM_SAMPLES as Float;
            let rand_v = if rand_v > 1.0 { rand_v - 1.0 } else { rand_v };
            lambda[i] = Self::sample_one(rand_v);
        }

        Self { lambda }
    }

    /// Sample a single wavelength prefering values in the visible wavelength
    #[inline]
    pub fn sample_one(rand_v: Float) -> Float {
        538.0
            - 138.888889 * (0.85691062 - SAMPLE_VISIBLE_INTEGRAL * rand_v * 0.0072).atanh()
    }

    /// Sample a single wavelength uniformly at random over the whole spectrum
    #[inline]
    pub fn sample_one_uniform(rand_v: Float) -> Float {
        LAMBDA_MIN + rand_v * (LAMBDA_MAX - LAMBDA_MIN)
    }

    fn pdf_one(lambda: Float) -> Float {
        if lambda < LAMBDA_MIN || lambda > LAMBDA_MAX {
            0.0
        } else {
            1.0 / (SAMPLE_VISIBLE_INTEGRAL * (0.0072 * (lambda - 538.05)).cosh().powi(2))
        }
    }

    /// Iterator to the sampled wavelengths
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=&Float> {
        self.lambda.iter()
    }
}
