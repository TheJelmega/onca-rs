//! Reimplementation of std::borrow types to use onca types instead of other std types

use core::{borrow::Borrow, fmt, hash::Hash, ops::{Add, Deref, DerefPure}};






/// A generalization of `Clone` to borrowed data.
/// 
/// Some types make it possible to go from borrowed to owned, usually by implementing the `Clone` trait.
/// But `Clone` works only for going from `&T` to `T`.
/// The `ToOwned` triat generates `Clone` to construct owned data from any borrow of a given type.
/// 
/// `U` represenst any additional type needed to be able to convert it into an Owned.
pub trait ToOwned<U> {
    /// The resulting type after obtaining ownership.
    type Owned: Borrow<Self>;

    /// Creates owned data from borrowed data, usually by cloning.
    /// 
    /// # Examples:
    /// 
    /// ```
    /// let s: &str = "a";
    /// let ss: String = s.to_owned();
    /// 
    /// let a: &[i32] = &[1, 2];
    /// let aa = DynArr<i32> = a.to_owned();
    /// ```
    #[must_use = "cloning is often expensive and is not expected to have side effects"]
    fn to_owned(&self) -> Self::Owned;

    /// Uses borrowed data to replace owned data, usually by cloning.
    /// 
    /// This is borrow-generalized version of [`Clone::clone_from`].
    /// 
    /// # Examples
    /// 
    /// Basic usage:
    /// 
    /// ```
    /// let mut s: String = String::new();
    /// "hello".clone_into(&mut s);
    /// 
    /// let mut a: DynArr<u32>
    /// ```
    fn clone_into(&self, target: &mut Self::Owned) {
        *target = self.to_owned();
    }
}

impl<T: Clone> ToOwned<()> for T {
    type Owned = T;

    fn to_owned(&self) -> Self::Owned {
        self.clone()
    }
    
    fn clone_into(&self, target: &mut Self::Owned) {
        target.clone_from(self)
    }
}

/// A clone-on-write smart pointer.
/// 
/// The type `Cow` is a smart poiter providing clone-on-write functionality: it can  enclose and provide immutable access to borrowed data,
/// and clone the data lazily when mutation or ownership is required.
/// The type is designed to work with generally borrowed data via the `Borrow` trait.
/// 
/// `Cow` implements `Deref`, which means that you can call non-mutationg methods directly on the data is encloses.
/// If mutation is desired, `to_mut` will obtain a mutable reference to an owned value, cloning if neccesary.
/// 
/// // TODO: Rc and Arc
/// If you need reference-counting pointers, note that [`Rc::make_mut`][crate::mem::Rc::make_mut] and [`Arc::make_mut`][core::mem::Arc::make_mut]
/// can provide clone-on-write functionality as well.
/// 
/// # Examples
/// 
/// ```
/// use onca_common::borrow::Cow;
/// 
/// fn abs_all(input: &mut Cow<'a, [i32]>) {
///     for i in 0..input.len() {
///         let a = input[i];
///         if a < 0 {
///             // Clone into a dynarr if not already owned.
///             input.to_mut()[i] = -a;
///         }
///     }
/// }
/// 
/// // No clone occurs because `input` doesn't  need to be mutated.
/// let slice = [0, 1, 2];
/// let mut input = Cow::from(&slice[..]);
/// abs_all(&mut input);
/// 
/// // Clone occurs because `input` needs to be mutated.
/// let slice = [-1, 0, 1];
/// let mut input = Cow::from(&slice[..]);
/// abs_all(&mut input);
/// 
/// // No clone occurs because `input` is already owned.
/// let mut input = Cow::from(dynarr![-1, 0, 1]);
/// abs_all(&mut input);
/// ```
/// 
/// Another example showing how to keep `Cow` in a struct:
/// 
/// ```
/// use onca_common::borrow::Cow;
/// 
/// struct Items<'a, X> where [X]: ToOwned<Owned = DynArr<X>> {
///     value: Cow<'a, [X]>
/// }
/// 
/// impl<'a, X: Clone + 'a> Items<'a, X> where [X]: ToOwned<Owned = DynArr<X>> {
///     fn new(a: Cow<'a, [X]>) -> Self {
///         Items { values: a }
///     }
/// }
/// 
/// // Creates a container from borrowed values of a slice
/// let readonly = [1, 2];
/// let borrowed = Items::new((&readonly[..]).into());
/// match borrowed {
///     Items { values: Cow::Borrowed(b) } => println!("borrowed {b:?}"),
///     _ => panic!("expect borrowed value"),
/// }
/// 
/// let mut clone_on_write = borrowed;
/// // Mutates the data from slice into owned dynarr and pushses a new value on top
/// clone_on_write.values.to_mut().push(3);
/// println!("clone_on@rite = {:?}", clone_on_write.values);
/// 
/// // The data was mutated. Let's check it out
/// match clone_on_write {
///     Items { values: Cow::Owned(_) } => println!("clone_on_write contains owned data"),
///     _ => panic!("expect owned data"),
/// }
/// ```
pub enum Cow<'a, B: ?Sized + ToOwned<U> + 'a, U = ()> {
    /// Borrowed data
    Borrowed(&'a B),
    /// Owned data
    Owned(<B as ToOwned<U>>::Owned)
}

impl<B: ?Sized + ToOwned<U>, U> Clone for Cow<'_, B, U> {
    fn clone(&self) -> Self {
        match *self {
            Self::Borrowed(b) => Self::Borrowed(b),
            Self::Owned(ref o) => {
                let b: &B = o.borrow();
                Self::Owned(b.to_owned())
            }
        }
    }
}

