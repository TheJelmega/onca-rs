#![allow(unused)]

///! Path manipulation.
/// 
/// This module provides two types, [`PathBuf`] and [`Path`] (akin to [`String`] and [`str`]), for working with paths abstractly.
/// Thses are thin wrappers around [`String`] and [`str`] respectively, meaning that unline [`std:path::PathBuf`] and [`std::path::Path`],
/// additional conversion might need to take place at the point of use on certain platforms, but the cost of thses conversions should in general not be an issue.

use std::{mem, collections::TryReserveError, str::FromStr, iter::{self, FusedIterator}, fmt, ops::Deref, borrow::Borrow, hash::Hash, cmp, io};

use onca_common::strings::StringExtensions;
use onca_common_macros::EnumDisplay;

use crate::{file, get_working_dir, Entry};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum State {
    Root, // `C:/`, `//server/share`, `mount:/`, or `/`
    Body, // foo/bar/baz
    Done
}

pub(crate) fn is_sep_byte(b: u8) -> bool {
    b == b'/' || b == b'\\'
}

fn is_sep(c: char) -> bool {
    c == '/' || c == '\\'
}

fn iter_after<'a, 'b, I, J>(mut iter: I, mut prefix: J) -> Option<I> where
    I: Iterator<Item = Component<'a>> + Clone,
    J: Iterator<Item = Component<'b>>
{
    loop {
        let mut iter_next = iter.clone();
        match (iter_next.next(), prefix.next()) {
            (Some(ref x), Some(ref y)) if x == y => (),
            (Some(_), Some(_))                   => return None,
            (Some(_), None)                      => return Some(iter),
            (None, None)                         => return Some(iter),
            (None, Some(_))                      => return None,
        }
        iter = iter_next;
    }
}

fn rsplit_file_at_dot(file: &str) -> (Option<&str>, Option<&str>) {
    if file == ".." {
        return (Some(file), None);
    }

    let mut iter = file.rsplitn(2, |c| c == '.');
    let after = iter.next();
    let before = iter.next();
    if before == Some("") {
        (Some(file), None)
    } else {
        (before, after)
    }
}

fn split_file_at_dot(file: &str) -> (&str, Option<&str>) {
    if file == ".." {
        return (file, None);
    }

    let i = match file[1..].chars().position(|c| c == '.') {
        Some(i) => i + 1,
        None    => return (file, None),
    };
    let before = &file[..i];
    let after = &file[(i + 1)..];
    (before, Some(after))
}

//--------------------------------------------------------------

/// An error returned from [`Path::strip_prefix`] if the prefix was not found.
/// 
/// This is created by the [`Path::strip_prefix`] method on [`Path`].
/// See its documentation for more info.
/// 
/// [`Path::strip_prefix`]: Path::strip_prefix
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StripPrefixError(());

/// An error returned when a path is invalid.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InvalidPathError((&'static str, usize));

//--------------------------------------------------------------

/// File path root.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Root<'a> {
    /// DOS-like drive root, e.g. `C:/`
    Drive(u8),
    /// Universal Naming convention (windows network drive) root, e.g. `//server/share` 
    WinShare { server: &'a str, share: &'a str },
    /// Common root across filesystems, e.g. `/`
    Common,
    /// Virtual file system root, e.g. `loc:/`
    VFS(&'a str)
}

impl<'a> Root<'a> {
    /// Get the length of the root (in bytes)
    #[inline]
    pub fn len(&self) -> usize {
        match *self {
            Root::Drive(_)              => 2,
            Root::WinShare { server, share } => 4 + server.len() + share.len(),
            Root::Common                => 1,
            Root::VFS(name)             => 2 + name.len(),
        }
    }
}

//--------------------------------------------------------------

/// A structure wrapping a root as well as its unparsed string representation.
/// 
/// In aditon to the parsed [`Root`] information returned by [`kind`], the `RootComponent` also holds the raw and unparsed [`str`] slice, returned by [`as_str`]
/// 
/// Instances of this `struct` can be obtained by matching against teh [`Root` variant] on [`Component`] 
/// 
/// [`as_str`]: RootComponent::as_str
/// [`Root` variant]: Component::Root
#[derive(Clone, Copy, Eq, Debug)]
pub struct RootComponent<'a> {
    /// The prefix as an unparsed `str` slice
    raw: &'a str,
    /// The parsed prefix data
    parsed: Root<'a>,
}

impl<'a> RootComponent<'a> {
    /// Returns the parsed root data.
    /// 
    /// See [`Root`]'s documentation for more information on the different kinds of roots.
    #[must_use]
    pub fn kind(&self) -> Root<'a> {
        self.parsed
    }

    /// Return the [`str`] slice for this root.
    #[must_use]
    pub fn as_str(&self) -> &'a str {
        self.raw
    }
}

impl<'a> PartialEq for RootComponent<'a> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.parsed, &other.parsed)
    }
}

impl<'a> PartialOrd for RootComponent<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&self.parsed, &other.parsed)
    }
}

impl<'a> Ord for RootComponent<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&self.parsed, &other.parsed)
    }
}

impl Hash for RootComponent<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.parsed.hash(state);
    }
}

//--------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct VFSMacro<'a> {
    name: &'a str,
    raw:  &'a str,
}

impl<'a> VFSMacro<'a> {
    /// Get the length of the macro (in bytes)
    #[must_use]
    pub fn len(&self) -> usize {
        2 + self.name.len()
    }

    /// Get the string representing the macro
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.raw
    }

    /// Get the name of the macro
    pub fn name(&self) -> &str {
        self.name
    }
}

impl AsRef<str> for VFSMacro<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for VFSMacro<'_> {
    fn as_ref(&self) -> &Path {
        unsafe { Path::new_unchecked(self.as_str()) }
    }
}

//------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum VFSPart<'a> {
    Text(&'a str),
    Macro(VFSMacro<'a>),
}

impl<'a> VFSPart<'a> {
    /// Get the length of the part (in bytes)
    pub fn len(&self) -> usize {
        match self {
            VFSPart::Text(s)  => s.len(),
            VFSPart::Macro(m) => m.len(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            VFSPart::Text(s) => s,
            VFSPart::Macro(m) => m.as_str(),
        }
    }
}

impl AsRef<str> for VFSPart<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for VFSPart<'_> {
    fn as_ref(&self) -> &Path {
        unsafe { Path::new_unchecked(self.as_str()) }
    }
}

pub struct VFSParts<'a> {
    path: &'a str,
}

//------------------------------

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct VFSComponent<'a>(&'a str);

impl<'a> VFSComponent<'a> {
    /// Get the length of the part (in bytes)
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

impl AsRef<str> for VFSComponent<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for VFSComponent<'_> {
    fn as_ref(&self) -> &Path {
        unsafe { Path::new_unchecked(self.as_str()) }
    }
}

impl<'a> Iterator for VFSComponent<'a> {
    type Item = VFSPart<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len() == 0 {
            None
        } else if self.0.starts_with('<') { // macro
            // Valid path should never have a macro that doesn't end on `>`
            let end = self.0.as_bytes().iter().position(|&b| b == b'>').unwrap();
            let full_macro = &self.0[..end + 1];
            let macro_name = &self.0[1..end];
            if end == self.0.len() {
                self.0 = "";
            } else {
                self.0 = &self.0[end + 1..];
            }
            Some(VFSPart::Macro(VFSMacro { name: macro_name, raw: full_macro }))
        } else { // text
            let end = self.0.as_bytes().iter().position(|&b| b == b'<').unwrap_or(self.0.len());
            let text = &self.0[..end];
            if end == self.0.len() {
                self.0 = "";
            } else {
                self.0 = &self.0[end..];
            }
            Some(VFSPart::Text(text))
        }
    }
}

impl<'a> DoubleEndedIterator for VFSComponent<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.0.len() == 0 {
            None
        } else if self.0.ends_with('>') { // macro
            let end = self.0.as_bytes().iter().rposition(|&b| b == b'<')?;
            let full_macro = &self.0[end..];
            let macro_name = &self.0[end + 1..self.0.len() - 1];
            self.0 = &self.0[..end];
            Some(VFSPart::Macro(VFSMacro { name: macro_name, raw: full_macro }))
        } else { // text
            let end = self.0.as_bytes().iter().rposition(|&b| b == b'>').unwrap_or_default();
            let text = &self.0[end + 1..];
            self.0 = &self.0[..end];
            Some(VFSPart::Text(text))
        }
    }
}

impl FusedIterator for VFSComponent<'_> {}

//--------------------------------------------------------------

