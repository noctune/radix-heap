use criterion::{black_box, Bencher, Criterion};
use criterion::{criterion_group, criterion_main};
use radix_heap::{Radix, RadixHeapMap};
use rand::XorShiftRng;
use rand::{Rand, Rng};
use std::collections::BinaryHeap;

fn sort_radix<T: Copy + Ord + Radix + Rand>(b: &mut Bencher) {
    let data: Vec<T> = XorShiftRng::new_unseeded().gen_iter().take(10000).collect();
    let mut heap = RadixHeapMap::new();

    b.iter(|| {
        heap.extend(data.iter().map(|&k| (k, ())));

        while let Some(a) = heap.pop() {
            black_box(a);
        }

        heap.clear();
    });
}

fn sort_binary<T: Copy + Ord + Radix + Rand>(b: &mut Bencher) {
    let data: Vec<T> = XorShiftRng::new_unseeded().gen_iter().take(10000).collect();
    let mut heap = BinaryHeap::<T>::new();

    b.iter(|| {
        heap.extend(data.iter());

        while let Some(a) = heap.pop() {
            black_box(a);
        }

        heap.clear();
    });
}

fn pushpop_radix(b: &mut Bencher) {
    let mut heap = RadixHeapMap::<i32, ()>::new();

    b.iter(|| {
        heap.push(0, ());

        for _ in 0..10000 {
            let (n, _) = heap.pop().unwrap();

            for i in 0..4 {
                heap.push(n - i, ());
            }
        }

        heap.clear();
    });
}

fn pushpop_binary(b: &mut Bencher) {
    let mut heap = BinaryHeap::<i32>::new();

    b.iter(|| {
        heap.push(0);

        for _ in 0..10000 {
            let n = heap.pop().unwrap();

            for i in 0..4 {
                heap.push(n - i);
            }
        }

        heap.clear();
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sort_radix 8", sort_radix::<u8>);
    c.bench_function("sort_radix 16", sort_radix::<u16>);
    c.bench_function("sort_radix 32", sort_radix::<u32>);
    c.bench_function("sort_radix 64", sort_radix::<u64>);
    c.bench_function("sort_binary 8", sort_binary::<u8>);
    c.bench_function("sort_binary 16", sort_binary::<u16>);
    c.bench_function("sort_binary 32", sort_binary::<u32>);
    c.bench_function("sort_binary 64", sort_binary::<u64>);
    c.bench_function("pushpop_radix", pushpop_radix);
    c.bench_function("pushpop_binary", pushpop_binary);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
