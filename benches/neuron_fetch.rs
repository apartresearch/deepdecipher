use criterion::{criterion_group, criterion_main, Criterion};
use deepdecipher::data::Database;

fn create_neuron_page(c: &mut Criterion) {
    let state =
        tokio::runtime::deepdecipher::server::State::new(Database::open("data.db")).unwrap();
    c.bench_function("solu-1l", move |b| {
        b.to_async(tokio::runtime::Runtime).iter(|| async {})
    })
}

criterion_group!(benhmarks, create_neuron_page);

criterion_main!(benchmarks);
