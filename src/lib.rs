use std::convert::TryFrom;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::ops::{self, RangeBounds};
use std::slice::{Iter, IterMut, SliceIndex};
use std::vec::IntoIter;

#[cfg(feature = "serde")]
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

/// Non empty vector, ensure non empty by construction.
/// Inherits `Vec`'s methods through `Deref` trait, not implement `DerefMut`.
/// Overridden these methods:
/// * `len` returns `NonZeroUsize` and `is_empty` always returns `false`.
/// * `first(_mut)`, `last(_mut)`, `split_first(_mut)`, `split_last(_mut)` don't return `Option`.
/// * `pop` returns `None` if there is only one element in it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonEmpty<T>(Vec<T>);

impl<T> NonEmpty<T> {
    #[inline]
    pub fn new(v: T) -> Self {
        Self(vec![v])
    }

    /// Constructs a non-empty vec without checking its size.
    ///
    /// # Safety
    /// `vec` should not be empty.
    #[inline]
    pub unsafe fn new_unchecked(vec: Vec<T>) -> Self {
        Self(vec)
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *const T {
        self.0.as_mut_ptr()
    }

    #[inline]
    pub fn len(&self) -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(self.0.len()) }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        false
    }

    #[inline]
    pub fn first(&self) -> &T {
        unsafe { self.0.get_unchecked(0) }
    }

    #[inline]
    pub fn first_mut(&mut self) -> &mut T {
        unsafe { self.0.get_unchecked_mut(0) }
    }

    #[inline]
    pub fn last(&self) -> &T {
        let i = self.len().get() - 1;
        unsafe { self.0.get_unchecked(i) }
    }

    #[inline]
    pub fn last_mut(&mut self) -> &mut T {
        let i = self.len().get() - 1;
        unsafe { self.0.get_unchecked_mut(i) }
    }

    #[inline]
    pub fn split_first(&self) -> (&T, &[T]) {
        (&self[0], &self[1..])
    }

    #[inline]
    pub fn split_first_mut(&mut self) -> (&mut T, &mut [T]) {
        let split = self.0.split_at_mut(1);
        (&mut split.0[0], split.1)
    }

    #[inline]
    pub fn split_last(&self) -> (&T, &[T]) {
        let len = self.len().get();
        (&self[len - 1], &self[..(len - 1)])
    }

    #[inline]
    pub fn split_last_mut(&mut self) -> (&mut T, &mut [T]) {
        let i = self.len().get() - 1;
        let split = self.0.split_at_mut(i);
        (&mut split.1[0], split.0)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.0.len() <= 1 {
            None
        } else {
            self.0.pop()
        }
    }

    #[inline]
    pub fn push(&mut self, v: T) {
        self.0.push(v)
    }

    #[inline]
    pub fn truncate(&mut self, len: NonZeroUsize) {
        self.0.truncate(len.get())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.0.iter_mut()
    }
}

impl<T> From<(Vec<T>, T)> for NonEmpty<T> {
    fn from((mut xs, x): (Vec<T>, T)) -> NonEmpty<T> {
        xs.push(x);
        NonEmpty(xs)
    }
}

impl<T> From<(T, Vec<T>)> for NonEmpty<T> {
    fn from((x, mut xs): (T, Vec<T>)) -> NonEmpty<T> {
        xs.insert(0, x);
        NonEmpty(xs)
    }
}

impl<T> From<NonEmpty<T>> for Vec<T> {
    fn from(v: NonEmpty<T>) -> Self {
        v.0
    }
}

/// Returns a unit-length vector containing the default element value.
impl<T: Default> Default for NonEmpty<T> {
    fn default() -> Self {
        ne_vec![T::default()]
    }
}

#[derive(Debug, PartialEq)]
pub struct EmptyError;

impl<T> TryFrom<Vec<T>> for NonEmpty<T> {
    type Error = EmptyError;
    fn try_from(xs: Vec<T>) -> Result<Self, Self::Error> {
        if xs.is_empty() {
            Err(EmptyError)
        } else {
            Ok(NonEmpty(xs))
        }
    }
}

