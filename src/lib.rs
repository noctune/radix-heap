#![deny(missing_docs)]

//! A monotone priority queue implemented with a radix heap.
//!
//! A monotone priority queue is a variant of priority queues (itself a
//! generalization of heaps) that requires that the extracted elements follow a
//! monotonic sequence. This means that you cannot insert an element into a
//! radix heap that is smaller than the last extracted element.
//!
//! The key of the last extracted element is called the "top" key of the radix
//! heap. Thus any value pushed onto the heap must be larger than or equal to
//! the top key.
//!
//! In return for this restriction, the radix heap does O(1) inserts. Popping an
//! element is O(log m) where m is the difference between a popped key and the
//! top key at the time the element was inserted. Note that this does not depend
//! on the number of elements in the radix heap. This means that for workloads
//! where this difference is bounded by a constant, the radix heap has O(1) pop.
//!
//! # Example
//!
//! ```
//! extern crate radix_heap;
//!
//! let mut heap = radix_heap::RadixHeapMap::new();
//! heap.push(7, 'a');
//! heap.push(2, 'b');
//! heap.push(9, 'c');
//!
//! assert!(heap.top() == None);
//! assert!(heap.pop() == Some((9, 'c')));
//! assert!(heap.top() == Some(9));
//! assert!(heap.pop() == Some((7, 'a')));
//! assert!(heap.top() == Some(7));
//! assert!(heap.pop() == Some((2, 'b')));
//! assert!(heap.top() == Some(2));
//! assert!(heap.pop() == None);
//! ```

use std::{
    cmp::max, cmp::Reverse, default::Default, fmt, iter::FromIterator, iter::FusedIterator,
    mem::swap, num::Wrapping,
};

use ieee754::Ieee754;
use ordered_float::NotNaN;

#[derive(Clone)]
struct Bucket<K, V> {
    max: Option<K>,
    elems: Vec<(K, V)>,
}

impl<K, V> Bucket<K, V> {
    fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }

    fn drain(&mut self) -> std::vec::Drain<(K, V)> {
        self.max = None;
        self.elems.drain(..)
    }

    fn iter(&self) -> std::slice::Iter<(K, V)> {
        self.elems.iter()
    }

    fn into_iter(self) -> std::vec::IntoIter<(K, V)> {
        self.elems.into_iter()
    }

    fn clear(&mut self) {
        self.max = None;
        self.elems.clear();
    }

    fn shrink_to_fit(&mut self) {
        self.elems.shrink_to_fit();
    }
}

impl<K: Ord + Copy, V> Bucket<K, V> {
    fn push(&mut self, key: K, value: V) {
        self.max = max(self.max, Some(key));
        self.elems.push((key, value));
    }

    fn pop(&mut self) -> Option<(K, V)> {
        self.elems.pop()
    }
}

impl<K, V> Default for Bucket<K, V> {
    fn default() -> Bucket<K, V> {
        Bucket {
            max: None,
            elems: Vec::new(),
        }
    }
}

/// A montone priority queue implemented with a radix heap.
///
/// This will be a max-heap.
///
/// A monotone priority queue is a variant of priority queues (itself a
/// generalization of heaps) that requires that the extracted elements follow a
/// monotonic sequence. This means that you cannot insert an element into a
/// radix heap that is smaller than the last extracted element.
///
/// The key of the last extracted element is called the "top" key of the radix
/// heap. Thus any value pushed onto the heap must be larger than or equal to
/// the top key.
///
/// In return for this restriction, the radix heap does O(1) inserts. Popping an
/// element is O(log m) where m is the difference between a popped key and the
/// top key at the time the element was inserted. Note that this does not depend
/// on the number of elements in the radix heap. This means that for workloads
/// where this difference is bounded by a constant, the radix heap has O(1) pop.
///
/// It is a logic error for a key to be modified in such a way that the
/// item's ordering relative to any other item, as determined by the `Ord`
/// trait, changes while it is in the heap. This is normally only possible
/// through `Cell`, `RefCell`, global state, I/O, or unsafe code.
#[derive(Clone)]
pub struct RadixHeapMap<K, V> {
    len: usize,

    /// The current top key, or none if one is not set yet.
    top: Option<K>,

    /// The K::RADIX_BITS + 1 number of buckets the items can land in.
    ///
    /// TODO: when rust supports associated consts as array sizes, use a fixed
    /// array instead of a vec.
    buckets: Vec<Bucket<K, V>>,

    /// The initial entries before a top key is found.
    initial: Bucket<K, V>,
}

