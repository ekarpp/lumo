use super::*;

#[derive(Clone)]
/// Struct to represent the wavelengths a `Color` is sampled at
pub struct ColorWavelength {
    lambda: [Float; SPECTRUM_SAMPLES],
}

impl Default for ColorWavelength {
    fn default() -> Self {
        let step = (LAMBDA_MAX - LAMBDA_MIN) / SPECTRUM_SAMPLES as Float;

        let lambda = (0..SPECTRUM_SAMPLES)
            .map(|i| LAMBDA_MIN + i as Float * step)
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { lambda }
    }
}

impl ColorWavelength {
    /// PDF for the sampled wavelengths
    pub fn pdf(&self) -> Float {
        1.0 / (LAMBDA_MAX - LAMBDA_MIN)
    }

    /// Sample a single wavelength uniformly at random
    pub fn sample_one(u: Float) -> Float {
        u * LAMBDA_MIN + (1.0 - u) * LAMBDA_MAX
    }

    /// Sample `SPECTRUM_SAMPLES` wavelengths uniformly at random
    pub fn sample(u: Float) -> Self {
        let l0 = u * LAMBDA_MIN + (1.0 - u) * LAMBDA_MAX;

        let map = |v: Float| -> Float {
            let dl = (LAMBDA_MAX - LAMBDA_MIN) / SPECTRUM_SAMPLES as Float;
            if v + dl <= LAMBDA_MAX {
                v + dl
            } else {
                LAMBDA_MIN + (v + dl - LAMBDA_MAX)
            }
        };

        let mut lambda: [Float; SPECTRUM_SAMPLES] = [l0, 0.0, 0.0, 0.0];
        for i in 1..SPECTRUM_SAMPLES {
            lambda[i] = map(lambda[i-1]);
        }

        Self { lambda }
    }

    /// Iterator to the sampled wavelengths
    pub fn iter(&self) -> impl Iterator<Item=&Float> {
        self.lambda.iter()
    }
}
