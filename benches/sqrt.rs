use apecrunch::number::Number;
use criterion::*;

pub fn sqrt_benchmark(c: &mut Criterion) {
    let mut val = Number::from_str("2").unwrap();
    c.bench_function("square root of two", |b: &mut Bencher| b.iter(|| {val.sqrt(6);}));
}

criterion_group!(sqrt, sqrt_benchmark);
criterion_main!(sqrt);