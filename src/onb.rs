use crate::DVec3;

/* w normalized. returns orthonormal uvw-basis */
pub fn uvw_basis(w: DVec3) -> (DVec3, DVec3) {
    let a = if w.x.abs() > 0.9 {
        DVec3::new(0.0, 1.0, 0.0)
    } else {
        DVec3::new(1.0, 0.0, 0.0)
    };

    let v = w.cross(a).normalize();
    let u = w.cross(v);

    (u, v)
}

/* transform k to uvw orthonormal basis */
pub fn to_uvw_basis(k: DVec3, u: DVec3, v: DVec3, w: DVec3) -> DVec3 {
    k.x*u + k.y*v + k.z*w
}