/// A single component of a path.
/// 
/// A `Component` roughly corresponds to a substring between path separators, e.g. `/`.
/// 
/// This `enum` is created by iterating over [`Components`], which in turn is crated by the [`components`] method on [`Path`]
/// 
/// [`components`]: Path::components
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Component<'a> {
    /// A root component
    Root(RootComponent<'a>),
    /// A reference to the parent directory, i.e. `.`.
    CurDir,
    /// A reference to the parent directory, i.e. `..`.
    ParentDir,
    /// A normal component, e.g. `a` and `b` in `a/b`.
    /// 
    /// This variant is the most common one, as it represetns references to files or directories.
    Normal(&'a str),
    /// A virtual file system path component.
    VFS(VFSComponent<'a>),
    /// Component with a search wildcard
    Wildcard(&'a str)
}

impl <'a> Component<'a> {
    /// Get the underlying raw [`str`] slice.
    pub fn as_str(self) -> &'a str {
        match self {
            Component::Root(root)  => root.as_str(),
            Component::CurDir      => ".",
            Component::ParentDir   => "..",
            Component::Normal(s)   => s,
            Component::VFS(vfs)    => vfs.as_str(),
            Component::Wildcard(s) => s,
        }
    }
}

impl AsRef<str> for Component<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for Component<'_> {
    fn as_ref(&self) -> &Path {
        unsafe { Path::new_unchecked(self.as_str()) }
    }
}

//--------------------------------------------------------------

/// An interator over the [`Component`]s of a [`Path`].
/// 
/// This is creted by the [`Path::components`].
/// See its documentation for more info.
/// 
/// [`Path::components`]: Path::components
#[derive(Clone)]
pub struct Components<'a> {
    path:  &'a str,
    root:  Option<Root<'a>>,
    front: State,
    back:  State,
}

impl fmt::Debug for Components<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a>(&'a Path);

        impl fmt::Debug for DebugHelper<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.components()).finish()
            }
        }
        
        f.debug_tuple("Components").field(&DebugHelper(self.as_path())).finish()
    }
}

impl<'a> Components<'a> {
    // How long is the root, if any?
    #[inline]
    fn root_len(&self) -> usize {
        self.root.as_ref().map(Root::len).unwrap_or(0)
    }

    /// How much of the root is left from the point of view of iteration?
    #[inline]
    fn root_remaining(&self) -> usize {
        if self.front == State::Root { self.root_len() } else { 0 }
    }

    /// Given the iteration so far, how muc hof the pre-State::Body path is left
    #[inline]
    fn len_before_body(&self) -> usize {
        let cur_dir = if self.front <= State::Root && self.include_cur_dir() { 1 } else { 0 };
        self.root_remaining() + cur_dir
    }

    /// Is the iteration complete?
    fn is_finished(&self) -> bool {
        self.front == State::Done || self.back == State::Done || self.front > self.back
    }

    fn is_sep_byte(&self, b: u8) -> bool {
        is_sep_byte(b)
    }

    /// Extract a slice corresponding to the portion of the path remaining for iteration
    #[must_use]
    pub fn as_path(&self) -> &'a Path {
        let mut comps = self.clone();
        if comps.front == State::Body {
            comps.trim_left();
        }
        if comps.back == State::Body {
            comps.trim_right();
        }
        unsafe { Path::new_unchecked(comps.path) }
    }

    // Is teh *original* path rooted?
    pub fn has_root(&self) -> bool {
        self.root.is_some()
    }

    /// Should the normalized path include the leading `.`?
    fn include_cur_dir(&self) -> bool {
        if self.has_root() {
            return false;
        }
        let mut iter = self.path[self.root_remaining()..].as_bytes().iter();
        match (iter.next(), iter.next()) {
            (Some(b'.'), None) => true,
            (Some(b'.'), Some(&b)) => self.is_sep_byte(b),
            _ => false
        }
    }

    fn parse_single_component<'b>(&self, comp: &'b str) -> Option<Component<'b>> {
        match comp {
            "."                         => None,
            ".."                        => Some(Component::ParentDir),
            ""                          => None,
            c if c.contains('<')        => Some(Component::VFS(VFSComponent(comp))),
            c if c.contains(['*', '?']) => Some(Component::Wildcard(comp)),
            _                           => Some(Component::Normal(comp))
        }
    }

    fn parse_next_component(&self) -> (usize, Option<Component<'a>>) {
        debug_assert!(self.front == State::Body);
        let (extra, comp) = match self.path.bytes().position(|b| self.is_sep_byte(b)) {
            Some(i) => (1, &self.path[..i]),
            None    => (0, self.path),
        };
        (comp.len() + extra, self.parse_single_component(comp))
    }

    fn parse_next_component_back(&self) -> (usize, Option<Component<'a>>) {
        debug_assert!(self.back == State::Body);
        let start = self.len_before_body();
        let (extra, comp) = match self.path[start..].bytes().rposition(|b| self.is_sep_byte(b)) {
            Some(i) => (1, &self.path[start + i + 1..]),
            None    => (0, &self.path[start..]),
        };
        (comp.len() + extra, self.parse_single_component(comp))
    }

    /// Trim away repeated separators (i.e. empty components) on the left
    fn trim_left(&mut self) {
        while !self.path.is_empty() {
            let (size, comp) = self.parse_next_component();
            if comp.is_some() {
                return;
            } else {
                self.path = &self.path[size..]
            }
        }
    }

    /// Trim away repeated separators (i.e. empty components) on the left
    fn trim_right(&mut self) {
        while self.path.len() > self.len_before_body() {
            let (size, comp) = self.parse_next_component_back();
            if comp.is_some() {
                return;
            } else {
                self.path = &self.path[size..]
            }
        }
    }
}

impl AsRef<Path> for Components<'_> {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<str> for Components<'_> {
    fn as_ref(&self) -> &str {
        self.as_path().as_str()
    }
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.is_finished() {
            match self.front {
                State::Root if self.root_len() > 0 => {
                    self.front = State::Body;
                    debug_assert!(self.root_len() <= self.path.len());
                    let raw = &self.path[..self.root_len()];
                    self.path = &self.path[self.root_len()..];
                    return Some(Component::Root(RootComponent { raw, parsed: self.root.unwrap() }));
                },
                State::Root => {
                    self.front = State::Body;
                    if self.include_cur_dir() {
                        debug_assert!(!self.path.is_empty());
                        self.path = &self.path[1..];
                        return Some(Component::CurDir);
                    }
                },
                State::Body if !self.path.is_empty() => {
                    let (size, comp) = self.parse_next_component();
                    self.path = &self.path[size..];
                    if comp.is_some() {
                        return comp;
                    }
                },
                State::Body => self.front = State::Done,
                State::Done => unreachable!()
            }
        }
        None
    }
}

impl<'a> DoubleEndedIterator for Components<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while !self.is_finished() {
            match self.back {
                State::Body if self.path.len() > self.len_before_body() => {
                    let (size, comp) = self.parse_next_component_back();
                    self.path = &self.path[..(self.path.len() - size)];
                    if comp.is_some() {
                        return comp;
                    }
                },
                State::Body => {
                    self.back = State::Root;
                    if self.include_cur_dir() {
                        self.path = &self.path[..(self.path.len() - 1)];
                        return Some(Component::CurDir);
                    }
                },
                State::Root if self.root_len() > 0 => {
                    self.back = State::Done;
                    return Some(Component::Root(RootComponent { raw: self.path, parsed: self.root.unwrap() }))
                },
                State::Root => self.back = State::Done,
                State::Done => unreachable!(),
            }
        }
        None
    }
}

impl FusedIterator for Components<'_> {}

impl<'a> PartialEq for Components<'a> {
    fn eq(&self, other: &Self) -> bool {
        // Faast path for exact matches, e.g. hash map lookups.
        // Don't explicitly compare the root
        if self.path.len() == other.path.len() &&
            self.front == other.front &&
            self.back == State::Body &&
            other.back == State::Body
        {
            // PERF: possible future improvement: this could bail out earlier if there werer a reverse memcmp back to front
            if self.path == other.path {
                return true;
            }
        }

        // Compare back to from, since absloute paths often share long prefixes
        Iterator::eq(self.clone().rev(), other.clone().rev())
    }
}

impl Eq for Components<'_> {}

impl<'a> PartialOrd for Components<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(compare_components(self.clone(), other.clone()))
    }
}

impl<'a> Ord for Components<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        compare_components(self.clone(), other.clone())
    }
}

