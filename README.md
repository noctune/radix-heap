# radix-heap

Fast monotone priority queues.

A monotone priority queue is a priority queue that requires the extracted elements follow a
monotonic sequence. This means that, for a max-radix-heap, you cannot insert an element into a
radix heap that is larger than the last extracted element.

The key of the last extracted element is called the "top" key of the radix heap. Thus any value
pushed onto the heap must be less than or equal to the top key.

In return for this restriction, the radix heap provides two major benefits:

- Inserts are `O(1)` and popping an element is amortized `O(log m)` where `m` is the difference
  between a popped key and the top key at the time the element was inserted.
  
  Note that this is independent of the number of elements in the radix heap. This means that for
  workloads where this difference is bounded by a constant, the radix heap has O(1) pop.

- It is trivial to implement first-in-last-out order for equal keys in a radix heap. When
  implementing pathfinding, this corresponds to "tie-breaking" which can significantly improve
  performance. This is also possible to implement with a binary heap, but comes for free with a
  radix heap.

- A radix heap has generally better cache coherence than a binary heap.

# Performance

Here is a summary of the benchmarks from running them on my machine:

```text
astar_radix             time:   [2.6594 us 2.6622 us 2.6651 us]
astar_binary            time:   [5.3698 us 5.3762 us 5.3827 us]
pushpop_radix           time:   [97.601 us 97.784 us 97.987 us]
pushpop_binary          time:   [507.28 us 507.44 us 507.60 us]
```

`astar` is a benchmark using a map from the
[2D Pathfinding Banchmarks](https://movingai.com/benchmarks/grids.html).

`pushpop` is a more heap-focused benchmark where values are repeatedly pushed and popped off a heap.

# Example

```
let mut heap = radix_heap::RadixHeapMap::new();

heap.push(7, 'a').unwrap();
heap.push(2, 'b').unwrap();
heap.push(9, 'c').unwrap();

assert!(heap.top() == None);
assert!(heap.pop() == Some((9, 'c')));
assert!(heap.top() == Some(9));
assert!(heap.pop() == Some((7, 'a')));
assert!(heap.top() == Some(7));
assert!(heap.pop() == Some((2, 'b')));
assert!(heap.top() == Some(2));
assert!(heap.pop() == None);
```
