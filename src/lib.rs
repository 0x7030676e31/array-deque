//! A fixed-capacity circular buffer (ring buffer) implementation.
//!
//! This crate provides [`ArrayDeque`], a double-ended queue with a fixed capacity
//! that uses a circular buffer for efficient operations at both ends. Unlike
//! [`std::collections::VecDeque`], this implementation has a compile-time fixed
//! capacity and will overwrite old elements when full.
//!
//! # Examples
//!
//! ```
//! use array_deque::ArrayDeque;
//!
//! let mut deque = ArrayDeque::new(3);
//! deque.push_back(1);
//! deque.push_back(2);
//! deque.push_back(3);
//!
//! assert_eq!(deque.pop_front(), Some(1));
//! assert_eq!(deque.pop_back(), Some(3));
//! ```
//!
//! # Features
//!
//! - **serde**: Enable serialization and deserialization support with serde.

use std::alloc::{self, Layout};

use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use core::{fmt, ptr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A fixed-capacity double-ended queue implemented as a circular buffer.
///
/// `ArrayDeque` provides efficient insertion and removal at both ends with O(1)
/// time complexity. When the deque reaches its capacity, new elements will
/// overwrite the oldest elements (FIFO behavior for overwrites).
///
/// # Examples
///
/// ```
/// use array_deque::ArrayDeque;
///
/// let mut deque = ArrayDeque::new(3);
/// deque.push_back(1);
/// deque.push_back(2);
/// deque.push_front(0);
///
/// assert_eq!(deque.len(), 3);
/// assert_eq!(deque[0], 0);
/// assert_eq!(deque[1], 1);
/// assert_eq!(deque[2], 2);
/// ```
///
/// # Capacity and Overflow Behavior
///
/// When the deque is full and a new element is added:
/// - `push_back` will overwrite the front element
/// - `push_front` will overwrite the back element
///
/// ```
/// use array_deque::ArrayDeque;
///
/// let mut deque = ArrayDeque::new(2);
/// deque.push_back(1);
/// deque.push_back(2);
/// deque.push_back(3); // Overwrites 1
///
/// assert_eq!(deque.pop_front(), Some(2));
/// assert_eq!(deque.pop_front(), Some(3));
/// ```
pub struct ArrayDeque<T> {
    /// Raw pointer to the allocated memory
    ptr: *mut T,
    /// Maximum capacity of the deque
    cap: usize,
    /// Current number of elements
    len: usize,
    /// Index of the front element
    idx: usize,
    /// Marker for the generic type
    _marker: PhantomData<T>,
}

impl<T> ArrayDeque<T> {
    /// Creates a new `ArrayDeque` with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `cap` - The fixed capacity of the deque. Must be greater than zero.
    ///
    /// # Panics
    ///
    /// Panics if `cap` is zero or if memory allocation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let deque: ArrayDeque<i32> = ArrayDeque::new(10);
    /// assert_eq!(deque.capacity(), 10);
    /// assert!(deque.is_empty());
    /// ```
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0, "Capacity must be greater than zero");
        let layout = Layout::array::<T>(cap).expect("Invalid layout");
        let ptr = unsafe { alloc::alloc(layout) as *mut T };
        if ptr.is_null() {
            panic!("Failed to allocate memory");
        }
        Self {
            ptr,
            cap,
            len: 0,
            idx: 0,
            _marker: PhantomData,
        }
    }

    /// Appends an element to the back of the deque.
    ///
    /// If the deque is at capacity, this will overwrite the front element
    /// and advance the front pointer.
    ///
    /// # Arguments
    ///
    /// * `value` - The element to append
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// deque.push_back(1);
    /// deque.push_back(2);
    /// assert_eq!(deque.len(), 2);
    /// ```
    pub fn push_back(&mut self, value: T) {
        let write_idx = (self.idx + self.len) % self.cap;
        unsafe {
            ptr::write(self.ptr.add(write_idx), value);
        }
        if self.len == self.cap {
            self.idx = (self.idx + 1) % self.cap;
        } else {
            self.len += 1;
        }
    }

    /// Prepends an element to the front of the deque.
    ///
    /// If the deque is at capacity, this will overwrite the back element.
    ///
    /// # Arguments
    ///
    /// * `value` - The element to prepend
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// deque.push_front(1);
    /// deque.push_front(2);
    /// assert_eq!(deque[0], 2);
    /// assert_eq!(deque[1], 1);
    /// ```
    pub fn push_front(&mut self, value: T) {
        self.idx = (self.idx + self.cap - 1) % self.cap;
        if self.len == self.cap {
            let drop_idx = (self.idx + self.len) % self.cap;
            unsafe {
                ptr::drop_in_place(self.ptr.add(drop_idx));
            }
        } else {
            self.len += 1;
        }
        unsafe {
            ptr::write(self.ptr.add(self.idx), value);
        }
    }

    /// Removes and returns the last element from the deque.
    ///
    /// # Returns
    ///
    /// `Some(T)` if the deque is not empty, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// deque.push_back(1);
    /// deque.push_back(2);
    /// assert_eq!(deque.pop_back(), Some(2));
    /// assert_eq!(deque.pop_back(), Some(1));
    /// assert_eq!(deque.pop_back(), None);
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let tail_idx = (self.idx + self.len - 1) % self.cap;
        self.len -= 1;
        Some(unsafe { ptr::read(self.ptr.add(tail_idx)) })
    }

    /// Removes and returns the first element from the deque.
    ///
    /// # Returns
    ///
    /// `Some(T)` if the deque is not empty, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// deque.push_back(1);
    /// deque.push_back(2);
    /// assert_eq!(deque.pop_front(), Some(1));
    /// assert_eq!(deque.pop_front(), Some(2));
    /// assert_eq!(deque.pop_front(), None);
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let front_idx = self.idx;
        self.idx = (self.idx + 1) % self.cap;
        self.len -= 1;
        Some(unsafe { ptr::read(self.ptr.add(front_idx)) })
    }

    /// Returns an iterator over the elements of the deque.
    ///
    /// The iterator yields elements from front to back.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// deque.push_back(1);
    /// deque.push_back(2);
    /// deque.push_back(3);
    ///
    /// let mut iter = deque.iter();
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), Some(&3));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len).map(move |i| {
            let idx = (self.idx + i) % self.cap;
            unsafe { &*self.ptr.add(idx) }
        })
    }

    /// Returns the maximum capacity of the deque.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let deque: ArrayDeque<i32> = ArrayDeque::new(10);
    /// assert_eq!(deque.capacity(), 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Returns the number of elements currently in the deque.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// assert_eq!(deque.len(), 0);
    /// deque.push_back(1);
    /// assert_eq!(deque.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the deque contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// assert!(deque.is_empty());
    /// deque.push_back(1);
    /// assert!(!deque.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if the deque has reached its maximum capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(2);
    /// assert!(!deque.is_full());
    /// deque.push_back(1);
    /// deque.push_back(2);
    /// assert!(deque.is_full());
    /// ```
    pub fn is_full(&self) -> bool {
        self.len == self.cap
    }

    /// Removes all elements from the deque.
    ///
    /// This operation properly drops all contained elements and resets
    /// the deque to an empty state.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// deque.push_back(1);
    /// deque.push_back(2);
    /// deque.clear();
    /// assert!(deque.is_empty());
    /// assert_eq!(deque.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        for i in 0..self.len {
            let idx = (self.idx + i) % self.cap;
            unsafe {
                ptr::drop_in_place(self.ptr.add(idx));
            }
        }
        self.len = 0;
        self.idx = 0;
    }
}

