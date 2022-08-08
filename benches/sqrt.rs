use apecrunch::number::Number;
use criterion::*;

pub fn sqrt_benchmark(c: &mut Criterion) {
    let val1 = Number::from_str("2").unwrap();

    let val2 = Number::from_str("9999").unwrap();

    c.bench_function("square root of two", |b: &mut Bencher| {
        b.iter(|| {
            val1.sqrt(6);
        })
    });

    c.bench_function(
        "square root of nine-thousand, nine-hundred and ninty-nine",
        |b: &mut Bencher| {
            b.iter(|| {
                val2.sqrt(6);
            })
        },
    );
}

criterion_group!(sqrt, sqrt_benchmark);
criterion_main!(sqrt);