impl<T> ops::Deref for NonEmpty<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.0.deref()
    }
}

impl<T> AsRef<[T]> for NonEmpty<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for NonEmpty<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
    }
}

impl<T> AsRef<Vec<T>> for NonEmpty<T> {
    fn as_ref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T, I: SliceIndex<[T]>> ops::Index<I> for NonEmpty<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        ops::Index::index(self.as_slice(), index)
    }
}
impl<T, I: SliceIndex<[T]>> ops::IndexMut<I> for NonEmpty<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        ops::IndexMut::index_mut(self.as_mut_slice(), index)
    }
}

impl<T> IntoIterator for NonEmpty<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<'a, T> IntoIterator for &'a NonEmpty<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut NonEmpty<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T> NonEmpty<T> {
    /// Removes the specified range from the vector in bulk, returning the removed items as an iterator.
    /// # Panics
    /// If the range specified would remove all elements from the vector. There must be at least 1 element left over.
    /// # Examples
    /// Removing all but the first element.
    /// ```
    /// # use non_empty_vec::{NonEmpty, ne_vec};
    /// let mut v = ne_vec!(0, 1, 2, 3, 4, 5);
    /// let removed: Vec<_> = v.drain(1..).collect();
    /// assert_eq!(removed, vec![1, 2, 3, 4, 5]);
    /// assert_eq!(v, ne_vec![0]);
    /// ```
    ///
    /// Removing all but the last element.
    /// ```
    /// # use non_empty_vec::{NonEmpty, ne_vec};
    /// let mut v = ne_vec!(0, 1, 2, 3, 4, 5);
    /// let removed: Vec<_> = v.drain(..v.len().get()-1).collect();
    /// assert_eq!(removed, vec![0, 1, 2, 3, 4]);
    /// assert_eq!(v, ne_vec![5]);
    /// ```
    /// Removing all elements (these panic).
    /// ```should_panic
    /// # use non_empty_vec::ne_vec;
    /// # let mut v = ne_vec!(0, 1, 2, 3, 4, 5);
    /// v.drain(..);
    /// ```
    /// ```should_panic
    /// # use non_empty_vec::ne_vec;
    /// # let mut v = ne_vec!(0, 1, 2, 3, 4, 5);
    /// v.drain(0..v.len().get());
    /// ```
    #[track_caller]
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> std::vec::Drain<T> {
        // whether or not there is space leftover in the start of the vector.
        let leftover_start = match range.start_bound() {
            core::ops::Bound::Included(&start) => start > 0,
            core::ops::Bound::Excluded(_) => true,
            core::ops::Bound::Unbounded => false,
        };
        if !leftover_start {
            // whether or not there is space leftover in the end of the vector.
            let leftover_end = match range.end_bound() {
                core::ops::Bound::Excluded(&end) => end < self.len().get(),
                core::ops::Bound::Included(&end) => end < self.len().get() - 1,
                core::ops::Bound::Unbounded => false,
            };
            if !leftover_end {
                panic!(
                    "range specified for `NonEmpty::drain` must leave at least one element left"
                );
            }
        }
        self.0.drain(range)
    }

