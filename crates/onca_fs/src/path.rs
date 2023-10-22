//! Path manipulation, which is based on a re-implemntation of rust's [`std::path`], allowing it to work with onca types and allocators.
//! 
//! This module provides two types, [`PathBuf`] and [`Path`] (akin to [`String`] and [`str`]), for working with paths abstractly.
//! These are thin wrappers around [`String`] and [`str`] respectively, meaning that unlike [`std::path::PathBuf`] and [`std::path::Path`], 
//! additional conversions need to take place at the point of use on certain platforms (e.g Window), but the cost of these conversions should in general not be an issue.
//! 
//! ## Case sensitivity
//! 
//! Unless otherwise indicated, path methods that do not access the filesystem such as [`Path::starts_with`] and `[`Path::end_with`], are case sensitive, no matter the platform or filesystem.
//! An exception to this is made for Windows drive letters
//! 
//! ## Assumptions and generalizations
//! 
//! There are also additional assumptions and generalization that are made, which allow for a more uniform use of a [`Path`] with less reliance on some OS-specific quirks
//! * All [`Path`]s use a '/' as a file separator, including Windows. While the use of `\` on Windows or with other filesystems that support this are handled, `/` should be prefered where-ever possible.
//! * On Windows, all path are assumed to be long path, allowing for more than 260 bytes in a path. 
//! * On Windows, verbatim paths (starting with '\\?\`) are supported, they are not required and it is *preferable* to avoid them is possible. 
//!   If they have to be used, they are expected to use `\` in the path, as `/` is not handled.
//! * It is assumed that OS/filesystem specific paths can be handled on system that themselves do not support those representations, as this could be required when communication with other code/systems.
//! 
//! ## VFS support
//! 
//! Onca's [`Path`] also supports features specific to Onca's _**V**irtual **F**ile **S**ystem_, where the following additional parts exists:
//! * `@{vfs_root}`, which is a special prefix, which can point to a mounted drive or 'device' in the VFS
//! * `${vfs_macro}`, which is a special value that will be replaced in the path, whose value will be concatinated with whathever is on it's left or right, but this cannot contain any file separators
//!
//! For more info about the VFS specific parts, see it's documentation
//! 
//! ## Notes
//! 
//! The internal representation may change in the future, but it would need to be proven to give a substantial benefit over the current representation.
//! As it's highly likely that the conversion to the OS/filesystem specific representation is negligable compared to the time needed by the filesystem to do an operation.

// NOTE(jel): std::path::* supports redox, so if we ever plan on supporting it, check its source

use core::{
    cmp,
    fmt,
    iter,
    borrow::Borrow,
    hash::{Hash, Hasher},
    iter::FusedIterator,
    str::FromStr,
    ops::Deref,
};
use std::collections::TryReserveError;
use onca_core::prelude::*;

//--------------------------------------------------------------

pub(crate) fn is_sep_byte(b: u8) -> bool {
    b == b'/' || b == b'\\'
}

pub(crate) fn is_verbatim_sep(b: u8) -> bool {
    b == b'\\'
}

//--------------------------------------------------------------

/// Windows path prefixes, e.g. `C:` or `\\server\share`.
/// 
/// Windows uses a variety of path prefix styles, including references to drive volumes (like `C:`), network shared folders (like `\\server\share`), and others.
/// In addition, some path prefixes are "verbatim" (i.e. prefixed with `\\?\`, in which case '/' ) is *not* treated as a separator and essentially no normalization is performed.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Prefix<'a> {
    /// Verbatim prefix, e.g. `\\?\cat_pics`.
    /// 
    /// Verbatim prefixes consist of `\\?\` immediately followed by the given component.
    Verbatim(&'a str),

    /// Verbatim prefix using Windows` _**U**niform **N**aming **C**onvention_, e.g. `\\?\UNC\server\share`.
    /// 
    /// Verbatim prefixes consist of `\\?\UNC\` immediately followed by teh server's hostname and a share name.
    VerbatimUNC(&'a str, &'a str),

    /// Verbatim disk prefix, e.g. `\\?\C:`.
    /// 
    /// Verbatim disk prefixes consist of `\\?\` immediately followed by the drive letter and `:`
    VerbatimDisk(u8),

    /// Device namespace prefix, e.g. `\\.\COM42`.
    /// 
    /// Device namespace prefixes consist of `\\.\` (possible using `/` instead of `\`), immediately followed by the device name.
    DeviceNS(&'a str),

    /// Prefix using Windows` _**U**niform **N**aming **C**onvention_, e.g. `\\server\share`.
    /// 
    /// UNC prefixes consist of the server's hostname and a share name.
    UNC(&'a str, &'a str),

    // Prefix `C:` for a given disk drive.
    Disk(u8)
}

impl<'a> Prefix<'a> {
    #[inline]
    fn len(&self) -> usize {
        use self::Prefix::*;
        match *self {
            Verbatim(x)       => 4 + x.len(),
            VerbatimUNC(x, y) => 8 + x.len() + if y.len() > 0 { 1 + y.len() } else { 0 },
            VerbatimDisk(_)   => 6,
            UNC(x, y)         => 2 + x.len() + if y.len() > 0 { 1 + y.len() } else { 0 },
            DeviceNS(x)       => 4 + x.len(),
            Disk(_)           => 2,
        }
    }

    /// Determines if the prefix is verbatim, i.e. begins with '\\?\`
    #[must_use]
    #[inline]
    pub fn is_verbatim(&self) -> bool {
        use self::Prefix::*;
        matches!(*self, Verbatim(_) | VerbatimDisk(_) | VerbatimUNC(..))
    }

    #[inline]
    fn is_drive(&self) -> bool {
        matches!(*self, Prefix::Disk(_))
    }

    #[inline]
    fn has_implicit_root(&self) -> bool {
        !self.is_drive()
    }
}

//--------------------------------------------------------------

/// Determines whether the character is one of the permitted path separators for the current platform
#[must_use]
pub fn is_separator(c: char) -> bool {
    c.is_ascii() && is_sep_byte(c as u8)
}

/// The primary separator of path components
pub const MAIN_SEP : char = '/';

/// The primary separator of path components
pub const MAIN_SEP_STR : &str = "/";

//--------------------------------------------------------------

fn iter_after<'a, 'b, I, J>(mut iter: I, mut prefix: J) -> Option<I>
    where I : Iterator<Item = Component<'a>> + Clone,
          J : Iterator<Item = Component<'b>>
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

//--------------------------------------------------------------

/// Says whether the first byte after the prefix is a separator
fn has_physical_root(s: &str, prefix: Option<Prefix<'_>>) -> bool {
    let path = if let Some(p) = prefix { &s[p.len()..] } else { s };
    !path.is_empty() && is_sep_byte(path.as_bytes()[0])
}

// Basic workhorse for splitting stem and extension
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
        None => return (file, None)
    };
    let before = &file[..i];
    let after = &file[(i + 1)..];
    (before, Some(after))
}

