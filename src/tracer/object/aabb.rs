use super::*;

/// Axis aligned bounding box
#[derive(Copy, Clone)]
pub struct AaBoundingBox {
    /// Minimum values along each axis
    pub ax_min: Point,
    /// Maximum values along each axis
    pub ax_max: Point,
}

impl Default for AaBoundingBox {
    fn default() -> Self {
        Self {
            ax_min: Point::splat(crate::INF),
            ax_max: Point::splat(-crate::INF),
        }
    }
}

impl AaBoundingBox {
    /// Constructs a AABB of given size.
    ///
    /// # Arguments
    /// * `ax_min` - The minimum values in each dimension
    /// * `ax_max` - The maxiumum values in each dimension
    pub fn new(ax_min: Point, ax_max: Point) -> Self {
        Self { ax_min, ax_max }
    }

    /// Find `t_start` and `t_end` for ray intersection
    #[inline(always)]
    pub fn intersect(&self, origin: Point, inv_dir: Direction) -> (Float, Float) {
        let ro_min = (self.ax_min - origin) * inv_dir;
        let ro_max = (self.ax_max - origin) * inv_dir;

        let t_start = ro_min.min(ro_max);
        let t_end = ro_max.max(ro_min);

        let t_start = t_start.max_element();
        let t_end = t_end.min_element();

        (t_start, t_end * (1.0 + 2.0 * efloat::gamma(3)))
    }

    pub fn center(&self) -> Vec3 {
        self.ax_min + (self.ax_max - self.ax_min) / 2.0
    }

    /// Combine self and other to a new bigger AABB
    pub fn merge(&self, other: &Self) -> Self {
        Self::new(self.ax_min.min(other.ax_min), self.ax_max.max(other.ax_max))
    }

    /// Returns the surface area of the AABB
    pub fn area(&self) -> Float {
        let bb_dim = self.ax_max - self.ax_min;

        2.0 * (bb_dim.x * bb_dim.y + bb_dim.x * bb_dim.z + bb_dim.y * bb_dim.z)
    }

    /// Does `point` along `axis` cut `self`?
    pub fn cuts(&self, axis: Axis, point: Float) -> bool {
        match axis {
            Axis::X => self.ax_min.x < point && point < self.ax_max.x,
            Axis::Y => self.ax_min.y < point && point < self.ax_max.y,
            Axis::Z => self.ax_min.z < point && point < self.ax_max.z,
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        [Axis::X, Axis::Y, Axis::Z].iter().all(|ax| self.cuts(*ax, point.axis(*ax)))
    }

    pub fn extent(&self) -> Vec3 {
        self.ax_max - self.ax_min
    }

    /// Maximum value along `axis`
    pub fn max(&self, axis: Axis) -> Float {
        match axis {
            Axis::X => self.ax_max.x,
            Axis::Y => self.ax_max.y,
            Axis::Z => self.ax_max.z,
        }
    }

    /// Minimum value along `axis`
    pub fn min(&self, axis: Axis) -> Float {
        match axis {
            Axis::X => self.ax_min.x,
            Axis::Y => self.ax_min.y,
            Axis::Z => self.ax_min.z,
        }
    }

    /// Split `self` along `axis` at `value`. Returns (left, right)
    pub fn split(&self, axis: Axis, value: Float) -> (Self, Self) {
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
