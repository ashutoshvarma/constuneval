//! Fork of std::borrow::UnevalCow with more proper Debug trait.

pub use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::iter::FromIterator;
use core::ops::{Add, AddAssign, Deref};
use std::borrow::ToOwned;
use std::ffi::{CStr, CString, OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::rc::Rc;

use std::fmt;
use std::string::String;

use UnevalCow::*;

impl<'a, B: ?Sized> Borrow<B> for UnevalCow<'a, B>
where
    B: ToOwned,
    <B as ToOwned>::Owned: 'a,
{
    fn borrow(&self) -> &B {
        &**self
    }
}

/// # Differences between `UnevalCow` and `UnevalCow`
/// - All experimental features are stripped for now like `clone_from()`
///   `is_owned()`, `is_borrowed()`, `extend_one()` etc.
/// - Debug trait is modified to prevent dereferencing and preferring `Borrowed` over
///   `Owned`.
/// - Due to the use of [std::any::TypeId] inside Debug trait, for `UnevalCow::<T>` T must have
///   lifetime of `'static` if you plan to use `Debug` trait.
///
/// ## Debug Trait Rules
/// - `UnevalCow::Borrowed(T)`          ==>  `UnevalCow::Borrowed(&T)`
/// - `UnevalCow::Borrowed(&[T])`       ==>  `UnevalCow::Borrowed(&[T])`
/// - `UnevalCow::Borrowed(Box<[T]>)`   ==>  `UnevalCow::Borrowed(&[T])`
///
/// - `UnevalCow::Owned(T)`             ==>  `UnevalCow::Borrowed(&T)`
/// - `UnevalCow::Owned(&[T])`          ==>  `UnevalCow::Borrowed(&[T])`
/// - `UnevalCow::Owned(Box<[T]>)`      ==>  `UnevalCow::Borrowed(&[T])`
///
///
/// # UnevalCow
/// A fork of [std::borrow::Cow] smart pointer.
///
/// The type `UnevalCow` is a smart pointer providing clone-on-write functionality: it
/// can enclose and provide immutable access to borrowed data, and clone the
/// data lazily when mutation or ownership is required. The type is designed to
/// work with general borrowed data via the `Borrow` trait.
///
/// `UnevalCow` implements `Deref`, which means that you can call
/// non-mutating methods directly on the data it encloses. If mutation
/// is desired, `to_mut` will obtain a mutable reference to an owned
/// value, cloning if necessary.
///
/// # Examples
///
/// ```
/// use constuneval::UnevalCow;
///
/// fn abs_all(input: &mut UnevalCow<[i32]>) {
///     for i in 0..input.len() {
///         let v = input[i];
///         if v < 0 {
///             // Clones into a vector if not already owned.
///             input.to_mut()[i] = -v;
///         }
///     }
/// }
///
/// // No clone occurs because `input` doesn't need to be mutated.
/// let slice = [0, 1, 2];
/// let mut input = UnevalCow::from(&slice[..]);
/// abs_all(&mut input);
///
/// // Clone occurs because `input` needs to be mutated.
/// let slice = [-1, 0, 1];
/// let mut input = UnevalCow::from(&slice[..]);
/// abs_all(&mut input);
///
/// // No clone occurs because `input` is already owned.
/// let mut input = UnevalCow::from(vec![-1, 0, 1]);
/// abs_all(&mut input);
/// ```
///
/// Another example showing how to keep `UnevalCow` in a struct:
///
/// ```
/// use constuneval::UnevalCow;
///
/// struct Items<'a, X: 'a> where [X]: ToOwned<Owned = Vec<X>> {
///     values: UnevalCow<'a, [X]>,
/// }
///
/// impl<'a, X: Clone + 'a> Items<'a, X> where [X]: ToOwned<Owned = Vec<X>> {
///     fn new(v: UnevalCow<'a, [X]>) -> Self {
///         Items { values: v }
///     }
/// }
///
/// // Creates a container from borrowed values of a slice
/// let readonly = [1, 2];
/// let borrowed = Items::new((&readonly[..]).into());
/// match borrowed {
///     Items { values: UnevalCow::Borrowed(b) } => println!("borrowed {:?}", b),
///     _ => panic!("expect borrowed value"),
/// }
///
/// let mut clone_on_write = borrowed;
/// // Mutates the data from slice into owned vec and pushes a new value on top
/// clone_on_write.values.to_mut().push(3);
/// println!("clone_on_write = {:?}", clone_on_write.values);
///
/// // The data was mutated. Let check it out.
/// match clone_on_write {
///     Items { values: UnevalCow::Owned(_) } => println!("clone_on_write contains owned data"),
///     _ => panic!("expect owned data"),
/// }
/// ```

pub enum UnevalCow<'a, B: ?Sized + 'a>
where
    B: ToOwned,
{
    /// Borrowed data.
    Borrowed(&'a B),

    /// Owned data.
    Owned(<B as ToOwned>::Owned),
}

impl<B: ?Sized + ToOwned> Clone for UnevalCow<'_, B> {
    fn clone(&self) -> Self {
        match *self {
            Borrowed(b) => Borrowed(b),
            Owned(ref o) => {
                let b: &B = o.borrow();
                Owned(b.to_owned())
            }
        }
    }

    // fn clone_from(&mut self, source: &Self) {
    //     match (self, source) {
    //         (&mut Owned(ref mut dest), &Owned(ref o)) => o.borrow().clone_into(dest),
    //         (t, s) => *t = s.clone(),
    //     }
    // }
}

impl<B: ?Sized + ToOwned> UnevalCow<'_, B> {
    pub fn to_mut(&mut self) -> &mut <B as ToOwned>::Owned {
        match *self {
            Borrowed(borrowed) => {
                *self = Owned(borrowed.to_owned());
                match *self {
                    Borrowed(..) => unreachable!(),
                    Owned(ref mut owned) => owned,
                }
            }
            Owned(ref mut owned) => owned,
        }
    }

    /// Extracts the owned data.
    ///
    /// Clones the data if it is not already owned.
    ///
    /// # Examples
    ///
    /// Calling `into_owned` on a `UnevalCow::Borrowed` clones the underlying data
    /// and becomes a `UnevalCow::Owned`:
    ///
    /// ```
    /// use constuneval::UnevalCow;
    ///
    /// let s = "Hello world!";
    /// let cow = UnevalCow::Borrowed(s);
    ///
    /// assert_eq!(
    ///   cow.into_owned(),
    ///   String::from(s)
    /// );
    /// ```
    ///
    /// Calling `into_owned` on a `UnevalCow::Owned` is a no-op:
    ///
    /// ```
    /// use constuneval::UnevalCow;
    ///
    /// let s = "Hello world!";
    /// let cow: UnevalCow<str> = UnevalCow::Owned(String::from(s));
    ///
    /// assert_eq!(
    ///   cow.into_owned(),
    ///   String::from(s)
    /// );
    /// ```
    pub fn into_owned(self) -> <B as ToOwned>::Owned {
        match self {
            Borrowed(borrowed) => borrowed.to_owned(),
            Owned(owned) => owned,
        }
    }
}

