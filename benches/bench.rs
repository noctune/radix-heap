use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::io::BufRead;

use criterion::{black_box, Bencher, Criterion};
use criterion::{criterion_group, criterion_main};
use radix_heap::RadixHeapMap;

const MAP: &[u8] = include_bytes!("den203d.map");

type Pos = (u32, u32);

struct Bool2D {
    height: u32,
    width: u32,
    values: Vec<bool>,
}

impl Bool2D {
    fn new(height: u32, width: u32) -> Bool2D {
        Bool2D {
            height,
            width,
            values: vec![false; (height * width) as usize],
        }
    }

    fn parse_map(mut bytes: &[u8]) -> Bool2D {
        let mut line = String::new();

        bytes.read_line(&mut line).unwrap();
        assert_eq!(line, "type octile\n");
        line.clear();

        bytes.read_line(&mut line).unwrap();
        let mut words = line.split_whitespace();
        assert_eq!(words.next(), Some("height"));
        let height: u32 = words.next().unwrap().parse().unwrap();
        line.clear();

        bytes.read_line(&mut line).unwrap();
        let mut words = line.split_whitespace();
        assert_eq!(words.next(), Some("width"));
        let width: u32 = words.next().unwrap().parse().unwrap();
        line.clear();

        bytes.read_line(&mut line).unwrap();
        assert_eq!(line, "map\n");
        line.clear();

        let mut values = Vec::new();
        for _ in 0..height {
            bytes.read_line(&mut line).unwrap();
            values.extend(line.as_bytes().iter().map(|&x| x == b'.'));
            line.clear();
        }

        Bool2D {
            height,
            width,
            values,
        }
    }

    fn clear(&mut self) {
        self.values.fill(false);
    }

    fn get_mut(&mut self, pos: Pos) -> &mut bool {
        &mut self.values[(pos.0 * self.height + pos.1) as usize]
    }
}

struct AStarEntry {
    pos: Pos,
    cost: u32,
    full_cost: u32,
}

fn astar<H: HeapMap>(b: &mut Bencher) {
    let map = Bool2D::parse_map(MAP);

    let from = (40, 75);
    let to = (20, 10);

    let mut visited = Bool2D::new(map.height, map.width);

    let manhattan =
        |pos: Pos| (pos.0.max(to.0) - pos.0.min(to.0)) + (pos.1.max(to.1) - pos.1.min(to.1));

    let mut heap = H::new();

    b.iter(|| {
        heap.clear();
        visited.clear();

        heap.push(AStarEntry {
            pos: from,
            cost: 0,
            full_cost: manhattan(from),
        });

        loop {
            if let Some(AStarEntry { pos, cost, .. }) = heap.pop() {
                if pos == to {
                    assert_eq!(black_box(cost), 85);
                    break;
                }

                for neighbor in [
                    (pos.0 + 1, pos.1),
                    (pos.0, pos.1 + 1),
                    (pos.0, pos.1 - 1),
                    (pos.0 - 1, pos.1),
                ] {
                    let visited = visited.get_mut(neighbor);

                    if !*visited {
                        let neighbor_cost = cost + 1;
                        heap.push(AStarEntry {
                            pos: neighbor,
                            cost: neighbor_cost,
                            full_cost: neighbor_cost + manhattan(neighbor),
                        });
                        *visited = true;
                    }
                }
            } else {
                panic!("no path")
            }
        }
    });
}

trait HeapMap {
    fn new() -> Self;
    fn clear(&mut self);
    fn push(&mut self, entry: AStarEntry);
    fn pop(&mut self) -> Option<AStarEntry>;
}

impl HeapMap for RadixHeapMap<Reverse<u32>, (Pos, u32)> {
    #[inline]
    fn new() -> Self {
        RadixHeapMap::new()
    }

    #[inline]
    fn clear(&mut self) {
        self.clear()
    }

    #[inline]
    fn push(&mut self, entry: AStarEntry) {
        self.push(Reverse(entry.full_cost), (entry.pos, entry.cost))
    }

    #[inline]
    fn pop(&mut self) -> Option<AStarEntry> {
        self.pop()
            .map(|(Reverse(full_cost), (pos, cost))| AStarEntry {
                pos,
                cost,
                full_cost,
            })
    }
}

impl HeapMap for BinaryHeap<AStarEntry> {
    #[inline]
    fn new() -> Self {
        BinaryHeap::new()
    }

    #[inline]
    fn clear(&mut self) {
        self.clear();
    }

    #[inline]
    fn push(&mut self, entry: AStarEntry) {
        self.push(entry)
    }

    #[inline]
    fn pop(&mut self) -> Option<AStarEntry> {
        self.pop()
    }
}
impl PartialOrd for AStarEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .full_cost
            .cmp(&self.full_cost)
            .then_with(|| self.cost.cmp(&other.cost))
    }
}

impl PartialEq for AStarEntry {
    fn eq(&self, other: &Self) -> bool {
        self.full_cost == other.full_cost && self.cost == other.cost
    }
}

impl Eq for AStarEntry {}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "astar_radix",
        astar::<RadixHeapMap<Reverse<u32>, (Pos, u32)>>,
    );
    c.bench_function("astar_binary", astar::<BinaryHeap<AStarEntry>>);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
