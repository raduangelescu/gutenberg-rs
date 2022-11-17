use criterion::{self, criterion_group, criterion_main, Criterion, Throughput};

/// Just parse - no decoding overhead
pub fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_document");

    group.finish();
}

criterion_group!(benches, bench_parse,);
criterion_main!(benches);
