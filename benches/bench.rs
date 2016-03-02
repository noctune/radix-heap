#![feature(test)]

extern crate rand;
extern crate test;
extern crate radix_heap;

use std::collections::BinaryHeap;
use radix_heap::RadixHeapMap;
use test::{Bencher, black_box};
use rand::{thread_rng, Rng};

#[bench]
fn extend_radix(b: &mut Bencher) {
    let data: Vec<u32> = thread_rng().gen_iter().take(10000).collect();
    let mut heap = RadixHeapMap::new();
    
    b.iter(|| {
        heap.extend(data.iter().map(|&k| (k,())));
        
        while let Some(a) = heap.pop() {
            black_box(a);
        }
        
        heap.clear();
    });
}

#[bench]
fn extend_binary(b: &mut Bencher) {
    let data: Vec<u32> = thread_rng().gen_iter().take(10000).collect();
    let mut heap = BinaryHeap::<u32>::new();
    
    b.iter(|| {
        heap.extend(data.iter());
        
        while let Some(a) = heap.pop() {
            black_box(a);
        }
        
        heap.clear();
    });
}

#[bench]
fn pushpop_radix(b: &mut Bencher) {
    let mut heap = RadixHeapMap::<i32, ()>::new();
    
    b.iter(|| {
        heap.push(0, ());
        
        for _ in 0..10000 {
            let (n,_) = heap.pop().unwrap();
            
            for i in 0..2 {
                heap.push(n - i, ());
            }
        }
        
        heap.clear();
    });
}

#[bench]
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