fn compare_components(mut left: Components<'_>, mut right: Components<'_>) -> cmp::Ordering {
    // Fast path for long shared prefixes
    //
    // - compare raw chars to find first mismatch
    // - backtrack to find separator before mismatch to avoid ambiguous parings
    // - if found, update state to only do a component wise comparison on the remainder, otherwise do it on the full path
    //
    // The fast path isn't atake for path with a RootComponent, to avoid backtracking into the middle of one
    if left.root.is_none() && right.root.is_none() && left.front == right.front {
        // PERF: possible future improvement: SIMD comparison
        let first_difference = match left.path.chars().zip(right.path.chars()).position(|(a, b)| a != b) {
            Some(diff)                                  => diff,
            None if left.path.len() == right.path.len() => return cmp::Ordering::Equal,
            None                                        => left.path.len().min(right.path.len())
        };

        if let Some(previous_sep) = left.path[..first_difference].bytes().rposition(|c| (c as char).is_ascii() && left.is_sep_byte(c)) {
            let mismatched_component = previous_sep + 1;
            left.path = &left.path[mismatched_component..];
            left.front = State::Body;
            right.path = &right.path[mismatched_component..];
            right.front = State::Body;
        }
    }

    Iterator::cmp(left, right)
}

//------------------------------

/// An iterator over the [`Component`]s of a [`Path`], as [`str`] slices.
/// 
/// This is created by the [`Path::iter`].
/// See its documentation for more info.
/// 
/// [`Path::iter`]: Path::iter
pub struct Iter<'a>(Components<'a>);

impl<'a> Iter<'a> {
    /// Extracts the slice corresponding to the portion of the path remaining for iteration.
    #[must_use]
    #[inline]
    pub fn as_path(&self) -> &'a Path {
        self.0.as_path()
    }
}

impl AsRef<Path> for Iter<'_> {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<str> for Iter<'_> {
    fn as_ref(&self) -> &str {
        self.as_path().as_str()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Component::as_str)
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(Component::as_str)
    }
}

impl FusedIterator for Iter<'_> {}

//--------------------------------------------------------------

/// An iterator over [`Path`] and it's ancestors.
/// 
/// This is cratred by [`Path::ancestors`].
/// 
/// [`Path::ancestors`]: Path::ancestors
#[derive(Clone, Copy, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Ancestors<'a> {
    next: Option<&'a Path>
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = &'a Path;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;
        self.next = next.and_then(Path::parent);
        next
    }
}

impl FusedIterator for Ancestors<'_> {}

//--------------------------------------------------------------


/// An owned, mutable path (akin to [`String`]).
/// 
/// Provides methods like [`push`] and [`set_extension`] tht can mutate the path in-place.
/// Also implements [`Deref`] to [`Path`], meaning that all methods on [`Path`] slices are available on `PathBuf` as well.
/// 
/// A `PathBuf` is always null-terminated.
/// 
/// [`push`]: PathBuf::push
/// [`set_extension`]: PathBuf::set_extension
#[derive(Clone)]
pub struct PathBuf(String);

impl PathBuf {
    /// Creates an empty `PathBuf`
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(String::new())
    }

    /// Create a new `PathBuf` with at least the specified capacity.
    #[must_use]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }

    /// Create a `PathBuf` from a [`str`] slice.
    #[must_use]
    pub fn from_str(s: &str) -> Result<Self, InvalidPathError> {
        let path = Path::new(s)?;
        Ok(Self::from_path(path))
    }

    /// Create a `PathBuf` from a [`Path`] slice.
    #[must_use]
    #[inline]
    pub fn from_path(p: &Path) -> Self {
        let mut string = String::from(p.as_str());
        string.null_terminate();
        Self(string)
    }

    /// Create a `PathBuf` from a utf-8 encoded [`u8`] slice, including invalid characters.
    #[must_use]
    pub fn from_utf8_lossy(s: &[u8]) -> Result<Self, InvalidPathError> {
        let mut string = String::from_utf8_lossy(s).into_owned();

        // Validate path
        Path::new(&string)?;

        string.null_terminate();
        Ok(Self(string))
    }

    /// Coerces to a [`Path`] slice
    #[must_use]
    #[inline]
    pub fn as_path(&self) -> &Path {
        unsafe { Path::new_unchecked(&self.0) }
    }

    /// Get a pointer to the internal string.
    #[must_use]
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// Extends `self` with `path`.
    /// 
    /// If `path` is absolute, it replaces the current path
    /// 
    /// Any reference to `.` will be removed
    pub fn push<P:AsRef<Path>>(&mut self, path: P) {
        self._push(path.as_ref())
    }

    pub fn _push(&mut self, path: &Path) {
        // In general, a separator is needed if the rightmost byte is not a separator
        let mut need_sep = self.as_mut_vec().last().map(|&c| !is_sep_byte(c)).unwrap_or(false);

        // In the special case of a root, do not add a sparator
        let comps = self.components();

        if comps.root_len() > 0 &&
           comps.root_len() == comps.path.len()
        {
            need_sep = false;
        }

        // absolute `path` replaces `self`
        if path.is_absolute() || path.has_root() {
            self.as_mut_vec().truncate(0);
        } else if need_sep {
            self.0.push('/')
        }

        self.0.push_str(path.as_str());
    }

    /// Truncates `self` to [`Path::parent`].
    /// 
    /// [`Path::parent`]: Path::parent
    pub fn pop(&mut self) -> bool {
        match self.parent().map(|p| p.as_str().len()) {
            Some(len) => {
                self.as_mut_vec().truncate(len);
                true
            },
            None => false,
        }
    }

    /// Changes [`self.file_name`] to `file_name`.
    /// 
    /// If [self.file_name`] is [`None`], this is equivalent to pushing `file_name`.
    /// 
    /// Otherwise it is equivalent to calling [`PathBuf::pop`] and then pushing `file_name`.
    /// The new path will be a sibling of hte original path (that is, it will have the same parent).
    /// 
    /// [`self.file_name`]: Path::file_name
    /// [`PathBuf::pop`]: PathBuf::pop
    pub fn set_file_name<S: AsRef<str>>(&mut self, file_name: S) {
        self._set_file_name(file_name.as_ref())
    }

    pub fn _set_file_name(&mut self, file_name: &str) {
        if self.file_name().is_some() {
            let popped = self.pop();
            debug_assert!(popped);
        }
        self.push(unsafe { Path::new_unchecked(file_name) });
    }

    /// Update [`self.extension`] to `extension`.
    /// 
    /// Returns `false` and does nothing if [`self.file_name`] is [`None`], returns `true` and updates the extension otherwise.
    /// 
    /// If [`self.extension`] is [`None`], the extension is added, otherwise it is replaced.
    /// 
    /// [`self.extension`]: Path::extension
    pub fn set_extension<S: AsRef<str>>(&mut self, extension: S) -> bool {
        self._set_extension(extension.as_ref())
    }

    pub fn _set_extension(&mut self, extension: &str) -> bool {
        let file_stem = match self.file_stem() {
            None    => return false,
            Some(f) => f.as_bytes(),
        };

        // Truncate until right after the file stem
        let end_file_stem = file_stem[file_stem.len()..].as_ptr();
        let start = self.0.as_bytes().as_ptr();
        let arr = self.as_mut_vec();
        arr.truncate(unsafe { end_file_stem.offset_from(start) as usize });

        // Add the new extension, if any
        let new = extension.as_bytes();
        if !new.is_empty() {
            arr.reserve_exact(new.len() + 1);
            arr.push(b'.');
            arr.extend_from_slice(new);
        }
        true
    }

    /// Normalize the path (in this case, a fully qualified path will be returned).
    /// 
    /// This cannot handle paths that are handled by the VFS
    /// 
    /// # Error
    /// 
    /// Returns an error if the path could not be normalized
    pub fn normalize(&mut self) -> io::Result<()> {
        use fmt::Write;

        let mut new_path = String::with_capacity(self.capacity());
 
        let mut components = self.components();
        if components.has_root() {
            match components.root.unwrap() {
                Root::Drive(drive) => if cfg!(windows) {
                    write!(&mut new_path, "{}:/", drive as char);
                } else {
                    return Err(io::Error::other("Cannot normalize a windows file path on a non-windows OS"));
                },
                Root::WinShare { server, share } => if cfg!(windows) {
                    write!(&mut new_path, "//{server}/{share}/");
                } else {
                    return Err(io::Error::other("Cannot normalize a windows file path on a non-windows OS"));
                },
                Root::Common => if cfg!(windows) {
                    return Err(io::Error::other("Cannot normalize a non-windows file path on windows"));
                } else {
                    new_path.push('/');
                },
                Root::VFS(_) => return Err(io::Error::other("Cannot normalize a virtual file system path without the virtual file system")),
            }

            // Pop root from the components.
            components.next().unwrap();
        } else {
            let cur_dir = get_working_dir()?;
            new_path.push_str(cur_dir.as_str());
            new_path.push('/')
        }

        let mut new_path = PathBuf(new_path);
        for comp in components {
            match comp {
                // Root is handled above
                Component::Root(root) => unreachable!(),
                Component::CurDir => (),
                Component::ParentDir => _ = new_path.pop(),
                Component::Normal(p) => new_path.push(unsafe { Path::new_unchecked(p) }),
                Component::VFS(_) => return Err(io::Error::other("Cannot normalize a path that includes a macro without the virtual file system")),
                Component::Wildcard(_) => return Err(io::Error::other("Cannot normalize a path that includes a wildcard")),
            }
        }
        *self = new_path;

        Ok(())
    }

    pub(crate) fn normalize_internal(&mut self, allow_vfs: bool) -> io::Result<()> {
        use fmt::Write;

        let mut new_path = String::with_capacity(self.capacity());
 
        let mut components = self.components();
        if components.has_root() {
            match components.root.unwrap() {
                Root::Drive(drive) => if cfg!(windows) {
                    write!(&mut new_path, "{}:/", drive as char);
                } else {
                    return Err(io::Error::other("Cannot normalize a windows file path on a non-windows OS"));
                },
                Root::WinShare { server, share } => if cfg!(windows) {
                    write!(&mut new_path, "//{server}/{share}/");
                } else {
                    return Err(io::Error::other("Cannot normalize a windows file path on a non-windows OS"));
                },
                Root::Common => if cfg!(windows) {
                    return Err(io::Error::other("Cannot normalize a non-windows file path on windows"));
                } else {
                    new_path.push('/');
                },
                Root::VFS(vfs) => if (allow_vfs) {
                    write!(&mut new_path, "{vfs}:/");
                } else {
                    return Err(io::Error::other("Cannot normalize a virtual file system path without the virtual file system"));
                },
            }

            // Pop root from the components.
            components.next().unwrap();
        } else {
            let cur_dir = get_working_dir()?;
            new_path.push_str(cur_dir.as_str());
            new_path.push('/')
        }

        let mut new_path = PathBuf(new_path);
        for comp in components {
            match comp {
                // Root is handled above
                Component::Root(root) => unreachable!(),
                Component::CurDir => (),
                Component::ParentDir => _ = new_path.pop(),
                Component::Normal(p) => new_path.push(unsafe { Path::new_unchecked(p) }),
                Component::VFS(_) => return Err(io::Error::other("Cannot normalize a path that includes a macro without the virtual file system")),
                Component::Wildcard(_) => return Err(io::Error::other("Cannot normalize a path that includes a wildcard")),
            }
        }
        *self = new_path;

        Ok(())
    }

    /// Consumed the `PathBuf`, yielding the internal [`String`] storage.
    #[must_use = "`self` will be dropped if the result it not used"]
    pub fn into_string(self) -> String {
        self.0
    }

    /// Returns the `PathBuf`'s capacity, in bytes
    #[must_use]
    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Truncates this `PathBuf`, removing all contents
    /// 
    /// While this means the `PathBuf` will have a length of zero, if does not touch its capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Reserves capacity for at least `additional` bytes more than the current length. The allocator may reserve more space to speculatively avoid frequent alloctions.
    /// After calling `reserve`, capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is lready sufficient.
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity overflows `usize`.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Tries to reserve capacity for at least `additional` byes more than the current length.
    /// The allocator may reserve more space to speculatively avoid frequent allocations.
    /// After calling `try_reserve`, capacity will be greater than or equal to `self.len() + additional` if it return `Ok(())`.
    /// Does nothing if capacity is already sufficient.
    /// This method preserves the contents even if an error occurs.
    /// 
    /// # Error
    /// 
    /// If the capacity overflows, or the allocator reports a failure, then an error is returned
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve(additional)
    }

    /// Reserves the minimum capacity for at least `additional` bytes more than the current length.
    /// Unlike [`reserve`], this will not deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `reserve_exact`, capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if the capacity is already sufficient.
    /// 
    /// # Panics
    /// 
    /// Panics if the new capacity overflows `usize`.
    /// 
    /// [`reserve`]: PathBuf::reserve
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    
    /// Tries to reserve the minimum capacity for at least `additional` bytes more than the current length.
    /// Unlike [`try_reserve`], this will not deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `try_reserve_exact`, capacity will be greater than or equal to `self.len() + additional` if it return `Ok(())`.
    /// Does nothing if capacity is already sufficient.
    /// This method preserves the contents even if an error occurs.
    /// 
    /// Note that the allocator may give the allocation more space than it requests.
    /// Therefore, capacity can not be relied upon to be precisely minimal.
    /// Prefer `try_reserve` if future insertions are expected
    /// 
    /// # Error
    /// 
    /// If the capacity overflows, or the allocator reports a failure, then an error is returned
    /// 
    /// [`try_reserve`]: PathBuf::try_reserve
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve_exact(additional)
    }

    /// Shrinks the capacity of this `PathBuf` to match its lenght.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Shrinks the capacity of this `PathBuf` with a lower bound.
    /// 
    /// The capacity will remain at least as large as both the lenght than the supplied value.
    /// 
    /// If the current capacity is less than the lower limit, this is a no-op
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity)
    }

    #[inline]
    fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        unsafe { self.0.as_mut_vec() }
    }
}

