use super::*;
use crate::{Axis, efloat};

/// Axis aligned bounding box
#[derive(Copy, Clone)]
pub struct AaBoundingBox {
    /// Minimum values along each axis
    pub ax_min: DVec3,
    /// Maximum values along each axis
    pub ax_max: DVec3,
}

impl Default for AaBoundingBox {
    fn default() -> Self {
        Self {
            ax_min: DVec3::splat(INFINITY),
            ax_max: DVec3::splat(-INFINITY),
        }
    }
}

impl AaBoundingBox {
    /// Constructs a AABB of given size.
    ///
    /// # Arguments
    /// * `ax_min` - The minimum values in each dimension
    /// * `ax_max` - The maxiumum values in each dimension
    pub fn new(ax_min: DVec3, ax_max: DVec3) -> Self {
        Self { ax_min, ax_max }
    }

    /// Find `t_start` and `t_end` for ray intersection
    pub fn intersect(&self, r: &Ray) -> (f64, f64) {
        let ro_min = (self.ax_min - r.origin) / r.dir;
        let ro_max = (self.ax_max - r.origin) / r.dir;

        let t_start = ro_min.min(ro_max);
        let t_end = ro_max.max(ro_min);

        let t_start = t_start.max_element();
        let t_end = t_end.min_element();

        (t_start, t_end * (1.0 + 2.0 * efloat::gamma(2.0)))
    }

    /// Combine self and other to a new bigger AABB
    pub fn merge(&self, other: &Self) -> Self {
        Self::new(self.ax_min.min(other.ax_min), self.ax_max.max(other.ax_max))
    }

    /// Returns the surface area of the AABB
    pub fn area(&self) -> f64 {
        let bb_dim = self.ax_max - self.ax_min;

        2.0 * (bb_dim.x * bb_dim.y + bb_dim.x * bb_dim.z + bb_dim.y * bb_dim.z)
    }

    /// Does `point` along `axis` cut `self`?
    pub fn cuts(&self, axis: Axis, point: f64) -> bool {
        match axis {
            Axis::X => self.ax_min.x < point && point < self.ax_max.x,
            Axis::Y => self.ax_min.y < point && point < self.ax_max.y,
            Axis::Z => self.ax_min.z < point && point < self.ax_max.z,
        }
    }

    /// Maximum value along `axis`
    pub fn max(&self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.ax_max.x,
            Axis::Y => self.ax_max.y,
            Axis::Z => self.ax_max.z,
        }
    }

    /// Minimum value along `axis`
    pub fn min(&self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.ax_min.x,
            Axis::Y => self.ax_min.y,
            Axis::Z => self.ax_min.z,
        }
    }

    /// Split `self` along `axis` at `value`. Returns (left, right)
    pub fn split(&self, axis: Axis, value: f64) -> (Self, Self) {
        let mut ax_mid_max = self.ax_max;
        let mut ax_mid_min = self.ax_min;
        match axis {
            Axis::X => {
                ax_mid_max.x = value;
                ax_mid_min.x = value;
            }
            Axis::Y => {
                ax_mid_max.y = value;
                ax_mid_min.y = value;
            }
            Axis::Z => {
                ax_mid_max.z = value;
                ax_mid_min.z = value;
            }
        }

        (
            Self::new(self.ax_min, ax_mid_max),
            Self::new(ax_mid_min, self.ax_max),
        )
    }
}