//--------------------------------------------------------------

/// Component parsing works by a double-ended state machine; the cursors at the fron and back of the path each keep track of what parts of hte path have been consumed so far.
/// 
/// Going front to back, a path is made up of a prefix, a starting directory component, and a body (of normal components)
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
enum State {
    Prefix,   // c:
    StartDir, // / or . or nothing
    Body,     // foo/bar/baz
    Done
}

/// A structure wrapping a Windows path prefix as well as its unparsed string representation
/// 
/// In addition to the parsed  [`Prefix`] information returned by [`kind`], `PrefixComponent` also holds the raw and unparsed [`str`] slice, returned by [`as_str`]
/// 
/// Instances of this `struct` can be obtained by matching against the [`Prefix` variant] on [`Component`]
/// 
/// Does not occur on Posix.
/// 
/// [`as_str`]: PrefixComponent::as_str
/// [`kind`]: PrefixComponent::kind
/// [`Prefix` variant]: Component::Prefix
#[derive(Clone, Copy, Eq, Debug)]
pub struct PrefixComponent<'a> {
    /// The prefix as an unparsed `str` slice
    raw    : &'a str,

    /// The parsed prefix data
    parsed : Prefix<'a>
}

impl<'a> PrefixComponent<'a> {
    /// Returns the parse prefix data
    /// 
    /// See ['Prefix']'s documentation for more information on th different kinds of prefixes
    #[must_use]
    #[inline]
    pub fn kind(&self) -> Prefix<'a> {
        self.parsed
    }

    /// Returns the [`str`] slice for this prefix
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.raw
    }
}

impl<'a> PartialEq for PrefixComponent<'a> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.parsed, &other.parsed)
    }
}

impl<'a> PartialOrd for PrefixComponent<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&self.parsed, &other.parsed)
    }
}

impl<'a> Ord for PrefixComponent<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&self.parsed, &other.parsed)
    }
}

impl Hash for PrefixComponent<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.parsed.hash(state);
    }
}

/// A single component of a path
/// 
/// A `Component` roughly corresponds to a substring between path separators (`\` or `/`)
/// 
/// This `enum` is created by iterating over [`Components`], which in turn is created by the [`components`](Path::components) method on [`Path`]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Component<'a> {
    /// A Windows path prefix, e.g. `C:` or `\\server\share`.
    /// 
    /// There is a large variety of prefix types, see [`Prefix`]'s documentation for more
    /// 
    /// Does not occur on Posix
    Prefix(PrefixComponent<'a>),

    /// The root directory component, apperas after any prefixes and before anything else.
    /// 
    /// It represents a separator that designated that a path start from root.
    RootDir,

    /// A reference to the current directory, i.e. `.`.
    CurDir,

    /// A reference to the parent directory, i.e. `..`.
    ParentDir,

    /// A normal component, e.g. `a` and `b` in `a/b`
    /// 
    /// This variant is the most common one, it represents references to files or directories
    Normal(&'a str),

    // TODO(jel): VFS Macro
}

impl<'a> Component<'a> {
    /// Extract the underlying [`str`] slice
    pub fn as_str(self) -> &'a str {
        match self {
            Component::Prefix(p) => p.as_str(),
            Component::RootDir => MAIN_SEP_STR,
            Component::CurDir => ".",
            Component::ParentDir => "..",
            Component::Normal(path) => path,
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
        self.as_str().as_ref()
    }
}

/// An iterator over the [`Component`]s of a [`Path`].
/// 
/// This `struct` is created by the [`components`] method on [`Path`].
/// See its documentation for more info.
/// 
/// [`components`]: Path::components
#[derive(Clone)]
pub struct Components<'a> {
    // The path left to parse components from
    path              : &'a str,
    // The prefix as it was originally parse, if any
    prefix            : Option<Prefix<'a>>,
    // `true` if path *physically* has a root separator; for most WIndows prefixes, it may have a "logical" root separator for the purposes of normalization, e.g. \\server\share == \\server\share\.
    has_physical_root : bool,
    // The iterator is double-ended, and thses two states keep track of what has been produces for either end
    front             : State,
    back              : State
}