/*
impl<T: ?Sized + AsRef<str>> From<&T> for PathBuf {
    fn from(s: &T) -> Self {
        PathBuf::from_str(s.as_ref())
    }
}
*/

impl From<String> for PathBuf {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<PathBuf> for String {
    fn from(value: PathBuf) -> Self {
        value.0
    }
}

impl FromStr for PathBuf {
    type Err = InvalidPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PathBuf::from_str(s)
    }
}

impl<P: AsRef<Path>> iter::FromIterator<P> for PathBuf {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        let mut buf = PathBuf::new();
        buf.extend(iter);
        buf
    }
}

impl<P: AsRef<Path>> iter::Extend<P> for PathBuf {
    fn extend<T: IntoIterator<Item = P>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |p| self.push(p))
    }
}

impl fmt::Debug for PathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl Deref for PathBuf {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        unsafe { Path::new_unchecked(&self.0) }
    }
}

impl Borrow<Path> for PathBuf {
    fn borrow(&self) -> &Path {
        self.deref()
    }
}

impl Default for PathBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for PathBuf {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        unsafe { Path::new_unchecked(self) }
    }
}

impl AsRef<str> for PathBuf {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'a> IntoIterator for &'a PathBuf {
    type Item = &'a str;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Hash for PathBuf {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_path().hash(state);
    }
}

//--------------------------------------------------------------

/// Form of the path, i.e. what state the path is it
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum PathForm {
    /// The path is a raw path, meaning that the path:
    /// - can be absolute or relative.
    /// - may have a vfs root.
    /// - may contain references to a parent dir (or current dir)
    /// - may contain vfs macros
    Raw,
    /// The path is unnormalized, meaning that the path:
    /// - can be absolute or relative.
    /// - may have a vfs root.
    /// - may contain references to a parent dir.
    /// - does not contain VFS macros.
    Unnormalized,
    /// The path is normalized, meaning that the path:
    /// - is absolute
    /// - may have a vfs root.
    /// - does not contain any references to a current or parent dir.
    /// - does not contain any VFS macros.
    Normalized,
    /// The path if fully qualified, meaning that if the path to the directory or file was requested, it would be the path that would be returned.
    /// 
    /// In other words, it means that the path:
    /// - is absolute.
    /// - does not have a vfs root.
    /// - does not contain any references to a current or parent dir.
    /// - does not contain any VFS macros.
    FullyQualified,
}

/// A slice of a path (akin to [`str`]).
/// 
/// This type supports a number of operations for inspecting a path, including breaking the path into its components (separated by '/').
/// 
/// This is a n *unsized* type, meaning that it must always be used behind a pointer or reference.
/// For an owned version of this type, see [`PathBuf`]
#[derive(Eq)]
pub struct Path(str);

impl Path {
    /// Create a path from a string slice.
    /// 
    /// No validation happens at this point, so this means that the path may contain invalid characters or unclosed macros.
    #[must_use]
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> Result<&Self, InvalidPathError> {
        let path: &Path = unsafe { mem::transmute(s.as_ref()) };
        path.validate()?;
        Ok(path)
    }