impl<K: Radix + Ord + Copy, V> RadixHeapMap<K, V> {
    /// Create an empty `RadixHeapMap`
    pub fn new() -> RadixHeapMap<K, V> {
        RadixHeapMap {
            len: 0,
            top: None,
            buckets: (0..=K::RADIX_BITS).map(|_| Bucket::default()).collect(),
            initial: Bucket::default(),
        }
    }

    /// Create an empty `RadixHeapMap` with the top key set to a specific
    /// value.
    ///
    /// This can be more efficient if you have a known minimum bound of the
    /// items being pushed to the heap.
    pub fn new_at(top: K) -> RadixHeapMap<K, V> {
        RadixHeapMap {
            len: 0,
            top: Some(top),
            buckets: (0..=K::RADIX_BITS).map(|_| Bucket::default()).collect(),
            initial: Bucket::default(),
        }
    }

    /// Drops all items form the `RadixHeapMap` and sets the top key to `None`.
    pub fn clear(&mut self) {
        self.len = 0;
        self.top = None;
        self.initial.clear();

        for bucket in &mut self.buckets {
            bucket.clear();
        }
    }

    /// Drop all items from the `RadixHeapMap` and sets the top key to a
    /// specific value.
    ///
    /// This can be more efficient if you have a known minimum bound of the
    /// items being pushed to the heap.
    pub fn clear_to(&mut self, top: K) {
        self.clear();
        self.top = Some(top);
    }

    #[inline]
    fn push_nocheck(&mut self, key: K, value: V, top: K) {
        self.buckets[key.radix_distance(&top) as usize].push(key, value);
    }

    #[inline]
    fn repush_bucket<F>(&mut self, mut bucket: F)
    where
        F: FnMut(&mut Self) -> &mut Bucket<K, V>,
    {
        let mut repush = Bucket::default();

        swap(bucket(self), &mut repush);
        let top = repush.max.expect("Expected non-empty bucket");
        self.top = Some(top);

        for (k, v) in repush.drain() {
            self.push_nocheck(k, v, top);
        }

        // Swap `repush` back again (purely to save memory allocation,
        // they are both empty at this point but `repush` has some
        // memory allocated)
        swap(bucket(self), &mut repush);

        debug_assert!(repush.is_empty());
    }

    /// Pushes a new key value pair onto the heap.
    ///
    /// Panics
    /// ------
    /// Panics if the key is more than the current top key.
    #[inline]
    pub fn push(&mut self, key: K, value: V) {
        if let Some(top) = self.top {
            assert!(key <= top, "Key must be lower or equal to current top key");
            self.push_nocheck(key, value, top);
        } else {
            self.initial.push(key, value);
        }

        self.len += 1;
    }

    /// Sets the top value to the current maximum key value in the heap
    pub fn constrain(&mut self) {
        if self.top.is_some() {
            let index = self
                .buckets
                .iter()
                .enumerate()
                .find(|&(_, bucket)| !bucket.is_empty())
                .map(|(i, _)| i);

            if let Some(index) = index {
                if index != 0 {
                    self.repush_bucket(|x| &mut x.buckets[index]);
                }
            }
        } else if !self.initial.is_empty() {
            self.repush_bucket(|x| &mut x.initial);
        }
    }

    /// Remove the greatest element from the heap and returns it, or `None` if
    /// empty.
    ///
    /// If there is a tie between multiple elements, the last inserted element
    /// will be popped first.
    ///
    /// This will set the top key to the extracted key.
    #[inline]
    pub fn pop(&mut self) -> Option<(K, V)> {
        self.constrain();

        let pop = self
            .buckets
            .first_mut()
            .expect("Expected at least one bucket")
            .pop();

        if pop.is_some() {
            self.len -= 1;
        }

        pop
    }

    /// Returns the number of elements in the heap
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if there is no elements in the heap
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The current top value. All keys pushed onto the heap must be smaller
    /// than this value.
    pub fn top(&self) -> Option<K> {
        self.top
    }

    /// Discards as much additional capacity as possible.
    pub fn shrink_to_fit(&mut self) {
        self.initial.shrink_to_fit();

        for bucket in &mut self.buckets {
            bucket.shrink_to_fit();
        }
    }

