use core::fmt;
use core::mem::MaybeUninit;
use core::ops::{Index, IndexMut};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A fixed-capacity, stack-allocated double-ended queue backed by a circular buffer.
///
/// `StackArrayDeque<T, N>` stores up to `N` elements inline on the stack with no heap
/// allocation. All insertions and removals at either end run in O(1) time. Once full,
/// further `push_back` calls will overwrite the oldest front element, and
/// `push_front` calls will overwrite the oldest back element (FIFO overwrite behavior).
///
/// # Examples
///
/// ```rust
/// use array_deque::StackArrayDeque;
///
/// let mut dq: StackArrayDeque<i32, 3> = StackArrayDeque::new();
/// dq.push_back(10);
/// dq.push_back(20);
/// dq.push_front(5);
/// assert_eq!(dq.len(), 3);
/// assert_eq!(dq[0], 5);
/// assert_eq!(dq[2], 20);
///
/// // Overflow: overwrites the front (5)
/// dq.push_back(30);
/// assert_eq!(dq.pop_front(), Some(20));
/// ```
pub struct StackArrayDeque<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
    idx: usize,
}

impl<T, const N: usize> StackArrayDeque<T, N> {
    /// Creates a new empty `StackArrayDeque`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::StackArrayDeque;
    ///
    /// let deque: StackArrayDeque<i32, 10> = StackArrayDeque::new();
    /// assert_eq!(deque.capacity(), 10);
    /// assert!(deque.is_empty());
    /// ```
    pub const fn new() -> Self {
        assert!(N > 0, "StackArrayDeque capacity must be greater than 0");
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
            idx: 0,
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
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
    /// deque.push_back(1);
    /// deque.push_back(2);
    /// assert_eq!(deque.len(), 2);
    /// ```
    pub fn push_back(&mut self, value: T) {
        let write_idx = (self.idx + self.len) % N;
        self.data[write_idx].write(value);

        if self.len == N {
            self.idx = (self.idx + 1) % N;
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
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
    /// deque.push_front(1);
    /// deque.push_front(2);
    /// assert_eq!(deque[0], 2);
    /// assert_eq!(deque[1], 1);
    /// ```
    pub fn push_front(&mut self, value: T) {
        self.idx = (self.idx + N - 1) % N;

        if self.len == N {
            let drop_idx = (self.idx + self.len) % N;
            unsafe {
                self.data[drop_idx].assume_init_drop();
            }
        } else {
            self.len += 1;
        }

        self.data[self.idx].write(value);
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
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
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
        let tail_idx = (self.idx + self.len - 1) % N;
        self.len -= 1;
        Some(unsafe { self.data[tail_idx].assume_init_read() })
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
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
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
        self.idx = (self.idx + 1) % N;
        self.len -= 1;
        Some(unsafe { self.data[front_idx].assume_init_read() })
    }

    /// Returns a reference to the front element without removing it.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut dq: StackArrayDeque<i32, 3> = StackArrayDeque::new();
    /// assert_eq!(dq.front(), None);
    /// dq.push_back(42);
    /// assert_eq!(dq.front(), Some(&42));
    /// ```
    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.data[self.idx].assume_init_ref() })
        }
    }

    /// Returns a reference to the back element without removing it.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut dq: StackArrayDeque<i32, 3> = StackArrayDeque::new();
    /// dq.push_back(1);
    /// dq.push_back(2);
    /// assert_eq!(dq.back(), Some(&2));
    /// ```
    pub fn back(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let back_idx = (self.idx + self.len - 1) % N;
            Some(unsafe { self.data[back_idx].assume_init_ref() })
        }
    }

    /// Returns an iterator over the elements of the deque.
    ///
    /// The iterator yields elements from front to back.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
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
            let idx = (self.idx + i) % N;
            unsafe { self.data[idx].assume_init_ref() }
        })
    }

    /// Returns the maximum capacity of the deque.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::StackArrayDeque;
    ///
    /// let deque: StackArrayDeque<i32, 10> = StackArrayDeque::new();
    /// assert_eq!(deque.capacity(), 10);
    /// ```
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns the number of elements currently in the deque.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
    /// assert_eq!(deque.len(), 0);
    /// deque.push_back(1);
    /// assert_eq!(deque.len(), 1);
    /// ```
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the deque contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
    /// assert!(deque.is_empty());
    /// deque.push_back(1);
    /// assert!(!deque.is_empty());
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if the deque has reached its maximum capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut deque: StackArrayDeque<i32, 2> = StackArrayDeque::new();
    /// assert!(!deque.is_full());
    /// deque.push_back(1);
    /// deque.push_back(2);
    /// assert!(deque.is_full());
    /// ```
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    /// Removes all elements from the deque.
    ///
    /// This operation properly drops all contained elements and resets
    /// the deque to an empty state.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
    /// deque.push_back(1);
    /// deque.push_back(2);
    /// deque.clear();
    /// assert!(deque.is_empty());
    /// assert_eq!(deque.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        for i in 0..self.len {
            let idx = (self.idx + i) % N;
            unsafe {
                self.data[idx].assume_init_drop();
            }
        }
        self.len = 0;
        self.idx = 0;
    }
}