impl<B: ?Sized + ToOwned<U>, U> Cow<'_, B, U> {
    /// Returns true if the data is borrowed, i.e. if `to_mut` would require additional work.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use onca_common::borrow::Cow;
    /// 
    /// let cow = Cow::Borrowed("moo");
    /// assert!(cow.is_borrowed());
    /// 
    /// let bull: Cow<'_, str> = Cow::owned("...moo?".to_string());
    /// assert!(!bull.is_borrowed());
    /// ```
    pub const fn is_borrowed(&self) -> bool {
        match *self {
            Self::Borrowed(_) => true,
            Self::Owned(_) => false,
        }
    }

    /// Acquires a mutable refernce to the owned from of the data.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use onca_common::borrow::Cow;
    /// 
    /// let cow: Cow<'_, str> = Cow::Owned("moo".to_string());
    /// assert!(cow.is_owned());
    /// 
    /// let bull = Cow::Borrowed("...moo?");
    /// assert!(!bull.is_owned());
    /// ```
    pub const fn is_owned(&self) -> bool {
        match *self {
            Self::Borrowed(_) => false,
            Self::Owned(_) => true,
        }
    }

    /// Acquires a mutable reference to teh owned form of the data.
    /// 
    /// Clones the data if it is not already owned.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use onca_common::borrow::Cow;
    /// 
    /// let mut cow = Cow::Borrowed("foo");
    /// cow.to_mut().make_ascii_uppercase();
    /// 
    /// assert_eq!(cow, Cow::Owned(String::from("FOO")) as Cow<'_ str>);
    /// ```
    pub fn to_mut(&mut self) -> &mut <B as  ToOwned<U>>::Owned {
        match *self {
            Self::Borrowed(borrowed) => {
                *self = Self::Owned(borrowed.to_owned());
                match *self {
                    Self::Borrowed(..) => unreachable!(),
                    Self::Owned(ref mut owned) => owned,
                }
            },
            Self::Owned(ref mut owned) => owned,
        }
    }

    /// Extracts the owned data.
    /// 
    /// Clones the data if it is not already owned.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use onca_common::borrow::Cow;
    /// 
    /// let s = "Hello world!";
    /// let cow = Cow::Borrowed(s);
    /// 
    /// assert_eq!(cow.into_owned(), String::from(s));
    /// ```
    /// 
    /// Calling `into_owned` on a `Cow::Owned` returns the owned data.
    /// The data is moved out of the `Cow` without being cloned.
    pub fn into_owned(self) -> <B as ToOwned<U>>::Owned {
        match self {
            Self::Borrowed(borrowed) => borrowed.to_owned(),
            Self::Owned(owned) => owned,
        }
    }
}

impl<B: ?Sized + ToOwned<U>, U> Deref for Cow<'_, B, U> where
    B::Owned: Borrow<B>
{
    type Target = B;

    fn deref(&self) -> &Self::Target {
        match *self {
            Self::Borrowed(borrowed) => borrowed,
            Self::Owned(ref owned) => owned.borrow()
        }
    }
}

unsafe impl<B: ?Sized + ToOwned<U>, U> DerefPure for Cow<'_, B, U> where B::Owned: Borrow<B> {}

impl<'a, 'b, B: ?Sized, C: ?Sized, U> PartialEq<Cow<'b, C, U>> for Cow<'a, B, U> where
    B: PartialEq<C> + ToOwned<U>,
    C: ToOwned<U>
{
    #[inline]
    fn eq(&self, other: &Cow<'b, C, U>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<B: ?Sized + PartialOrd + ToOwned<U>, U> PartialOrd for Cow<'_, B, U> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<B: ?Sized + Eq + ToOwned<U>, U> Eq for Cow<'_, B, U> {}

impl<B: ?Sized + Ord + ToOwned<U>, U> Ord for Cow<'_, B, U> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<B: ?Sized, U> fmt::Debug for Cow<'_, B,  U> where
    B: fmt::Debug + ToOwned<U, Owned: fmt::Debug>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrowed(ref b) => fmt::Debug::fmt(b, f),
            Self::Owned(ref o) => fmt::Debug::fmt(o, f),
        }
    }
}

impl<B: ?Sized, U> fmt::Display for Cow<'_, B, U> where
    B: fmt::Display + ToOwned<U, Owned: fmt::Display>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrowed(ref b) => fmt::Display::fmt(b, f),
            Self::Owned(ref o) => fmt::Display::fmt(o, f),
        }
    }
}

impl<B: ?Sized, U> Default for Cow<'_, B, U> where
    B: ToOwned<U, Owned: Default>
{
    /// Creates an owned Cow<'a, B> with the default value for the contained owned value.
    fn default() -> Self {
        Self::Owned(<B as ToOwned<U>>::Owned::default())
    }
}

impl<B: ?Sized + Hash + ToOwned<U>, U> Hash for Cow<'_, B, U> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T: ?Sized + ToOwned<U>, U> AsRef<T> for Cow<'_, T, U> {
    fn as_ref(&self) -> &T {
        self
    }
}


// impl<'a> Add<&'a str> for Cow<'a, str> {
//     type Output = Cow<'a, str>;

//     fn add(self, rhs: &'a str) -> Self::Output {
//         self += rhs;
//         self
//     }
// }