    /// Returns an iterator of all key-value pairs in the RadixHeapMap in arbitrary order
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            cur_bucket: self.initial.iter(),
            buckets: self.buckets.iter(),
            size: self.len,
        }
    }

    /// Returns an iterator of all keys in the RadixHeapMap in arbitrary order
    pub fn keys(&self) -> Keys<K, V> {
        Keys(self.iter())
    }

    /// Returns an iterator of all values in the RadixHeapMap in arbitrary order
    pub fn values(&self) -> Values<K, V> {
        Values(self.iter())
    }
}

impl<K: Radix + Ord + Copy, V> Default for RadixHeapMap<K, V> {
    fn default() -> RadixHeapMap<K, V> {
        RadixHeapMap::new()
    }
}

impl<K: Radix + Ord + Copy, V> FromIterator<(K, V)> for RadixHeapMap<K, V> {
    fn from_iter<I>(iter: I) -> RadixHeapMap<K, V>
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut heap = RadixHeapMap::new();

        for (k, v) in iter {
            heap.push(k, v);
        }

        heap
    }
}

impl<K: Radix + Ord + Copy, V> Extend<(K, V)> for RadixHeapMap<K, V> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        for (k, v) in iter {
            self.push(k, v);
        }
    }
}

impl<'a, K: Radix + Ord + Copy + 'a, V: Copy + 'a> Extend<&'a (K, V)> for RadixHeapMap<K, V> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a (K, V)>,
    {
        for &(k, v) in iter {
            self.push(k, v);
        }
    }
}

impl<K: Radix + Ord + Copy + fmt::Debug, V: fmt::Debug> fmt::Debug for RadixHeapMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// An owning iterator over key-value pairs in a RadixHeapMap.
#[derive(Clone)]
pub struct IntoIter<K, V> {
    cur_bucket: std::vec::IntoIter<(K, V)>,
    buckets: std::vec::IntoIter<Bucket<K, V>>,
    size: usize,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let pair @ Some(_) = self.cur_bucket.next() {
                self.size -= 1;
                return pair;
            } else {
                self.cur_bucket = self.buckets.next()?.into_iter();
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }

    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Item),
    {
        self.cur_bucket.for_each(&mut f);
        self.buckets.for_each(|b| b.into_iter().for_each(&mut f));
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {}

impl<K, V> FusedIterator for IntoIter<K, V> {}

/// An iterator over key-value pairs in a RadixHeapMap.
#[derive(Clone)]
pub struct Iter<'a, K, V> {
    cur_bucket: std::slice::Iter<'a, (K, V)>,
    buckets: std::slice::Iter<'a, Bucket<K, V>>,
    size: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = &'a (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let pair @ Some(_) = self.cur_bucket.next() {
                self.size -= 1;
                return pair;
            } else {
                self.cur_bucket = self.buckets.next()?.iter();
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }

    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Item),
    {
        self.cur_bucket.for_each(&mut f);
        self.buckets.for_each(|b| b.iter().for_each(&mut f));
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}

impl<'a, K, V> FusedIterator for Iter<'a, K, V> {}

/// An iterator over keys in a RadixHeapMap.
#[derive(Clone)]
pub struct Keys<'a, K, V>(Iter<'a, K, V>);

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Item),
    {
        self.0.for_each(|(k, _)| f(k))
    }
}

impl<'a, K, V> ExactSizeIterator for Keys<'a, K, V> {}

impl<'a, K, V> FusedIterator for Keys<'a, K, V> {}

/// An iterator over values in a RadixHeapMap.
#[derive(Clone)]
pub struct Values<'a, K, V>(Iter<'a, K, V>);

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Item),
    {
        self.0.for_each(|(_, v)| f(v))
    }
}

impl<'a, K, V> ExactSizeIterator for Values<'a, K, V> {}

impl<'a, K, V> FusedIterator for Values<'a, K, V> {}

impl<K: Radix + Ord + Copy, V> IntoIterator for RadixHeapMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            cur_bucket: self.initial.into_iter(),
            buckets: self.buckets.into_iter(),
            size: self.len,
        }
    }
}

impl<'a, K: Radix + Ord + Copy, V> IntoIterator for &'a RadixHeapMap<K, V> {
    type Item = &'a (K, V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A number that can be compared using radix distance
pub trait Radix {
    /// The number of high bits in a row that this and `other` has in common
    ///
    /// Eg. the radix similarity of 001001 and 000001 is 2 because they share
    /// the 2 high bits.
    fn radix_similarity(&self, other: &Self) -> u32;

    /// Opposite of `radix_similarity`. If `radix_distance` returns 0, then `radix_similarity`
    /// returns `radix_bits` and vice versa.
    fn radix_distance(&self, other: &Self) -> u32 {
        Self::RADIX_BITS - self.radix_similarity(other)
    }