impl<T, const N: usize> Drop for StackArrayDeque<T, N> {
    /// Properly drops all contained elements.
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N: usize> Default for StackArrayDeque<T, N> {
    /// Creates an empty deque.
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for StackArrayDeque<T, N> {
    /// Formats the deque as a debug list showing all elements from front to back.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Clone, const N: usize> Clone for StackArrayDeque<T, N> {
    /// Creates a deep copy of the deque with the same capacity and elements.
    fn clone(&self) -> Self {
        let mut new = StackArrayDeque::new();
        for item in self.iter() {
            new.push_back(item.clone());
        }
        new
    }
}

impl<T: PartialEq, const N: usize> PartialEq for StackArrayDeque<T, N> {
    /// Compares two deques for equality based on their elements and order.
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}

impl<T: Eq, const N: usize> Eq for StackArrayDeque<T, N> {}

impl<T, const N: usize> Index<usize> for StackArrayDeque<T, N> {
    type Output = T;

    /// Provides indexed access to elements in the deque.
    ///
    /// Index 0 corresponds to the front element, index 1 to the second element, etc.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds (>= len()).
    fn index(&self, i: usize) -> &Self::Output {
        assert!(i < self.len);
        let idx = (self.idx + i) % N;
        unsafe { self.data[idx].assume_init_ref() }
    }
}

impl<T, const N: usize> IndexMut<usize> for StackArrayDeque<T, N> {
    /// Provides mutable indexed access to elements in the deque.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds (>= len()).
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        assert!(i < self.len);
        let idx = (self.idx + i) % N;
        unsafe { self.data[idx].assume_init_mut() }
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize, const N: usize> Serialize for StackArrayDeque<T, N> {
    /// Serializes the deque as a sequence of its elements from front to back.
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
impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for StackArrayDeque<T, N> {
    /// Deserializes a sequence into a `StackArrayDeque`, erroring if it exceeds capacity.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<T> = Vec::deserialize(deserializer)?;
        if vec.len() > N {
            return Err(serde::de::Error::custom(
                "Too many elements for StackArrayDeque capacity",
            ));
        }
        let mut deque = StackArrayDeque::new();
        for item in vec {
            deque.push_back(item);
        }
        Ok(deque)
    }
}

impl<T, const N: usize> Extend<T> for StackArrayDeque<T, N> {
    /// Extends the deque with items from an iterator, pushing to the back.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::StackArrayDeque;
    ///
    /// let mut dq: StackArrayDeque<i32, 5> = StackArrayDeque::new();
    /// dq.extend([1,2,3]);
    /// assert_eq!(dq.len(), 3);
    /// ```
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item);
        }
    }
}

impl<T, const N: usize> FromIterator<T> for StackArrayDeque<T, N> {
    /// Creates a deque by collecting an iterator into its back, up to capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_deque::StackArrayDeque;
    ///
    /// let dq: StackArrayDeque<_, 4> = (0..3).collect();
    /// assert_eq!(dq.len(), 3);
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut dq = StackArrayDeque::new();
        dq.extend(iter);
        dq
    }
}