impl<T> Drop for ArrayDeque<T> {
    /// Properly deallocates the deque's memory and drops all contained elements.
    fn drop(&mut self) {
        self.clear();
        let layout = Layout::array::<T>(self.cap).expect("Invalid layout");
        unsafe {
            alloc::dealloc(self.ptr.cast(), layout);
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for ArrayDeque<T> {
    /// Formats the deque as a debug list showing all elements from front to back.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Clone> Clone for ArrayDeque<T> {
    /// Creates a deep copy of the deque with the same capacity and elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// deque.push_back(1);
    /// deque.push_back(2);
    /// let cloned = deque.clone();
    /// assert_eq!(deque.len(), cloned.len());
    /// assert_eq!(deque[0], cloned[0]);
    /// ```
    fn clone(&self) -> Self {
        let mut new = ArrayDeque::new(self.cap);
        for item in self.iter() {
            new.push_back(item.clone());
        }
        new
    }
}

impl<T: PartialEq> PartialEq for ArrayDeque<T> {
    /// Compares two deques for equality based on their elements and order.
    ///
    /// Two deques are equal if they have the same length and all elements
    /// compare equal in the same order.
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}

impl<T: Eq> Eq for ArrayDeque<T> {}

impl<T> Index<usize> for ArrayDeque<T> {
    type Output = T;

    /// Provides indexed access to elements in the deque.
    ///
    /// Index 0 corresponds to the front element, index 1 to the second element, etc.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds (>= len()).
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// deque.push_back(10);
    /// deque.push_back(20);
    /// assert_eq!(deque[0], 10);
    /// assert_eq!(deque[1], 20);
    /// ```
    fn index(&self, i: usize) -> &Self::Output {
        assert!(i < self.len);
        let idx = (self.idx + i) % self.cap;
        unsafe { &*self.ptr.add(idx) }
    }
}

impl<T> IndexMut<usize> for ArrayDeque<T> {
    /// Provides mutable indexed access to elements in the deque.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds (>= len()).
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// deque.push_back(10);
    /// deque[0] = 42;
    /// assert_eq!(deque[0], 42);
    /// ```
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        assert!(i < self.len);
        let idx = (self.idx + i) % self.cap;
        unsafe { &mut *self.ptr.add(idx) }
    }
}

impl<T> Extend<T> for ArrayDeque<T> {
    /// Extends the deque with the contents of an iterator.
    ///
    /// Elements are added to the back of the deque. If the deque becomes full
    /// during extension, older elements will be overwritten.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(5);
    /// deque.extend(vec![1, 2, 3]);
    /// assert_eq!(deque.len(), 3);
    /// ```
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item);
        }
    }
}

