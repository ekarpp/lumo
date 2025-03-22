use super::*;

/// Number of samples for dense spectrum, sample every 5nm from 360nm to 830nm
// every 5nm for [360, 830]
pub const DENSE_SAMPLES: usize = 95;

/// Spectrum sampled at 5nm steps from 360nm to 830nm
#[derive(Clone)]
pub struct DenseSpectrum {
    values: [Float; DENSE_SAMPLES],
}

impl DenseSpectrum {
    const STEP: Float = (LAMBDA_MAX - LAMBDA_MIN) / (DENSE_SAMPLES as Float - 1.0);

    /// New dense specturm from array of samples
    pub const fn new(values: [Float; DENSE_SAMPLES]) -> Self {
        Self { values }
    }

    /// New constant dense spectrum
    pub const fn from_constant(constant: Float) -> Self {
        Self::new([constant; DENSE_SAMPLES])
    }

    /// Are we a constant spectrum?
    pub fn is_constant(&self) -> bool {
        // cache?
        let lc = self.values[0];
        self.values.iter().all(|v| lc == *v)
    }

    /// New dense spectrum from `(wavelength, value)` pairs (sorted by wavelength)
    pub fn from_points(points: Vec<(Float, Float)>) -> Self {
        assert!(points.is_sorted_by(|l,r| l.0 <= r.0));
        let mut values = [0.0; DENSE_SAMPLES];

        for i in 0..DENSE_SAMPLES {
            let lambda = LAMBDA_MIN + i as Float * Self::STEP;
            let b1 = points.partition_point(|(l, _)| *l < lambda).min(points.len());

            if b1 < points.len() && points[b1].0 == lambda {
                values[i] = points[b1].1;
                continue;
            }

            let (l1, i1) = if b1 == points.len() {
                (lambda, 0.0)
            } else {
                points[b1]
            };

            let (l0, i0) = if b1 == 0 {
                (lambda, 0.0)
            } else {
                points[b1 - 1]
            };
            let dl = l1 - l0;
            let x1 = (lambda - l0) / dl;
            let x0 = 1.0 - x1;

            values[i] = x0 * i0 + x1 * i1;
        }

        Self { values }
    }

    /// Sample the dense spectrum at `lambda`
    pub fn sample(&self, lambda: &ColorWavelength) -> Color {
        let samples: [Float; SPECTRUM_SAMPLES] = lambda.iter()
            .map(|wl| self.sample_one(*wl))
            .collect::<Vec<Float>>().try_into().unwrap();
        Color::from(samples)
    }

    /// Sample the dense spectrum at one wavelength `lambda`
    pub fn sample_one(&self, lambda: Float) -> Float {
        let b1 = ((lambda - LAMBDA_MIN) / Self::STEP).ceil() as usize;
        let l1 = LAMBDA_MIN + Self::STEP * b1 as Float;

        if lambda == 0.0 {
            0.0
        } else if lambda == l1 {
            self.values[b1]
        } else {
            let b0 = b1 - 1;
            let l0 = l1 - Self::STEP;

            let x1 = (lambda - l0) / Self::STEP;
            let x0 = 1.0 - x1;

            let i1 = self.values[b1];
            let i0 = self.values[b0];

            i0 * x0 + i1 * x1
        }
    }

    /// Convert the dense spectrum to XYZ vector
    pub const fn to_xyz(&self) -> XYZ {
        XYZ::new(
            self.dot(xyz::cie1931::X) / xyz::cie1931::Y_INTEGRAL,
            self.dot(xyz::cie1931::Y) / xyz::cie1931::Y_INTEGRAL,
            self.dot(xyz::cie1931::Z) / xyz::cie1931::Y_INTEGRAL,
        )
    }

    const fn dot(&self, rhs: Self) -> Float {
        let mut sum = 0.0;
        let mut i = 0;
        while i < DENSE_SAMPLES {
            sum += self.values[i] * rhs.values[i];
            i += 1;
        }
        sum
    }
}
