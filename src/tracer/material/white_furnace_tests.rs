use super::*;
use crate::{ Point, rng::{self, Xorshift} };
use crate::tracer::ray::Ray;
use crate::tracer::object::{ Object, Disk };

/* testing framework to make sure materials don't give more energy than they receive,
 * "the white furnace"
 */
const NUM_RUNS: usize = 100;
const NUM_SAMPLES: usize = 16_384;
const MAX_RADIANCE: Float = 1.01;

macro_rules! test_material {
    ( $( $name:ident, $mat:expr, $mode:expr ),* ) => {
        $(
            mod $name {
                use super::*;

                #[test]
                fn white_furnace() {
                    let mut rng = Xorshift::default();

                    let d = Disk::new(
                        Point::ZERO,
                        Normal::Z,
                        1.0,
                        $mat
                    );
                    let xo = Point::Z;
                    let wi = Direction::NEG_Z;
                    let r = Ray::new(xo, wi);
                    for _ in 0..NUM_RUNS {
                        let wo = rng::maps::square_to_hemisphere(rng.gen_vec2());

                        let radiance = match d.hit(&r, 0.0, crate::INF) {
                            None => Color::WHITE,
                            Some(h) => furnace_sample(wo, h, &mut rng, $mode)
                        };

                        let pass = radiance.max() < MAX_RADIANCE;
                        if !pass {
                            println!("L: {}, wo: {}", radiance, wo);
                        }
                        assert!(pass);
                    }
                }
            }
        )*
    }
}

fn white_tex() -> Texture {
    Texture::from(Spectrum::WHITE)
}

fn dielectric(roughness: Float, eta: Float) -> Material {
    Material::transparent(white_tex(), roughness, eta)
}

fn conductor(roughness: Float, eta: Float, k: Float) -> Material {
    Material::metal(white_tex(), roughness, eta, k)
}

test_material!{
    lambertian,  Material::lambertian(Spectrum::WHITE), Transport::Radiance,
    diffuse,     Material::diffuse(white_tex()),        Transport::Radiance,

    conductor75_eta15_k3, conductor(0.75, 1.5, 3.0), Transport::Radiance,
    conductor50_eta15_k3, conductor(0.75, 1.5, 3.0), Transport::Radiance,
    conductor25_eta15_k3, conductor(0.75, 1.5, 3.0), Transport::Radiance,
    conductor10_eta15_k3, conductor(0.75, 1.5, 3.0), Transport::Radiance,
    conductor00_eta15_k3, conductor(0.75, 1.5, 3.0), Transport::Radiance,

    conductor75_eta25_k0, conductor(0.75, 2.5, 0.0), Transport::Radiance,
    conductor50_eta25_k0, conductor(0.75, 2.5, 0.0), Transport::Radiance,
    conductor25_eta25_k0, conductor(0.75, 2.5, 0.0), Transport::Radiance,
    conductor10_eta25_k0, conductor(0.75, 2.5, 0.0), Transport::Radiance,
    conductor00_eta25_k0, conductor(0.75, 2.5, 0.0), Transport::Radiance,

    dielectric75_eta15_rad, dielectric(0.75, 1.5), Transport::Radiance,
    dielectric50_eta15_rad, dielectric(0.50, 1.5), Transport::Radiance,
    dielectric25_eta15_rad, dielectric(0.25, 1.5), Transport::Radiance,
    dielectric10_eta15_rad, dielectric(0.10, 1.5), Transport::Radiance,
    dielectric00_eta15_rad, dielectric(0.00, 1.5), Transport::Radiance,

    dielectric75_eta25_rad, dielectric(0.75, 2.5), Transport::Radiance,
    dielectric50_eta25_rad, dielectric(0.50, 2.5), Transport::Radiance,
    dielectric25_eta25_rad, dielectric(0.25, 2.5), Transport::Radiance,
    dielectric10_eta25_rad, dielectric(0.10, 2.5), Transport::Radiance,
    dielectric00_eta25_rad, dielectric(0.00, 2.5), Transport::Radiance,


    dielectric75_eta15_imp, dielectric(0.75, 1.5), Transport::Importance,
    dielectric50_eta15_imp, dielectric(0.50, 1.5), Transport::Importance,
    dielectric25_eta15_imp, dielectric(0.25, 1.5), Transport::Importance,
    dielectric10_eta15_imp, dielectric(0.10, 1.5), Transport::Importance,
    dielectric00_eta15_imp, dielectric(0.00, 1.5), Transport::Importance,

    dielectric75_eta25_imp, dielectric(0.75, 2.5), Transport::Importance,
    dielectric50_eta25_imp, dielectric(0.50, 2.5), Transport::Importance,
    dielectric25_eta25_imp, dielectric(0.25, 2.5), Transport::Importance,
    dielectric10_eta25_imp, dielectric(0.10, 2.5), Transport::Importance,
    dielectric00_eta25_imp, dielectric(0.00, 2.5), Transport::Importance
}

fn furnace_sample(wo: Direction, h: Hit, rng: &mut Xorshift, mode: Transport) -> Color {
    let m = h.material;

    let mut misses = 0;
    let mut radiance = Color::BLACK;
    let lambda = ColorWavelength::sample(rng.gen_float());

    for _ in 0..NUM_SAMPLES {
        match m.bsdf_sample(wo, &h, rng.gen_float(), rng.gen_vec2()) {
            None => misses += 1,
            Some(wi) => radiance += m.bsdf_f(wo, wi, &lambda, mode, &h)
                * m.shading_cosine(wi, h.ns)
                / m.bsdf_pdf(wo, wi, &h, false),
        }
    }

    radiance / (NUM_SAMPLES - misses) as Float
}