/// An iterator over the [`Component`]s of a [`Path`], as [`str`] slices.
///
/// This `struct` is crated by the [`iter`] method on [`Path`].
/// See its documentation for more.
/// 
/// [`iter`]: Path::iter
#[derive(Clone)]
pub struct Iter<'a> {
    inner : Components<'a>
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
    // How long is the prefix, if any?
    #[inline]
    fn prefix_len(&self) -> usize {
        self.prefix.as_ref().map(Prefix::len).unwrap_or(0)
    }

    #[inline]
    fn prefix_verbatim(&self) -> bool {
        self.prefix.as_ref().map(Prefix::is_verbatim).unwrap_or(false)
    }

    // How much of the prefix is left from the point of vies of iteration?
    #[inline]
    fn prefix_remaining(&self) -> usize {
        if self.front == State::Prefix { self.prefix_len() } else { 0 }
    }

    // Given the iteration so far, how much of the pre-State::Body path is left?
    #[inline]
    fn len_before_body(&self) -> usize {
        let root = if self.front <= State::StartDir && self.has_physical_root { 1 } else { 0 };
        let cur_dir = if self.front <= State::StartDir && self.include_cur_dir() { 1 } else { 0 };
        self.prefix_remaining() + root + cur_dir
    }

    // is the iteration complete?
    #[inline]
    fn finished(&self) -> bool {
        self.front == State::Done || self.back == State::Done || self.front > self.back
    }

    #[inline]
    fn is_sep_byte(&self, b: u8) -> bool {
        if self.prefix_verbatim() { is_verbatim_sep(b) } else { is_sep_byte(b) }
    }

    /// Extracts a slice corresponding to the portion of the path remaining for iteration
    #[must_use]
    pub fn as_path(&self) -> &'a Path {
        let mut comps = self.clone();
        if comps.front == State::Body {
            comps.trim_left();
        }
        if comps.back == State::Body {
            comps.trim_right();
        }
        Path::new(comps.path)
    }

    // Is the *original* path rooted?
    pub fn has_root(&self) -> bool {
        if self.has_physical_root {
            return true;
        }
        if let Some(p) = self.prefix {
            if p.has_implicit_root() {
                return true;
            }
        }
        false
    }

    // Should the normalized path include the leading . ?
    fn include_cur_dir(&self) -> bool {
        if self.has_root() {
            return false;
        }
        let mut iter = self.path[self.prefix_remaining()..].as_bytes().iter();
        match (iter.next(), iter.next()) {
            (Some(b'.'), None) => true,
            (Some(b'.'), Some(&b)) => self.is_sep_byte(b),
            _ => false
        }
    }

    // Parse a given str into the corresponding path component
    fn parse_single_component<'b>(&self, comp: &'b str) -> Option<Component<'b>> {
        match comp {
            "." if self.prefix_verbatim() => Some(Component::CurDir),
            "."                           => None, // . components are normalized away, except at the beginning of a path, which is treaded separately via `include_cur_dir`
            ".."                          => Some(Component::ParentDir),
            ""                            => None,
            _                             => Some(Component::Normal(comp))
        }
    }

    // Parse a component from th left, saying how many chars to consume to remove the component
    fn parse_next_component(&self) -> (usize, Option<Component<'a>>) {
        debug_assert!(self.front == State::Body);
        let (extra, comp) = match self.path.bytes().position(|b| self.is_sep_byte(b)) {
            None => (0, self.path),
            Some(i) => (i, &self.path[..i])
        };
        (comp.len() + extra, self.parse_single_component(comp))
    }

    // Parse a component from the right, saying how many chars to consume to remove the component
    fn parse_next_component_back(&self) -> (usize, Option<Component<'a>>) {
        debug_assert!(self.back == State::Body);
        let start = self.len_before_body();
        let (extra, comp) = match self.path[start..].bytes().rposition(|b| self.is_sep_byte(b)) {
            None => (0, &self.path[start..]),
            Some(i) => (1, &self.path[(start + i + 1)..]),
        };
        (comp.len() + extra, self.parse_single_component(comp))
    }

    // trim away repeasted separators (i.e. empty components) on the left
    fn trim_left(&mut self) {
        while !self.path.is_empty() {
            let (size, comp) = self.parse_next_component();
            if comp.is_none() {
                return;
            } else {
                self.path = &self.path[size..];
            }
        }
    }

    // trim away repeasted separators (i.e. empty components) on the right
    fn trim_right(&mut self) {
        while !self.path.len() > self.len_before_body() {
            let (size, comp) = self.parse_next_component_back();
            if comp.is_some() {
                return;
            } else {
                self.path = &self.path[..(self.path.len() - size)];
            }
        }
    }
}

impl AsRef<Path> for Components<'_> {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<str> for Components<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_path().as_str()
    }
}

impl fmt::Debug for Iter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a>(&'a Path);

        impl fmt::Debug for DebugHelper<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }

        f.debug_tuple("Iter").field(&DebugHelper(self.as_path())).finish()
    }
}

impl<'a> Iter<'a> {
    /// Extracts a slice corresponding to the portion of the path remaining for iteration
    #[must_use]
    #[inline]
    pub fn as_path(&self) -> &'a Path {
        self.inner.as_path()
    }
}

impl AsRef<Path> for Iter<'_> {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<str> for Iter<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_path().as_str()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Component::as_str)
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(Component::as_str)
    }
}

impl FusedIterator for Iter<'_> {}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.finished() {
            match self.front {
                State::Prefix if self.prefix_len() > 0 => {
                    self.front = State::StartDir;
                    debug_assert!(self.prefix_len() <= self.path.len());
                    let raw = &self.path[..self.prefix_len()];
                    self.path = &self.path[self.prefix_len()..];
                    return Some(Component::Prefix(PrefixComponent { raw: raw, parsed: self.prefix.unwrap() }));
                },
                State::Prefix => {
                    self.front = State::StartDir;
                },
                State::StartDir => {
                    self.front = State::Body;
                    if self.has_physical_root {
                        debug_assert!(!self.path.is_empty());
                        self.path = &self.path[1..];
                        return Some(Component::RootDir);
                    } else if let Some(p) = self.prefix {
                        if p.has_implicit_root() && !p.is_verbatim() {
                            return Some(Component::RootDir);
                        }
                    } else if self.include_cur_dir() {
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
                State::Body => {
                    self.front = State::Done;
                }
                State::Done => unreachable!(),
            }
        }
        None
    }
}

impl<'a> DoubleEndedIterator for Components<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        while !self.finished() {
            match self.back {
                State::Body if self.path.len() > self.len_before_body() => {
                    let (size, comp) = self.parse_next_component_back();
                    self.path = &self.path[..(self.path.len() - size)];
                    if comp.is_some() {
                        return comp;
                    }
                },
                State::Body => {
                    self.back = State::StartDir;
                },
                State::StartDir => {
                    self.back = State::Prefix;
                    if self.has_physical_root {
                        self.path = &self.path[..(self.path.len() - 1)];
                        return Some(Component::RootDir);
                    } else if let Some(p) = self.prefix {
                        if p.has_implicit_root() && !p.is_verbatim() {
                            return Some(Component::RootDir);
                        }
                    } else if self.include_cur_dir() {
                        self.path = &self.path[..(self.path.len() - 1)];
                        return Some(Component::CurDir);
                    }
                },
                State::Prefix if self.prefix_len() > 0 => {
                    self.back = State::Done;
                    return Some(Component::Prefix(PrefixComponent { raw: self.path, parsed: self.prefix.unwrap() }));
                },
                State::Prefix => {
                    self.back = State::Done;
                    return None;
                }
                State::Done => unreachable!()
            }
        }
        None
    }
}

