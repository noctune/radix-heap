# radix-heap

[![Build Status](https://travis-ci.org/Noctune/radix-heap.svg?branch=master)](https://travis-ci.org/Noctune/radix-heap)

A radix heap is kind of *monotone* priority queue. Monotone means, for a max-
heap, that items pushed onto the heap must be smaller or equal to the last item
that was popped off the heap. This restriction allows for a better asymptotic
runtime for certain algorithms.

See [the documentation](http://noctune.github.io/radix-heap) for more details.
