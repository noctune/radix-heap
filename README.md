# radix-heap

[![Build Status](https://travis-ci.org/Noctune/radix-heap.svg?branch=master)](https://travis-ci.org/Noctune/radix-heap)

A radix heap is kind of *monotone* priority queue. Monotone means, for a max-
heap, that items pushed onto the heap must be smaller or equal to the last item
that was popped off the heap. It has O(1) push and amortized O(log n) pop. The
pop cost can also be described as amortixed O(log m) where m is the difference
between the value popped *p* and the and the last popped value at the time of 
*p*'s insertion. 

See [the documentation](https://docs.rs/radix-heap/) for more details.

# Performance

Here is a summary of the benchmarks from running them on my machine:

    extend_radix 8          time:   [19.475 us 19.479 us 19.483 us]
    extend_radix 16         time:   [20.746 us 20.783 us 20.842 us]
    extend_radix 32         time:   [19.624 us 19.629 us 19.637 us]
    extend_radix 64         time:   [31.839 us 31.869 us 31.906 us]
    extend_binary 8         time:   [95.684 us 95.693 us 95.706 us]
    extend_binary 16        time:   [91.204 us 91.260 us 91.315 us]
    extend_binary 32        time:   [92.068 us 93.321 us 95.324 us]
    extend_binary 64        time:   [90.484 us 90.590 us 90.718 us]
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

The gist of it is that radix heaps are better for smaller keys. The last two
benchmarks repeatedly push values that are slightly larger than the last popped
value. The radix heap performs significantly better at this. This use case is
something that would be common in for example Dijkstra's algorithm.