impl FusedIterator for Components<'_> {}

impl<'a> PartialEq for Components<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Components{ path: _, front: _, back: _, has_physical_root: _, prefix: _ } = self;

        // Fast path for exact matches, e.g. for hasmap lookups.
        // Don't explicitly compare the prefix of has_physical_root fields since they'll either be covered by the `path` buffer  or are only relevant for `prefix_verbatim()`.
        if self.path.len() == other.path.len() &&
           self.front == other.front &&
           self.back == State::Body &&
           other.back == State::Body &&
           self.prefix_verbatim() == other.prefix_verbatim()
        {
            // possible future improvement: this could bail out earlier if ther ewer a reverse memcmp/bcmp comparing back to front
            if self.path == other.path {
                return true;
            }
        }

        // compare back to front since absolute paths oftern share long prefixes
        Iterator::eq(self.clone().rev(), other.clone().rev())
    }
}

impl Eq for Components<'_> {}

impl<'a> PartialOrd for Components<'a> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(compare_components(self.clone(), other.clone()))
    }
}

impl<'a> Ord for Components<'a> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        compare_components(self.clone(), other.clone())
    }
}

fn compare_components(mut left: Components<'_>, mut right: Components<'_>) -> cmp::Ordering {
    // Fast path for long shared prefixes
    //
    // - compare raw chars to find first mismatch
    // - backtrack to find separator before mismatch to avoid ambiguous parsings of '.' or '..' characters
    // - if found, update state to only do a component-wise comparison on the remainder, otherwise do it on the full path
    //
    // The fast path isn't taken for path with a PrefixComponent to avoid backtracking into the middle of one
    if left.prefix.is_none() && right.prefix.is_none() && left.front == right.front {
        // possible future improvement: simd comparison
        let first_difference = match left.path.chars().zip(right.path.chars()).position(|(a, b)| a != b) {
            None if left.path.len() == right.path.len() => return cmp::Ordering::Equal,
            None => left.path.len().min(right.path.len()),
            Some(diff) => diff
        };

        if let Some(previous_sep) = left.path[..first_difference].bytes().rposition(|c| (c as char).is_ascii() && left.is_sep_byte(c)) {
            let mismatched_component_start = previous_sep + 1;
            left.path = &left.path[mismatched_component_start..];
            left.front = State::Body;
            right.path = &right.path[mismatched_component_start..];
            right.front = State::Body;
        }
    }

    Iterator::cmp(left, right)
}

/// An iterator over [`Path`] and its ancestors.
/// 
/// This `struct` is created by the [`ancestors`] method on [`Path`].
/// See its documentation for more
#[derive(Clone, Copy, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Ancestors<'a> {
    next: Option<&'a Path>,
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = &'a Path;

    #[inline]
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
/// This type provides methods like [`push`] and [`set_extension`] that mutate the path in place. Italso implements [`Deref`] to [`Path`], meaning that all methods on [`Path`] slices are available on `PathBuf` values as well.
/// 
/// [`push`]: PathBuf::push
/// [`set_extension`]: PathBuf::set_extension
pub struct PathBuf {
    inner : String
}

