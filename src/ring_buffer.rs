//! Items related to the implementation of ring buffers.
//!
//! The primary items of interest in this module include:
//!
//! - The [Slice](./trait.Slice.html) and [SliceMut](./trait.SliceMut.html) traits - implemented
//! for types that may be used as the underlying buffer in `Fixed` and `Bounded` ring buffers.
//! - The [Fixed](./struct.Fixed.html) ring buffer type.
//! - The [Bounded](./struct.Bounded.html) ring buffer type.

use Box;
use core::mem;
use core::iter::{Chain, Cycle, FromIterator, Skip, Take};
use core::ptr;
use core::ops::{Index, IndexMut};
use core::slice;
use Vec;

////////////////////////
///// SLICE TRAITS /////
////////////////////////

/// Types that may be used as a data slice for `Fixed` and `Bounded` ring buffers.
pub trait Slice {
    /// The type contained within the slice.
    type Element;
    /// Borrow the data slice.
    fn slice(&self) -> &[Self::Element];
}

/// Types that may be used as a data slice for mutable `Fixed` and `Bounded` ring buffers.
pub trait SliceMut: Slice {
    /// Mutably borrow the data slice.
    fn slice_mut(&mut self) -> &mut [Self::Element];
}

/// Types that may be used as a constant-length buffer underlying a `Bounded` ring buffer.
pub trait FixedSizeArray {
    /// The constant length.
    const LEN: usize;
}

impl<'a, T> Slice for &'a [T] {
    type Element = T;
    #[inline]
    fn slice(&self) -> &[Self::Element] {
        self
    }
}

impl<'a, T> Slice for &'a mut [T] {
    type Element = T;
    #[inline]
    fn slice(&self) -> &[Self::Element] {
        self
    }
}

impl<'a, T> SliceMut for &'a mut [T] {
    #[inline]
    fn slice_mut(&mut self) -> &mut [Self::Element] {
        self
    }
}

impl<T> Slice for Box<[T]> {
    type Element = T;
    #[inline]
    fn slice(&self) -> &[Self::Element] {
        &self[..]
    }
}

impl<T> SliceMut for Box<[T]> {
    #[inline]
    fn slice_mut(&mut self) -> &mut [Self::Element] {
        &mut self[..]
    }
}

impl<T> Slice for Vec<T> {
    type Element = T;
    #[inline]
    fn slice(&self) -> &[Self::Element] {
        &self[..]
    }
}

impl<T> SliceMut for Vec<T> {
    #[inline]
    fn slice_mut(&mut self) -> &mut [Self::Element] {
        &mut self[..]
    }
}

macro_rules! impl_slice_for_arrays {
    ($($N:expr)*) => {
        $(
            impl<T> Slice for [T; $N] {
                type Element = T;
                #[inline]
                fn slice(&self) -> &[Self::Element] {
                    &self[..]
                }
            }
            impl<T> SliceMut for [T; $N] {
                #[inline]
                fn slice_mut(&mut self) -> &mut [Self::Element] {
                    &mut self[..]
                }
            }
            impl<T> FixedSizeArray for [T; $N] {
                const LEN: usize = $N;
            }
        )*
    };
}

impl_slice_for_arrays! {
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34
    35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63 64 65
    66 67 68 69 70 71 72 73 74 75 76 77 78 79 80 81 82 83 84 85 86 87 88 89 90 91 92 93 94 95 96
    97 98 99 100 101 102 103 104 105 106 107 108 109 110 111 112 113 114 115 116 117 118 119 120
    121 122 123 124 125 126 127 128 256 512 1024 2048 4096 8192
}

/////////////////////////////
///// FIXED RING BUFFER /////
/////////////////////////////