impl<T> FromIterator<T> for ArrayDeque<T> {
    /// Creates a deque from an iterator.
    ///
    /// The capacity is set to the number of elements in the iterator
    /// (with a minimum of 1).
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let deque: ArrayDeque<i32> = (1..4).collect();
    /// assert_eq!(deque.len(), 3);
    /// assert_eq!(deque[0], 1);
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<T> = iter.into_iter().collect();
        let mut deque = ArrayDeque::new(vec.len().max(1));
        deque.extend(vec);
        deque
    }
}

impl<T> From<&[T]> for ArrayDeque<T>
where
    T: Clone,
{
    /// Creates a deque from a slice by cloning all elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let slice = &[1, 2, 3];
    /// let deque: ArrayDeque<i32> = ArrayDeque::from(slice);
    /// assert_eq!(deque.len(), 3);
    /// ```
    fn from(slice: &[T]) -> Self {
        let mut deque = ArrayDeque::new(slice.len());
        for item in slice {
            deque.push_back(item.clone());
        }
        deque
    }
}

impl<T, const N: usize> From<[T; N]> for ArrayDeque<T> {
    /// Creates a deque from an array by taking ownership of all elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let array = [1, 2, 3];
    /// let deque: ArrayDeque<i32> = ArrayDeque::from(array);
    /// assert_eq!(deque.len(), 3);
    /// ```
    fn from(array: [T; N]) -> Self {
        let mut deque = ArrayDeque::new(N.max(1));
        for item in array {
            deque.push_back(item);
        }
        deque
    }
}

impl<T> From<Vec<T>> for ArrayDeque<T> {
    /// Creates a deque from a vector by taking ownership of all elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let vec = vec![1, 2, 3];
    /// let deque: ArrayDeque<i32> = ArrayDeque::from(vec);
    /// assert_eq!(deque.len(), 3);
    /// ```
    fn from(vec: Vec<T>) -> Self {
        let mut deque = ArrayDeque::new(vec.len().max(1));
        for item in vec {
            deque.push_back(item);
        }
        deque
    }
}

impl<T: Clone> From<&Vec<T>> for ArrayDeque<T> {
    /// Creates a deque from a vector reference by cloning all elements.
    fn from(vec: &Vec<T>) -> Self {
        let mut deque = ArrayDeque::new(vec.len().max(1));
        for item in vec {
            deque.push_back(item.clone());
        }
        deque
    }
}