impl PathBuf {
    #[inline]
    fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        unsafe { self.inner.as_mut_vec() }
    }

    /// Creates an empty `PathBuf` with the given allocator.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self { inner: String::new() }
    }

    /// Creates a new `PathBuf` with a given capacity and allocator, used to created the internal [`String`].
    /// See [`with_capacity`] defined on [`String`]
    /// 
    /// [`with_capacity`]: String::with_capacity
    #[must_use]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: String::with_capacity(capacity) }
    }

    /// Create a `PathBuf` from a [`str`] slice.
    #[must_use]
    #[inline]
    pub fn from_str(s: &str) -> Self {
        Self { inner: String::from(s) }
    }

    /// Create a `PathBuf` from a [`u8`] slice, including invalid characters.
    #[must_use]
    #[inline]
    pub fn from_utf8_lossy(s: &[u8]) -> Self {
        Self { inner: String::from_utf8_lossy(s).into() }
    }

    /// Coerces to a [`Path`] slice.
    #[must_use]
    #[inline]
    pub fn as_path(&self) -> &Path {
        self
    }

    /// Get a pointer to the first character of the path.
    #[must_use]
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.inner.as_ptr()
    }

    /// Extends `self` with `path`
    /// 
    /// If `path` is absolute, it replaces the current path
    /// 
    /// On Windows:
    /// 
    /// * if `path` has a root but no prefix (e.g. `\windows`), it replaces everyting except for the prefix (if any) of `self`
    /// * if `path` has a prefix but no root, it replaces `self`
    /// * if `self` is a verbatim prefix (e.g. `\\?\windows`) and `path` is not empty, the new path is normalized: all references to `.` and `..` are removed
    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        self._push(path.as_ref())
    }

    fn _push(&mut self, path: &Path) {
        // in general, a separator is needed if the rightmost byte is not a separator
        let mut need_sep = self.as_mut_vec().last().map(|c| !is_sep_byte(*c)).unwrap_or(false);

        // in the special case of `C:` on Windows, od *not* ass a separator
        let comps = self.components();

        if comps.prefix_len() > 0 &&
           comps.prefix_len() == comps.path.len() &&
           comps.prefix.unwrap().is_drive()
        {
            need_sep = false;
        }

        // absolute `path` replaces `self`
        if path.is_absolute() || path.prefix().is_some() {
            self.as_mut_vec().truncate(0);

        // verbatim paths need . and .. removed
        } else if comps.prefix_verbatim() && !path.inner.is_empty() {
            let mut buf : Vec<_> = comps.collect();
            for c in path.components() {
                match c {
                    Component::RootDir => {
                        buf.truncate(1);
                        buf.push(c);
                    },
                    Component::CurDir => (),
                    Component::ParentDir => {
                        if let Some(Component::Normal(_)) = buf.last() {
                            buf.pop();
                        }
                    },
                    _ => buf.push(c)
                }
            }

            let mut res = String::new();
            let mut need_sep = false;

            for c in buf {
                if need_sep && c != Component::RootDir {
                    res.push(MAIN_SEP);
                }
                res.push_str(c.as_str());

                need_sep = match c {
                    Component::RootDir => false,
                    Component::Prefix(prefix) => {
                        !prefix.parsed.is_drive() && prefix.parsed.len() > 0
                    },
                    _ => true
                }
            }

            self.inner = res;
            return;

        // `path` has a root but no prefix, e.g. `\windows` (Windows only)
        } else if path.has_root() {
            let prefix_len = self.components().prefix_remaining();
            self.as_mut_vec().truncate(prefix_len);

        // `path` is a pure relative path
        } else if need_sep {
            self.inner.push(MAIN_SEP);
        }

        self.inner.push_str(path.as_str())
    }


    /// Truncates `self` to [`self.parent`].
    /// 
    /// If [`self.filename`] was [`None`], this is equivalent to pushing `file_name`.
    /// 
    /// Otherwise it is equicalent to calling [`pop`] and then pushing `file_anem`.
    /// The new path will be a sbiling of the original path (that is, it will have the same parent.)
    /// 
    /// [`self.file_name`]: Path::file_name
    /// [`pop`]: PathBuf::pop
    pub fn pop(&mut self) -> bool {
        match self.parent().map(|p| p.as_str().len()) {
            Some(len) => {
                self.as_mut_vec().truncate(len);
                true
            },
            None => false
        }
    }

    /// Updates [`self.file_name`] to `file_name`
    /// 
    /// If [`self.filename`] was [`None`], this is equivalent to pushing `file_name`
    /// 
    /// Otherwise it is equivalent to calling [`pop`] and then pushing `file_name`.
    /// The new path will be a sibling of the original path (That is, it will have the same parent).
    /// 
    /// [`self.file_name`]: Path::file_name
    /// [`pop`]: PathBuf::pop
    pub fn set_file_name<S: AsRef<str>>(&mut self, file_name: S) {
        self._set_file_name(file_name.as_ref())
    }

    pub fn _set_file_name(&mut self, file_name: &str) {
        if self.file_name().is_some() {
            let popped = self.pop();
            debug_assert!(popped);
        }
        self.push(file_name);
    }

    /// Updates [`self.extension`] to `extension`
    /// 
    /// Returns `false` and does nothing if [`self.file_name`] is [`None`], returns `true` and updates the extension otherwise
    /// 
    /// If [`self.extension] is [`None`], the extension is added; otherwise it is replaced.
    pub fn set_extension<S: AsRef<str>>(&mut self, extension: S) -> bool {
        self._set_extension(extension.as_ref())
    }

    pub fn _set_extension(&mut self, extension: &str) -> bool {
        let file_stem = match self.file_stem() {
            None => return false,
            Some(f) => f.as_bytes(),
        };

        // truncate until right after the file stem
        let end_file_stem = file_stem[file_stem.len()..].as_ptr();
        let start = self.inner.as_bytes().as_ptr();
        let arr = self.as_mut_vec();
        arr.truncate(unsafe { end_file_stem.offset_from(start) as usize });

        // add the new extension, if any
        let new = extension.as_bytes();
        if !new.is_empty() {
            arr.reserve_exact(new.len() + 1);
            arr.push(b'.');
            arr.extend_from_slice(new);
        }
        true
    }

    /// Consumes the `PathBuf`, yielding its internal [`String`] storage.
    #[must_use = "`self` will be fropped if the result is not used"]
    #[inline]
    pub fn into_string(self) -> String {
        self.inner
    }

    /// Invokes [`capacity`] on the underlying instance of [`String`].
    /// 
    /// [`capacity`]: String::capacity
    #[must_use]
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    // Invokes [`clear`] on the underlying instance of [`String`].
    /// 
    /// [`clear`]: String::clear
    #[must_use]
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    // Invokes [`clear`] on the underlying instance of [`String`].
    /// 
    /// [`clear`]: String::clear
    #[must_use]
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    // Invokes [`clear`] on the underlying instance of [`String`].
    /// 
    /// [`clear`]: String::clear
    #[must_use]
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve(additional)
    }

    // Invokes [`clear`] on the underlying instance of [`String`].
    /// 
    /// [`clear`]: String::clear
    #[must_use]
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional)
    }

    // Invokes [`clear`] on the underlying instance of [`String`].
    /// 
    /// [`clear`]: String::clear
    #[must_use]
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve_exact(additional)
    }

    // Invokes [`clear`] on the underlying instance of [`String`].
    /// 
    /// [`clear`]: String::clear
    #[must_use]
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }

    // Invokes [`clear`] on the underlying instance of [`String`].
    /// 
    /// [`clear`]: String::clear
    #[must_use]
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity)
    }

    /// Null terminate the path
    pub fn null_terminate(&mut self) {
        self.inner.null_terminate();
    }
}

impl Clone for PathBuf {
    #[inline]
    fn clone(&self) -> Self {
        PathBuf { inner: self.inner.clone() }
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner.clone_from(&source.inner)
    }
}

impl<T: ?Sized + AsRef<str>> From<&T> for PathBuf {
    fn from(s: &T) -> Self {
        PathBuf::from_str(s.as_ref())
    }
}

impl From<String> for PathBuf {
    /// Converts a [`String`] into a [`PathBuf`]
    /// 
    /// This conversion does not allocate or copy memory
    fn from(s: String) -> Self {
        PathBuf { inner: s }
    }
}

impl From<PathBuf> for String {
    fn from(path_buf: PathBuf) -> Self {
        path_buf.inner
    }
}

impl FromStr for PathBuf {
    type Err = core::convert::Infallible;

    /// Create a `PathBuf` from a [`str`] slice, using the default allocator
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PathBuf::from_str(s))
    }
}

