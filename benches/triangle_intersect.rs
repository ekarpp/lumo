use spuristo::tracer::*;
use criterion::{criterion_group, criterion_main, Criterion};

use std::f64::INFINITY;
use glam::DVec3;

fn triangle_intersect(c: &mut Criterion) {
    let t = Triangle::new((DVec3::X, DVec3::Y, DVec3::Z), Material::Blank);

    let r = Ray::new(
        DVec3::ZERO,
        DVec3::ONE,
    );

    c.bench_function("Benchmark triangle intersection", |b| {
        b.iter(|| t.hit(&r, 0.0, INFINITY))
    });
}

criterion_group!(benches, triangle_intersect);
criterion_main!(benches);
