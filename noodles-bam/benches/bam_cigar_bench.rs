use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use noodles_sam::record::Cigar;
use rand::Rng;

use noodles_bam::record::codec::decoder::{get_cigar, get_cigar_no_reserve};
use noodles_bam::record::codec::encoder::put_cigar;

fn function_1(cigar_buf: &Vec<u8>, n_ops: &usize) {
    let mut cigar = Cigar::default();

    get_cigar(&mut cigar_buf.as_slice(), &mut cigar, *n_ops).unwrap();
}

fn function_2(cigar_buf: &Vec<u8>, n_ops: &usize) {
    let mut cigar = Cigar::default();

    get_cigar_no_reserve(&mut cigar_buf.as_slice(), &mut cigar, *n_ops).unwrap();
}

fn generate_cigar_string(n: usize) -> Cigar {
    let mut rng = rand::thread_rng();
    let operations = ['M', 'I', 'D', 'N', 'S', 'H', 'P', '=', 'X'];

    let cigar_str = (0..n)
        .map(|_| {
            let len: i32 = rng.gen_range(1..20);
            let op = operations[rng.gen_range(0..operations.len())];

            format!("{}{}", len, op)
        })
        .collect::<Vec<String>>()
        .join("");

    Cigar::from_str(&cigar_str).unwrap()
}

// Returns the buffer and the number of operations
fn generate_cigar_strings(n: usize) -> Vec<(Vec<u8>, usize)> {
    let cigars = (0..n)
        .map(|_| {
            let cigar = generate_cigar_string(10);

            let n_ops = cigar.len();

            let mut cigar_buf = Vec::new();
            put_cigar(&mut cigar_buf, &cigar).unwrap();

            (cigar_buf, n_ops)
        })
        .collect();

    cigars
}

fn compare_functions(c: &mut Criterion) {
    let inputs = generate_cigar_strings(10000);

    let mut group = c.benchmark_group("functions_comparison");

    group.bench_with_input("get_cigar_with_reserve", &inputs, |b, input_batch| {
        b.iter(|| {
            for (buf, n_ops) in input_batch {
                function_1(black_box(&buf), black_box(&n_ops));
            }
        })
    });

    group.bench_with_input("get_cigar_without_reserve", &inputs, |b, input_batch| {
        b.iter(|| {
            for (buf, n_ops) in input_batch {
                function_2(black_box(&buf), black_box(&n_ops));
            }
        })
    });

    group.finish();
}

criterion_group!(benches, compare_functions);
criterion_main!(benches);
