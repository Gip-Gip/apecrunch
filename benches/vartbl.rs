use apecrunch::number::Number;
use apecrunch::parser::Token;
use apecrunch::variable::VarTable;
use apecrunch::variable::Variable;
use criterion::*;
use uuid::Uuid;

pub fn vartbl_benchmark(c: &mut Criterion) {
    let mut vars = Vec::<Variable>::new();

    for _ in 0..1_000 {
        let var = Variable::new(
            &Uuid::new_v4().to_string(),
            Token::Number(Number::neg_one()),
        );

        vars.push(var);
    }

    c.bench_function("add variables to vartable", |b: &mut Bencher| {
        b.iter(|| {
            let mut vartbl = VarTable::new();

            for var in &vars {
                vartbl.add(var.clone()).unwrap();
            }
        })
    });

    let mut vartbl = VarTable::new();

    for var in &vars {
        vartbl.add(var.clone()).unwrap();
    }

    c.bench_function("remove variables from vartable", |b: &mut Bencher| {
        b.iter(|| {
            let mut vartbl = vartbl.clone();

            for var in &vars {
                vartbl.remove(&var.id).unwrap();
            }
        })
    });

    c.bench_function("get variables from vartable", |b: &mut Bencher| {
        b.iter(|| {
            let mut vartbl = vartbl.clone();

            for var in &vars {
                vartbl.get(&var.id).unwrap();
            }
        })
    });

    c.bench_function("store functions to vartable", |b: &mut Bencher| {
        b.iter(|| {
            let mut vartbl = vartbl.clone();

            for var in &vars {
                vartbl.store(var.clone()).unwrap();
                vartbl.store(var.clone()).unwrap();
            }
        })
    });
}

criterion_group!(vartbl, vartbl_benchmark);
criterion_main!(vartbl);
