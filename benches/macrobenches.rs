use criterion::{self, criterion_group, criterion_main, Criterion};
use gutenberg_rs::rdf_parser::parse_rdfs_from_content;

static SAMPLE_1: &str = include_str!("../tests/documents/pg1.rdf");
static SAMPLE_2: &str = include_str!("../tests/documents/pg25.rdf");
static SAMPLE_3: &str = include_str!("../tests/documents/pg732.rdf");
static SAMPLE_4: &str = include_str!("../tests/documents/pg1000.rdf");
static SAMPLE_5: &str = include_str!("../tests/documents/pg90907.rdf");
static SAMPLE_6: &str = include_str!("../tests/documents/pg41418.rdf");

/// Just parse - no decoding overhead
pub fn bench_parse(c: &mut Criterion) {
    let documents = vec![
        SAMPLE_1.to_string(),
        SAMPLE_2.to_string(),
        SAMPLE_3.to_string(),
        SAMPLE_4.to_string(),
        SAMPLE_5.to_string(),
        SAMPLE_6.to_string(),
    ];
    c.bench_function("parse_rdfs_from_content", |b| {
        b.iter(|| parse_rdfs_from_content(&documents, false).unwrap())
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