impl<P: AsRef<Path>> iter::FromIterator<P> for PathBuf {
    fn from_iter<I: IntoIterator<Item = P>>(iter: I) -> Self {
        let mut buf = PathBuf::new();
        buf.extend(iter);
        buf
    }
}

impl<P: AsRef<Path>> iter::Extend<P> for PathBuf {
    fn extend<I: IntoIterator<Item = P>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |p| self.push(p));
    }

    //#[inline]
    //fn extend_one(&mut self, item: P) {
    //    self.push(p.as_ref())
    //}
}

impl fmt::Debug for PathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl Deref for PathBuf {
    type Target = Path;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Path::new(&self.inner)
    }
}

impl Borrow<Path> for PathBuf {
    #[inline]
    fn borrow(&self) -> &Path {
        self.deref()
    }
}

impl Default for PathBuf {
    #[inline]
    fn default() -> Self {
        PathBuf::new()
    }
}

impl PartialEq for PathBuf {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.components() == other.components()
    }
}

impl Eq for PathBuf {}

impl PartialOrd for PathBuf {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(compare_components(self.components(), other.components()))
    }
}

impl Ord for PathBuf {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        compare_components(self.components(), other.components())
    }
}

impl Hash for PathBuf {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_path().hash(state)
    }
}

impl AsRef<str> for PathBuf {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.inner[..]
    }
}

//--------------------------------------------------------------

/// A slice of a path (akin to [`str`]).
/// 
/// This type supports a number of operations for inspecting a pth, including breaking the path into its components (separated by either `/` or `\` on Windows, and `/` on Posix)
/// 
/// This is an *unsized* type, meaning that it must always be used behind a pointer like `&`.
/// For an owned version of this type, see [`PathBuf`]
pub struct Path {
    inner: str
}

/// An error returned from [`Path::strip_prefix`] if the prefix was not found.
/// 
/// This `struct` is created by the [`strip_prefix`] method on [`Path`].
/// See its documentation for more
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StripPrefixError(());

impl Path {
    /// Directly wraps a string slice a `Path` slice.
    /// 
    /// This is a const-free conversions.
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Path {
        unsafe { &*(s.as_ref() as *const str as *const Path) }
    }

