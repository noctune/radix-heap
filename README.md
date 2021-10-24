# radix-heap

Fast monotone priority queues.

A monotone priority queue is a variant of priority queues (itself a
generalization of heaps) that requires that the extracted elements follow a
monotonic sequence. This means that, for a max-radix-heap, you cannot insert
an element into a radix heap that is larger than the last extracted element.

The key of the last extracted element is called the "top" key of the radix
heap. Thus any value pushed onto the heap must be less than or equal to
the top key.

In return for this restriction, the radix heap does O(1) inserts. Popping an
element is O(log m) where m is the difference between a popped key and the
top key at the time the element was inserted. Note that this does not depend
on the number of elements in the radix heap. This means that for workloads
where this difference is bounded by a constant, the radix heap has O(1) pop.

# Performance

Here is a summary of the benchmarks from running them on my machine:

```text
sort_radix 8            time:   [281.10 us 281.95 us 283.09 us]
sort_radix 16           time:   [491.76 us 492.83 us 494.20 us]
sort_radix 32           time:   [526.14 us 526.24 us 526.34 us]
sort_radix 64           time:   [636.50 us 637.68 us 639.16 us]
sort_binary 8           time:   [578.36 us 579.05 us 579.87 us]
sort_binary 16          time:   [580.02 us 583.73 us 587.80 us]
sort_binary 32          time:   [581.35 us 581.42 us 581.50 us]
sort_binary 64          time:   [592.38 us 592.81 us 593.45 us]
pushpop_radix           time:   [147.26 us 147.60 us 148.13 us]
pushpop_binary          time:   [552.32 us 552.71 us 553.24 us]
```

The gist of it is that radix heaps are better for smaller keys. The last two
benchmarks repeatedly push values that are slightly larger than the last popped
value. The radix heap performs significantly better at this. This use case is
something that would be common in for example Dijkstra's algorithm.

# Example

```
extern crate radix_heap;
let mut heap = radix_heap::RadixHeapMap::new();

heap.push(7, 'a');
heap.push(2, 'b');
heap.push(9, 'c');

assert!(heap.top() == None);
assert!(heap.pop() == Some((9, 'c')));
assert!(heap.top() == Some(9));
assert!(heap.pop() == Some((7, 'a')));
assert!(heap.top() == Some(7));
assert!(heap.pop() == Some((2, 'b')));
assert!(heap.top() == Some(2));
assert!(heap.pop() == None);
```