use crate::{Vec2, Float};

/// Filters used to construct a pixel from samples
#[derive(Clone, Copy)]
pub enum Filter {
    /// Constant box filter
    Box,
    /// Triangle filter
    Triangle,
    /// Gaussian filter with `alpha`
    Gaussian(Float),
}

impl Filter {
    /// Evaluate the filter at `px` which is in `[0.0, 1.0] x [0.0, 1.0]`
    pub fn eval(&self, px: Vec2) -> Float {
        match self {
            Filter::Box => 1.0,
            Filter::Triangle => {
                let offset = (1.0 - px.abs()).max(Vec2::ZERO);
                offset.x * offset.y
            }
            Filter::Gaussian(alpha) => {
                let exp = (-alpha).exp();
                let gauss = |x: Float| -> Float {
                    ((-alpha * x * x).exp() - exp).max(0.0)
                };

                gauss(px.x) * gauss(px.y)
            }
        }
    }
}
