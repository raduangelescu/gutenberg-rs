use criterion::{self, criterion_group, criterion_main, Criterion};

fn read(c: &mut Criterion) {
    let group = c.benchmark_group("read");

    group.finish();
}

criterion_group!(benches, read,);
criterion_main!(benches);