    /// The value returned by `radix_similarty` if all bits are equal
    const RADIX_BITS: u32;
}

macro_rules! radix_wrapper_impl {
    ($t:ident) => {
        impl<T: Radix> Radix for $t<T> {
            #[inline]
            fn radix_similarity(&self, other: &$t<T>) -> u32 {
                self.0.radix_similarity(&other.0)
            }

            const RADIX_BITS: u32 = T::RADIX_BITS;
        }
    };
}

radix_wrapper_impl!(Reverse);
radix_wrapper_impl!(Wrapping);

macro_rules! radix_int_impl {
    ($t:ty) => {
        impl Radix for $t {
            #[inline]
            fn radix_similarity(&self, other: &$t) -> u32 {
                (self ^ other).leading_zeros()
            }

            const RADIX_BITS: u32 = (std::mem::size_of::<$t>() * 8) as u32;
        }
    };
}

radix_int_impl!(i8);
radix_int_impl!(i16);
radix_int_impl!(i32);
radix_int_impl!(i64);
radix_int_impl!(isize);

radix_int_impl!(u8);
radix_int_impl!(u16);
radix_int_impl!(u32);
radix_int_impl!(u64);
radix_int_impl!(usize);

macro_rules! radix_float_impl {
    ($t:ty) => {
        impl Radix for NotNaN<$t> {
            #[inline]
            fn radix_similarity(&self, other: &NotNaN<$t>) -> u32 {
                self.bits().radix_similarity(&other.bits())
            }

            const RADIX_BITS: u32 = <$t as Ieee754>::Bits::RADIX_BITS;
        }
    };
}

radix_float_impl!(f32);
radix_float_impl!(f64);

impl Radix for () {
    #[inline]
    fn radix_similarity(&self, _: &()) -> u32 {
        0
    }
    const RADIX_BITS: u32 = 0;
}

macro_rules! radix_tuple_impl {
    ($(
        $Tuple:ident {
            $(($idx:tt) -> $T:ident)+
        }
    )+) => {
        $(
            impl<$($T:Radix),+> Radix for ($($T,)+) {
                #[inline]
                fn radix_similarity(&self, other: &($($T,)+)) -> u32 {
                    let similarity = 0;

                    $(
                        let s = self.$idx.radix_similarity(&other.$idx);
                        let similarity = similarity + s;
                        if s < <$T as Radix>::RADIX_BITS { return similarity }
                    )+

                    return similarity;
                }
                const RADIX_BITS: u32 = 0 $(+<$T as Radix>::RADIX_BITS)+;
            }
        )+
    }
}

radix_tuple_impl! {
    Tuple1 {
        (0) -> A
    }
    Tuple2 {
        (0) -> A
        (1) -> B
    }
    Tuple3 {
        (0) -> A
        (1) -> B
        (2) -> C
    }
    Tuple4 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
    }
    Tuple5 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
    }
    Tuple6 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
    }
    Tuple7 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
    }
    Tuple8 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
    }
    Tuple9 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
    }
    Tuple10 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
    }
    Tuple11 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
    }
    Tuple12 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
        (11) -> L
    }
}

#[cfg(test)]
mod tests {
    extern crate quickcheck;

    use self::quickcheck::{quickcheck, TestResult};
    use super::Radix;
    use super::RadixHeapMap;
    use ordered_float::NotNaN;
    use std::cmp::Reverse;
    use std::f32;

    #[test]
    fn radix_dist() {
        assert!(4u32.radix_distance(&2) == 3);
        assert!(3u32.radix_distance(&2) == 1);
        assert!(2u32.radix_distance(&2) == 0);
        assert!(1u32.radix_distance(&2) == 2);
        assert!(0u32.radix_distance(&2) == 2);
    }

    #[test]
    fn clear() {
        let mut heap = RadixHeapMap::new();
        heap.push(0u32, 'a');
        heap.clear();
        assert!(heap.pop().is_none());
    }

    #[test]
    fn push_pop() {
        let mut heap = RadixHeapMap::new();
        heap.push(0u32, 'a');
        heap.push(3, 'b');
        heap.push(2, 'c');

        assert!(heap.len() == 3);
        assert!(!heap.is_empty());

        assert!(heap.pop() == Some((3, 'b')));
        assert!(heap.pop() == Some((2, 'c')));
        assert!(heap.pop() == Some((0, 'a')));
        assert!(heap.pop() == None);

        assert!(heap.len() == 0);
        assert!(heap.is_empty());
    }

