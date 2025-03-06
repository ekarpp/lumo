use super::*;

#[test]
fn complex_div_zero() {
    let a = Complex::new(1.23, 4.56);

    let nan = a / Complex::from(0.0);
    assert!(nan.Re.is_nan() && nan.Im.is_nan());

    let nan = a / 0.0;
    assert!(nan.Re.is_nan() && nan.Im.is_nan());

    let nan = 1.0 / Complex::from(0.0);
    assert!(nan.Re.is_nan() && nan.Im.is_nan());
}

#[test]
fn complex_div_id() {
    let a = Complex::new(1.23, 4.56);
    let id = a / a;
    assert!(id.Re == 1.0 && id.Im == 0.0);
}

#[test]
fn complex_sqrt_zero() {
    assert!(Complex::from(0.0).sqrt().norm() == 0.0);
}

#[test]
fn complex_sqrt_norm() {
    let a = Complex::new(1.23, 4.56);
    assert!((a.norm() - a.sqrt().norm_sqr()).abs() < crate::EPSILON);
}

#[test]
fn complex_sub_id() {
    let a = Complex::new(1.23, 4.56);
    assert!((a - a).norm() == 0.0);
}