/// An owning iterator that moves elements out of a `StackArrayDeque`.
///
/// This is returned by `into_iter()`.
pub struct StackArrayDequeIntoIter<T, const N: usize> {
    deque: StackArrayDeque<T, N>,
    pos: usize,
}

impl<T, const N: usize> Iterator for StackArrayDequeIntoIter<T, N> {
    type Item = T;
    /// Advances and returns the next element, front to back.
    fn next(&mut self) -> Option<T> {
        if self.pos >= self.deque.len {
            None
        } else {
            // read element at front+pos
            let idx = (self.deque.idx + self.pos) % N;
            self.pos += 1;
            // safety: we know these slots were initialized
            Some(unsafe { self.deque.data[idx].assume_init_read() })
        }
    }
}

impl<T, const N: usize> IntoIterator for StackArrayDeque<T, N> {
    type Item = T;
    type IntoIter = StackArrayDequeIntoIter<T, N>;
    /// Consumes the deque and returns an iterator over its elements.
    fn into_iter(self) -> Self::IntoIter {
        StackArrayDequeIntoIter {
            deque: self,
            pos: 0,
        }
    }
}

/// A borrowed iterator over `&T` from a `StackArrayDeque`.
///
/// This is returned by `iter()` and `&deque.into_iter()`.
pub struct StackArrayDequeIter<'a, T, const N: usize> {
    deque: &'a StackArrayDeque<T, N>,
    pos: usize,
}

