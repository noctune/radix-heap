//! A monotone priority queue implemented with a radix heap.
//!
//! A monotone priority queue is a priority queue that only allows keys to be
//! inserted if their priority is less than or equal to that of the last
//! extracted key.
//!
//! In this implementation, the last extracted value is known as the "top"
//! value. The top value does not necessarily have to be the last extracted key.
//! It can be lower, or initially it can be any value. Note that the constraint
//! above still applies in such a case.
//!
//! Insertion is O(1) time complexity, while popping is amortized O(log n).
//! More precisely, popping is O(d) where d is the radix distance from the key
//! to the top value at the time of insertion. This can give better asymptotic
//! running times for certain algorithms like for example Djikstra's algorithm.
//!
//! #Example
//!
//! ```
//! extern crate radix_heap;
//!
//! let mut heap = radix_heap::RadixHeapMap::new();
//! heap.push(7, 'a');
//! heap.push(2, 'b');
//! heap.push(9, 'c');
//! 
//! assert!(heap.pop() == Some((9, 'c')));
//! assert!(heap.top() == 9);
//! assert!(heap.pop() == Some((7, 'a')));
//! assert!(heap.top() == 7);
//! assert!(heap.pop() == Some((2, 'b')));
//! assert!(heap.top() == 2);
//! assert!(heap.pop() == None);
//! ```

extern crate ieee754;
extern crate revord;
extern crate ordered_float;
extern crate num;

use std::fmt;
use std::iter::FromIterator;
use std::cmp::max;
use std::mem::swap;
use std::default::Default;
use std::num::Wrapping;
use revord::RevOrd;
use ieee754::Ieee754;
use ordered_float::NotNaN;
use num::Bounded;

#[derive(Clone)]
struct Bucket<K,V> {
    max: K,
    elems: Vec<(K,V)>,
}

/// A montone priority queue implemented with a radix heap.
///
/// This will be a max-heap.
///
/// It is a logic error for a key to be modified in such a way that the
/// item's ordering relative to any other item, as determined by the `Ord`
/// trait, changes while it is in the heap. This is normally only possible
/// through `Cell`, `RefCell`, global state, I/O, or unsafe code.
#[derive(Clone)]
pub struct RadixHeapMap<K,V> {
    len: usize,
    top: K,
    buckets: Vec<Bucket<K,V>>,
}

impl<K: Radix + Ord + Copy + Bounded, V> RadixHeapMap<K,V> {
    /// Creates a new `RadixHeapMap` with the highest possible top value such
    /// that all keys are allowed.
    pub fn new() -> RadixHeapMap<K,V> {
        RadixHeapMap::new_at(K::max_value())
    }
    
    /// Clears a new `RadixHeapMap` and sets te top value as high as possible
    /// value, so all keys can be inserted into the heap.
    pub fn clear(&mut self) {
        self.clear_to(K::max_value());
    }
}

impl<K: Radix + Ord + Copy,V> RadixHeapMap<K,V> {
    
    /// Creates a new RadixHeapMap with a specific top value
    pub fn new_at(top: K) -> RadixHeapMap<K,V> {
        let maxdist = K::radix_bits();
        let mut buckets = Vec::with_capacity(maxdist as usize + 1);
        
        for _ in 0..maxdist + 1 {
            buckets.push(Bucket{
                max: top,
                elems: Vec::new()
            });
        }
        
        RadixHeapMap { len: 0, top: top, buckets: buckets } 
    }
    
    /// Clears the elements in the heap and sets the top value to a new value
    pub fn clear_to(&mut self, top: K) {
        self.top = top;
        self.len = 0;
        
        for bucket in &mut self.buckets {
            bucket.elems.clear();
        }
    }
    
    #[inline]
    fn push_nocheck(&mut self, key: K, value: V) {
        let bucket = &mut self.buckets[key.radix_similarity(&self.top) as usize];
        
        bucket.max = max(bucket.max, key);
        
        if bucket.elems.is_empty() {
            bucket.max = key;
        }
        
        bucket.elems.push((key,value));
    }
    
    /// Pushes a new key value pair onto the heap.
    /// 
    /// Panics
    /// ------
    /// Panics if the key is more than the current top value.
    #[inline]
    pub fn push(&mut self, key: K, value: V) {
        assert!(key <= self.top);
        self.len += 1;
        self.push_nocheck(key, value);
    }
    
    /// Sets the top value to the current maximum key value in the heap
    pub fn constrain(&mut self) {
        if self.buckets.last().unwrap().elems.is_empty() {
            let bucket = self.buckets[..self.buckets.len() - 1].iter()
                .enumerate()
                .rev()
                .filter(|&(_, ref bucket)| !bucket.elems.is_empty())
                .next()
                .map(|(i, bucket)| (i, bucket.max));
            
            if let Some((index,max)) = bucket {
                self.top = max;
                let mut repush = Vec::new();
                
                swap(&mut self.buckets[index].elems, &mut repush);
                
                for (k,v) in repush.drain(..) {
                    self.push_nocheck(k, v);
                }
                
                // Swap `repush` back again (purely to save memory allocation,
                // they are both empty at this point but `repush` has some
                // memory allocated)
                swap(&mut self.buckets[index].elems, &mut repush);
            }
        }
    }
    
