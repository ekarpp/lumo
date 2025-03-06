use super::*;

pub fn f(
    lambda: &ColorWavelength,
    h: &Hit,
    sigma_t: &Spectrum,
    sigma_s: &Spectrum,
) -> Color {
    let transmittance = (-sigma_t.sample(lambda) * h.t).exp();
    // cancel out the transmittance pdf taken from scene transmitance
    let pdf = (transmittance * sigma_t.sample(lambda)).mean()
        / transmittance.mean();
    if pdf == 0.0 {
        Color::WHITE
    } else {
        sigma_s.sample(lambda) / pdf
    }
}

/* Henyey-Greenstein (1941) */
pub fn sample(
    wo: Direction,
    g: Float,
    rand_sq: Vec2,
) -> Option<Direction> {
    let cos_theta = if g.abs() < 1e-3 {
        1.0 - 2.0 * rand_sq.x
    } else {
        let g2 = g * g;
        let fract = (1.0 - g2)
            / (1.0 - g + 2.0 * g * rand_sq.x);
        (1.0 + g2 - fract * fract) / (2.0 * g)
    };
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();

    let phi = 2.0 * crate::PI * rand_sq.y;

    // wo can be from instance?
    let uvw = Onb::new(wo.normalize());
    let wi = uvw.to_world(Direction::new(
        sin_theta * phi.cos(),
        sin_theta * phi.sin(),
        cos_theta
    ));

    Some( wi )
}

pub fn pdf(
    wo: Direction,
    wi: Direction,
    g: Float,
) -> Float {
    let cos_theta = wo.dot(wi);

    let g2 = g * g;
    let denom = 1.0 + g2 + 2.0 * g * cos_theta;

    (1.0 - g2) / (4.0 * crate::PI * denom * denom.max(0.0).sqrt())
}