    /// Calls a predicate with every element of this vector, removing each element for which the predicate returns `true`.
    /// All removed elements are yielded from the returned iterator.
    /// # Examples
    /// Normal use.
    /// ```
    /// // Filter out odd entries
    /// # use non_empty_vec::ne_vec;
    /// let mut v = ne_vec![1,2,3,4,5,6];
    /// assert!(v.drain_filter(|i| *i % 2 == 1).eq([1, 3, 5]));
    /// assert_eq!(v, ne_vec![2, 4, 6]);
    /// ```
    /// At least one element is always left behind.
    /// ```
    /// // When there's only one element left, the predicate never even gets called on it.
    /// # use non_empty_vec::ne_vec;
    /// let mut v = ne_vec![1];
    /// v.drain_filter(|_| unreachable!());
    /// assert_eq!(v, ne_vec![1]);
    ///
    /// // This also applies if all elements before the final get removed.
    /// let mut v = ne_vec![1, 2, 3, 4, 5];
    /// let removed = v.drain_filter(|&mut i| if i < 5 {
    ///     true
    /// } else {
    ///     unreachable!()
    /// });
    /// assert!(removed.eq(1..=4));
    /// assert_eq!(v, ne_vec![5]);
    /// ```
    /// Lazy execution.
    /// ```
    /// // Nothing gets removed until the iterator is consumed
    /// # use non_empty_vec::ne_vec;
    /// let mut v = ne_vec![1,2,3,4];
    /// v.drain_filter(|_| true);
    /// assert_eq!(v, ne_vec![1,2,3,4]);
    /// ```
    #[inline]
    pub fn drain_filter<F>(&mut self, f: F) -> DrainFilter<T, F>
    where
        F: FnMut(&mut T) -> bool,
    {
        DrainFilter::new(self, f)
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    _p: PhantomData<&'a mut NonEmpty<T>>,
    items: *mut T,
    f: F,

    // Always `0 <= left <= i < r <= right <= old_len`

    // any items to the left of `left` are initialized and staying in the vector.
    left: usize,
    // any items between `left` and `i` are uninitialized memory.
    i: usize,
    // any items between `i` and `r` are initialized and not searched yet.
    r: usize,
    // any items between `r` and `right` and uninitialized memory.
    right: usize,
    // any items to the right of `right` are initialized and staying in the vector.
    old_len: usize,
}
impl<'a, T, F> DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    #[inline]
    pub fn new(vec: &'a mut NonEmpty<T>, f: F) -> Self {
        let left = 0;
        let i = 0;
        let old_len = vec.0.len();
        let right = old_len;
        let r = right;
        Self {
            _p: PhantomData,
            items: vec.0.as_mut_ptr(),
            f,
            left,
            i,
            r,
            right,
            old_len,
        }
    }

    /// Removes the first unsearched element from the front of the vector.
    fn pop_front(&mut self) -> Option<T> {
        if self.r - self.i > 1 {
            let item = unsafe {
                // Move out the first item. This is valid since the source
                // will be uninitialized next line.
                let item = std::ptr::read(self.items.add(self.i));
                // due to the invariants of this type,
                // we have just marked the former first item as uninitialized.
                self.i += 1;
                item
            };
            Some(item)
        } else {
            None
        }
    }
    /// Removes the first unsearched element from the back of the vector.
    fn pop_back(&mut self) -> Option<T> {
        if self.r - self.i > 1 {
            let item = unsafe {
                let item = std::ptr::read(self.items.add(self.r - 1));
                self.r -= 1;
                item
            };
            Some(item)
        } else {
            None
        }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    fn insert_front(&mut self, item: T) {
        #[cold]
        #[cfg_attr(debug_assertions, track_caller)]
        fn do_panic() -> ! {
            panic!("no uninitialized space available in front!");
        }

        if self.left >= self.i {
            do_panic();
        }
        unsafe {
            std::ptr::write(self.items.add(self.left), item);
            self.left += 1;
        }
    }
    #[cfg_attr(debug_assertions, track_caller)]
    fn insert_back(&mut self, item: T) {
        #[cold]
        #[cfg_attr(debug_assertions, track_caller)]
        fn do_panic() -> ! {
            panic!("no uninitialized space available in the back!");
        }

        if self.right <= self.r {
            do_panic();
        }
        unsafe {
            std::ptr::write(self.items.add(self.right - 1), item);
            self.right -= 1;
        }
    }
}
impl<'a, T, F> Drop for DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    fn drop(&mut self) {
        // Move unsearched elements to the front of the vector
        while let Some(item) = self.pop_front() {
            self.insert_front(item);
        }
        // Move items at the end to the front.
        while self.right < self.old_len {
            // We no longer care about updateing `i` and `r` anymore.

            let item = unsafe {
                let item = std::ptr::read(self.items.add(self.right - 1));
                self.right += 1;
                item
            };
            unsafe {
                std::ptr::write(self.items.add(self.left), item);
                self.left += 1;
            }
        }
    }
}

