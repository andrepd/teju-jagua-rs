use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

const NUMS: &[f64] = &[0., -69., 123406000., 0.1234, 2.718281828459045, 1.7976931348623157e308];

fn benchmark_id(x: f64) -> BenchmarkId {
    BenchmarkId::from_parameter(ryu::Buffer::new().format(x))
}

fn teju_exp(c: &mut Criterion) {
    let mut g = c.benchmark_group("teju_exp");

    for num in NUMS {
        g.bench_with_input(benchmark_id(*num), num, |b, &num| {
            b.iter(|| teju::Buffer::new().format_exp_finite(black_box(num)).len() );
        });
    }
    g.finish();
}

fn teju_general(c: &mut Criterion) {
    let mut g = c.benchmark_group("teju_general");

    for num in NUMS {
        g.bench_with_input(benchmark_id(*num), num, |b, &num| {
            b.iter(|| teju::Buffer::new().format_finite(black_box(num)).len() );
        });
    }
    g.finish();
}

fn ryu(c: &mut Criterion) {
    let mut g = c.benchmark_group("ryu");

    for num in NUMS {
        g.bench_with_input(benchmark_id(*num), num, |b, &num| {
            b.iter(|| ryu::Buffer::new().format_finite(black_box(num)).len() );
        });
    }
    g.finish();
}

fn std(c: &mut Criterion) {
    let mut g = c.benchmark_group("std");

    use std::io::Write;
    let mut buf = [0u8; 80];
    for num in NUMS {
        g.bench_with_input(benchmark_id(*num), num, |b, &num| {
            b.iter(|| write!(buf.as_mut_slice(), "{}", black_box(num)) );
        });
    }
    g.finish();
}

criterion_group!(bench, teju_general, teju_exp, ryu, std);

criterion_main!(bench);