    /// Pops the largest element of the heap. This might decrease the top value.
    #[inline]
    pub fn pop(&mut self) -> Option<(K,V)> {
        self.constrain();
        
        let pop = self.buckets.last_mut().unwrap().elems.pop();
        
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
    pub fn top(&self) -> K {
        self.top
    }
    
    /// Discards as much additional capacity as possible.
    pub fn shrink_to_fit(&mut self) {
        for bucket in &mut self.buckets {
            bucket.elems.shrink_to_fit();
        }
    }
}

impl<K: Radix + Ord + Copy + Bounded, V> Default for RadixHeapMap<K,V> {
    fn default() -> RadixHeapMap<K,V> {
        RadixHeapMap::new()
    }
}

impl<K: Radix + Ord + Copy + Bounded, V> FromIterator<(K,V)> for RadixHeapMap<K,V> {
    fn from_iter<I>(iter: I) -> RadixHeapMap<K,V> where
        I: IntoIterator<Item=(K,V)>
    {
        let mut heap = RadixHeapMap::new();
        
        for (k,v) in iter {
            heap.len += 1;
            heap.push_nocheck(k,v);
        }
        
        heap
    }
}

impl<K: Radix + Ord + Copy, V> Extend<(K,V)> for RadixHeapMap<K,V> {
    fn extend<I>(&mut self, iter: I) where
        I: IntoIterator<Item=(K,V)>
    {
        for (k,v) in iter {
            self.push(k,v);
        }
    }
}

impl<'a, K: Radix + Ord + Copy + 'a, V: Copy + 'a> Extend<&'a (K,V)> for RadixHeapMap<K,V> {
    fn extend<I>(&mut self, iter: I) where
        I: IntoIterator<Item=&'a (K,V)>
    {
        for &(k,v) in iter {
            self.push(k,v);
        }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for RadixHeapMap<K,V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let entries = self.buckets.iter().flat_map(|b| b.elems.iter());
        f.debug_list().entries(entries).finish()
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
        Self::radix_bits() - self.radix_similarity(other)
    }
    
    /// The value returned by `radix_similarty` if all bits are equal
    fn radix_bits() -> u32;
}

macro_rules! radix_wrapper_impl {
    ($t:ident) => {
        impl<T: Radix> Radix for $t<T> {
            #[inline]
            fn radix_similarity(&self, other: &$t<T>) -> u32 {
                self.0.radix_similarity(&other.0)
            }
            
            #[inline]
            fn radix_bits() -> u32 {
                T::radix_bits()
            }
        }
    }
}

radix_wrapper_impl!(RevOrd);
radix_wrapper_impl!(Wrapping);

macro_rules! radix_int_impl {
    ($t:ty) => {
        impl Radix for $t {
            #[inline]
            fn radix_similarity(&self, other: &$t) -> u32 {
                (self ^ other).leading_zeros()
            }
            
            #[inline]
            fn radix_bits() -> u32 {
                (std::mem::size_of::<$t>() * 8) as u32
            }
        }
    }
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
                self.0.bits().radix_similarity(&other.0.bits())
            }
            
            #[inline]
            fn radix_bits() -> u32 {
                <$t as Ieee754>::Bits::radix_bits()
            }
        }
    }
}

radix_float_impl!(f32);
radix_float_impl!(f64);


macro_rules! e {
    ($e:expr) => { $e }
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
                        let s = e!(self.$idx.radix_similarity(&other.$idx));
                        let similarity = similarity + s;
                        if s < <$T as Radix>::radix_bits() { return similarity }
                    )+
                    
                    return similarity;
                }
                
                #[inline]
                fn radix_bits() -> u32 {
                    0 $(+e!(<$T as Radix>::radix_bits()))+
                }
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
    
    use std::f32;
    use ordered_float::NotNaN;
    use super::Radix;
    use super::RadixHeapMap;
    use super::num::Bounded;
    use self::quickcheck::{TestResult, quickcheck};
    
    #[test]
    fn radix_dist() {
        assert!(4u32.radix_distance(&2) == 3);
        assert!(3u32.radix_distance(&2) == 1);
        assert!(2u32.radix_distance(&2) == 0);
        assert!(1u32.radix_distance(&2) == 2);
        assert!(0u32.radix_distance(&2) == 2);
    }
    
    #[test]
    fn push_pop() {
        let mut heap = RadixHeapMap::new();
        heap.push(0u32, 'a');
        heap.push(3, 'b');
        heap.push(2, 'c');
        
        assert!(heap.len() == 3); 
        assert!(!heap.is_empty()); 
        
        assert!(heap.pop() == Some((3,'b'))); 
        assert!(heap.pop() == Some((2,'c'))); 
        assert!(heap.pop() == Some((0,'a'))); 
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
        
        assert!(heap.pop() == Some((3,'b')));
        heap.push(4, 'd');
    }
    
    #[test]
    fn sort() {
        fn prop<T: Ord + Radix + Copy + Bounded>(mut xs: Vec<T>) -> bool {
            let mut heap = xs.iter().map(|&d| (d,())).collect::<RadixHeapMap<_,_>>();
            
            xs.sort();
            
            while xs.pop() == heap.pop().map(|(k,_)| k) {
                if xs.is_empty() {
                    return true;
                }
            }
            
            return false;
        }
        
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
            
            let mut xs: Vec<_> = xs.into_iter().map(|x| NotNaN(x)).collect();
            
            let mut heap = RadixHeapMap::new_at(NotNaN(f32::INFINITY));
            heap.extend(xs.iter().map(|&d| (d,())));
            
            xs.sort();
            
            while xs.pop() == heap.pop().map(|(k,_)| k) {
                if xs.is_empty() {
                    return TestResult::passed();
                }
            }
            
            return TestResult::failed();
        }
        
        quickcheck(prop as fn(Vec<f32>) -> TestResult);
    }
}
