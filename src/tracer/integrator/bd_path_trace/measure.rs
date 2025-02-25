use super::*;

/// Helper to convert area measure `pdf` to solid angle
#[allow(dead_code)]
pub fn area_to_sa(pdf: Float, xo: Point, xi: Point, wi: Direction, ngi: Normal) -> Float {
    pdf * xo.distance_squared(xi) / ngi.dot(wi).abs()
}

/// Helper to convert SA measure `pdf` to area
pub fn sa_to_area(pdf: Float, xo: Point, xi: Point, wi: Direction, ngi: Normal) -> Float {
    pdf * wi.dot(ngi).abs() / xo.distance_squared(xi)
}
