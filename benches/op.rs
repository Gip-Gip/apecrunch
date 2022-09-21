use apecrunch::op_engine::get_equality;
use apecrunch::parser::parse_str;
use apecrunch::session::Session;
use criterion::*;

pub fn op_benchmark(c: &mut Criterion) {
    let mut session = Session::_new_test().unwrap();

    let input1 = parse_str("2+2", &mut session).unwrap();
    let input2 = parse_str("((6.1--2.22)^2 + (-24-10.5)^2)^0.5", &mut session).unwrap();

    c.bench_function("get equality of '2+2'", |b: &mut Bencher| {
        b.iter(|| get_equality(&input1, &mut session))
    });

    c.bench_function(
        "get equality of 'sqrt((6.1--2.22)^2 + (-24-10.5)^2)'",
        |b: &mut Bencher| b.iter(|| get_equality(&input2, &mut session)),
    );

    session._test_purge().unwrap();
}

criterion_group!(op, op_benchmark);
criterion_main!(op);
