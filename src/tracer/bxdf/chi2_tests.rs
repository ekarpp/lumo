use super::*;
use crate::{ math::simpson_integration, rng::Xorshift };
use crate::tracer::{ Spectrum, Texture };
use std::io::Write;
use std::fs::{self, File};
use std::path::Path;

const TMP_DIR: &str = "./tmp/";
const THETA_BINS: usize = 10;
const PHI_BINS: usize = 2 * THETA_BINS;
const NUM_SAMPLES: usize = THETA_BINS * PHI_BINS * 1_000;
const CHI2_RUNS: usize = 20;
const CHI2_SLEVEL: Float = 0.01;
const CHI2_TOLERANCE: Float = (NUM_SAMPLES as Float) * 1e-5;
const CHI2_MIN_FREQ: Float = 5.0;

macro_rules! test_bxdf {
    ( $( $name:ident, $bxdf:expr ),* ) => {
        $(
            mod $name {
                use super::*;

                #[test]
                fn chi2() {
                    let mut rng = Xorshift::default();

                    for _ in 0..CHI2_RUNS {
                        let wo = rng::maps::square_to_hemisphere(rng.gen_vec2());
                        assert!(chi2_pass(wo, &mut rng, $bxdf));
                    }
                }
            }
        )*
    }
}

fn mfd(roughness: Float, eta: Float) -> MfDistribution {
    MfDistribution::new(
        roughness, eta, 0.0, /* k has no effect on sampling */
        Texture::from(Spectrum::WHITE),
        Texture::from(Spectrum::WHITE),
        Texture::from(Spectrum::WHITE),
    )
}

test_bxdf!{
    lambertian, BxDF::Lambertian(Spectrum::WHITE),
    diffuse,    BxDF::MfDiffuse(mfd(1.0, 1.5)),

    conductor75, BxDF::MfConductor(mfd(0.75, 1.5)),
    conductor50, BxDF::MfConductor(mfd(0.50, 1.5)),
    conductor25, BxDF::MfConductor(mfd(0.25, 1.5)),
    conductor10, BxDF::MfConductor(mfd(0.10, 1.5)),

    dielectric75_eta15, BxDF::MfDielectric(mfd(0.75, 1.5)),
    dielectric50_eta15, BxDF::MfDielectric(mfd(0.50, 1.5)),
    dielectric25_eta15, BxDF::MfDielectric(mfd(0.25, 1.5)),
    dielectric10_eta15, BxDF::MfDielectric(mfd(0.10, 1.5)),

    dielectric75_eta25, BxDF::MfDielectric(mfd(0.75, 2.5)),
    dielectric50_eta25, BxDF::MfDielectric(mfd(0.50, 2.5)),
    dielectric25_eta25, BxDF::MfDielectric(mfd(0.25, 2.5)),
    dielectric10_eta25, BxDF::MfDielectric(mfd(0.10, 2.5))
}

fn write_tables(
    actual: [usize; PHI_BINS*THETA_BINS],
    expected: [Float; PHI_BINS*THETA_BINS],
    wo: Direction,
) {
    if !fs::exists(TMP_DIR).expect("IO error") {
        fs::create_dir(TMP_DIR).expect("Unable to create temp dir");
    }
    let file_name = format!("{}lumo_chi2_{}.json", TMP_DIR, rng::gen_seed());
    let path = Path::new(&file_name);
    let mut tmp_file = File::create(&file_name)
        .expect("Unable to create temporary file");

    write!(tmp_file,
           "{{\"expected\":{:?},\"actual\":{:?},\"wo\":{}}}",
           expected, actual, wo
    )
        .expect("Unable to write to temporary file");

    println!("Dumped tables to {}", path.to_str().unwrap());
}

