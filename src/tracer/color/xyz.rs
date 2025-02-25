#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;

/// Integral of the XYZ 1931 Y-curve
pub const CIE_Y: Float = 106.856895;

/// xy-coordinates of the D65 illuminant
pub const w_D65: Vec2 = Vec2::new(0.3127, 0.3290);

/// XYZ values from xyY values
pub const fn from_xyY(xy: Vec2, Y: Float) -> Vec3 {
    if xy.y == 0.0 {
        Vec3::ZERO
    } else {
        Vec3::new(
            xy.x * Y / xy.y,
            Y,
            (1.0 - xy.x - xy.y) * Y / xy.y,
        )
    }
}


/// XYZ 1931 X-curve sampled at `lambda`
pub fn X(lambda: &ColorWavelength) -> Color {
    // Wyman 2013
    let x_sample = |lambda: Float| -> Float {
        let c0 = 442.0;
        let s0 = (lambda - c0) * (if lambda < c0 { 0.0624 } else { 0.0374 });
        let c1 = 599.8;
        let s1 = (lambda - c1) * (if lambda < c1 { 0.0264 } else { 0.0323 });
        let c2 = 501.1;
        let s2 = (lambda - c2) * (if lambda < c2 { 0.0490 } else { 0.0382 });

        0.362 * (-0.5 * s0 * s0).exp()
            + 1.056 * (-0.5 * s1 * s1).exp()
            - 0.065 * (-0.5 * s2 * s2).exp()
    };

    let samples: [Float; SPECTRUM_SAMPLES] = lambda.iter()
        .map(|l| x_sample(*l))
        .collect::<Vec<Float>>().try_into().unwrap();
    Color::from(samples)
}

/// XYZ 1931 Y-curve sampled at `lambda`
pub fn Y(lambda: &ColorWavelength) -> Color {
    // Wyman 2013
    let y_sample = |lambda: Float| -> Float {
        let c0 = 568.8;
        let s0 = (lambda - c0) * (if lambda < c0 { 0.0213 } else { 0.0247 });
        let c1 = 530.9;
        let s1 = (lambda - c1) * (if lambda < c1 { 0.0613 } else { 0.0322 });

        0.821 * (-0.5 * s0 * s0).exp()
            + 0.286 * (-0.5 * s1 * s1).exp()
    };

    let samples: [Float; SPECTRUM_SAMPLES] = lambda.iter()
        .map(|l| y_sample(*l))
        .collect::<Vec<Float>>().try_into().unwrap();
    Color::from(samples)
}

/// XYZ 1931 Z-curve sampled at `lambda`
pub fn Z(lambda: &ColorWavelength) -> Color {
    // Wyman 2013
    let z_sample = |lambda: Float| -> Float {
        let c0 = 437.0;
        let s0 = (lambda - c0) * (if lambda < c0 { 0.0845 } else { 0.0278 });
        let c1 = 459.0;
        let s1 = (lambda - c1) * (if lambda < c1 { 0.0385 } else { 0.0725 });

        1.217 * (-0.5 * s0 * s0).exp()
            + 0.681 * (-0.5 * s1 * s1).exp()
    };

    let samples: [Float; SPECTRUM_SAMPLES] = lambda.iter()
        .map(|l| z_sample(*l))
        .collect::<Vec<Float>>().try_into().unwrap();
    Color::from(samples)
}