impl<T: Clone, const N: usize> From<&[T; N]> for ArrayDeque<T> {
    /// Creates a deque from an array reference by cloning all elements.
    fn from(array: &[T; N]) -> Self {
        let mut deque = ArrayDeque::new(N.max(1));
        for item in array {
            deque.push_back(item.clone());
        }
        deque
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for ArrayDeque<T> {
    /// Serializes the deque as a sequence of its elements.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.len))?;
        for item in self.iter() {
            seq.serialize_element(item)?;
        }
        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for ArrayDeque<T> {
    /// Deserializes a sequence into a deque.
    ///
    /// The capacity is set to the number of elements in the sequence.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<T> = Vec::deserialize(deserializer)?;
        Ok(ArrayDeque::from(vec))
    }
}

impl<T> IntoIterator for ArrayDeque<T> {
    type Item = T;
    type IntoIter = ArrayDequeIntoIter<T>;

    /// Creates a consuming iterator that moves all elements out of the deque.
    ///
    /// The deque cannot be used after calling this method.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// deque.push_back(1);
    /// deque.push_back(2);
    ///
    /// let vec: Vec<i32> = deque.into_iter().collect();
    /// assert_eq!(vec, vec![1, 2]);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        ArrayDequeIntoIter {
            deque: self,
            pos: 0,
        }
    }
}

/// An iterator that moves elements out of an `ArrayDeque`.
///
/// This struct is created by the `into_iter` method on `ArrayDeque`.
pub struct ArrayDequeIntoIter<T> {
    deque: ArrayDeque<T>,
    pos: usize,
}

impl<T> Iterator for ArrayDequeIntoIter<T> {
    type Item = T;

    /// Advances the iterator and returns the next element.
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.deque.len {
            return None;
        }
        let idx = (self.deque.idx + self.pos) % self.deque.cap;
        self.pos += 1;
        Some(unsafe { ptr::read(self.deque.ptr.add(idx)) })
    }

    /// Returns the bounds on the remaining length of the iterator.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.deque.len - self.pos;
        (remaining, Some(remaining))
    }
}

impl<'a, T> IntoIterator for &'a ArrayDeque<T> {
    type Item = &'a T;
    type IntoIter = ArrayDequeIter<'a, T>;

    /// Creates an iterator over references to the deque's elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut deque = ArrayDeque::new(3);
    /// deque.push_back(1);
    /// deque.push_back(2);
    ///
    /// for item in &deque {
    ///     println!("{}", item);
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        ArrayDequeIter {
            deque: self,
            pos: 0,
        }
    }
}

/// An iterator over references to the elements of an `ArrayDeque`.
///
/// This struct is created by the `iter` method on `ArrayDeque` or by
/// using `&deque` in a for loop.
pub struct ArrayDequeIter<'a, T> {
    deque: &'a ArrayDeque<T>,
    pos: usize,
}

impl<'a, T> Iterator for ArrayDequeIter<'a, T> {
    type Item = &'a T;