    /// Yields the underlying [`str`] slice.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Converts a `Path` into an owned [`PathBuf`]
    #[must_use = "this returns the result of the opration, without modifying the original"]
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from_str(&self.inner)
    }

    /// Converts a `Path` into an owned [`PathBuf`] and null terminates it
    #[must_use = "this returns the result of the opration, without modifying the original"]
    pub fn to_null_terminated_path_buf(&self) -> PathBuf {
        let mut buf = PathBuf::from_str(&self.inner);
        buf.null_terminate();
        buf
    }

    // NOTE(jel): Rust also supports Redox in this call, something we are currently not doing, or care about atm
    /// Returns `true` if the `Path` is absolute, i.e. if it is independent from the current directory
    /// 
    /// * On Windows, a path is absolute if it has a prefix and starts with the root: `C:\windows` is absolute, while `C:temp` and `\temp` are not
    /// * On Posix, a path is absolute if it starts with the root, so `is_absolute` and [`has_root`] are equivalent
    /// 
    /// [`has_root`]: Path::has_root
    #[must_use]
    pub fn is_absolute(&self) -> bool {
        self.has_root() && (cfg!(any(unix, target_os = "wasi")) || self.prefix().is_some())
    }

    /// Returns `true` if the `Path` is relative, i.e. not absolute
    /// 
    /// See [`is_absolute`]'s documentation for more details
    /// 
    /// [`is_absolute`]: Path::is_absolute
    #[must_use]
    #[inline]
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    fn prefix(&self) -> Option<Prefix<'_>> {
        self.components().prefix
    }

    /// Returns `true` if the `Path` has a root
    /// 
    /// * On Windows, a path has a root if it:
    ///     * has no prefix and begins with a separator, e.g. `\windows`
    ///     * has a prefix followed by a separator, e.g. `C:\windows` but not `C:windows`
    ///     * has a non-disk prefix, e.g. `\\server\share`
    #[must_use]
    #[inline]
    pub fn has_root(&self) -> bool {
        self.components().has_root()
    }

    /// Returns the `Path` without its final component, it there is one.
    /// 
    /// Returns [`None`] if the path terminates in a root or prefix
    #[must_use]
    pub fn parent(&self) -> Option<&Path> {
        let mut comps = self.components();
        let comp = comps.next_back();
        comp.and_then(|p| match p {
            Component::Normal(_) | Component::CurDir | Component::ParentDir => {
                Some(comps.as_path())
            },
            _ => None
        })
    }

    /// Produces an iterator over `Path` and its ancestors
    /// 
    /// The iterator will yield the `Path` that is returned if the [`parent`] method is used zero or more times.
    /// That means, the iterator will yield `&self`, `&self.parent().unwrap()`, `&self.parent().unwrap().parent().unwrap()`, and so on.
    /// If the [`parent`] method returns [`None`], the iterator will do likewise. The iterator will always at least on value, namely `&self`
    pub fn ancestors(&self) -> Ancestors<'_> {
        Ancestors { next: Some(&self) }
    }

    /// Returns the final component of the `Path`, if there is one
    /// 
    /// If the path is a normal file, this is the file name.
    /// It it's the path of a directory, this is the directory name
    /// 
    /// Returns[`None`] if th path terminates in `..`.
    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.components().next_back().and_then(|p| match p {
            Component::Normal(p) => Some(p),
            _ => None
        })
    }

    /// Returns a path that, when joined onta `base`, yields `self`
    /// 
    /// # Errors
    /// 
    /// If `base` is not a prefix of `self` (i.e. [`starts_with`] returns false), returns [`Err`]
    #[must_use]
    pub fn strip_prefix<P : AsRef<Path>>(&self, base: P) -> Result<&Path, StripPrefixError> {
        self._strip_prefix(base.as_ref())
    }

    fn _strip_prefix(&self, base: &Path) -> Result<&Path, StripPrefixError> {
        iter_after(self.components(), base.components())
            .map(|c| c.as_path())
            .ok_or(StripPrefixError(()))
    }

    /// Determines wheter `base` is a prefix of `self`.
    /// 
    /// Only considers whole path components to match.
    #[must_use]
    pub fn starts_with<P: AsRef<Path>>(&self, base: P) -> bool {
        self._starts_with(base.as_ref())
    }

    fn _starts_with(&self, base: &Path) -> bool {
        iter_after(self.components(), base.components()).is_some()
    }

    /// Determines whterh `child` is a suffix of `self`.
    /// 
    /// Only considers whole path components to match
    pub fn ends_with<P: AsRef<Path>>(&self, child: P) -> bool {
        self._ends_with(child.as_ref())
    }

    fn _ends_with(&self, child: &Path) -> bool {
        iter_after(self.components().rev(), child.components().rev()).is_some()
    }

    /// Extract the stem (non-extension) porion of [`self.file_name`].
    /// 
    /// The stem is:
    /// * [`None`], if there is not file name;
    /// * The entire file name if there is no embedded '.';
    /// * The entire file name if the file name begins with `.` and has no other `.`s within;
    /// * Otherwise, the portion of the file name before the final `.`
    ///
    /// [`self.file_name`]: Path::file_name
    #[must_use]
    pub fn file_stem(&self) -> Option<&str> {
        self.file_name().map(rsplit_file_at_dot).and_then(|(before, after)| before.or(after))
    }

    /// Extracts the prefix of [`self.file_name`]
    /// 
    /// The prefix is:
    /// * [`None`], if ther eis no file name;
    /// * The entire file name if there is no embedded '.';
    /// * The portion of hte file name before the first non-beginning '.';
    /// * The entire file name if the file name begins with `.` and has no other `.`s within;
    /// * The protion of the file name before the second `.` if the file name begins with a `.`
    /// 
    /// [`self.file_name`]: Path::file_name
    #[must_use]
    pub fn file_prefix(&self) -> Option<&str> {
        self.file_name().map(split_file_at_dot).and_then(|(before, _after)| Some(before))
    }

    /// Extract the extension of [`self.file_name`], if possible.
    /// 
    /// The extension is:
    /// * [`None`], if there is no file name;
    /// * [`None`], if there is no embedded `.`;
    /// * [`None`], if the file name begins with a `.` and has no other `.`s within;
    /// Otherwise, the protion of the file name after the final `.`
    /// 
    /// [`self.file_name`]: Path::file_name
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.file_name().map(rsplit_file_at_dot).and_then(|(before, after)| before.and(after))
    }

    /// Creates an owned [`PathBuf`] with ` paht ` ajoinged to `self`.
    /// 
    /// See [`PathBuf::push`] for more details on what it means to adjoin a path
    #[must_use]
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self._join(path.as_ref())
    }

    fn _join(&self, path: &Path) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.push(path);
        buf
    }

    /// Creates an owned [`PathBuf`] like `self` but with the given file name.
    /// 
    /// See [`PathBut::set_file_name`] for more details
    #[must_use]
    pub fn with_file_name<S: AsRef<str>>(&self, file_name: S) -> PathBuf {
        self._with_file_name(file_name.as_ref())
    }

    fn _with_file_name(&self, file_name: &str) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.set_file_name(file_name);
        buf
    }

    /// Creates an owned [`PathBuf`] lie `self` but with the given extension.
    /// 
    /// See [`PathBuf::set_extension`] for more details
    #[must_use]
    pub fn with_extension<S: AsRef<str>>(&self, file_name: S) -> PathBuf {
        self._with_extension(file_name.as_ref())
    }

    fn _with_extension(&self, file_name: &str) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.set_extension(file_name);
        buf
    }
     
    
    /// Produces an iterator over the [`Component`]s of the path.
    /// 
    /// Then parsing the path, there is a small amount of normalization:
    /// * Repeated separators are ignored, so `a/b` and `a//b` both have `a` and `b` as components.
    /// * Occurances of `.` are normalized away, expect if they are at the beginning of the path.
    ///   For example, `a/./b`, `a/b/`, `a/b/.` and `a/b` all have `a` and `b` as components, but `./a/b` starts with an additional [`CurDir`] component
    /// * A trailing slash is normalized away, `/a/b` and `/a/b/` are equivalent.
    /// 
    /// Note that no other normalization takes place; in particular, `a/c` and `/a/b/../c` are distinct, to account for the possibility that `b` is a symbolic link (so its parent isn't `a`)
    /// 
    /// [`CurDir`]: Component::CurDir
    pub fn components(&self) -> Components<'_> {
        let prefix = parse_prefix(self.as_str());
        Components { 
            path: self.as_str(), 
            prefix, 
            has_physical_root: has_physical_root(self.as_str(), prefix),
            front: State::Prefix, 
            back: State::Body
        }
    }

    /// Produces an iterator over the apth's components viewed as [`str`] slices
    /// 
    /// For more information about the particulars of how the path is separated into components, see [`components`]
    /// 
    /// [`components`]: Path::components
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter { inner: self.components() }
    }

    // TODO(jel): function to retrieve file metadata, unction to canonicalize the path, function to reads a link an returns the path it points to, etc: See rust's path

}

impl AsRef<str> for Path {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl PartialEq for Path {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.components() == other.components()
    }
}

impl Eq for Path {}

impl PartialOrd for Path {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(compare_components(self.components(), other.components()))
    }
}

impl Ord for Path {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        compare_components(self.components(), other.components())
    }
}

impl Hash for Path {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bytes = self.as_str().as_bytes();
        let (prefix_len, verbatim) = match parse_prefix(&self.inner) {
            Some(prefix) => {
                prefix.hash(state);
                (prefix.len(), prefix.is_verbatim())
            },
            None => (0, false)
        };
        
        let bytes = &bytes[prefix_len..];

        let mut component_start = 0;
        let mut bytes_hashed = 0;

