#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{
    alloc::{Layout, alloc, dealloc},
    vec::Vec,
};

#[cfg(feature = "std")]
use std::{
    alloc::{Layout, alloc, dealloc},
    vec::Vec,
};

use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use core::{fmt, ptr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A fixed-capacity, heap-allocated double-ended queue backed by a circular buffer.
///
/// `ArrayDeque<T>` allocates a buffer on the heap with the given capacity. All
/// insertions and removals at either end run in O(1) time. Once full, further
/// `push_back` calls overwrite the oldest front element, and `push_front` calls
/// overwrite the oldest back element (FIFO overwrite behavior).
///
/// # Examples
///
/// ```rust
/// use array_deque::ArrayDeque;
///
/// let mut dq = ArrayDeque::new(3);
/// dq.push_back("a");
/// dq.push_back("b");
/// dq.push_front("c");
/// assert_eq!(dq.len(), 3);
/// assert_eq!(dq[0], "c");
///
/// // Overflow: overwrites the back ("b")
/// dq.push_front("x");
/// assert_eq!(dq.pop_back(), Some("a"));
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
        let ptr = unsafe { alloc(layout) as *mut T };

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

    /// Returns a reference to the front element without removing it.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut dq = ArrayDeque::new(3);
    /// assert_eq!(dq.front(), None);
    /// dq.push_back(42);
    /// assert_eq!(dq.front(), Some(&42));
    /// ```
    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { &*self.ptr.add(self.idx) })
        }
    }

    /// Returns a reference to the back element without removing it.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut dq = ArrayDeque::new(3);
    /// dq.push_back(1);
    /// dq.push_back(2);
    /// assert_eq!(dq.back(), Some(&2));
    /// ```
    pub fn back(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let back_idx = if self.idx + self.len <= self.cap {
                self.idx + self.len - 1
            } else {
                (self.idx + self.len - 1) % self.cap
            };
            Some(unsafe { &*self.ptr.add(back_idx) })
        }
    }

    /// Returns an iterator over the elements of the deque (front to back).
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut dq = ArrayDeque::new(3);
    /// dq.push_back(1);
    /// dq.push_back(2);
    /// let v: Vec<_> = dq.iter().cloned().collect();
    /// assert_eq!(v, vec![1,2]);
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
    /// let dq: ArrayDeque<i32> = ArrayDeque::new(5);
    /// assert_eq!(dq.capacity(), 5);
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
    /// let mut dq = ArrayDeque::new(3);
    /// assert_eq!(dq.len(), 0);
    /// dq.push_back(1);
    /// assert_eq!(dq.len(), 1);
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
    /// let mut dq = ArrayDeque::new(3);
    /// assert!(dq.is_empty());
    /// dq.push_back(1);
    /// assert!(!dq.is_empty());
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
    /// let mut dq = ArrayDeque::new(2);
    /// dq.push_back(1);
    /// dq.push_back(2);
    /// assert!(dq.is_full());
    /// ```
    pub fn is_full(&self) -> bool {
        self.len == self.cap
    }

    /// Removes all elements from the deque, properly dropping them,
    /// and resets it to an empty state.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut dq = ArrayDeque::new(3);
    /// dq.push_back(1);
    /// dq.clear();
    /// assert!(dq.is_empty());
    /// assert_eq!(dq.len(), 0);
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
    /// Drops all elements and deallocates the heap buffer.
    fn drop(&mut self) {
        self.clear();
        let layout = Layout::array::<T>(self.cap).expect("Invalid layout");
        unsafe {
            dealloc(self.ptr.cast(), layout);
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for ArrayDeque<T> {
    /// Formats the deque as a debug list (front to back).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Clone> Clone for ArrayDeque<T> {
    /// Creates a deep copy of the deque with identical capacity and contents.
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
    /// Two deques are equal if they have the same length
    /// and each element compares equal in order.
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}

impl<T: Eq> Eq for ArrayDeque<T> {}

impl<T> Index<usize> for ArrayDeque<T> {
    type Output = T;

    /// Indexed access into the deque (0 is front).
    ///
    /// # Panics
    ///
    /// Panics if `index >= len()`.
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "Index out of bounds");
        let actual_idx = self.idx + index;
        let actual_idx = if actual_idx >= self.cap {
            actual_idx - self.cap
        } else {
            actual_idx
        };
        unsafe { &*self.ptr.add(actual_idx) }
    }
}

