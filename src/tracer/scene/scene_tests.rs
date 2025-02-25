use super::*;
use crate::tracer::{Instanceable, Plane, Sphere};
use crate::{Point, Direction};

/* light at y = 2, plane at y = 1 perp to z */
fn scene(m: Material) -> Scene {
    let mut scene = Scene::default();

    scene.add_light(Sphere::new(
        crate::EPSILON,
        Material::Light(Texture::from(Color::WHITE)))
                    .translate(0.0, 2.0, 0.0)
    );

    scene.add(Plane::new(
        Point::new(0.0, 1.0, 0.0),
        Point::new(0.0, -1.0, 0.0),
        m,
    ));

    scene
}

#[test]
fn light_no_pass() {
    let s = scene(Material::mirror());
    let r = Ray::new(Point::ZERO, Direction::Y);
    assert!(s.hit_light(&r, s.lights[0].as_ref()).is_none());
}

#[test]
fn object_behind_light() {
    let s = scene(Material::mirror());
    let r = Ray::new(3.0 * Point::Y, Direction::NEG_Y);
    assert!(s.hit_light(&r, s.lights[0].as_ref()).is_some());
}

#[test]
fn hits_closest() {
    let mut s = Scene::default();

    s.add(Plane::new(
        Point::Y,
        Point::NEG_Y,
        Material::Blank,
    ));

    s.add(Plane::new(
        2.0 * Point::Y,
        Point::NEG_Y,
        Material::mirror(),
    ));

    let r = Ray::new(Point::ZERO, Point::Y);
    let is_blank = |h: &Hit| -> bool { matches!(h.material, Material::Blank) };
    assert!(s.hit(&r).filter(is_blank).is_some());
}
