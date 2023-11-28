use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geometry_tools::bounding::calculate_bounding_sphere_from_spheres;

fn criterion_benchmark(c: &mut Criterion) {
    let spheres = vec![(glam::Vec3A::ZERO, 1.0); 10000];

    c.bench_function("calculate_bounding_sphere_from_spheres", |b| {
        b.iter(|| calculate_bounding_sphere_from_spheres(black_box(&spheres)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
