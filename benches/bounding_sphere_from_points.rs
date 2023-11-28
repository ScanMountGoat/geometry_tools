use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geometry_tools::bounding::calculate_bounding_sphere_from_points;

fn criterion_benchmark(c: &mut Criterion) {
    let positions = vec![glam::Vec3A::ZERO; 10000];

    c.bench_function("calculate_bounding_sphere_from_points", |b| {
        b.iter(|| calculate_bounding_sphere_from_points(black_box(&positions)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