/// A ring buffer with a fixed length.
///
/// *AKA Circular buffer, cyclic buffer, FIFO queue.*
///
/// Elements are pushed and popped from the buffer simultaneously in order to retain a consistent
/// length.
///
/// A `Fixed` ring buffer can be created around any type with a slice to write to.
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     // From a fixed size array.
///     sample::ring_buffer::Fixed::from([1, 2, 3, 4]);
///
///     // From a Vec.
///     sample::ring_buffer::Fixed::from(vec![1, 2, 3, 4]);
///
///     // From a Boxed slice.
///     sample::ring_buffer::Fixed::from(vec![1, 2, 3].into_boxed_slice());
///
///     // From a mutably borrowed slice.
///     let mut slice = [1, 2, 3, 4];
///     sample::ring_buffer::Fixed::from(&mut slice[..]);
///
///     // An immutable ring buffer from an immutable slice.
///     let slice = [1, 2, 3, 4];
///     sample::ring_buffer::Fixed::from(&slice[..]);
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Fixed<S> {
    first: usize,
    data: S,
}

impl<S> Fixed<S>
where
    S: Slice,
{
    /// The fixed length of the buffer.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let rb = ring_buffer::Fixed::from([0; 4]);
    ///     assert_eq!(rb.len(), 4);
    /// }
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.data.slice().len()
    }

    /// Push the given item onto the back of the queue and return the item at the front of the
    /// queue, ensuring that the length is retained.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let mut rb = ring_buffer::Fixed::from([0, 1, 2, 3]);
    ///     assert_eq!(rb.push(4), 0);
    ///     assert_eq!(rb.push(5), 1);
    ///     assert_eq!(rb.push(6), 2);
    ///     assert_eq!(rb.push(7), 3);
    ///     assert_eq!(rb.push(8), 4);
    ///     assert_eq!([rb[0], rb[1], rb[2], rb[3]], [5, 6, 7, 8]);
    /// }
    /// ```
    pub fn push(&mut self, item: S::Element) -> S::Element
    where
        S: SliceMut,
    {
        let mut next_index = self.first + 1;
        if next_index == self.len() {
            next_index = 0;
        }
        // We know there is a fixed length so we can safely avoid bounds checking.
        let old_item =
            unsafe { mem::replace(self.data.slice_mut().get_unchecked_mut(self.first), item) };
        self.first = next_index;
        old_item
    }

    /// Borrows the item at the given index.
    ///
    /// If `index` is out of range it will be looped around the length of the data slice.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let mut rb = ring_buffer::Fixed::from([0, 1, 2]);
    ///     assert_eq!(*rb.get(0), 0);
    ///     assert_eq!(*rb.get(1), 1);
    ///     assert_eq!(*rb.get(2), 2);
    ///     assert_eq!(*rb.get(3), 0);
    ///     assert_eq!(*rb.get(4), 1);
    ///     assert_eq!(*rb.get(5), 2);
    /// }
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> &S::Element {
        let wrapped_index = (self.first + index) % self.len();
        &self.data.slice()[wrapped_index]
    }

    /// Mutably borrows the item at the given index.
    ///
    /// If `index` is out of range it will be looped around the length of the data slice.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> &mut S::Element
    where
        S: SliceMut,
    {
        let wrapped_index = (self.first + index) % self.len();
        &mut self.data.slice_mut()[wrapped_index]
    }

    /// Sets the index of the first element within the data slice.
    ///
    /// If `index` is out of range it will be looped around the length of the data slice.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let mut rb = ring_buffer::Fixed::from([0, 1, 2, 3]);
    ///     assert_eq!(rb[0], 0);
    ///     rb.set_first(2);
    ///     assert_eq!(rb[0], 2);
    ///     rb.set_first(5);
    ///     assert_eq!(rb[0], 1);
    /// }
    /// ```
    #[inline]
    pub fn set_first(&mut self, index: usize) {
        self.first = index % self.len();
    }

    /// The start and end slices that make up the ring buffer.
    ///
    /// These two slices chained together represent all elements within the buffer in order.
    ///
    /// The first slice is always aligned contiguously behind the second slice.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// fn main() {
    ///     let mut ring_buffer = sample::ring_buffer::Fixed::from([0; 4]);
    ///     assert_eq!(ring_buffer.slices(), (&[0, 0, 0, 0][..], &[][..]));
    ///     ring_buffer.push(1);
    ///     ring_buffer.push(2);
    ///     assert_eq!(ring_buffer.slices(), (&[0, 0][..], &[1, 2][..]));
    ///     ring_buffer.push(3);
    ///     ring_buffer.push(4);
    ///     assert_eq!(ring_buffer.slices(), (&[1, 2, 3, 4][..], &[][..]));
    /// }
    /// ```
    #[inline]
    pub fn slices(&self) -> (&[S::Element], &[S::Element]) {
        let (end, start) = self.data.slice().split_at(self.first);
        (start, end)
    }

    /// The same as the `slices` method, but returns mutable slices instead.
    #[inline]
    pub fn slices_mut(&mut self) -> (&mut [S::Element], &mut [S::Element])
    where
        S: SliceMut,
    {
        let (end, start) = self.data.slice_mut().split_at_mut(self.first);
        (start, end)
    }

    /// Produce an iterator that repeatedly yields a reference to each element in the buffer.
    #[inline]
    pub fn iter_loop(&self) -> Skip<Cycle<slice::Iter<S::Element>>> {
        self.data.slice().iter().cycle().skip(self.first)
    }

    /// Produce an iterator that yields a reference to each element in the buffer.
    #[inline]
    pub fn iter(&self) -> Take<Skip<Cycle<slice::Iter<S::Element>>>> {
        self.iter_loop().take(self.data.slice().len())
    }

    /// Produce an iterator that yields a mutable reference to each element in the buffer.
    #[inline]
    pub fn iter_mut(&mut self) -> Chain<slice::IterMut<S::Element>, slice::IterMut<S::Element>>
    where
        S: SliceMut,
    {
        let (start, end) = self.slices_mut();
        start.iter_mut().chain(end.iter_mut())
    }

    /// Creates a `Fixed` ring buffer from its starting index and data buffer type.
    ///
    /// **Panic!**s if the given index is out of range of the given data slice.
    ///
    /// **Note:** This method should only be necessary if you require specifying a first index.
    /// Please see the `ring_buffer::Fixed::from` function for a simpler constructor that does not
    /// require a `first` index.
    #[inline]
    pub fn from_raw_parts(first: usize, data: S) -> Self {
        assert!(first < data.slice().len());
        Fixed { first, data }
    }

    /// Creates a `Fixed` ring buffer from its starting index and data buffer type.
    ///
    /// This method is unsafe as there is no guarantee that `first` < `data.slice().len()`.
    #[inline]
    pub unsafe fn from_raw_parts_unchecked(first: usize, data: S) -> Self {
        Fixed { first, data }
    }

    /// Consumes the `Fixed` ring buffer and returns its parts:
    ///
    /// - `usize` is an index into the first element of the buffer.
    /// - `S` is the buffer data.
    #[inline]
    pub fn into_raw_parts(self) -> (usize, S) {
        let Fixed { first, data } = self;
        (first, data)
    }
}

