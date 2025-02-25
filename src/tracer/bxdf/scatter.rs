use super::*;

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