impl<'a, T, const N: usize> Iterator for StackArrayDequeIter<'a, T, N> {
    type Item = &'a T;
    /// Advances and returns the next reference, front to back.
    fn next(&mut self) -> Option<&'a T> {
        if self.pos >= self.deque.len {
            None
        } else {
            let idx = (self.deque.idx + self.pos) % N;
            self.pos += 1;
            Some(unsafe { self.deque.data[idx].assume_init_ref() })
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a StackArrayDeque<T, N> {
    type Item = &'a T;
    type IntoIter = StackArrayDequeIter<'a, T, N>;
    /// Borrows the deque and returns an iterator over `&T`.
    fn into_iter(self) -> Self::IntoIter {
        StackArrayDequeIter {
            deque: self,
            pos: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
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
        let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
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
        let mut deque: StackArrayDeque<i32, 5> = StackArrayDeque::new();
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
        let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.clear();
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn capacity() {
        let deque: StackArrayDeque<i32, 5> = StackArrayDeque::new();
        assert_eq!(deque.capacity(), 5);
        assert!(deque.is_empty());
    }

    #[test]
    fn clone() {
        let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        let cloned_deque = deque.clone();
        assert_eq!(cloned_deque.len(), 2);
        assert_eq!(cloned_deque[0], 1);
        assert_eq!(cloned_deque[1], 2);
    }

    #[test]
    fn index() {
        let mut deque: StackArrayDeque<i32, 5> = StackArrayDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);
        assert_eq!(deque[0], 1);
        assert_eq!(deque[1], 2);
        assert_eq!(deque[2], 3);
        assert!(std::panic::catch_unwind(|| {
            let _ = deque[3];
        })
        .is_err());
    }

    #[test]
    fn index_mut() {
        let mut deque: StackArrayDeque<i32, 5> = StackArrayDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);
        deque[0] = 10;
        assert_eq!(deque[0], 10);
        assert_eq!(deque[1], 2);
        assert_eq!(deque[2], 3);
        assert!(std::panic::catch_unwind(|| {
            let _ = deque[3];
        })
        .is_err());
    }

    #[test]
    fn iter_empty() {
        let deque: StackArrayDeque<i32, 5> = StackArrayDeque::new();
        let mut iter = deque.iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_full() {
        let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
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
        let mut deque: StackArrayDeque<i32, 5> = StackArrayDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        let mut iter = deque.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn is_empty() {
        let deque: StackArrayDeque<i32, 5> = StackArrayDeque::new();
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn is_full() {
        let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
        assert!(!deque.is_full());
        deque.push_back(1);
        deque.push_back(2);
        assert!(!deque.is_full());
        deque.push_back(3);
        assert!(deque.is_full());
    }

    #[test]
    fn clear_empty() {
        let mut deque: StackArrayDeque<(), 3> = StackArrayDeque::new();
        deque.clear();
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn clear_non_empty() {
        let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.clear();
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn overflow_behavior_push_back() {
        let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);
        assert!(deque.is_full());

        // This should overwrite the front element (1)
        deque.push_back(4);
        assert_eq!(deque.len(), 3);
        assert_eq!(deque[0], 2);
        assert_eq!(deque[1], 3);
        assert_eq!(deque[2], 4);
    }

    #[test]
    fn overflow_behavior_push_front() {
        let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);
        assert!(deque.is_full());

        // This should overwrite the back element (3)
        deque.push_front(0);
        assert_eq!(deque.len(), 3);
        assert_eq!(deque[0], 0);
        assert_eq!(deque[1], 1);
        assert_eq!(deque[2], 2);
    }

    #[test]
    fn default() {
        let deque: StackArrayDeque<i32, 5> = StackArrayDeque::default();
        assert!(deque.is_empty());
        assert_eq!(deque.capacity(), 5);
    }

    #[test]
    fn debug_format() {
        let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        let debug_str = format!("{:?}", deque);
        assert_eq!(debug_str, "[1, 2]");
    }

    #[test]
    fn partial_eq() {
        let mut deque1: StackArrayDeque<i32, 3> = StackArrayDeque::new();
        let mut deque2: StackArrayDeque<i32, 3> = StackArrayDeque::new();

        assert_eq!(deque1, deque2);

        deque1.push_back(1);
        deque1.push_back(2);

        deque2.push_back(1);
        deque2.push_back(2);

        assert_eq!(deque1, deque2);

        deque2.push_back(3);
        assert_ne!(deque1, deque2);
    }

    #[test]
    fn circular_behavior() {
        let mut deque: StackArrayDeque<i32, 3> = StackArrayDeque::new();

        // Fill the deque
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);

        // Remove from front and add to back (should wrap around)
        assert_eq!(deque.pop_front(), Some(1));
        deque.push_back(4);

        assert_eq!(deque[0], 2);
        assert_eq!(deque[1], 3);
        assert_eq!(deque[2], 4);

        // Remove from back and add to front (should wrap around)
        assert_eq!(deque.pop_back(), Some(4));
        deque.push_front(0);

        assert_eq!(deque[0], 0);
        assert_eq!(deque[1], 2);
        assert_eq!(deque[2], 3);
    }

    #[test]
    fn mixed_operations() {
        let mut deque: StackArrayDeque<i32, 4> = StackArrayDeque::new();

        deque.push_back(1);
        deque.push_front(0);
        deque.push_back(2);
        deque.push_front(-1);

        assert_eq!(deque.len(), 4);
        assert!(deque.is_full());

        assert_eq!(deque[0], -1);
        assert_eq!(deque[1], 0);
        assert_eq!(deque[2], 1);
        assert_eq!(deque[3], 2);

        assert_eq!(deque.pop_front(), Some(-1));
        assert_eq!(deque.pop_back(), Some(2));
        assert_eq!(deque.len(), 2);

        assert_eq!(deque[0], 0);
        assert_eq!(deque[1], 1);
    }

    #[test]
    fn single_capacity() {
        let mut deque: StackArrayDeque<i32, 1> = StackArrayDeque::new();
        assert_eq!(deque.capacity(), 1);

        deque.push_back(1);
        assert_eq!(deque.len(), 1);
        assert!(deque.is_full());
        assert_eq!(deque[0], 1);

        // Pushing another element should overwrite
        deque.push_back(2);
        assert_eq!(deque.len(), 1);
        assert_eq!(deque[0], 2);

        assert_eq!(deque.pop_front(), Some(2));
        assert!(deque.is_empty());
    }
}
