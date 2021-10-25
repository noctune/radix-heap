# radix-heap

Fast monotone priority queues.

A monotone priority queue is a priority queue that requires the extracted elements follow a
monotonic sequence. This means that, for a max-radix-heap, you cannot insert an element into a
radix heap that is larger than the last extracted element.

The key of the last extracted element is called the "top" key of the radix heap. Thus any value
pushed onto the heap must be less than or equal to the top key.

In return for this restriction, the radix heap provides two major benefits:

- Inserts are `O(1)` and popping an elemtn is amortized `O(log m)` where m is the difference between
  a popped key and the top key at the time the element was inserted.
  
  Note that this is independent of the number of elements in the radix heap. This means that for
  workloads where this difference is bounded by a constant, the radix heap has O(1) pop.

- It is trivial to implement a radix heap with first-in-last-out order for equal keys. When
  implementing pathfinding 

# Performance

Here is a summary of the benchmarks from running them on my machine:

```text
astar_radix             time:   [2.5387 us 2.5431 us 2.5477 us]
astar_binary            time:   [24.220 us 24.231 us 24.243 us]
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