#![allow(dead_code)]
use super::*;

macro_rules! illuminants {
    ( $( $name:ident ),* ) => {
        $(
            pub const $name: &'static DenseSpectrum =
                &DenseSpectrum::new(samples::illuminants::$name::SAMPLES);
        )*
    }
}

illuminants! { A, D50, D65, F2, F7, CORNELL }