    fn validate(&self) -> Result<(), InvalidPathError> {
        if self.0.is_empty() {
            return Err(InvalidPathError(("A path slice cannot be empty.", 0)))
        }

        // Validate macro begin/ends + invalid characters
        let bytes = self.0.as_bytes();
        let mut macro_iter = bytes.iter();
        let mut in_macro = false;
        let mut pre_first_slash = true;
        for (idx, c) in self.0.bytes().enumerate() {
            match c {
                b'<' => if in_macro {
                    return Err(InvalidPathError(("Invalid macro start", idx)));
                } else {
                    in_macro = true;
                },
                b'>' => if in_macro {
                    in_macro = false;
                } else {
                    return Err(InvalidPathError(("Invalid macro start", idx)));
                },
                b':' => if !pre_first_slash {
                    return Err(InvalidPathError(("Path contains invalid `:`", idx)));
                },
                b'/' | b'\\' => pre_first_slash = false,
                b'"' | b'|' => return Err(InvalidPathError(("Path contains invalid characters, any of the following: :\"|", idx))),
                _ => {},
            }
        }
        if in_macro {
            return Err(InvalidPathError(("Macro has not been closed", self.0.len())));
        }
        Ok(())
    }

    /// Directly wraps a string slice as a `Path` slice, without checking for invalid character/macros.
    #[must_use]
    #[inline(always)]
    pub unsafe fn new_unchecked<S: AsRef<str> + ?Sized>(s: &S) -> &Self {
        mem::transmute(s.as_ref())
    }

    /// Yields the underlying [`str`] slice.
    #[must_use]
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the lenght of the path
    #[must_use]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Converts a `Path` into an owned [`PathBuf`].
    #[must_use]
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from_path(self)
    }

    /// Check if the `Path` is absolute, i.e. if it is independant from the current directory.
    /// 
    /// * On windows, a path is absolute when it has a root, with the exception of `/temp`, which is relative to the current drive.
    #[must_use]
    pub fn is_absolute(&self) -> bool {
        let mut components = self.components();
        components.has_root() && (!cfg!(windows) || components.next().map_or(false, |val| match val {
            Component::Root(root) => !matches!(root.kind(), Root::Common),
            _ => false,
        }))
    }

    /// Check if the path is relative, i.e. not absolute.
    #[must_use]
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    /// Check if the `Path` has a root.
    #[must_use]
    pub fn has_root(&self) -> bool {
        self.components().has_root()
    }

    /// Returns the `Path` without its final component, if there is one.
    /// 
    /// Returns [`None`] if the path terminates in a root.
    #[must_use]
    pub fn parent(&self) -> Option<&Path> {
        let mut comps = self.components();
        let comp = comps.next_back();
        comp.and_then(|p| match p {
            Component::Normal(_) | Component::CurDir | Component::ParentDir => {
                Some(comps.as_path())
            },
            _ => None,
        }).filter(|p| !p.0.is_empty())
    }

    /// Produces an iterator over `Path` and its ancestors.
    /// 
    /// The iterator will yield the `Path` that is returned if the ['parent`] method is used zero or more times.
    /// That means, the iterator will yield `&self`, `&self.parent().unwrap()`, `&self.parent().unwrap().parent().unwrap()`, and so on.
    /// If the [`parent`] method returns [`None`], the iterator will do likewise.
    /// The iterator will always at least on value, namely `&self`.
    /// 
    /// [`parent`]: Self::parent
    #[must_use]
    pub fn ancestors(&self) -> Ancestors<'_> {
        Ancestors { next: Some(&self) }
    }

    /// Returns the final component of the [`Path`], if there is one.
    /// 
    /// If the path is a normal file, this is the file name.
    /// If it's the path of directory, this is the directory name.
    /// 
    /// Returns [`None`] if the path terminates in `..`.
    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.components().next_back().and_then(|p| match p {
            Component::Normal(p) => Some(p),
            Component::VFS(vfs)  => Some(vfs.as_str()),
            _                    => None,
        })
    }

    /// Returns a path, that when joint onto `base`, yields `self`.
    /// 
    /// # Errors
    /// 
    /// If `base` is not a prefix of `self`, i.e. [`starts_with`] returns false, retuns [`Err`].
    #[must_use]
    pub fn strip_prefix<P: AsRef<Path>>(&self, base: P) -> Result<&Path, StripPrefixError> {
        self._strip_prefix(base.as_ref())
    }

    fn _strip_prefix(&self, base: &Path) -> Result<&Path, StripPrefixError> {
        iter_after(self.components(), base.components())
            .map(|c| c.as_path())
            .ok_or(StripPrefixError(()))
    }

    /// Determines whether `base` is a prefix of `self`.
    /// 
    /// Only considers whole path component to match.
    #[must_use]
    pub fn starts_with<P: AsRef<Path>>(&self, base: P) -> bool {
        self._starts_with(base.as_ref())
    }

    fn _starts_with(&self, base: &Path) -> bool {
        iter_after(self.components(), base.components()).is_some()
    }

    /// Determines whether `child` is a suffix of self.
    /// 
    /// Only considers whole path components to match.
    #[must_use]
    pub fn ends_with<P: AsRef<Path>>(&self, child: P) -> bool {
        self._ends_with(child.as_ref())
    }

    fn _ends_with(&self, child: &Path) -> bool {
        iter_after(self.components().rev(), child.components().rev()).is_some()
    }

    /// Extract teh stem (non-extension) potion of [`Path::filename`].
    /// 
    /// The stem is:
    /// - [`None`] if there is not a filename,
    /// - The entire filename if there is no embedded '.',
    /// - The entire filename if the file name begins with `.` and has no other '.'s wihitn.
    /// - Ohterwise, the poriton of the file name before the final '.'
    /// 
    /// [`Path::filename`]: Path::filename
    #[must_use]
    pub fn file_stem(&self) -> Option<&str> {
        self.file_name().map(rsplit_file_at_dot).and_then(|(before, after)| before.or(after))
    }

    /// Extracts the prefix of [`Path::file_prefix`].
    /// 
    /// The prefix is:
    /// - [`None`] if there is no filename,
    /// - The entier file name if there is no embedded `.`,
    /// - The portion of the filename before the first non-beginning `.`,
    /// - The entire filename if the file name begins with `.` and has no other `.`s within,
    /// - The poriton of the file name before the second `.` if ther filename begins with a `.`.
    /// 
    /// [`Path::file_prefix`]: Path::file_prefix
    #[must_use]
    pub fn file_prefix(&self) -> Option<&str> {
        self.file_name().map(split_file_at_dot).and_then(|(before, after)| Some(before))
    }

    /// Extract teh extension of [`Path::file_name`]
    /// 
    /// The extension is:
    /// - [`None`] if there is no filename,
    /// - [`None`] if there is no embedded `.`,
    /// - [`None`] if the file name begins a with a `.` and has no other `.`,
    /// - Otherwise, the prtion fo the file name after the final `.`.
    /// 
    /// [`Path::file_name`]: Path::file_name
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.file_name().map(rsplit_file_at_dot).and_then(|(before, after)| before.and(after))
    }

    /// Is an absolute path referencing a windows path?
    pub fn is_windows_path(&self) -> bool {
        if let Some(Component::Root(root)) = self.components().next() {
            matches!(root.kind(), Root::Drive(_) | Root::WinShare { .. })
        } else {
            false
        }
    }

    /// Does the path contain any VFS macros?
    pub fn has_macros(&self) -> bool {
        self.0.find('<').is_some()
    }

    /// Does the path have a VFS root?
    pub fn has_vfs_root(&self) -> bool {
        if let Some(Component::Root(root)) = self.components().next() {
            matches!(root.kind(), Root::VFS(_))
        } else {
            false
        }
    }

    /// Is the path a search path, i.e. does it contain a wildcard.
    pub fn is_search_path(&self) -> bool {
        self.0.contains(|c| c == '*' || c == '?')
    }

    /// Check if the path is a VFS path, i.e. does the path either contain macros, or start with a VFS root?
    pub fn is_vfs_path(&self) -> bool {
        self.has_vfs_root() || self.has_macros()
    }

    /// Get the form the path is currently in
    pub fn get_form(&self) -> PathForm {
        let mut comps = self.components();

        // Assume we start with fully qualified, lower if this is not the case
        let mut form = if comps.has_root() { PathForm::FullyQualified } else { PathForm::Unnormalized };

        for comp in comps {
            match comp {
                Component::Root(root) => match root.kind() {
                    Root::Common if cfg!(windows)                 => form = PathForm::Normalized,
                    Root::VFS(_)                                  => form = PathForm::Normalized,
                    Root::Drive(_)
                    | Root::WinShare { .. } if cfg!(not(windows)) => form = PathForm::Normalized,
                    _                                             => {},
                },
                Component::CurDir      => form = PathForm::Unnormalized,
                Component::ParentDir   => form = PathForm::Unnormalized,
                Component::Normal(_)   => {},
                Component::VFS(_)      => return PathForm::Raw,
                Component::Wildcard(_) => form = PathForm::Unnormalized,
            }
        }
        form
    }

    /// Create an owned [`PathBuf`] with `path` adjoined to `self`.
    /// 
    /// See [`PathBuf::push`] for more details on what t means to adjoin a path.
    /// 
    /// [`PathBuf::push`]: PathBuf::push
    #[must_use]
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self._join(path.as_ref())
    }

    fn _join(&self, path: &Path) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.push(path);
        buf
    }

    /// Creates an owned [`PathBuf`] like `self`, but with the given filename.
    /// 
    /// See [`PathBuf::set_file_name`] for more details.
    /// 
    /// [`PathBuf::set_file_name`]: PathBuf::set_file_name
    #[must_use]
    pub fn with_file_name<S: AsRef<str>>(&self, file_name: S) -> PathBuf {
        self._with_file_name(file_name.as_ref())
    }

    fn  _with_file_name(&self, file_name: &str) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.set_file_name(file_name);
        buf
    }

    /// Creates an owned [`PathBuf`] like `self`, but with the given extension.
    /// 
    /// see [`Path::set_extension`] for more details.
    /// 
    /// [`Path::set_extension`]: Path::set_extension
    #[must_use]
    pub fn with_extension<S: AsRef<str>>(&self, extension: S) -> PathBuf {
        self._with_extension(extension.as_ref())
    }

    fn _with_extension(&self, extension: &str) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.set_extension(extension);
        buf
    }

    /// Produces an iterator over the [`Component`]s of the path.
    /// 
    /// When parsing the path, there is a small amount of normalization:
    /// - Repeated separators are ignored, so `a/b` and `a//b` both have `a` and `b` as components.
    /// - Occurances of `.` are normalized away, except if htey are  at teh beginning of the path.
    ///   For exzample, `a/./b`, `a/b/`, and `a/b` all have `a` and `b` as components, but `./a/b` starts with a n additional [`CurDir`] component
    /// - A trailing slash is normalzied away `a/b` and `a/b/` are equivalent
    pub fn components(&self) -> Components<'_> {
        let root = parse_root(self.as_str());
        Components {
            path: self.as_str(),
            root,
            front: State::Root,
            back: State::Body,
        }
    }

    /// Produces an iterator over the path's components viewed as [`str`] slices.
    /// 
    /// For more information about the particulars of how the path is separated into components, see [`Path::components`]
    /// 
    /// [`Path::components`]: Path::components
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.components())
    }
}

