use super::*;
use crate::consts::EPSILON;
use crate::Sphere;

/* light at y = 2, plane at y = 1 perp to z */
fn scene(m: Material) -> Scene {
    Scene::new(
        vec![
            Sphere::new(
                DVec3::new(0.0, 2.0, 0.0),
                EPSILON,
                Material::Light(Texture::Solid(DVec3::ONE))
            ),
            Plane::new(
                DVec3::new(0.0, 1.0, 0.0),
                DVec3::new(0.0, -1.0, 0.0),
                m
            ),
        ],
    )
}

/*
#[test]
fn light_pass_glass() {
    let s = scene(Material::Glass);
    let r = Ray::new(
        DVec3::ZERO,
        DVec3::new(0.0, 1.0, 0.0),
    );
    assert!(s.hit_light(&r, &s.objects[0]).is_some());
}
*/

#[test]
fn light_no_pass() {
    let s = scene(Material::Mirror);
    let r = Ray::new(
        DVec3::ZERO,
        DVec3::new(0.0, 1.0, 0.0),
    );
    assert!(s.hit_light(&r, s.objects[0].as_ref()).is_none());
}

#[test]
fn object_behind_light() {
    let s = scene(Material::Mirror);
    let r = Ray::new(
        DVec3::new(0.0, 3.0, 0.0),
        DVec3::new(0.0, -1.0, 0.0),
    );
    assert!(s.hit_light(&r, s.objects[0].as_ref()).is_some());
}

#[test]
fn hits_closest() {
    let s = Scene::new(
        vec![
            Plane::new(
                DVec3::new(0.0, 1.0, 0.0),
                DVec3::new(0.0, -1.0, 0.0),
                Material::Glass
            ),
            Plane::new(
                DVec3::new(0.0, 2.0, 0.0),
                DVec3::new(0.0, -1.0, 0.0),
                Material::Mirror
            ),
        ],
    );
    let r = Ray::new(
        DVec3::ZERO,
        DVec3::new(0.0, 1.0, 0.0),
    );
    let is_glass = |h: &Hit| -> bool {
        matches!(h.object.material(), Material::Glass)
    };
    assert!(s.hit(&r).filter(is_glass).is_some());
}
