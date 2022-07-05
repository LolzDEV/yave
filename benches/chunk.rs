use criterion::{criterion_group, criterion_main, Criterion};
use yave::world::chunk::Chunk;

pub fn generate(c: &mut Criterion) {
    c.bench_function("Chunk data generation", |b| {
        b.iter(|| {
            Chunk::new(0, 0);
        })
    });
}

criterion_group!(benches, generate);
criterion_main!(benches);