impl AsRef<str> for Path {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

/*
impl AsRef<Path> for str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for String {
    fn as_ref(&self) -> &Path {
       Path::new(self)
    }
}
*/

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.components() == other.components()
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(compare_components(self.components(), other.components()))
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        compare_components(self.components(), other.components())
    }
}

impl Hash for Path {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let bytes = self.as_str().as_bytes();
        let root_len = match parse_root(&self.0) {
            Some(root) => {
                root.hash(state);
                root.len()
            },
            None => 0
        };

        let bytes = &bytes[root_len..];

        let mut component_start = 0;
        let mut bytes_hashed = 0;

        for i in 0..bytes.len() {
            let is_sep = is_sep_byte(bytes[i]);
            if is_sep {
                if i > component_start {
                    let to_hash = &bytes[component_start..i];
                    state.write(to_hash);
                    bytes_hashed += to_hash.len();
                }

                // Skip over separator and optionally a folliwnig CurDir item since components() would normalize there away
                component_start = i + 1;

                let tail = &bytes[component_start..];

                component_start += match tail {
                    [b'.']                                     => 1,
                    [b'.', sep, ..] if self::is_sep_byte(*sep) => 1,
                    _                                          => 0
                }
            }
        }

        if component_start < bytes.len() {
            let to_hash = &bytes[component_start..];
            state.write(to_hash);
            bytes_hashed += to_hash.len();
        }

        state.write_usize(bytes_hashed);
    }
}

//--------------------------------------------------------------

macro_rules! impl_cmp {
    ($lhs:ty, $rhs:ty) => {
        impl_cmp!{ @internal $lhs, $rhs }
        impl_cmp!{ @internal $rhs, $lhs }
    };
    (@internal $lhs:ty, $rhs:ty) => {
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                <Path as PartialEq>::eq(self, other)
            }
        }

        impl<'a, 'b> PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<cmp::Ordering> {
                <Path as PartialOrd>::partial_cmp(self, other)
            }
        }
    };
}

impl_cmp!(PathBuf, Path);
impl_cmp!(PathBuf, &'a Path);

//--------------------------------------------------------------

pub fn parse_root(path: &str) -> Option<Root<'_>> {
    use Root::*;

    if path.starts_with(r"\\") || path.starts_with("//") {
        let path = &path[2..];
        let (server, path) = parse_next_component(path);
        let (share, _) = parse_next_component(path);
        Some(WinShare { server, share })
    } else if path.starts_with(is_sep) {
        Some(Common)
    } else if let Some(drive) = parse_drive(path) {
        Some(Drive(drive))
    } else if let Some(Some(colon)) = path.bytes().enumerate().find_map(|(idx, c)| if c == b':' {
            Some(Some(idx))
        } else if is_sep_byte(c) {
            Some(None)
        } else {
            None
        }
    ) {
        // VFS prefix
        Some(VFS(&path[..colon]))
    } else { // no prefix
        None
    }
}

// Parses a drive prefix, e.g. "C:/"
fn parse_drive(path: &str) -> Option<u8> {
    // In most DOS systems, it is not possible to have more than 26 drive letters
    // See https://en.wikipedia.org/wiki/Drive_letter_assignment#Common_assignments
    fn is_valid_drive_letter(drive: &u8) -> bool {
        drive.is_ascii_alphabetic()
    }

    match path.as_bytes() {
        [drive, b':', ..] if is_valid_drive_letter(drive) => Some(drive.to_ascii_uppercase()),
        _ => None
    }
}