        for i in 0..bytes.len() {
            let is_sep = if verbatim { is_verbatim_sep(bytes[i]) } else { is_sep_byte(bytes[i]) };
            if is_sep {
                if i > component_start {
                    let to_hash = &bytes[component_start..i];
                    state.write(to_hash);
                    bytes_hashed += to_hash.len();
                }

                // skip over separator and optionally a following CurDir item since components() would normalize these away
                component_start = i + 1;

                let tail = &bytes[component_start..];

                component_start += match tail {
                    [b'.'] => 1,
                    [b'.', sep @ _, ..] if self::is_sep_byte(*sep) => 1,
                    _ => 0
                };
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

impl AsRef<Path> for Path {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}

impl AsRef<Path> for str {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for String {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl<'a> IntoIterator for &'a PathBuf {
    type Item = &'a str;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

macro_rules! impl_cmp {
    ($lhs:ty, $rhs: ty) => {
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                <Path as PartialEq>::eq(self, other)
            }
        }

        impl<'a, 'b> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                <Path as PartialEq>::eq(self, other)
            }
        }

        impl<'a, 'b> PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<cmp::Ordering> {
                <Path as PartialOrd>::partial_cmp(self, other)
            }
        }

        impl<'a, 'b> PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<cmp::Ordering> {
                <Path as PartialOrd>::partial_cmp(self, other)
            }
        }
    };
}

impl_cmp!(PathBuf, Path);
impl_cmp!(PathBuf, &'a Path);


//--------------------------------------------------------------

struct PrefixParser<'a, const LEN: usize> {
    path   : &'a str,
    prefix : [u8; LEN],
} 

impl<'a, const LEN: usize> PrefixParser<'a, LEN> {
    #[inline]
    fn get_prefix(path: &str) -> [u8; LEN] {
        let mut prefix = [0; LEN];
        // SAFETY: Only ASCII characters are modified
        for (i, &ch) in path.as_bytes().iter().take(LEN).enumerate() {
            prefix[i] = ch;
        }
        prefix
    }

    fn new(path: &'a str) -> Self {
        Self { path: path, prefix: [0; LEN] }
    }

    fn as_slice(&self) -> PrefixParseSlice<'a, '_> {
        PrefixParseSlice { path: self.path, prefix: &self.prefix[..LEN.min(self.path.len())], index: 0 }
    }
}

struct PrefixParseSlice<'a, 'b> {
    path  : &'a str,
    prefix: &'b [u8],
    index : usize
}

impl<'a> PrefixParseSlice<'a, '_> {
    fn strip_prefix(&self, prefix: &str) -> Option<Self> {
        self.prefix[self.index..]
            .starts_with(prefix.as_bytes())
            .then(|| Self { index: self.index + prefix.len(), ..*self })
    }

    fn prefix_bytes(&self) -> &'a [u8] {
        &self.path.as_bytes()[..self.index]
    }

    fn finish(self) -> &'a str {
        &self.path[self.index..]
    }
}

pub fn parse_prefix(path: &str) -> Option<Prefix<'_>> {
    use Prefix::*;

    let parser = PrefixParser::<8>::new(path);
    let parser = parser.as_slice();

    if let Some(parser) = parser.strip_prefix(r"\\") {
        // \\
        //
        // The meaning of verbatim paths can chagne when they use a different separator
        if let Some(parser) = parser.strip_prefix(r"?\") {
            if !parser.prefix_bytes().iter().any(|&x| x == b'/') {
                // \\?\
                if let Some(parser) = parser.strip_prefix(r"UNC\") {
                    let path = parser.finish();
                    let (server, path) = parse_next_component(path);
                    let (share, _) = parse_next_component(path);

                    return Some(VerbatimUNC(server, share));
                } else {
                    let path = parser.finish();

                    // in verbatim paths only recognize the exact drive
                    if let Some(drive) = parse_drive_exact(path) {
                        return Some(VerbatimDisk(drive));
                    } else {
                        let (prefix, _) = parse_next_component(path);
                        return Some(Verbatim(prefix));
                    }
                }
            }
        } 
        if let Some(parser) = parser.strip_prefix(r".\") {
            // \\.\COM42
            let path = parser.finish();
            let (prefix, _) = parse_next_component(path);
            Some(DeviceNS(prefix))
        } else {
            let path = parser.finish();
            let (server, path) = parse_next_component(path);
            let (share, _) = parse_next_component(path);

            if !server.is_empty() && !share.is_empty() {
                Some(UNC(server, share))
            } else {
                // no valid prefix beginning with "\\" recognized
                None
            }
        }
    } else if let Some(drive) = parse_drive(path) {
        // C:
        Some(Disk(drive))
    } else {
        // no prefix
        None
    }
}

// Parses a drive prefix, e.g. "C:" an "C:\whatever"
fn parse_drive(path: &str) -> Option<u8> {
    // In most DOS systems, it is not possible to have more than 26 drive letters.
    // See <https://en.wikipedia.org/wiki/Drive_letter_assignment#Common_assignments>.
    fn is_valid_drive_letter(drive: &u8) -> bool {
        drive.is_ascii_alphabetic()
    }

    match path.as_bytes() {
        [drive, b':', ..] if is_valid_drive_letter(drive) => Some(drive.to_ascii_uppercase()),
        _ => None
    }
}

// Parses a drive prefix exactly, e.g. "c:"
fn parse_drive_exact(path: &str) -> Option<u8> {
    // only parse two bytes: the drive letter and the driver separator
    if path.as_bytes().get(2).map(|&x| is_sep_byte(x)).unwrap_or(true) {
        parse_drive(path)
    } else {
        None
    }
}

// Parse the next path component
//
// Returns the next component and the rest of the path excluding the component and separator.
fn parse_next_component(path: &str) -> (&str, &str) {
    match path.bytes().position(|x| is_sep_byte(x)) {
        Some(separator_start) => {
            let separator_end = separator_start + 1;
            let component = &path.as_bytes()[..separator_start];

            // Panic safe
            // The max 'separator_end` is `bytes.len()` and bytes[bytes.len()..]` is a valid index
            let path = &path.as_bytes()[separator_end..];

            unsafe { (core::str::from_utf8_unchecked(component), core::str::from_utf8_unchecked(path)) }
        },
        None => (path, "")
    }
}