    /// Advances the iterator and returns the next element reference.
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.deque.len {
            return None;
        }
        let idx = (self.deque.idx + self.pos) % self.deque.cap;
        self.pos += 1;
        unsafe { Some(&*self.deque.ptr.add(idx)) }
    }

    /// Returns the bounds on the remaining length of the iterator.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.deque.len - self.pos;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for ArrayDequeIter<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut deque = ArrayDeque::new(3);
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);
        assert_eq!(deque.pop_front(), Some(1));
        assert_eq!(deque.pop_back(), Some(3));
        assert_eq!(deque.pop_front(), Some(2));
        assert_eq!(deque.pop_front(), None);
    }

    #[test]
    fn push_front_back() {
        let mut deque = ArrayDeque::new(3);
        deque.push_front(1);
        deque.push_front(2);
        deque.push_back(3);
        assert_eq!(deque.pop_front(), Some(2));
        assert_eq!(deque.pop_back(), Some(3));
        assert_eq!(deque.pop_front(), Some(1));
        assert_eq!(deque.pop_front(), None);
    }

    #[test]
    fn iter() {
        let mut deque = ArrayDeque::new(5);
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);
        let mut iter = deque.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn clear() {
        let mut deque = ArrayDeque::new(3);
        deque.push_back(1);
        deque.push_back(2);
        deque.clear();
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn capacity() {
        let deque = ArrayDeque::<i32>::new(5);
        assert_eq!(deque.capacity(), 5);
        assert!(deque.is_empty());
    }

    #[test]
    fn clone() {
        let mut deque = ArrayDeque::new(3);
        deque.push_back(1);
        deque.push_back(2);
        let cloned_deque = deque.clone();
        assert_eq!(cloned_deque.len(), 2);
        assert_eq!(cloned_deque[0], 1);
        assert_eq!(cloned_deque[1], 2);
    }

    #[test]
    fn from_iter() {
        let vec = vec![1, 2, 3];
        let deque: ArrayDeque<_> = vec.into_iter().collect();
        assert_eq!(deque.len(), 3);
        assert_eq!(deque[0], 1);
        assert_eq!(deque[1], 2);
        assert_eq!(deque[2], 3);
    }

    #[test]
    fn from_slice() {
        let slice = [1, 2, 3];
        let deque: ArrayDeque<_> = ArrayDeque::from(&slice[..]);
        assert_eq!(deque.len(), 3);
        assert_eq!(deque[0], 1);
        assert_eq!(deque[1], 2);
        assert_eq!(deque[2], 3);
    }

    #[test]
    fn from_array() {
        let array = [1, 2, 3];
        let deque: ArrayDeque<_> = ArrayDeque::from(array);
        assert_eq!(deque.len(), 3);
        assert_eq!(deque[0], 1);
        assert_eq!(deque[1], 2);
        assert_eq!(deque[2], 3);
    }

    #[test]
    fn index() {
        let mut deque = ArrayDeque::new(5);
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);
        assert_eq!(deque[0], 1);
        assert_eq!(deque[1], 2);
        assert_eq!(deque[2], 3);
        assert!(
            std::panic::catch_unwind(|| {
                let _ = deque[3];
            })
            .is_err()
        );
    }

    #[test]
    fn index_mut() {
        let mut deque = ArrayDeque::new(5);
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);
        deque[0] = 10;
        assert_eq!(deque[0], 10);
        assert_eq!(deque[1], 2);
        assert_eq!(deque[2], 3);
        assert!(
            std::panic::catch_unwind(|| {
                let _ = deque[3];
            })
            .is_err()
        );
    }

    #[test]
    fn extend() {
        let mut deque = ArrayDeque::new(5);
        deque.extend(vec![1, 2, 3]);
        assert_eq!(deque.len(), 3);
        assert_eq!(deque[0], 1);
        assert_eq!(deque[1], 2);
        assert_eq!(deque[2], 3);
    }

    #[test]
    fn into_iter() {
        let mut deque = ArrayDeque::new(5);
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);
        let mut iter = deque.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn into_iter_empty() {
        let deque: ArrayDeque<i32> = ArrayDeque::new(5);
        let mut iter = deque.into_iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn into_iter_full() {
        let mut deque = ArrayDeque::new(3);
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);
        let mut iter = deque.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn into_iter_partial() {
        let mut deque = ArrayDeque::new(5);
        deque.push_back(1);
        deque.push_back(2);
        let mut iter = deque.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_empty() {
        let deque: ArrayDeque<i32> = ArrayDeque::new(5);
        let mut iter = deque.iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_full() {
        let mut deque = ArrayDeque::new(3);
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);
        let mut iter = deque.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_partial() {
        let mut deque = ArrayDeque::new(5);
        deque.push_back(1);
        deque.push_back(2);
        let mut iter = deque.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn is_empty() {
        let deque: ArrayDeque<i32> = ArrayDeque::new(5);
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn is_full() {
        let mut deque = ArrayDeque::new(3);
        assert!(!deque.is_full());
        deque.push_back(1);
        deque.push_back(2);
        assert!(!deque.is_full());
        deque.push_back(3);
        assert!(deque.is_full());
    }

    #[test]
    fn clear_empty() {
        let mut deque = ArrayDeque::<()>::new(3);
        deque.clear();
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn clear_non_empty() {
        let mut deque = ArrayDeque::new(3);
        deque.push_back(1);
        deque.push_back(2);
        deque.clear();
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_serialize() {
        let mut deque = ArrayDeque::new(3);
        deque.push_back(1);
        deque.push_back(2);
        let serialized = serde_json::to_string(&deque).unwrap();
        assert_eq!(serialized, "[1,2]");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_deserialize() {
        let serialized = "[1,2,3]";
        let deque: ArrayDeque<i32> = serde_json::from_str(serialized).unwrap();
        assert_eq!(deque.len(), 3);
        assert_eq!(deque[0], 1);
        assert_eq!(deque[1], 2);
        assert_eq!(deque[2], 3);
    }
}