    #[test]
    fn rev_push_pop() {
        let mut heap = RadixHeapMap::new();
        heap.push(Reverse(0), 'a');
        heap.push(Reverse(3), 'b');
        heap.push(Reverse(2), 'c');

        assert!(heap.len() == 3);
        assert!(!heap.is_empty());

        assert!(heap.pop() == Some((Reverse(0), 'a')));
        assert!(heap.pop() == Some((Reverse(2), 'c')));
        assert!(heap.pop() == Some((Reverse(3), 'b')));
        assert!(heap.pop() == None);

        assert!(heap.len() == 0);
        assert!(heap.is_empty());
    }

    #[test]
    #[should_panic]
    fn push_pop_panic() {
        let mut heap = RadixHeapMap::new();
        heap.push(0u32, 'a');
        heap.push(3, 'b');

        assert!(heap.pop() == Some((3, 'b')));
        heap.push(4, 'd');
    }

    #[test]
    fn sort() {
        fn prop<T: Ord + Radix + Copy>(mut xs: Vec<T>) -> bool {
            let mut heap: RadixHeapMap<_, _> =
                xs.iter().enumerate().map(|(i, &d)| (d, i)).collect();

            xs.sort();

            while xs.pop() == heap.pop().map(|(k, _)| k) {
                if xs.is_empty() {
                    return true;
                }
            }

            return false;
        }

        quickcheck(prop as fn(Vec<()>) -> bool);
        quickcheck(prop as fn(Vec<u32>) -> bool);
        quickcheck(prop as fn(Vec<i32>) -> bool);
        quickcheck(prop as fn(Vec<(u32, i32)>) -> bool);
        quickcheck(prop as fn(Vec<u8>) -> bool);
        quickcheck(prop as fn(Vec<i16>) -> bool);
        quickcheck(prop as fn(Vec<(i64, usize)>) -> bool);
    }

    #[test]
    fn sort_float() {
        fn prop(xs: Vec<f32>) -> TestResult {
            if xs.iter().any(|x| x.is_nan()) {
                return TestResult::discard();
            }

            let mut xs: Vec<_> = xs.into_iter().map(|x| NotNaN::from(x)).collect();
            xs.sort();

            let mut heap: RadixHeapMap<_, _> =
                xs.iter().enumerate().map(|(i, &d)| (d, i)).collect();

            while xs.pop() == heap.pop().map(|(k, _)| k) {
                if xs.is_empty() {
                    return TestResult::passed();
                }
            }

            return TestResult::failed();
        }

        quickcheck(prop as fn(Vec<f32>) -> TestResult);
    }

    #[test]
    fn iter_yeilds_all_elements() {
        fn prop<T: Ord + Radix + Copy>(mut xs: Vec<T>) -> TestResult {
            let heap = xs.iter().map(|&d| (d, ())).collect::<RadixHeapMap<_, _>>();

            // Check that the iterator yields all elements inside the heap
            for (k, ()) in heap.iter() {
                for i in 0..xs.len() {
                    if xs[i] == *k {
                        xs.remove(i);
                        break;
                    }
                }
            }

            if xs.is_empty() {
                TestResult::passed()
            } else {
                TestResult::failed()
            }
        }

        quickcheck(prop as fn(Vec<u32>) -> TestResult);
        quickcheck(prop as fn(Vec<i32>) -> TestResult);
        quickcheck(prop as fn(Vec<(u32, i32)>) -> TestResult);
        quickcheck(prop as fn(Vec<u8>) -> TestResult);
        quickcheck(prop as fn(Vec<i16>) -> TestResult);
        quickcheck(prop as fn(Vec<(i64, usize)>) -> TestResult);
    }

    #[test]
    fn into_iter_inital() {
        let mut heap = RadixHeapMap::new();
        heap.push(1, 2);
        heap.push(5, 2);

        let mut vec: Vec<_> = heap.into_iter().collect();
        vec.sort();
        assert_eq!(vec, vec![(1, 2), (5, 2)]);
    }

    #[test]
    fn into_iter() {
        let mut heap = RadixHeapMap::new();
        heap.push(1, 2);
        heap.push(5, 4);
        heap.push(7, 1);

        assert_eq!(Some((7, 1)), heap.pop());

        let mut vec: Vec<_> = heap.into_iter().collect();
        vec.sort();
        assert_eq!(vec, vec![(1, 2), (5, 4)]);
    }
}