impl<S> From<S> for Fixed<S>
where
    S: Slice,
{
    /// Construct a `Fixed` ring buffer from the given data slice.
    ///
    /// **Panic!**s if the given `data` buffer is empty.
    #[inline]
    fn from(data: S) -> Self {
        Self::from_raw_parts(0, data)
    }
}

impl<S, T> FromIterator<T> for Fixed<S>
where
    S: Slice<Element = T> + FromIterator<T>,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let data = S::from_iter(iter);
        Self::from(data)
    }
}

impl<S> Index<usize> for Fixed<S>
where
    S: Slice,
{
    type Output = S::Element;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<S> IndexMut<usize> for Fixed<S>
where
    S: SliceMut,
{
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}

///////////////////////////////
///// BOUNDED RING BUFFER /////
///////////////////////////////

/// A ring buffer with an upper bound on its length.
///
/// *AKA Circular buffer, cyclic buffer, FIFO queue.*
///
/// Elements can be pushed to the back of the buffer and popped from the front.
///
/// Elements must be `Copy` due to the behaviour of the `push` and `pop` methods. If you require
/// working with non-`Copy` elements, the `std` `VecDeque` type may be better suited.
///
/// A `Bounded` ring buffer can be created from any type providing a slice to use for pushing and
/// popping elements.
///
/// ```
/// extern crate sample;
///
/// use sample::ring_buffer;
///
/// fn main() {
///     // From a fixed size array.
///     ring_buffer::Bounded::from([0; 4]);
///
///     // From a Vec.
///     ring_buffer::Bounded::from(vec![0; 4]);
///
///     // From a Boxed slice.
///     ring_buffer::Bounded::from(vec![0; 3].into_boxed_slice());
///
///     // From a mutably borrowed slice.
///     let mut slice = [0; 4];
///     ring_buffer::Bounded::from(&mut slice[..]);
///
///     // An immutable ring buffer from an immutable slice.
///     let slice = [0; 4];
///     ring_buffer::Bounded::from(&slice[..]);
/// }
/// ```
///
/// Two slightly more efficient constructors are provided for fixed-size arrays and boxed slices.
/// These are generally more efficient as they do not require initialising elements.
///
/// ```
/// extern crate sample;
///
/// use sample::ring_buffer;
///
/// fn main() {
///     // Fixed-size array.
///     ring_buffer::Bounded::<[i32; 4]>::array();
///
///     // Boxed slice.
///     let mut rb = ring_buffer::Bounded::boxed_slice(4);
///     rb.push(1);
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bounded<S> {
    start: usize,
    len: usize,
    data: S,
}

/// An iterator that drains the ring buffer by `pop`ping each element one at a time.
///
/// Note that only elements yielded by `DrainBounded::next` will be popped from the ring buffer.
/// That is, all non-yielded elements will remain in the ring buffer.
pub struct DrainBounded<'a, S: 'a> {
    bounded: &'a mut Bounded<S>,
}

impl<T> Bounded<Box<[T]>>
where
    T: Copy,
{
    /// A `Bounded` ring buffer that uses a `Box`ed slice with the given maximum length to
    /// represent the data.
    ///
    /// Slightly more efficient than using the similar `From` constructor as this creates the
    /// underlying slice with uninitialised memory.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let mut rb = ring_buffer::Bounded::boxed_slice(4);
    ///     assert_eq!(rb.max_len(), 4);
    ///     assert_eq!(rb.len(), 0);
    ///     rb.push(1);
    ///     rb.push(2);
    /// }
    /// ```
    pub fn boxed_slice(max_len: usize) -> Self {
        let mut vec = Vec::new();
        vec.reserve_exact(max_len);
        unsafe {
            vec.set_len(max_len);
            let data = vec.into_boxed_slice();
            Self::from_raw_parts(0, 0, data)
        }
    }
}

impl<S> Bounded<S>
where
    S: Slice,
    S::Element: Copy,
{
    /// A `Bounded` buffer that uses a fixed-size array to represent data.
    ///
    /// Slightly more efficient than using the similar `From` constructor as this creates the
    /// underlying array with uninitialised memory.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let mut rb = ring_buffer::Bounded::<[f32; 3]>::array();
    ///     assert_eq!(rb.len(), 0);
    ///     assert_eq!(rb.max_len(), 3);
    /// }
    /// ```
    pub fn array() -> Self
    where
        S: FixedSizeArray,
    {
        unsafe {
            let data = mem::uninitialized();
            Self::from_raw_parts(0, 0, data)
        }
    }

    /// The same as the `From` implementation, but assumes that the given `data` is full of valid
    /// elements and initialises the ring buffer with a length equal to `max_len`.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let mut rb = ring_buffer::Bounded::from_full([0, 1, 2, 3]);
    ///     assert_eq!(rb.len(), rb.max_len());
    ///     assert_eq!(rb.pop(), Some(0));
    ///     assert_eq!(rb.pop(), Some(1));
    ///     assert_eq!(rb.pop(), Some(2));
    ///     assert_eq!(rb.pop(), Some(3));
    ///     assert_eq!(rb.pop(), None);
    /// }
    /// ```
    pub fn from_full(data: S) -> Self {
        Self::from_raw_parts(0, data.slice().len(), data)
    }

    /// The maximum length that the `Bounded` buffer can be before pushing would overwrite the
    /// front of the buffer.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// fn main() {
    ///     let mut ring_buffer = sample::ring_buffer::Bounded::<[i32; 3]>::array();
    ///     assert_eq!(ring_buffer.max_len(), 3);
    /// }
    /// ```
    #[inline]
    pub fn max_len(&self) -> usize {
        self.data.slice().len()
    }

    /// The current length of the ring buffer.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// fn main() {
    ///     let mut ring_buffer = sample::ring_buffer::Bounded::<[i32; 3]>::array();
    ///     assert_eq!(ring_buffer.len(), 0);
    /// }
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether or not the ring buffer's length is equal to the maximum length.
    ///
    /// Equivalent to `self.len() == self.max_len()`.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let mut rb = ring_buffer::Bounded::<[i32; 2]>::array();
    ///     assert!(!rb.is_full());
    ///     rb.push(0);
    ///     rb.push(1);
    ///     assert!(rb.is_full());
    /// }
    /// ```
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == self.max_len()
    }

    /// The start and end slices that make up the ring buffer.
    ///
    /// These two slices chained together represent all elements within the buffer in order.
    ///
    /// The first slice is always aligned contiguously behind the second slice.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// fn main() {
    ///     let mut ring_buffer = sample::ring_buffer::Bounded::<[i32; 4]>::array();
    ///     assert_eq!(ring_buffer.slices(), (&[][..], &[][..]));
    ///     ring_buffer.push(1);
    ///     ring_buffer.push(2);
    ///     assert_eq!(ring_buffer.slices(), (&[1, 2][..], &[][..]));
    ///     ring_buffer.push(3);
    ///     ring_buffer.push(4);
    ///     assert_eq!(ring_buffer.slices(), (&[1, 2, 3, 4][..], &[][..]));
    ///     ring_buffer.push(5);
    ///     ring_buffer.push(6);
    ///     assert_eq!(ring_buffer.slices(), (&[3, 4][..], &[5, 6][..]));
    /// }
    /// ```
    #[inline]
    pub fn slices(&self) -> (&[S::Element], &[S::Element]) {
        let (end, start) = self.data.slice().split_at(self.start);
        if start.len() <= self.len {
            let end_len = self.len - start.len();
            (start, &end[..end_len])
        } else {
            (&start[..self.len], &end[..0])
        }
    }

    /// The same as the `slices` method, but returns mutable slices instead.
    #[inline]
    pub fn slices_mut(&mut self) -> (&mut [S::Element], &mut [S::Element])
    where
        S: SliceMut,
    {
        let (end, start) = self.data.slice_mut().split_at_mut(self.start);
        if start.len() <= self.len {
            let end_len = self.len - start.len();
            (start, &mut end[..end_len])
        } else {
            (&mut start[..self.len], &mut end[..0])
        }
    }

    /// Produce an iterator that yields a reference to each element in the buffer.
    ///
    /// This method uses the `slices` method internally.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let mut rb = ring_buffer::Bounded::<[i32; 3]>::array();
    ///     assert_eq!(rb.iter().count(), 0);
    ///     rb.push(1);
    ///     rb.push(2);
    ///     assert_eq!(rb.iter().cloned().collect::<Vec<_>>(), vec![1, 2]);
    /// }
    /// ```
    #[inline]
    pub fn iter(&self) -> Chain<slice::Iter<S::Element>, slice::Iter<S::Element>> {
        let (start, end) = self.slices();
        start.iter().chain(end.iter())
    }

    /// Produce an iterator that yields a mutable reference to each element in the buffer.
    ///
    /// This method uses the `slices_mut` method internally.
    #[inline]
    pub fn iter_mut(&mut self) -> Chain<slice::IterMut<S::Element>, slice::IterMut<S::Element>>
    where
        S: SliceMut,
    {
        let (start, end) = self.slices_mut();
        start.iter_mut().chain(end.iter_mut())
    }

    /// Borrows the item at the given index.
    ///
    /// Returns `None` if there is no element at the given index.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let mut rb = ring_buffer::Bounded::<[i32; 4]>::array();
    ///     assert_eq!(rb.get(1), None);
    ///     rb.push(0);
    ///     rb.push(1);
    ///     assert_eq!(rb.get(1), Some(&1));
    /// }
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> Option<&S::Element> {
        if index >= self.len {
            return None;
        }
        let wrapped_index = index % self.max_len();
        unsafe { Some(self.data.slice().get_unchecked(wrapped_index) as &_) }
    }

    /// Mutably borrows the item at the given index.
    ///
    /// Returns `None` if there is no element at the given index.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut S::Element>
    where
        S: SliceMut,
    {
        if index >= self.len {
            return None;
        }
        let wrapped_index = index % self.max_len();
        unsafe {
            Some(self.data.slice_mut().get_unchecked_mut(wrapped_index) as
                &mut _)
        }
    }

    /// Pushes the given element to the back of the buffer.
    ///
    /// If the buffer length is currently the max length, this replaces the element at the front of
    /// the buffer and returns it.
    ///
    /// If the buffer length is less than the max length, this pushes the element to the back of
    /// the buffer and increases the length of the buffer by `1`. `None` is returned.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// fn main() {
    ///     let mut ring_buffer = sample::ring_buffer::Bounded::<[i32; 3]>::array();
    ///     assert_eq!(ring_buffer.push(1), None);
    ///     assert_eq!(ring_buffer.push(2), None);
    ///     assert_eq!(ring_buffer.len(), 2);
    ///     assert_eq!(ring_buffer.push(3), None);
    ///     assert_eq!(ring_buffer.len(), 3);
    ///     assert_eq!(ring_buffer.push(4), Some(1));
    ///     assert_eq!(ring_buffer.len(), 3);
    /// }
    /// ```
    pub fn push(&mut self, elem: S::Element) -> Option<S::Element>
    where
        S: SliceMut,
    {
        // If the length is equal to the max, the buffer is full and we overwrite the start.
        if self.len == self.max_len() {
            let mut next_start = self.start + 1;

            // Wrap the start around the max length.
            if next_start >= self.max_len() {
                next_start = 0;
            }

            // Replace the element currently at the end.
            let old_elem =
                unsafe { mem::replace(self.data.slice_mut().get_unchecked_mut(self.start), elem) };

            self.start = next_start;
            return Some(old_elem);
        }

        // Otherwise the buffer is not full and has a free index to write to.
        let index = (self.start + self.len) % self.max_len();
        unsafe {
            ptr::write(self.data.slice_mut().get_unchecked_mut(index), elem);
        }
        self.len += 1;
        None
    }

    /// Pop an element from the front of the ring buffer.
    ///
    /// If the buffer is empty, this returns `None`.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let mut rb = ring_buffer::Bounded::from_full([0, 1, 2]);
    ///     assert_eq!(rb.len(), rb.max_len());
    ///     assert_eq!(rb.pop(), Some(0));
    ///     assert_eq!(rb.pop(), Some(1));
    ///     assert_eq!(rb.push(3), None);
    ///     assert_eq!(rb.pop(), Some(2));
    ///     assert_eq!(rb.pop(), Some(3));
    ///     assert_eq!(rb.pop(), None);
    /// }
    /// ```
    pub fn pop(&mut self) -> Option<S::Element>
    where
        S: SliceMut,
    {
        if self.len == 0 {
            return None;
        }

        let mut next_start = self.start + 1;

        // Wrap the start around the max length.
        if next_start >= self.max_len() {
            next_start = 0;
        }

        let old_elem = unsafe { ptr::read(self.data.slice_mut().get_unchecked_mut(self.start)) };

        self.start = next_start;
        self.len -= 1;
        Some(old_elem)
    }

    /// Produce an iterator that drains the ring buffer by `pop`ping each element one at a time.
    ///
    /// Note that only elements yielded by `DrainBounded::next` will be popped from the ring buffer.
    /// That is, all non-yielded elements will remain in the ring buffer.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::ring_buffer;
    ///
    /// fn main() {
    ///     let mut rb = ring_buffer::Bounded::from_full([0, 1, 2, 3]);
    ///     assert_eq!(rb.drain().take(2).collect::<Vec<_>>(), vec![0, 1]);
    ///     assert_eq!(rb.pop(), Some(2));
    ///     assert_eq!(rb.pop(), Some(3));
    ///     assert_eq!(rb.pop(), None);
    /// }
    /// ```
    pub fn drain(&mut self) -> DrainBounded<S> {
        DrainBounded {
            bounded: self,
        }
    }

    /// Creates a `Bounded` ring buffer from its start index, length and data slice.
    ///
    /// The maximum length of the `Bounded` ring buffer is assumed to the length of the given slice.
    ///
    /// **Note:** Existing elements within the given `data`'s `slice` will not be dropped when
    /// overwritten by calls to push. Thus, it is safe for the slice to contain uninitialized
    /// elements when using this method.
    ///
    /// **Note:** This method should only be necessary if you require specifying the `start` and
    /// initial `len`. Please see the `Bounded::array` and `Bounded::boxed_slice` functions for
    /// simpler constructor options that do not require manually passing indices.
    ///
    /// **Panic!**s if the following conditions are not met:
    ///
    /// - `start` < `data.slice().len()`
    /// - `len` <= `data.slice().len()`
    #[inline]
    pub fn from_raw_parts(start: usize, len: usize, data: S) -> Self {
        assert!(start < data.slice().len());
        assert!(len <= data.slice().len());
        Bounded { start, len, data }
    }

    /// Creates a `Bounded` ring buffer from its `start` index, `len` and data slice.
    ///
    /// This method is unsafe as there is no guarantee that either:
    ///
    /// - `start` < `data.slice().len()` or
    /// - `len` <= `data.slice().len()`.
    #[inline]
    pub unsafe fn from_raw_parts_unchecked(start: usize, len: usize, data: S) -> Self {
        Bounded { start, len, data }
    }

    /// Consumes the `Bounded` ring buffer and returns its parts:
    ///
    /// - The first `usize` is an index into the first element of the buffer.
    /// - The second `usize` is the length of the ring buffer.
    /// - `S` is the buffer data.
    ///
    /// This method is unsafe as the returned data may contain uninitialised memory in the case
    /// that the ring buffer is not full.
    #[inline]
    pub unsafe fn into_raw_parts(self) -> (usize, usize, S) {
        let Bounded { start, len, data } = self;
        (start, len, data)
    }
}

