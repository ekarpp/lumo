use crate::{ Normal, Direction };

/// Small utility struct for orthonormal basis
pub struct Onb {
    /// `x` (or `y`?) direction
    pub u: Normal,
    /// `y` (or `x`?) direction
    pub v: Normal,
    /// `z` direction
    pub w: Normal,
}

impl Onb {
    /// Creates a new ONB.
    ///
    /// # Arguments
    /// * `dir` - Direction of `z` axis. Not necessarily normalized.
    pub fn new(dir: Direction) -> Self {
        let w = dir.normalize();
        let (u, v) = w.any_orthonormal_pair();
        Self { u, v, w }
    }

    #[allow(dead_code)]
    pub fn new_from_basis(u: Normal, v: Normal, w: Normal) -> Self {
        let eps = 1e-5;
        // assert orthornomality
        assert!(u.is_normalized() && v.is_normalized() && w.is_normalized());
        assert!(u.dot(v).abs() < eps && u.dot(w).abs() < eps && v.dot(w).abs() < eps);
        Self { u, v, w }
    }

    /// Translate from the ONB to world basis
    ///
    /// # Arguments
    /// * `v` - The vector in ONB basis.
    pub fn to_world(&self, v: Direction) -> Direction {
        v.x * self.u + v.y * self.v + v.z * self.w
    }

    /// Translate from world basis to the ONB
    ///
    /// # Arguments
    /// * `v` - The vector in canonical basis
    pub fn to_local(&self, v: Direction) -> Direction {
        Direction::new(
            v.dot(self.u),
            v.dot(self.v),
            v.dot(self.w),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_directions() {
        let w = Direction::new(1.23, 4.56, 7.89);
        let uvw = Onb::new(w);

        let v = Direction::new(9.87, 6.54, 3.21);
        let vp = uvw.to_world(uvw.to_local(v));

        assert!(v.distance(vp) < 1e-10);
    }

    #[test]
    fn from_basis() {
        let u = Direction::new(1.23, 4.56, 7.89).normalize();
        let v = u.cross(Normal::X).normalize();
        let w = u.cross(v);
        let uvw = Onb::new_from_basis(u, v, w);

        let v = Direction::new(9.87, 6.54, 3.21);
        let vp = uvw.to_world(uvw.to_local(v));

        assert!(v.distance(vp) < 1e-10);
    }
}
