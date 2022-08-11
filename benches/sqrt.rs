use apecrunch::number::Number;
use criterion::*;

pub fn sqrt_benchmark(c: &mut Criterion) {
    let val1 = Number::from_str("2").unwrap();

    let val2 = Number::from_str("9999").unwrap();

    let val3 = Number::from_str("9999999999999999").unwrap();

    c.bench_function("square root of two", |b: &mut Bencher| {
        b.iter(|| {
            val1.root(&val1, 6);
        })
    });

    c.bench_function(
        "square root of nine-thousand, nine-hundred and ninty-nine",
        |b: &mut Bencher| {
            b.iter(|| {
                val2.root(&val1, 6);
            })
        },
    );

    c.bench_function(
        "square root of an obscenely large number",
        |b: &mut Bencher| {
            b.iter(|| {
                val3.root(&val1, 6);
            })
        },
    );
}

criterion_group!(sqrt, sqrt_benchmark);
criterion_main!(sqrt);