fn chi2_pass(wo: Direction, rng: &mut Xorshift, bxdf: BxDF) -> bool {
    let actual_freq = sample_frequencies(wo, rng, &bxdf);
    let expected_freq = compute_frequencies(wo, &bxdf);

    // degrees of freedom
    let mut dof = 0;
    // test statistic = sum_i (O_i - E_i)^2 / E_i
    let mut stat = 0.0;

    let mut pooled_samples = 0;
    let mut pooled_computed = 0.0;

    for phi_bin in 0..PHI_BINS {
        for theta_bin in 0..THETA_BINS {
            let idx = phi_bin + theta_bin * PHI_BINS;

            if expected_freq[idx] == 0.0 {
                if actual_freq[idx] as Float > CHI2_TOLERANCE {
                    println!(
                        "Found sampled value of {} at {} where expectation was zero.",
                        actual_freq[idx],
                        idx,
                    );
                    write_tables(actual_freq, expected_freq, wo);
                    return false;
                }
            } else if expected_freq[idx] < CHI2_MIN_FREQ {
                pooled_samples += actual_freq[idx];
                pooled_computed += expected_freq[idx];
            } else if pooled_computed < CHI2_MIN_FREQ {
                // make sure the pooled is high enough
                pooled_samples += actual_freq[idx];
                pooled_computed += expected_freq[idx];
            } else {
                let delta = actual_freq[idx] as Float - expected_freq[idx];
                stat += (delta * delta) / expected_freq[idx];
                dof += 1;
            }
        }
    }

    if pooled_samples + pooled_computed as usize > 0 {
        let delta = pooled_samples as Float - pooled_computed;
        stat += (delta * delta) / pooled_computed as Float;
        dof += 1;
    }

    // we assume that all parameters are known in the pearson chi2 test of goodness of fit
    dof -= 1;

    if dof == 0 {
        println!("Got 0 DoF!");
        false
    } else {
        // null hypothesis = our sampling follows the PDF

        /*
         * p-value, probability to get test statistic at least as extreme,
         * assuming null hypothesis holds
         */
        let pval = 1.0 - crate::math::chi2::chi2_cdf(dof, stat);
        println!("test statistic: {} p-value: {}", stat, pval);

        // we are possibly running multiple chi2 tests. apply Šidák correction
        let alpha = 1.0 - (1.0 - CHI2_SLEVEL).powf(1.0 / CHI2_RUNS as Float);

        let passed = pval >= alpha;

        if !passed {
            // write the tables in a pretty json (with wo)
            write_tables(actual_freq, expected_freq, wo);
        }

        passed
    }
}

fn sample_frequencies(
    wo: Direction,
    rng: &mut Xorshift,
    bxdf: &BxDF
) -> [usize; THETA_BINS*PHI_BINS] {
    let mut samples = [0; THETA_BINS*PHI_BINS];

    let theta_factor = THETA_BINS as Float / crate::PI;
    let phi_factor = PHI_BINS as Float / (2.0 * crate::PI);

    for _ in 0..NUM_SAMPLES {
        match bxdf.sample(wo, false, rng.gen_float(), rng.gen_vec2()) {
            None => (),
            Some(wi) => {
                let theta = spherical_utils::theta(wi);
                let phi = spherical_utils::phi(wi);

                let theta_bin = ((theta * theta_factor) as usize).clamp(0, THETA_BINS - 1);
                let phi_bin = ((phi * phi_factor) as usize).clamp(0, PHI_BINS - 1);

                samples[phi_bin + theta_bin * PHI_BINS] += 1;
            }
        }
    }

    samples
}

fn compute_frequencies(wo: Direction, bxdf: &BxDF) -> [Float; THETA_BINS*PHI_BINS] {
    let mut samples = [0.0; THETA_BINS*PHI_BINS];
    let mut ig = 0.0;
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
                bxdf.pdf(wo, wi, reflection) * theta.sin()
            };
            let integral = simpson_integration::simpson2d(f, theta0, theta1, phi0, phi1);
            ig += integral;
            samples[phi_bin + theta_bin * PHI_BINS] = integral * NUM_SAMPLES as Float;
        }
    }
    println!("integral: {}", ig);

    samples
}
