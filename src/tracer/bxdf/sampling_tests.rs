use super::*;
use crate::{ math::simpson_integration, rng::Xorshift };
use crate::tracer::{ DenseSpectrum, Spectrum, Texture };

// used for numerically integrating PDF over whole space
const THETA_BINS: usize = 80;
const PHI_BINS: usize = 2 * THETA_BINS;

const NUM_BINS: usize = 8;
const NUM_SAMPLES: usize = THETA_BINS * PHI_BINS * 1_000;
const BIN_TOLERANCE: Float = 5e-2 * NUM_BINS as Float;
const INT_TOLERANCE: Float = 1e-2;

macro_rules! test_bxdf {
    ( $( $name:ident, $bxdf:expr ),* ) => {
        $(
            mod $name {
                use super::*;

                #[test]
                fn sampling() {
                    let mut rng = Xorshift::default();
                    let bins = do_sampling(&mut rng, $bxdf);

                    // print bins first for debugging
                    println!("bin values:");
                    for i in 0..NUM_BINS {
                        for j in 0..NUM_BINS {
                            print!("{:.3} ", bins[i][j]);
                        }
                        println!();
                    }

                    let mut sum = 0.0;
                    println!("bin deltas:");
                    for i in 0..NUM_BINS {
                        for j in 0..NUM_BINS {
                            sum += bins[i][j];
                            let delta = (2.0 * crate::PI - bins[i][j]).abs();
                            print!("{:.3} ", delta);
                            assert!(delta < BIN_TOLERANCE);
                        }
                        println!();
                    }

                    sum /= (NUM_BINS * NUM_BINS) as Float;
                    let delta = (2.0 * crate::PI - sum).abs();
                    println!("delta: {}", delta);
                    assert!(delta < INT_TOLERANCE);
                }
            }
        )*
    }
}

fn mfd(roughness: Float, eta: Float) -> MfDistribution {
    MfDistribution::new(
        roughness,
        DenseSpectrum::from_constant(eta),
        DenseSpectrum::from_constant(0.0),
        Texture::from(Spectrum::WHITE),
        Texture::from(Spectrum::WHITE),
        Texture::from(Spectrum::WHITE),
    )
}

// TODO: support dielectrics
test_bxdf!{
    lambertian, BxDF::Lambertian(Spectrum::WHITE),
    diffuse,    BxDF::MfDiffuse(mfd(1.0, 1.5)),

    conductor75, BxDF::MfConductor(mfd(0.75, 1.5)),
    conductor50, BxDF::MfConductor(mfd(0.50, 1.5)),
    conductor25, BxDF::MfConductor(mfd(0.25, 1.5)),
    conductor10, BxDF::MfConductor(mfd(0.10, 1.5))
}

/* ripped from pbrt. take v = normal and sample NUM_SAMPLES times.
 * if sample is ok add 1 / pdf to a *bin*. each bin should converge to 2*PI.
 * split bins using spherical coordinates of the sampled vector.
 */
fn do_sampling(rng: &mut Xorshift, bxdf: BxDF) -> Vec<Vec<Float>> {
    let mut bins: Vec<Vec<Float>> = vec!();
    let mut num_failed: usize = 0;

    for i in 0..NUM_BINS {
        bins.push(vec!());
        for _ in 0..NUM_BINS {
            bins[i].push(0.0);
        }
    }

    let mut lambda = ColorWavelength::sample(rng.gen_float());
    // let wo face directly the normal
    let wo = Direction::Z;
    for _ in 0..NUM_SAMPLES {
        let wi = bxdf.sample(wo, false, &mut lambda, rng.gen_float(), rng.gen_vec2());
        match wi {
            None => num_failed += 1,
            Some(wi) => {
                let reflection = spherical_utils::cos_theta(wo)
                    * spherical_utils::cos_theta(wi) >= 0.0;
                let pdf = bxdf.pdf(wo, wi, reflection, &lambda);
                if pdf == 0.0 {
                    panic!("Sampled direction with 0 probability");
                }
                let wi_phi = spherical_utils::phi(wi) / (2.0 * crate::PI);
                let phi_idx = (wi_phi * NUM_BINS as Float) as usize;
                let wi_cos_theta = spherical_utils::cos_theta(wi);
                let theta_idx = (wi_cos_theta * NUM_BINS as Float) as usize;
                // can be out of bounds in rare cases. fix if it happens.
                bins[theta_idx][phi_idx] += 1.0 / pdf;
            }
        }
    }
    let good_samples = NUM_SAMPLES - num_failed;

    let integral = integrate_sphere(wo, &lambda, &bxdf);
    println!("integral over whole space: {}", integral);
    /* scale bins properly based on samples and integral over whole space of PDF.
     * (PDF for sampling microfacets does not always integrate to 1 over whole space)
     */
    for i in 0..NUM_BINS {
        for j in 0..NUM_BINS {
            bins[i][j] *= integral * (NUM_BINS * NUM_BINS) as Float
                / good_samples as Float;
        }
    }
    let fail_percent = 100.0 * num_failed as Float / NUM_SAMPLES as Float;
    println!("failed samples: {:.2} %", fail_percent);
    bins
}

fn integrate_sphere(wo: Direction, lambda: &ColorWavelength, bxdf: &BxDF) -> Float {
    let mut integral = 0.0;

    let theta_factor = crate::PI / THETA_BINS as Float;
    let phi_factor = (2.0 * crate::PI) / PHI_BINS as Float;

    for theta_bin in 0..THETA_BINS {
        let theta0 = theta_bin as Float * theta_factor;
        let theta1 = theta0 + theta_factor;
        for phi_bin in 0..PHI_BINS {
            let phi0 = phi_bin as Float * phi_factor;
            let phi1 = phi0 + phi_factor;
            let f = |theta: Float, phi: Float| {
                let wi = Direction::new(
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                );
                let reflection = spherical_utils::cos_theta(wo)
                    * spherical_utils::cos_theta(wi) >= 0.0;
                // pdf in solid angle, change to spherical coordinates
                bxdf.pdf(wo, wi, reflection, lambda) * theta.sin()
            };
            integral += simpson_integration::simpson2d(f, theta0, theta1, phi0, phi1);
        }
    }

    integral
}
