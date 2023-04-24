use super::*;
use crate::tracer::Sphere;
use crate::EPSILON;

/* light at y = 2, plane at y = 1 perp to z */
fn scene(m: Material) -> Scene {
    let mut scene = Scene::default();

    scene.add_light(Sphere::new(
        DVec3::new(0.0, 2.0, 0.0),
        EPSILON,
        Material::Light(Texture::Solid(DVec3::ONE)),
    ));

    scene.add(Plane::new(
        DVec3::new(0.0, 1.0, 0.0),
        DVec3::new(0.0, -1.0, 0.0),
        m,
    ));

    scene
}

#[test]
fn light_no_pass() {
    let s = scene(Material::Mirror);
    let r = Ray::new(DVec3::ZERO, DVec3::new(0.0, 1.0, 0.0));
    assert!(s.hit_light(&r, s.lights[0].as_ref()).is_none());
}

#[test]
fn object_behind_light() {
    let s = scene(Material::Mirror);
    let r = Ray::new(DVec3::new(0.0, 3.0, 0.0), DVec3::new(0.0, -1.0, 0.0));
    assert!(s.hit_light(&r, s.lights[0].as_ref()).is_some());
}

#[test]
fn hits_closest() {
    let mut s = Scene::default();

    s.add(Plane::new(
        DVec3::new(0.0, 1.0, 0.0),
        DVec3::new(0.0, -1.0, 0.0),
        Material::Blank,
    ));

    s.add(Plane::new(
        DVec3::new(0.0, 2.0, 0.0),
        DVec3::new(0.0, -1.0, 0.0),
        Material::Mirror,
    ));

    let r = Ray::new(DVec3::ZERO, DVec3::new(0.0, 1.0, 0.0));
    let is_blank = |h: &Hit| -> bool { matches!(h.material, Material::Blank) };
    assert!(s.hit(&r).filter(is_blank).is_some());
}