impl<B: ?Sized + ToOwned> Deref for UnevalCow<'_, B> {
    type Target = B;

    fn deref(&self) -> &B {
        match *self {
            Borrowed(borrowed) => borrowed,
            Owned(ref owned) => owned.borrow(),
        }
    }
}

impl<B: ?Sized> Eq for UnevalCow<'_, B> where B: Eq + ToOwned {}

impl<B: ?Sized> Ord for UnevalCow<'_, B>
where
    B: Ord + ToOwned,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<'a, 'b, B: ?Sized, C: ?Sized> PartialEq<UnevalCow<'b, C>> for UnevalCow<'a, B>
where
    B: PartialEq<C> + ToOwned,
    C: ToOwned,
{
    #[inline]
    fn eq(&self, other: &UnevalCow<'b, C>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<'a, B: ?Sized> PartialOrd for UnevalCow<'a, B>
where
    B: PartialOrd + ToOwned,
{
    #[inline]
    fn partial_cmp(&self, other: &UnevalCow<'a, B>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<B: ?Sized> fmt::Debug for UnevalCow<'_, B>
where
    B: fmt::Debug + 'static,
    B: ToOwned,
    <B as ToOwned>::Owned: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use core::any::TypeId;
        let ty_id = TypeId::of::<B>();
        match *self {
            Borrowed(ref b) => f.write_fmt(format_args!("UnevalCow::Borrowed( &{:?} )", b)),
            Owned(ref o) => {
                // if ty_id == TypeId::of::<<B as ToOwned>::Owned>() {
                //     return f.write_fmt(format_args!("UnevalCow::Owned( {:?} )", o));
                if ty_id == TypeId::of::<str>() {
                    return f.write_fmt(format_args!("UnevalCow::Borrowed( {:?} )", o));
                } else {
                    return f.write_fmt(format_args!("UnevalCow::Borrowed( &{:?} )", o));
                }
            }
        }
    }
}

impl<B: ?Sized> fmt::Display for UnevalCow<'_, B>
where
    B: fmt::Display,
    B: ToOwned,
    <B as ToOwned>::Owned: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Borrowed(ref b) => fmt::Display::fmt(b, f),
            Owned(ref o) => fmt::Display::fmt(o, f),
        }
    }
}

