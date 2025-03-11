use super::*;
use crate::tracer::{Instanceable, Disk, Sphere, Spectrum};
use crate::{ Point, Direction };

/* light at y = 2, disk at y = 1 perp to z */
fn scene(m: Material) -> Scene {
    let mut scene = Scene::default();

    scene.add_light(Sphere::new(
        crate::EPSILON,
        Material::light(Texture::from(Spectrum::WHITE)))
                    .translate(0.0, 2.0, 0.0)
    );

    scene.add(Disk::new(
        Point::new(0.0, 1.0, 0.0),
        Point::new(0.0, -1.0, 0.0),
        100.0,
        m,
    ));

    scene.build();
    scene
}

#[test]
fn light_no_pass() {
    let s = scene(Material::mirror());
    let r = Ray::new(Point::ZERO, Direction::Y);
    let mut rng = Xorshift::default();

    let (light, _) = s.get_light(s.sample_light(0.0));
    assert!(s.hit_light(&r, &mut rng, light).is_none());
}

#[test]
fn object_behind_light() {
    let s = scene(Material::mirror());
    let r = Ray::new(3.0 * Point::Y, -Direction::Y);
    let mut rng = Xorshift::default();

    let (light, _) = s.get_light(s.lights.sample_light(0.0));
    assert!(s.hit_light(&r, &mut rng, light).is_some());
}

#[test]
fn hits_closest() {
    let mut s = Scene::default();

    s.add(Disk::new(
        Point::Y,
        -Point::Y,
        100.0,
        Material::Blank,
    ));

    s.add(Disk::new(
        2.0 * Point::Y,
        -Point::Y,
        100.0,
        Material::mirror(),
    ));

    s.add_light(Disk::new(
        -100.0 * Point::Y,
        Point::Y,
        1.0,
        Material::light(Texture::from(Spectrum::WHITE))
    ));

    s.build();

    let r = Ray::new(Point::ZERO, Point::Y);
    let is_blank = |h: &Hit| -> bool { matches!(h.material, Material::Blank) };
    let mut rng = Xorshift::default();

    assert!(s.hit(&r, &mut rng).filter(is_blank).is_some());
}
