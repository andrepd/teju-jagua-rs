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

/*fn teju_core(c: &mut Criterion) {
    let mut g = c.benchmark_group("teju_core");

    for num in NUMS {
        g.bench_with_input(benchmark_id(*num), num, |b, &num| {
            b.iter(|| unsafe { teju::teju::mk_impl::Result::new(num) } );
        });
    }
    g.finish();
}*/

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

criterion_group!(microbench, teju_general, teju_exp, ryu, std);

//

fn read_distribution_file(name: &str) -> Vec<f64> {
    use std::io::{prelude::*, ErrorKind};
    let mut data = vec![];
    let fname = format!("{}/benches/resources/{}.bin", env!("CARGO_MANIFEST_DIR"), name);
    let mut file = std::fs::File::open(fname).unwrap();
    let mut buf = [0u8; 8];
    loop {
        match file.read_exact(&mut buf) {
            Ok(()) => data.push(f64::from_ne_bytes(buf)),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return data,
            Err(_) => panic!(),
        }
    }
}

fn benchmark_distribution_finite(c: &mut Criterion, name: &str) {
    let data = read_distribution_file(name);
    let mut g = c.benchmark_group(name);
    g.throughput(criterion::Throughput::Elements(data.len().try_into().unwrap()));
    g.bench_with_input(BenchmarkId::new("teju", data.len()), &data.len(), |b, _| {
        b.iter(|| {
            for &i in &data {
                let _ = teju::Buffer::new().format_finite(black_box(i));
            }
        });
    });
    g.bench_with_input(BenchmarkId::new("ryu", data.len()), &data.len(), |b, _| {
        b.iter(|| {
            for &i in &data {
                let _ = ryu::Buffer::new().format_finite(black_box(i));
            }
        });
    });
    g.bench_with_input(BenchmarkId::new("std", data.len()), &data.len(), |b, _| {
        b.iter(|| {
            use std::io::Write;
            let mut buf = [0u8; 80];
            for &i in &data {
                let _ = write!(buf.as_mut_slice(), "{}", black_box(i));
            }
        });
    });
}

fn benchmark_distribution(c: &mut Criterion, name: &str) {
    let data = read_distribution_file(name);
    let mut g = c.benchmark_group(name);
    g.throughput(criterion::Throughput::Elements(data.len().try_into().unwrap()));
    g.bench_with_input(BenchmarkId::new("teju", data.len()), &data.len(), |b, _| {
        b.iter(|| {
            for &i in &data {
                let _ = teju::Buffer::new().format(black_box(i));
            }
        });
    });
    g.bench_with_input(BenchmarkId::new("ryu", data.len()), &data.len(), |b, _| {
        b.iter(|| {
            for &i in &data {
                let _ = ryu::Buffer::new().format(black_box(i));
            }
        });
    });
    g.bench_with_input(BenchmarkId::new("std", data.len()), &data.len(), |b, _| {
        b.iter(|| {
            use std::io::Write;
            let mut buf = [0u8; 80];
            for &i in &data {
                let _ = write!(buf.as_mut_slice(), "{}", black_box(i));
            }
        });
    });
}

fn uniform_zero_to_one(c: &mut Criterion) {
    benchmark_distribution_finite(c, "uniform_zero_to_one")
}

fn unit_gaussian_around_zero(c: &mut Criterion) {
    benchmark_distribution_finite(c, "unit_gaussian_around_zero")
}

fn unit_gaussian_around_zero_with_nan(c: &mut Criterion) {
    benchmark_distribution(c, "unit_gaussian_around_zero_with_nan")
}

fn pareto_fat_tail(c: &mut Criterion) {
    benchmark_distribution_finite(c, "pareto_fat_tail")
}

fn poisson_very_large_mean(c: &mut Criterion) {
    benchmark_distribution_finite(c, "poisson_very_large_mean")
}

fn int32(c: &mut Criterion) {
    benchmark_distribution_finite(c, "int32")
}


criterion_group!(distributions,
    uniform_zero_to_one,
    unit_gaussian_around_zero,
    unit_gaussian_around_zero_with_nan,
    pareto_fat_tail,
    poisson_very_large_mean,
    int32,
);

//

criterion_main!(microbench, distributions);