impl<'a, T, F> Iterator for DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // Loop until either we find an element, or run out of elements to search.
        loop {
            // Remove the first unsearched element from the iterator.
            let mut item = self.pop_front()?;
            // If it passed the predicate, yield it.
            if (self.f)(&mut item) {
                break Some(item);
            }
            // Otherwise, place it back at the beginning of the vector.
            else {
                self.insert_front(item);
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let max = self.r - self.i - 1;
        (0, Some(max))
    }
}
impl<'a, T, F> DoubleEndedIterator for DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        // Loop until either we find an element or the list is depleted.
        loop {
            let mut item = self.pop_back()?;
            if (self.f)(&mut item) {
                break Some(item);
            } else {
                self.insert_back(item);
            }
        }
    }
}
impl<'a, T, F> FusedIterator for DrainFilter<'a, T, F> where F: FnMut(&mut T) -> bool {}

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for NonEmpty<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for NonEmpty<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_from(<Vec<T>>::deserialize(deserializer)?)
            .map_err(|_| D::Error::custom("empty vector"))
    }
}

/// Constructs a [`NonEmpty`] vector, similar to std's `vec` macro.
///
/// This macro will generally try to check the validity of the length at compile time if it can.
///
/// If the length is an expression (e.g. `ne_vec![(); { 0 }]`), the check is performed at runtime
/// to allow the length to be dynamic.
///
/// # Examples
/// Proper use.
/// ```
/// # use non_empty_vec::*;
/// # use std::convert::TryFrom;
/// assert_eq!(
///     ne_vec![1, 2, 3],
///     NonEmpty::try_from(vec![1, 2, 3_i32]).unwrap(),
/// );
///
/// assert_eq!(
///     ne_vec![1; 3],
///     NonEmpty::try_from(vec![1, 1, 1]).unwrap(),
/// );
/// ```
/// Improper use.
/// ```compile_fail
/// # use non_empty_vec::*;
/// let _ = ne_vec![];
/// ```
///
/// ```compile_fail
/// # use non_empty_vec::*;
/// let _ = ne_vec![1; 0];
/// ```
///
/// ```compile_fail
/// # use non_empty_vec::*;
/// let _ = ne_vec![1; 0usize];
/// ```
///
/// ```should_panic
/// # use non_empty_vec::*;
/// let n = 0;
/// let _ = ne_vec![1; n];
/// ```
#[macro_export]
macro_rules! ne_vec {
    () => {
        ::std::compile_error!("`NonEmpty` vector must be non-empty")
    };
    ($($x:expr),+ $(,)?) => {
        unsafe { $crate::NonEmpty::new_unchecked(vec![$($x),+]) }
    };
    ($elem:expr; 0) => {
        // if 0 is passed to the macro we can generate a good compile error
        ne_vec![]
    };
    ($elem:expr; $n:literal) => {{
        // extra guard to reject compilation if $n ends up being 0 in some other way (e.g. ne_vec![1; 0usize])
        const _ASSERT_NON_ZERO: [(); $n - 1] = [(); $n - 1];
        unsafe { $crate::NonEmpty::new_unchecked(vec![$elem; $n]) }
    }};
    ($elem:expr; $n:expr) => {{
        // if $n is an expression, we cannot check the length at compile time and do it at runtime
        if $n == 0 {
            ::std::panic!("`NonEmpty` vector must be non-empty");
        }
        unsafe { $crate::NonEmpty::new_unchecked(vec![$elem; $n]) }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // From
        let mut list: NonEmpty<i32> = (vec![1, 2], 3).into();
        assert_eq!(list, (1, vec![2, 3]).into());
        assert_eq!(&*list, &[1, 2, 3]);

        // Index
        list[0] = 2;
        assert_eq!(list[0], 2);
        list[0] = 1;
        assert_eq!(list[0], 1);

        // slice methods
        assert_eq!(list.len().get(), 3);
        assert_eq!(list.as_slice(), &[1, 2, 3]);

        // TryFrom
        assert_eq!(<NonEmpty<i32>>::try_from(vec![]).ok(), None);
        assert_eq!(
            &*<NonEmpty<i32>>::try_from(vec![1, 2, 3]).unwrap(),
            &[1, 2, 3]
        );

        // Iterator
        assert_eq!(
            list.iter().map(|n| n * 2).collect::<Vec<_>>(),
            vec![2, 4, 6]
        );

        // Single
        let single = NonEmpty::new(15_i32);
        assert_eq!(single.len().get(), 1);
        assert_eq!(single[0], 15);
    }

    #[test]
    fn default() {
        assert_eq!(NonEmpty::<i32>::default(), ne_vec![0]);
        assert_eq!(NonEmpty::<&str>::default(), ne_vec![""]);
    }

    #[test]
    fn into_iter() {
        let mut list = ne_vec![1, 2, 3];

        for (a, b) in [1, 2, 3].iter().zip(&list) {
            assert_eq!(a, b);
        }

        for a in &mut list {
            *a += 1;
        }
        assert_eq!(list.as_slice(), &[2, 3, 4]);

        for (a, b) in vec![2, 3, 4].into_iter().zip(list) {
            assert_eq!(a, b);
        }
    }

    #[test]
    fn drain_filter() {
        // Filter out odd numbers.
        let mut v = ne_vec![1, 2, 3, 4, 5, 6];
        assert!(v.drain_filter(|val| *val % 2 == 1).eq([1, 3, 5]));
        assert_eq!(v, ne_vec![2, 4, 6]);

        // singleton
        let mut v = ne_vec![1];
        for _ in v.drain_filter(|_| unreachable!()) {}
        assert_eq!(v, ne_vec![1]);

        // leftover
        let mut v = ne_vec![1, 2, 3];
        let removed = v.drain_filter(|&mut val| if val < 3 { true } else { unreachable!() });
        assert!(removed.eq([1, 2]));
        assert_eq!(v, ne_vec![3]);

        // double-ended, meet in middle
        let mut v = ne_vec![1, 2, 3, 4, 5, 6];
        let mut rem = v.drain_filter(|val| *val % 2 == 1);
        assert_eq!(rem.next(), Some(1));
        assert_eq!(rem.next_back(), Some(5));
        assert_eq!(rem.next_back(), Some(3));
        assert_eq!(rem.next(), None);
        assert_eq!(rem.next_back(), None);

        // rev
        let mut v = ne_vec![1, 2, 3, 4, 5, 6];
        let rem = v.drain_filter(|val| *val % 2 == 0).rev();
        assert!(rem.eq([6, 4, 2]));
        assert_eq!(v, ne_vec![1, 3, 5]);

        // singleton-back
        let mut v = ne_vec![1];
        for _ in v.drain_filter(|_| unreachable!()) {}
        assert_eq!(v, ne_vec![1]);

        // leftover-back
        let mut v = ne_vec![1, 2, 3];
        let removed = v
            .drain_filter(|&mut val| if val > 1 { true } else { unreachable!() })
            .rev();
        assert!(removed.eq([3, 2]));
        assert_eq!(v, ne_vec![1]);

        // meet in middle, unreachable
        let mut v = ne_vec![1, 2, 3];
        let mut rem = v.drain_filter(|&mut val| if val == 2 { unreachable!() } else { true });
        assert_eq!(rem.next_back(), Some(3));
        assert_eq!(rem.next(), Some(1));
        assert_eq!(rem.next_back(), None);
        assert_eq!(rem.next(), None);
        std::mem::drop(rem);
        assert_eq!(v, ne_vec![2]);
    }

    #[test]
    fn initialize_macro() {
        assert_eq!(ne_vec![1; 3].as_slice(), &[1, 1, 1]);
        assert_eq!(ne_vec!["string"; 5].as_slice(), &["string"; 5]);
    }

    #[test]
    #[should_panic]
    fn initialize_macro_zero_size() {
        // ne_vec![1; 0] results in a compile error
        let n = 0;
        let _ = ne_vec![1; n];
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize() {
        use serde_json;

        let vec: NonEmpty<u32> = (1, vec![]).into();
        assert_eq!(
            serde_json::from_str::<NonEmpty<u32>>(&serde_json::to_string(&vec).unwrap()).unwrap(),
            vec
        );
    }
}
