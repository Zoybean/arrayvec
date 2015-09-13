use std::borrow::Borrow;
use std::fmt;
use std::mem;
use std::ops::Deref;
use std::str;
use std::slice;

use array::Array;
use array::Index;
use CapacityError;

/// A string with a fixed capacity.
///
/// The `ArrayString` is a string backed by a fixed size array. It keeps track
/// of its length.
///
/// The string is a contiguous value that you can store directly on the stack
/// if needed.
#[derive(Copy)]
pub struct ArrayString<A: Array<Item=u8>> {
    xs: A,
    len: A::Index,
}

unsafe fn new_array<A: Array<Item=u8>>() -> A {
    // Note: Returning an uninitialized value here only works
    // if we can be sure the data is never used. The nullable pointer
    // inside enum optimization conflicts with this this for example,
    // so we need to be extra careful. See `NoDrop` enum.
    mem::uninitialized()
}

impl<A: Array<Item=u8>> ArrayString<A> {
    /// Create a new empty `ArrayString`.
    ///
    /// Capacity is inferred from the type parameter.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let mut string = ArrayString::<[_; 16]>::new();
    /// string.push_str("foo");
    /// assert_eq!(&string[..], "foo");
    /// assert_eq!(string.capacity(), 16);
    /// ```
    pub fn new() -> ArrayString<A> {
        unsafe {
            ArrayString {
                xs: new_array(),
                len: Index::from(0),
            }
        }
    }

    /// Return the capacity of the `ArrayString`.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let string = ArrayString::<[_; 3]>::new();
    /// assert_eq!(string.capacity(), 3);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize { A::capacity() }

    /// Adds the given char to the end of the string.
    ///
    /// Returns `Ok` if the push succeeds, and returns `Err` if the backing
    /// array is not large enough to fit the additional char.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let mut string = ArrayString::<[_; 2]>::new();
    ///
    /// string.push('a').unwrap();
    /// string.push('b').unwrap();
    /// let overflow = string.push('c');
    ///
    /// assert_eq!(&string[..], "ab");
    /// assert_eq!(overflow.err().map(|e| e.element), Some('c'));
    /// ```
    pub fn push(&mut self, c: char) -> Result<(), CapacityError<char>> {
        use std::fmt::Write;
        self.write_char(c).map_err(|_| CapacityError::new(c))
    }

    /// Adds the given string slice to the end of the string.
    ///
    /// Returns `Ok` if the push succeeds, and returns `Err` if the
    /// backing array is not large enough to fit the string.
    ///
    /// ```
    /// use arrayvec::ArrayString;
    ///
    /// let mut string = ArrayString::<[_; 2]>::new();
    ///
    /// string.push_str("a").unwrap();
    /// let overflow1 = string.push_str("bc");
    /// string.push_str("d").unwrap();
    /// let overflow2 = string.push_str("ef");
    ///
    /// assert_eq!(&string[..], "ad");
    /// assert_eq!(overflow1.err().map(|e| e.element), Some("bc"));
    /// assert_eq!(overflow2.err().map(|e| e.element), Some("ef"));
    /// ```
    pub fn push_str<'a>(&mut self, s: &'a str) -> Result<(), CapacityError<&'a str>> {
        use std::io::Write;

        if self.len() + s.len() > self.capacity() {
            return Err(CapacityError::new(s));
        }
        unsafe {
            let sl = slice::from_raw_parts_mut(self.xs.as_mut_ptr(), A::capacity());
            (&mut sl[self.len()..]).write(s.as_bytes()).unwrap();
            let newl = self.len() + s.len();
            self.set_len(newl);
        }
        Ok(())
    }

    /// Make the string empty.
    pub fn clear(&mut self) {
        unsafe {
            self.set_len(0);
        }
    }

    /// Set the strings's length.
    ///
    /// May panic if `length` is greater than the capacity.
    ///
    /// This function is `unsafe` because it changes the notion of the
    /// number of “valid” bytes in the string. Use with care.
    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        debug_assert!(length <= self.capacity());
        self.len = Index::from(length);
    }
}

impl<A: Array<Item=u8>> Deref for ArrayString<A> {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        unsafe {
            let sl = slice::from_raw_parts(self.xs.as_ptr(), self.len.to_usize());
            str::from_utf8_unchecked(sl)
        }
    }
}

impl<A: Array<Item=u8>> Borrow<str> for ArrayString<A> {
    fn borrow(&self) -> &str { self }
}

impl<A: Array<Item=u8>> AsRef<str> for ArrayString<A> {
    fn as_ref(&self) -> &str { self }
}

impl<A: Array<Item=u8>> fmt::Debug for ArrayString<A> where A::Item: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { (**self).fmt(f) }
}

/// `Write` appends written data to the end of the string.
impl<A: Array<Item=u8>> fmt::Write for ArrayString<A> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s).map_err(|_| fmt::Error)
    }
}

impl<A: Array<Item=u8> + Copy> Clone for ArrayString<A> {
    fn clone(&self) -> ArrayString<A> {
        *self
    }
}

