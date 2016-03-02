# radix-heap
A radix heap is kind of *monotone* priority queue. Monotone means, for a max-
heap, that items pushed onto the heap must be smaller or equal to the last item
that was popped off the heap. This restriction allows for a better asymptotic
runtime for certain algorithms.

See [the documentation](noctune.github.io/radix-heap) for more details.
