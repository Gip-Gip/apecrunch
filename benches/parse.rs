use apecrunch::parser::parse_str;
use apecrunch::session::Session;
use criterion::*;

pub fn parse_benchmark(c: &mut Criterion) {
    let input1 = "2+2";
    let input2 = "((6.1--2.22)^2 + (-24-10.5)^2)^0.5";

    let mut session = Session::_new_test().unwrap();

    c.bench_function("parse '2+2'", |b: &mut Bencher| {
        b.iter(|| {
            parse_str(input1, &mut session)
        })
    });

    c.bench_function("parse 'sqrt((6.1--2.22)^2 + (-24-10.5)^2)'", |b: &mut Bencher| {
        b.iter(|| {
            parse_str(input2, &mut session)
        })
    });

    session._test_purge().unwrap();
}

criterion_group!(parse, parse_benchmark);
criterion_main!(parse);