/// Parse the next path component
/// 
/// Returns the next component and the rest of hte path excluding the component and separator
fn parse_next_component(path: &str) -> (&str, &str) {
    match path.bytes().position(|x| is_sep_byte(x)) {
        Some(separator_start) => {
            let component = &path.as_bytes()[..separator_start];
            
            let separator_end = separator_start + 1;
            // Panic safe
            // The max `separator_end` is `bytes.len()` and `bytes[bytes.len()..]` is a vlaid index
            let path = &path.as_bytes()[separator_end..];

            unsafe { (std::str::from_utf8_unchecked(component), core::str::from_utf8_unchecked(path)) }
        },
        None => (path, "")
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_path() {
        let valid = Path::new("test/path");
        assert!(valid.is_ok());

        let invalid = Path::new("");
        assert!(invalid.is_err());

        // invalid chars: :\"|?*
        let invalid = Path::new("bl/h:");
        match invalid {
            Ok(_) => panic!("Found Ok(_), expected Err(_)"),
            Err(InvalidPathError((_, idx))) => assert_eq!(idx, 4),
        }
        let invalid = Path::new("blah\"");
        match invalid {
            Ok(_) => panic!("Found Ok(_), expected Err(_)"),
            Err(InvalidPathError((_, idx))) => assert_eq!(idx, 4),
        }
        let invalid = Path::new("blah|");
        match invalid {
            Ok(_) => panic!("Found Ok(_), expected Err(_)"),
            Err(InvalidPathError((_, idx))) => assert_eq!(idx, 4),
        }
        let invalid = Path::new("blah?");
        match invalid {
            Ok(_) => panic!("Found Ok(_), expected Err(_)"),
            Err(InvalidPathError((_, idx))) => assert_eq!(idx, 4),
        }
        let invalid = Path::new("blah*");
        match invalid {
            Ok(_) => panic!("Found Ok(_), expected Err(_)"),
            Err(InvalidPathError((_, idx))) => assert_eq!(idx, 4),
        }

        // valid with "invalid" char
        let valid = Path::new("C:/test");
        assert!(valid.is_ok());

        // Valid macro
        let valid = Path::new("<macro>");
        assert!(valid.is_ok());

        // Invalid macro
        let invalid = Path::new("blah>");
        match invalid {
            Ok(_) => panic!("Found Ok(_), expected Err(_)"),
            Err(InvalidPathError((_, idx))) => assert_eq!(idx, 4),
        }
        let invalid = Path::new("blah<<");
        match invalid {
            Ok(_) => panic!("Found Ok(_), expected Err(_)"),
            Err(InvalidPathError((_, idx))) => assert_eq!(idx, 5),
        }
        let invalid = Path::new("<blah");
        match invalid {
            Ok(_) => panic!("Found Ok(_), expected Err(_)"),
            Err(InvalidPathError((_, idx))) => assert_eq!(idx, 5),
        }
    }

    #[test]
    fn path_components() {
        let path = Path::new("a/b").unwrap();
        let mut comp = path.components();
        assert_eq!(comp.next(), Some(Component::Normal("a")));
        assert_eq!(comp.next(), Some(Component::Normal("b")));
        assert_eq!(comp.next(), None);

        let path = Path::new("C:/a/b").unwrap();
        let mut comp = path.components();
        match comp.next() {
            Some(comp) => match comp {
                Component::Root(root) => assert!(matches!(root.kind(), Root::Drive(b'C'))),
                _ => panic!("Expected root component"),
            },
            None => panic!("Root expected"),
        }
        assert_eq!(comp.next(), Some(Component::Normal("a")));
        assert_eq!(comp.next(), Some(Component::Normal("b")));
        assert_eq!(comp.next(), None);

        let path = Path::new("//server/share/a/b").unwrap();
        let mut comp = path.components();
        match comp.next() {
            Some(comp) => match comp {
                Component::Root(root) => assert!(matches!(root.kind(), Root::WinShare{ server: "server", share: "share" })),
                _ => panic!("Expected root component"),
            },
            None => panic!("Root expected"),
        }
        assert_eq!(comp.next(), Some(Component::Normal("a")));
        assert_eq!(comp.next(), Some(Component::Normal("b")));
        assert_eq!(comp.next(), None);

        let path = Path::new("/a/b").unwrap();
        let mut comp = path.components();
        match comp.next() {
            Some(comp) => match comp {
                Component::Root(root) => assert!(matches!(root.kind(), Root::Common)),
                _ => panic!("Expected root component"),
            },
            None => panic!("Root expected"),
        }
        assert_eq!(comp.next(), Some(Component::Normal("a")));
        assert_eq!(comp.next(), Some(Component::Normal("b")));
        assert_eq!(comp.next(), None);

        let path = Path::new("vfs:/a/b").unwrap();
        let mut comp = path.components();
        match comp.next() {
            Some(comp) => match comp {
                Component::Root(root) => assert!(matches!(root.kind(), Root::VFS("vfs"))),
                _ => panic!("Expected root component"),
            },
            None => panic!("Root expected"),
        }
        assert_eq!(comp.next(), Some(Component::Normal("a")));
        assert_eq!(comp.next(), Some(Component::Normal("b")));
        assert_eq!(comp.next(), None);

        let path = Path::new("a/../b").unwrap();
        let mut comp = path.components();
        assert_eq!(comp.next(), Some(Component::Normal("a")));
        assert_eq!(comp.next(), Some(Component::ParentDir));
        assert_eq!(comp.next(), Some(Component::Normal("b")));
        assert_eq!(comp.next(), None);

        let path = Path::new("a/b*").unwrap();
        let mut comp = path.components();
        assert_eq!(comp.next(), Some(Component::Normal("a")));
        assert_eq!(comp.next(), Some(Component::Wildcard("b*")));
        assert_eq!(comp.next(), None);
        
        let path = Path::new("a/b?").unwrap();
        let mut comp = path.components();
        assert_eq!(comp.next(), Some(Component::Normal("a")));
        assert_eq!(comp.next(), Some(Component::Wildcard("b?")));
        assert_eq!(comp.next(), None);
        
        let path = Path::new("a/b<macro>").unwrap();
        let mut comp = path.components();
        assert_eq!(comp.next(), Some(Component::Normal("a")));
        assert!(matches!(comp.next(), Some(Component::VFS(_))));
        assert_eq!(comp.next(), None);
    }

    #[test]
    fn path_components_rev() {
        let path = Path::new("a/b").unwrap();
        let mut comp = path.components();
        assert_eq!(comp.next_back(), Some(Component::Normal("b")));
        assert_eq!(comp.next_back(), Some(Component::Normal("a")));
        assert_eq!(comp.next_back(), None);

        let path = Path::new("C:/a/b").unwrap();
        let mut comp = path.components();
        match comp.next() {
            Some(comp) => match comp {
                Component::Root(root) => assert!(matches!(root.kind(), Root::Drive(b'C'))),
                _ => panic!("Expected root component"),
            },
            None => panic!("Root expected"),
        }
        assert_eq!(comp.next_back(), Some(Component::Normal("b")));
        assert_eq!(comp.next_back(), Some(Component::Normal("a")));
        assert_eq!(comp.next_back(), None);

        let path = Path::new("//server/share/a/b").unwrap();
        let mut comp = path.components();
        match comp.next() {
            Some(comp) => match comp {
                Component::Root(root) => assert!(matches!(root.kind(), Root::WinShare{ server: "server", share: "share" })),
                _ => panic!("Expected root component"),
            },
            None => panic!("Root expected"),
        }
        assert_eq!(comp.next_back(), Some(Component::Normal("b")));
        assert_eq!(comp.next_back(), Some(Component::Normal("a")));
        assert_eq!(comp.next_back(), None);

        let path = Path::new("/a/b").unwrap();
        let mut comp = path.components();
        match comp.next() {
            Some(comp) => match comp {
                Component::Root(root) => assert!(matches!(root.kind(), Root::Common)),
                _ => panic!("Expected root component"),
            },
            None => panic!("Root expected"),
        }
        assert_eq!(comp.next_back(), Some(Component::Normal("b")));
        assert_eq!(comp.next_back(), Some(Component::Normal("a")));
        assert_eq!(comp.next_back(), None);

        let path = Path::new("vfs:/a/b").unwrap();
        let mut comp = path.components();
        match comp.next() {
            Some(comp) => match comp {
                Component::Root(root) => assert!(matches!(root.kind(), Root::VFS("vfs"))),
                _ => panic!("Expected root component"),
            },
            None => panic!("Root expected"),
        }
        assert_eq!(comp.next_back(), Some(Component::Normal("b")));
        assert_eq!(comp.next_back(), Some(Component::Normal("a")));
        assert_eq!(comp.next_back(), None);

        let path = Path::new("a/../b").unwrap();
        let mut comp = path.components();
        assert_eq!(comp.next_back(), Some(Component::Normal("b")));
        assert_eq!(comp.next_back(), Some(Component::ParentDir));
        assert_eq!(comp.next_back(), Some(Component::Normal("a")));
        assert_eq!(comp.next_back(), None);

        let path = Path::new("a/b*").unwrap();
        let mut comp = path.components();
        assert_eq!(comp.next_back(), Some(Component::Wildcard("b*")));
        assert_eq!(comp.next_back(), Some(Component::Normal("a")));
        assert_eq!(comp.next_back(), None);
        
        let path = Path::new("a/b?").unwrap();
        let mut comp = path.components();
        assert_eq!(comp.next_back(), Some(Component::Wildcard("b?")));
        assert_eq!(comp.next_back(), Some(Component::Normal("a")));
        assert_eq!(comp.next_back(), None);
        
        let path = Path::new("a/b<macro>").unwrap();
        let mut comp = path.components();
        assert!(matches!(comp.next_back(), Some(Component::VFS(_))));
        assert_eq!(comp.next_back(), Some(Component::Normal("a")));
        assert_eq!(comp.next_back(), None);
    }

    #[test]
    fn path_macro() {
        let path = Path::new("<macro>").unwrap();
        let comp = path.components().next().unwrap();
        if let Component::VFS(mut vfs) = comp {
            assert_eq!(vfs.next(), Some(VFSPart::Macro(VFSMacro { name: "macro", raw: "<macro>" })));
            assert_eq!(vfs.next(), None)
        } else {
            panic!("Expected a VFS component");
        }
         
        let path = Path::new("<macro>test").unwrap();
        let comp = path.components().next().unwrap();
        if let Component::VFS(mut vfs) = comp {
            assert_eq!(vfs.next(), Some(VFSPart::Macro(VFSMacro { name: "macro", raw: "<macro>" })));
            assert_eq!(vfs.next(), Some(VFSPart::Text("test")));
            assert_eq!(vfs.next(), None)
        } else {
            panic!("Expected a VFS component");
        }
         
        let path = Path::new("test<macro>").unwrap();
        let comp = path.components().next().unwrap();
        if let Component::VFS(mut vfs) = comp {
            assert_eq!(vfs.next(), Some(VFSPart::Text("test")));
            assert_eq!(vfs.next(), Some(VFSPart::Macro(VFSMacro { name: "macro", raw: "<macro>" })));
            assert_eq!(vfs.next(), None)
        } else {
            panic!("Expected a VFS component");
        }

        let path = Path::new("a<macro>b").unwrap();
        let comp = path.components().next().unwrap();
        if let Component::VFS(mut vfs) = comp {
            assert_eq!(vfs.next(), Some(VFSPart::Text("a")));
            assert_eq!(vfs.next(), Some(VFSPart::Macro(VFSMacro { name: "macro", raw: "<macro>" })));
            assert_eq!(vfs.next(), Some(VFSPart::Text("b")));
            assert_eq!(vfs.next(), None)
        } else {
            panic!("Expected a VFS component");
        }

        let path = Path::new("a<macro><macro2>").unwrap();
        let comp = path.components().next().unwrap();
        if let Component::VFS(mut vfs) = comp {
            assert_eq!(vfs.next(), Some(VFSPart::Text("a")));
            assert_eq!(vfs.next(), Some(VFSPart::Macro(VFSMacro { name: "macro", raw: "<macro>" })));
            assert_eq!(vfs.next(), Some(VFSPart::Macro(VFSMacro { name: "macro2", raw: "<macro2>" })));
            assert_eq!(vfs.next(), None)
        } else {
            panic!("Expected a VFS component");
        }
    }

    #[test]
    fn path_macro_rev() {
        let path = Path::new("<macro>").unwrap();
        let comp = path.components().next().unwrap();
        if let Component::VFS(mut vfs) = comp {
            assert_eq!(vfs.next_back(), Some(VFSPart::Macro(VFSMacro { name: "macro", raw: "<macro>" })));
            assert_eq!(vfs.next_back(), None)
        } else {
            panic!("Expected a VFS component");
        }
         
        let path = Path::new("<macro>test").unwrap();
        let comp = path.components().next().unwrap();
        if let Component::VFS(mut vfs) = comp {
            assert_eq!(vfs.next_back(), Some(VFSPart::Text("test")));
            assert_eq!(vfs.next_back(), Some(VFSPart::Macro(VFSMacro { name: "macro", raw: "<macro>" })));
            assert_eq!(vfs.next_back(), None)
        } else {
            panic!("Expected a VFS component");
        }
         
        let path = Path::new("test<macro>").unwrap();
        let comp = path.components().next().unwrap();
        if let Component::VFS(mut vfs) = comp {
            assert_eq!(vfs.next_back(), Some(VFSPart::Macro(VFSMacro { name: "macro", raw: "<macro>" })));
            assert_eq!(vfs.next_back(), Some(VFSPart::Text("test")));
            assert_eq!(vfs.next_back(), None)
        } else {
            panic!("Expected a VFS component");
        }

        let path = Path::new("a<macro>b").unwrap();
        let comp = path.components().next().unwrap();
        if let Component::VFS(mut vfs) = comp {
            assert_eq!(vfs.next_back(), Some(VFSPart::Text("b")));
            assert_eq!(vfs.next_back(), Some(VFSPart::Macro(VFSMacro { name: "macro", raw: "<macro>" })));
            assert_eq!(vfs.next_back(), Some(VFSPart::Text("a")));
            assert_eq!(vfs.next_back(), None)
        } else {
            panic!("Expected a VFS component");
        }

        let path = Path::new("a<macro><macro2>").unwrap();
        let comp = path.components().next().unwrap();
        if let Component::VFS(mut vfs) = comp {
            assert_eq!(vfs.next_back(), Some(VFSPart::Macro(VFSMacro { name: "macro2", raw: "<macro2>" })));
            assert_eq!(vfs.next_back(), Some(VFSPart::Macro(VFSMacro { name: "macro", raw: "<macro>" })));
            assert_eq!(vfs.next_back(), Some(VFSPart::Text("a")));
            assert_eq!(vfs.next_back(), None)
        } else {
            panic!("Expected a VFS component");
        }
    }

    #[test]
    fn path_misc() {
        let path = Path::new("relative/path").unwrap();
        assert!(path.is_relative());

        let path = Path::new("C:/win/abs/path").unwrap();
        assert!(path.is_absolute());
        assert!(path.has_root());
        let path = Path::new("//server/share/win/abs/path").unwrap();
        assert!(path.is_absolute());
        assert!(path.has_root());
        let path = Path::new("vfs:/win/abs/path").unwrap();
        assert!(path.is_absolute());
        assert!(path.has_root());
        
        // This path is only relative on windows
        let path = Path::new("/win/abs/path").unwrap();
        assert_eq!(path.is_absolute(), cfg!(not(windows)));
        assert!(path.has_root());

        let path = Path::new("some/path/name.file.txt").unwrap();
        let parent = path.parent();
        assert_eq!(parent, Some(Path::new("some/path").unwrap()));
        let parent = parent.unwrap().parent();
        assert_eq!(parent, Some(Path::new("some").unwrap()));
        let parent = parent.unwrap().parent();
        assert_eq!(parent, None);

        let mut ancestors = path.ancestors();
        assert_eq!(ancestors.next(), Some(path));
        assert_eq!(ancestors.next(), Some(Path::new("some/path").unwrap()));
        assert_eq!(ancestors.next(), Some(Path::new("some").unwrap()));
        assert_eq!(ancestors.next(), None);

        assert_eq!(path.file_name(), Some("name.file.txt"));
        assert_eq!(path.extension(), Some("txt"));
        assert_eq!(path.file_stem(), Some("name.file"));
        assert_eq!(path.file_prefix(), Some("name"));

        assert!(path.starts_with(Path::new("some/path").unwrap()));
        assert!(path.ends_with(Path::new("name.file.txt").unwrap()));

        assert_eq!(path.strip_prefix(Path::new("some").unwrap()), Ok(Path::new("path/name.file.txt").unwrap()));


        let path = Path::new("relative/<path>").unwrap();
        assert_eq!(path.get_form(), PathForm::Raw);

        let path = Path::new("relative/path").unwrap();
        assert_eq!(path.get_form(), PathForm::Unnormalized);

        let path = Path::new("C:/absolute/../path").unwrap();
        assert_eq!(path.get_form(), PathForm::Unnormalized);

        let path = Path::new("vfs:/absolute/path").unwrap();
        assert_eq!(path.get_form(), PathForm::Normalized);

        let path = Path::new("C:/absolute/path").unwrap();
        assert_eq!(path.get_form(), PathForm::FullyQualified);
    }

    #[test]
    fn pathbuf() {
        // New
        let pathbuf = PathBuf::new();
        assert_eq!(pathbuf.capacity(), 0);

        let pathbuf = PathBuf::with_capacity(8);
        assert!(pathbuf.capacity() >= 8);

        let path = Path::new("some/path/file.txt").unwrap();
        let pathbuf = PathBuf::from_path(&path);
        assert_eq!(path, pathbuf);

        // push

        let path0 = Path::new("some/path").unwrap();
        let path1 = Path::new("file.txt").unwrap();
        let mut pathbuf = PathBuf::from_path(&path0);
        pathbuf.push(path1);
        assert_eq!(path, pathbuf);

        let path1 = Path::new("C:/file.txt").unwrap();
        let mut pathbuf = PathBuf::from_path(&path0);
        pathbuf.push(path1);
        assert_eq!(path1, pathbuf);

        let path0 = Path::new("some/path/").unwrap();
        let path1 = Path::new("C:/file.txt").unwrap();
        let mut pathbuf = PathBuf::from_path(&path0);
        pathbuf.push(path1);
        assert_eq!(path1, pathbuf);

        // pop

        let mut pathbuf = PathBuf::from_path(&path);
        pathbuf.pop();
        assert_eq!(path0, pathbuf);

        // set_file_name

        let path1 = Path::new("some/path/other_file.png").unwrap();
        let mut pathbuf = PathBuf::from_path(&path);
        pathbuf.set_file_name("other_file.png");
        assert_eq!(path1, pathbuf);
        
        let path0 = Path::new("some/..").unwrap();
        let path1 = Path::new("some/../other_file.png").unwrap();
        let mut pathbuf = PathBuf::from_path(&path0);
        pathbuf.set_file_name("other_file.png");
        assert_eq!(path1, pathbuf);

        // set extension
        let path1 = Path::new("some/path/file.txt").unwrap();
        let mut pathbuf = PathBuf::from_path(&path);
        pathbuf.set_extension("png");
    }
}