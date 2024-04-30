use criterion::{criterion_group, criterion_main, Criterion};
use rand::{thread_rng, RngCore};
use tzhandhsake::p2p::Nonce;

fn basic_inc<const N: usize>(nonce: &mut [u8; N], step: u8) {
    if step == 0 {
        return;
    };
    nonce
        .iter_mut()
        .rev()
        .try_fold(step, |acc, x| match x.overflowing_add(acc) {
            (res, true) => {
                *x = res;
                Some(1)
            }
            (res, false) => {
                *x = res;
                None
            }
        });
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Nonce inc");
    let mut rng = thread_rng();
    let mut nonce = [0u8; 24];
    rng.fill_bytes(&mut nonce);
    let mut p2pnonce = Nonce::from(nonce.clone());

    group.bench_function("basic_inc", |b| b.iter(|| basic_inc(&mut nonce, 1)));
    group.bench_function("ref impl", |b| b.iter(|| p2pnonce.inc()));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