impl<S> From<S> for Bounded<S>
where
    S: Slice,
    S::Element: Copy,
{
    /// Construct a `Bounded` ring buffer from the given data slice.
    ///
    /// **Panic!**s if the given `data` buffer is empty.
    #[inline]
    fn from(data: S) -> Self {
        Self::from_raw_parts(0, 0, data)
    }
}

impl<S, T> FromIterator<T> for Bounded<S>
where
    S: Slice<Element = T> + FromIterator<T>,
    T: Copy,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let data = S::from_iter(iter);
        Self::from(data)
    }
}

impl<S> Index<usize> for Bounded<S>
where
    S: Slice,
    S::Element: Copy,
{
    type Output = S::Element;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of range")
    }
}

impl<S> IndexMut<usize> for Bounded<S>
where
    S: SliceMut,
    S::Element: Copy,
{
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of range")
    }
}

impl<'a, S> Iterator for DrainBounded<'a, S>
where
    S: SliceMut,
    S::Element: Copy,
{
    type Item = S::Element;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.bounded.pop()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.bounded.len(), Some(self.bounded.len()))
    }
}

impl<'a, S> ExactSizeIterator for DrainBounded<'a, S>
where
    S: SliceMut,
    S::Element: Copy,
{
    fn len(&self) -> usize {
        self.bounded.len()
    }
}
