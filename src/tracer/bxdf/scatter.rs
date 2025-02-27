use super::*;

pub fn lambertian_f(spec: &Spectrum, lambda: &ColorWavelength) -> Color {
    spec.sample(lambda) / crate::PI
}

pub fn lambertian_sample(rand_sq: Vec2) -> Option<Direction> {
    Some( rand_utils::square_to_cos_hemisphere(rand_sq) )
}

pub fn lambertian_pdf(wo: Direction, wi: Direction) -> Float {
    if !spherical_utils::same_hemisphere(wo, wi) {
        0.0
    } else {
        let cos_theta = spherical_utils::cos_theta(wi);

        if cos_theta > 0.0 {
            cos_theta / crate::PI
        } else {
            0.0
        }
    }
}
