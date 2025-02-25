use super::*;
use crate::{ Point, rand_utils };
use crate::tracer::ray::Ray;
use crate::tracer::object::{ Object, Disk };

/* testing framework to make sure materials don't give more energy than they receive,
 * "the white furnace"
 */

const NUM_RUNS: usize = 100;
const NUM_SAMPLES: usize = 16_384;
const MAX_RADIANCE: Float = 1.01;

fn white_texture() -> Texture {
    Texture::from(Color::WHITE)
}

fn disk(material: Material) -> Disk {
    *Disk::new(
        Point::ZERO,
        Normal::Z,
        1.0,
        material,
    )
}

#[test]
fn lambert_white_furnace() {
    let m = Material::Standard(BSDF::new(BxDF::Lambertian(Color::WHITE)));
    test_material(m, Transport::Radiance);
}

#[test]
fn diffuse_white_furnace() {
    let m = Material::diffuse(white_texture());
    test_material(m, Transport::Radiance);
}

#[test]
fn conductor75_white_furnace() {
    let m = Material::metal(white_texture(), 0.75, 2.5, 0.0);
    test_material(m, Transport::Radiance);
}

#[test]
fn conductor50_white_furnace() {
    let m = Material::metal(white_texture(), 0.50, 2.5, 0.0);
    test_material(m, Transport::Radiance);
}

#[test]
fn conductor25_white_furnace() {
    let m = Material::metal(white_texture(), 0.25, 2.5, 0.0);
    test_material(m, Transport::Radiance);
}

#[test]
fn conductor10_white_furnace() {
    let m = Material::metal(white_texture(), 0.10, 2.5, 0.0);
    test_material(m, Transport::Radiance);
}

#[test]
fn conductor0_white_furnace() {
    let m = Material::metal(white_texture(), 0.00, 2.5, 0.0);
    test_material(m, Transport::Radiance);
}

#[test]
fn dielectric75_eta15_rad_white_furnace() {
    let m = Material::transparent(white_texture(), 0.75, 1.5);
    test_material(m, Transport::Radiance);
}

#[test]
fn dielectric50_eta15_rad_white_furnace() {
    let m = Material::transparent(white_texture(), 0.50, 1.5);
    test_material(m, Transport::Radiance);
}

#[test]
fn dielectric25_eta15_rad_white_furnace() {
    let m = Material::transparent(white_texture(), 0.25, 1.5);
    test_material(m, Transport::Radiance);
}

#[test]
fn dielectric10_eta15_rad_white_furnace() {
    let m = Material::transparent(white_texture(), 0.10, 1.5);
    test_material(m, Transport::Radiance);
}

#[test]
fn dielectric0_eta15_rad_white_furnace() {
    let m = Material::transparent(white_texture(), 0.00, 1.5);
    test_material(m, Transport::Radiance);
}

#[test]
fn dielectric75_eta25_rad_white_furnace() {
    let m = Material::transparent(white_texture(), 0.75, 2.5);
    test_material(m, Transport::Radiance);
}

#[test]
fn dielectric50_eta25_rad_white_furnace() {
    let m = Material::transparent(white_texture(), 0.50, 2.5);
    test_material(m, Transport::Radiance);
}

#[test]
fn dielectric25_eta25_rad_white_furnace() {
    let m = Material::transparent(white_texture(), 0.25, 2.5);
    test_material(m, Transport::Radiance);
}

#[test]
fn dielectric10_eta25_rad_white_furnace() {
    let m = Material::transparent(white_texture(), 0.10, 2.5);
    test_material(m, Transport::Radiance);
}

#[test]
fn dielectric0_eta25_rad_white_furnace() {
    let m = Material::transparent(white_texture(), 0.00, 2.5);
    test_material(m, Transport::Radiance);
}

#[test]
fn dielectric75_eta15_imp_white_furnace() {
    let m = Material::transparent(white_texture(), 0.75, 1.5);
    test_material(m, Transport::Importance);
}

#[test]
fn dielectric50_eta15_imp_white_furnace() {
    let m = Material::transparent(white_texture(), 0.50, 1.5);
    test_material(m, Transport::Importance);
}

#[test]
fn dielectric25_eta15_imp_white_furnace() {
    let m = Material::transparent(white_texture(), 0.25, 1.5);
    test_material(m, Transport::Importance);
}

#[test]
fn dielectric10_eta15_imp_white_furnace() {
    let m = Material::transparent(white_texture(), 0.10, 1.5);
    test_material(m, Transport::Importance);
}

#[test]
fn dielectric0_eta15_imp_white_furnace() {
    let m = Material::transparent(white_texture(), 0.00, 1.5);
    test_material(m, Transport::Importance);
}

#[test]
fn dielectric75_eta25_imp_white_furnace() {
    let m = Material::transparent(white_texture(), 0.75, 2.5);
    test_material(m, Transport::Importance);
}

#[test]
fn dielectric50_eta25_imp_white_furnace() {
    let m = Material::transparent(white_texture(), 0.50, 2.5);
    test_material(m, Transport::Importance);
}

#[test]
fn dielectric25_eta25_imp_white_furnace() {
    let m = Material::transparent(white_texture(), 0.25, 2.5);
    test_material(m, Transport::Importance);
}

#[test]
fn dielectric10_eta25_imp_white_furnace() {
    let m = Material::transparent(white_texture(), 0.10, 2.5);
    test_material(m, Transport::Importance);
}

#[test]
fn dielectric0_eta25_imp_white_furnace() {
    let m = Material::transparent(white_texture(), 0.00, 2.5);
    test_material(m, Transport::Importance);
}

fn test_material(m: Material, mode: Transport) {
    let d = disk(m);
    for _ in 0..NUM_RUNS {
        assert!(furnace_pass(&d, mode));
    }
}

fn furnace_pass(d: &Disk, mode: Transport) -> bool {
    let origin = Point::Z;
    let r = Ray::new(origin, Direction::NEG_Z);
    let wo = rand_utils::square_to_hemisphere(rand_utils::unit_square());

    let radiance = match d.hit(&r, 0.0, 1e10) {
        None => Color::WHITE,
        Some(h) => furnace_sample(wo, h, mode),
    };
    let pass = radiance.rgb.max_element() < MAX_RADIANCE;

    if !pass {
        println!("L: {}, wo: {}", radiance.rgb, wo);
    }

    pass
}

fn furnace_sample(wo: Direction, h: Hit, mode: Transport) -> Color {
    let m = h.material;

    let mut misses = 0;
    let mut radiance = Color::BLACK;

    for _ in 0..NUM_SAMPLES {
        match m.bsdf_sample(wo, &h, rand_utils::unit_square()) {
            None => misses += 1,
            Some(wi) => radiance += m.bsdf_f(wo, wi, mode, &h)
                * m.shading_cosine(wi, h.ns)
                / m.bsdf_pdf(wo, wi, &h, false),
        }
    }

    radiance / (NUM_SAMPLES - misses) as Float
}