impl<T> IndexMut<usize> for ArrayDeque<T> {
    /// Mutable indexed access into the deque (0 is front).
    ///
    /// # Panics
    ///
    /// Panics if `index >= len()`.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len);
        let idx = (self.idx + index) % self.cap;
        unsafe { &mut *self.ptr.add(idx) }
    }
}

impl<T> Extend<T> for ArrayDeque<T> {
    /// Extends the deque by pushing each item of the iterator to the back.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let mut dq = ArrayDeque::new(5);
    /// dq.extend([1,2,3]);
    /// assert_eq!(dq.len(), 3);
    /// ```
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item);
        }
    }
}

impl<T> FromIterator<T> for ArrayDeque<T> {
    /// Creates a deque from an iterator by collecting all items.
    /// Capacity == number of items (min 1).
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::ArrayDeque;
    ///
    /// let dq: ArrayDeque<_> = (0..3).collect();
    /// assert_eq!(dq.len(), 3);
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<T> = iter.into_iter().collect();
        let mut deque = ArrayDeque::new(vec.len().max(1));
        deque.extend(vec);
        deque
    }
}

impl<T: Clone> From<&[T]> for ArrayDeque<T> {
    /// Clones all elements from a slice into a new deque.
    fn from(slice: &[T]) -> Self {
        let mut deque = ArrayDeque::new(slice.len());
        for item in slice {
            deque.push_back(item.clone());
        }
        deque
    }
}

impl<T, const N: usize> From<[T; N]> for ArrayDeque<T> {
    /// Takes ownership of each element in the array.
    fn from(array: [T; N]) -> Self {
        let mut deque = ArrayDeque::new(N.max(1));
        for item in array {
            deque.push_back(item);
        }
        deque
    }
}

impl<T> From<Vec<T>> for ArrayDeque<T> {
    /// Takes ownership of each element in the vector.
    fn from(vec: Vec<T>) -> Self {
        let mut deque = ArrayDeque::new(vec.len().max(1));
        for item in vec {
            deque.push_back(item);
        }
        deque
    }
}

impl<T: Clone> From<&Vec<T>> for ArrayDeque<T> {
    /// Clones each element from the vector.
    fn from(vec: &Vec<T>) -> Self {
        let mut deque = ArrayDeque::new(vec.len().max(1));
        for item in vec {
            deque.push_back(item.clone());
        }
        deque
    }
}

impl<T: Clone, const N: usize> From<&[T; N]> for ArrayDeque<T> {
    /// Clones each element from the array reference.
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
    /// Serializes the deque as a sequence (front to back).
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
    /// Deserializes a sequence into a deque (capacity == item count).
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
    /// Consumes the deque and returns an iterator over its elements.
    fn into_iter(self) -> Self::IntoIter {
        ArrayDequeIntoIter {
            deque: self,
            pos: 0,
        }
    }
}

/// An owning iterator that moves elements out of an `ArrayDeque`.
///
/// Returned by `into_iter()`.
pub struct ArrayDequeIntoIter<T> {
    deque: ArrayDeque<T>,
    pos: usize,
}

impl<T> Iterator for ArrayDequeIntoIter<T> {
    type Item = T;

    /// Advances and returns the next element.
    fn next(&mut self) -> Option<T> {
        if self.pos >= self.deque.len {
            return None;
        }
        let idx = (self.deque.idx + self.pos) % self.deque.cap;
        self.pos += 1;
        Some(unsafe { ptr::read(self.deque.ptr.add(idx)) })
    }
}

impl<'a, T> IntoIterator for &'a ArrayDeque<T> {
    type Item = &'a T;
    type IntoIter = ArrayDequeIter<'a, T>;
    /// Borrows the deque and returns an iterator over `&T`.
    fn into_iter(self) -> Self::IntoIter {
        ArrayDequeIter {
            deque: self,
            pos: 0,
        }
    }
}

/// A borrowed iterator over `&T` from an `ArrayDeque`.
///
/// Returned by `iter()` and `&deque.into_iter()`.
pub struct ArrayDequeIter<'a, T> {
    deque: &'a ArrayDeque<T>,
    pos: usize,
}

impl<'a, T> Iterator for ArrayDequeIter<'a, T> {
    type Item = &'a T;

    /// Advances and returns the next reference.
    fn next(&mut self) -> Option<&'a T> {
        if self.pos >= self.deque.len {
            return None;
        }
        let idx = (self.deque.idx + self.pos) % self.deque.cap;
        self.pos += 1;
        unsafe { Some(&*self.deque.ptr.add(idx)) }
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