impl<B: ?Sized> Default for UnevalCow<'_, B>
where
    B: ToOwned,
    <B as ToOwned>::Owned: Default,
{
    /// Creates an owned UnevalCow<'a, B> with the default value for the contained owned value.
    fn default() -> Self {
        Owned(<B as ToOwned>::Owned::default())
    }
}

impl<B: ?Sized> Hash for UnevalCow<'_, B>
where
    B: Hash + ToOwned,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T: ?Sized + ToOwned> AsRef<T> for UnevalCow<'_, T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<'a> Add<&'a str> for UnevalCow<'a, str> {
    type Output = UnevalCow<'a, str>;

    #[inline]
    fn add(mut self, rhs: &'a str) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a> Add<UnevalCow<'a, str>> for UnevalCow<'a, str> {
    type Output = UnevalCow<'a, str>;

    #[inline]
    fn add(mut self, rhs: UnevalCow<'a, str>) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a> AddAssign<&'a str> for UnevalCow<'a, str> {
    fn add_assign(&mut self, rhs: &'a str) {
        if self.is_empty() {
            *self = UnevalCow::Borrowed(rhs)
        } else if !rhs.is_empty() {
            if let UnevalCow::Borrowed(lhs) = *self {
                let mut s = String::with_capacity(lhs.len() + rhs.len());
                s.push_str(lhs);
                *self = UnevalCow::Owned(s);
            }
            self.to_mut().push_str(rhs);
        }
    }
}

impl<'a> AddAssign<UnevalCow<'a, str>> for UnevalCow<'a, str> {
    fn add_assign(&mut self, rhs: UnevalCow<'a, str>) {
        if self.is_empty() {
            *self = rhs
        } else if !rhs.is_empty() {
            if let UnevalCow::Borrowed(lhs) = *self {
                let mut s = String::with_capacity(lhs.len() + rhs.len());
                s.push_str(lhs);
                *self = UnevalCow::Owned(s);
            }
            self.to_mut().push_str(&rhs);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Clone-on-write - src/alloc/vec.rs
////////////////////////////////////////////////////////////////////////////////

impl<'a, T: Clone> From<&'a [T]> for UnevalCow<'a, [T]> {
    fn from(s: &'a [T]) -> UnevalCow<'a, [T]> {
        UnevalCow::Borrowed(s)
    }
}

impl<'a, T: Clone> From<Vec<T>> for UnevalCow<'a, [T]> {
    fn from(v: Vec<T>) -> UnevalCow<'a, [T]> {
        UnevalCow::Owned(v)
    }
}

impl<'a, T: Clone> From<&'a Vec<T>> for UnevalCow<'a, [T]> {
    fn from(v: &'a Vec<T>) -> UnevalCow<'a, [T]> {
        UnevalCow::Borrowed(v.as_slice())
    }
}

impl<'a, T> FromIterator<T> for UnevalCow<'a, [T]>
where
    T: Clone,
{
    fn from_iter<I: IntoIterator<Item = T>>(it: I) -> UnevalCow<'a, [T]> {
        UnevalCow::Owned(FromIterator::from_iter(it))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Clone-on-write - src/std/path.rs
////////////////////////////////////////////////////////////////////////////////

impl<'a> From<&'a Path> for UnevalCow<'a, Path> {
    #[inline]
    fn from(s: &'a Path) -> UnevalCow<'a, Path> {
        UnevalCow::Borrowed(s)
    }
}

impl<'a> From<PathBuf> for UnevalCow<'a, Path> {
    #[inline]
    fn from(s: PathBuf) -> UnevalCow<'a, Path> {
        UnevalCow::Owned(s)
    }
}

impl<'a> From<&'a PathBuf> for UnevalCow<'a, Path> {
    #[inline]
    fn from(p: &'a PathBuf) -> UnevalCow<'a, Path> {
        UnevalCow::Borrowed(p.as_path())
    }
}

impl AsRef<Path> for UnevalCow<'_, OsStr> {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl From<UnevalCow<'_, Path>> for Box<Path> {
    #[inline]
    fn from(cow: UnevalCow<'_, Path>) -> Box<Path> {
        match cow {
            UnevalCow::Borrowed(path) => Box::from(path),
            UnevalCow::Owned(path) => Box::from(path),
        }
    }
}


impl<'a> From<UnevalCow<'a, Path>> for PathBuf {
    #[inline]
    fn from(p: UnevalCow<'a, Path>) -> Self {
        p.into_owned()
    }
}


////////////////////////////////////////////////////////////////////////////////
// Clone-on-write - src/alloc/string.rs
////////////////////////////////////////////////////////////////////////////////

impl<'a> Extend<UnevalCow<'a, str>> for String {
    fn extend<I: IntoIterator<Item = UnevalCow<'a, str>>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }

    // #[inline]
    // fn extend_one(&mut self, s: UnevalCow<'a, str>) {
    //     self.push_str(&s);
    // }
}

////////////////////////////////////////////////////////////////////////////////
// Clone-on-write - src/std/ffi/c_str.rs
////////////////////////////////////////////////////////////////////////////////

impl<'a> From<CString> for UnevalCow<'a, CStr> {
    #[inline]
    fn from(s: CString) -> UnevalCow<'a, CStr> {
        UnevalCow::Owned(s)
    }
}

impl<'a> From<&'a CStr> for UnevalCow<'a, CStr> {
    #[inline]
    fn from(s: &'a CStr) -> UnevalCow<'a, CStr> {
        UnevalCow::Borrowed(s)
    }
}

impl<'a> From<&'a CString> for UnevalCow<'a, CStr> {
    #[inline]
    fn from(s: &'a CString) -> UnevalCow<'a, CStr> {
        UnevalCow::Borrowed(s.as_c_str())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Clone-on-write - src/std/ffi/os_str.rs
////////////////////////////////////////////////////////////////////////////////

impl<'a> From<OsString> for UnevalCow<'a, OsStr> {
    #[inline]
    fn from(s: OsString) -> UnevalCow<'a, OsStr> {
        UnevalCow::Owned(s)
    }
}

impl<'a> From<&'a OsStr> for UnevalCow<'a, OsStr> {
    #[inline]
    fn from(s: &'a OsStr) -> UnevalCow<'a, OsStr> {
        UnevalCow::Borrowed(s)
    }
}

impl<'a> From<&'a OsString> for UnevalCow<'a, OsStr> {
    #[inline]
    fn from(s: &'a OsString) -> UnevalCow<'a, OsStr> {
        UnevalCow::Borrowed(s.as_os_str())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Clone-on-write - src/alloc/string.rs
////////////////////////////////////////////////////////////////////////////////

// impl ToString for UnevalCow<'_, str> {
//     #[inline]
//     fn to_string(&self) -> String {
//         self[..].to_owned()
//     }
// }

impl<'a> From<&'a str> for UnevalCow<'a, str> {
    #[inline]
    fn from(s: &'a str) -> UnevalCow<'a, str> {
        UnevalCow::Borrowed(s)
    }
}

impl<'a> From<String> for UnevalCow<'a, str> {
    #[inline]
    fn from(s: String) -> UnevalCow<'a, str> {
        UnevalCow::Owned(s)
    }
}

impl<'a> From<&'a String> for UnevalCow<'a, str> {
    #[inline]
    fn from(s: &'a String) -> UnevalCow<'a, str> {
        UnevalCow::Borrowed(s.as_str())
    }
}

impl<'a> FromIterator<char> for UnevalCow<'a, str> {
    fn from_iter<I: IntoIterator<Item = char>>(it: I) -> UnevalCow<'a, str> {
        UnevalCow::Owned(FromIterator::from_iter(it))
    }
}

impl<'a, 'b> FromIterator<&'b str> for UnevalCow<'a, str> {
    fn from_iter<I: IntoIterator<Item = &'b str>>(it: I) -> UnevalCow<'a, str> {
        UnevalCow::Owned(FromIterator::from_iter(it))
    }
}

impl<'a> FromIterator<String> for UnevalCow<'a, str> {
    fn from_iter<I: IntoIterator<Item = String>>(it: I) -> UnevalCow<'a, str> {
        UnevalCow::Owned(FromIterator::from_iter(it))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Clone-on-write - src/alloc/box.rs
////////////////////////////////////////////////////////////////////////////////

impl<T: Copy> From<UnevalCow<'_, [T]>> for Box<[T]> {
    #[inline]
    fn from(cow: UnevalCow<'_, [T]>) -> Box<[T]> {
        match cow {
            UnevalCow::Borrowed(slice) => Box::from(slice),
            UnevalCow::Owned(slice) => Box::from(slice),
        }
    }
}

impl From<UnevalCow<'_, str>> for Box<str> {
    #[inline]
    fn from(cow: UnevalCow<'_, str>) -> Box<str> {
        match cow {
            UnevalCow::Borrowed(s) => Box::from(s),
            UnevalCow::Owned(s) => Box::from(s),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Clone-on-write - src/alloc/sync.rs
////////////////////////////////////////////////////////////////////////////////

impl<'a, B> From<UnevalCow<'a, B>> for Arc<B>
where
    B: ToOwned + ?Sized,
    Arc<B>: From<&'a B> + From<B::Owned>,
{
    #[inline]
    fn from(cow: UnevalCow<'a, B>) -> Arc<B> {
        match cow {
            UnevalCow::Borrowed(s) => Arc::from(s),
            UnevalCow::Owned(s) => Arc::from(s),
        }
    }
}



////////////////////////////////////////////////////////////////////////////////
// Clone-on-write - src/alloc/rc.rs
////////////////////////////////////////////////////////////////////////////////


impl<'a, B> From<UnevalCow<'a, B>> for Rc<B>
where
    B: ToOwned + ?Sized,
    Rc<B>: From<&'a B> + From<B::Owned>,
{
    #[inline]
    fn from(cow: UnevalCow<'a, B>) -> Rc<B> {
        match cow {
            UnevalCow::Borrowed(s) => Rc::from(s),
            UnevalCow::Owned(s) => Rc::from(s),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Clone-on-write - src/alloc/boxed.rs
////////////////////////////////////////////////////////////////////////////////

