
use criterion::{self, criterion_group, criterion_main, Criterion};
use pretty_assertions::assert_eq;

fn read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read");
   
    group.finish();
}



criterion_group!(
    benches,
    read,
);
criterion_main!(benches);