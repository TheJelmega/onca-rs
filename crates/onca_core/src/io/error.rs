use core::{
    fmt,
    result,
    mem::{size_of, align_of},
    ptr::NonNull,
    marker::PhantomData
};
use std::{error, any::Any, fmt::Debug};

use crate::{mem::HeapPtr, alloc::{Allocation, UseAlloc}};


// TODO(jel): Split off the errors for their relavent systems, i.e. file system related errors only in onca_fs. if possible

/// A specialized [`Result`] type for I/O operations
/// 
/// This type is broadly uses across [`onca_core::io`] for any operation which may produce an error
/// 
/// This typedef is generally used to avoid writin out [`io::Error`] directly and is otherwise a direct mapping to [`Result`].
/// 
/// While usual Rust style is to import types directly, aliases of [`Result`] often are not, to make it easier to distinguish between them.
/// [`Result`] is generally assumed to be [`core::result::Result`][`Result`], and so users of this alias will generally use `io::Result` instead of shadowing the [prelude]'s import of [`core::result::Result`][`Result`].
/// 
/// [`onca_core::io`]: crate::io
/// [`io::Error`]: Error
/// [`Result`]: core::result::Result
/// [prelude]: core::prelude
/// 
pub type Result<T> = core::result::Result<T, Error>;

/// The error type for I/Ooperations of the [`Read`], [`Write`], [`Seek`], and associated traits.
/// 
/// Error mostly originiate from the undrlying OS, but custom instances of `Error` can be created with crafted error messages and a partivular calue of [`ErrorKind`]
/// 
/// [`Read`]: crate::io::Read
/// [`Write`]: crate::io::Write
/// [`Seek`]: crate::io::Seek

pub struct Error {
    repr: Repr
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

enum ErrorData<C> {
    Os(i32),
    Simple(ErrorKind),
    SimpleMessage(&'static SimpleMessage),
    Custom(C)
}

// `#[repr(align(4))`] is probably redundant, it should have that value or higher already.
// It's included just because repr_bitpacked's encoding require an alignment >= 4 (note that `#[repr(align)]` will not reduce the alignment required by the struct, only increase it).
//
// If we add more variants or ErrorData, this can be increased to 8.
#[repr(align(4))]
#[derive(Debug)]
pub(crate) struct SimpleMessage {
    kind    : ErrorKind,
    message : &'static str,
}

impl SimpleMessage {
    pub(crate) const fn new(kind: ErrorKind, message: &'static str) -> Self {
        Self { kind, message }
    }
}

macro_rules! const_io_error {
    ($kind:expr, $message:expr) => {
        $crate::io::error::Error::from_static_message({
            const MESSAGE_DATA: $crate::io::error::SimpleMessage = $crate::io::error::SimpleMessage::new($kind, $message);
            &MESSAGE_DATA
        })
    };
}
pub(crate) use const_io_error;

// As with 'SimpleMessage`: `$[repr(align(4))]` here is just because repr_bitpacked's encoding requires it.
// In practice it almost certainly will already be this high or higher
#[derive(Debug)]
#[repr(align(4))]
struct Custom {
    kind  : ErrorKind,
    error : HeapPtr<dyn error::Error + Send + Sync>
}

/// A list specifgying general categories of I/O error
/// 
/// This list is intended to change over time and it is not recommended to exhustively match agains it.
/// 
/// It is used with the ['io::Error`] type.
/// 
/// # Handling errors and matching on `ErrorKind`
/// 
/// In application code, use `match` for the `ErrorKind` values you are expecting; use `_` to match "all other errors"
/// 
/// In comprehensive and thorough tests that want to verify that a test doesn't return any known incorrect error kind, you may want to cut-and-pase the current list of errors from here into your test code,
/// and then match '_' as the correct case. 
/// This seems counterintuitive, but it will make your tests more robust.
/// In particular, if you want to verify that your code does produce an unrecognized error kind, the robust solution is to check for all the recognized error kinds and fail in those cases.
/// 
/// [`io::Error`]: Error
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ErrorKind {
    /// A socket address could not be bound because the addresss is already in use elsewhere.
    AddrInUse,
    /// A nonexistent interface was requested or the requested address was not local.
    AddrNotAvailable,
    /// An entity already exists, often a file.
    AlreadyExists,
    /// Program argument list too long
    /// 
    /// When trying to run an external program, a system or process limit on the size of the arguments would have been exceeded.
    ArgumentListTooLong,
    /// The operation failed because a pipe was closed.
    BrokenPipe,
    /// The connection was aborted (terminated) by the remote server.
    ConnectionAborted,
    /// The connection was refused by the remote server.
    ConnectionRefused,
    /// The connection was reset by the remote server.
    ConnectionReset,
    /// Cross-device or cross-filesystem (hard) link or rename.
    CrossesDevices,
    /// Deadlock (avoided).
    /// 
    /// A file locking operation would result in deadlock. This situation is typically detected, if at all, on a best-effor basis
    Deadlock,
    /// The filesystem object is, unexpectedly, a directory.
    /// 
    /// A directory was specified when a non-directory was expected.
    DirectoryNotEmpty,
    /// Executable file is busy.
    /// 
    /// An attempt was made to write to a file which is also in use as a running program. (Not all operation systems detect this situation.)
    ExecutableFileBusy,
    /// File larger than allowed or supported.
    /// 
    /// This might arise from a hard limit of the underlying filesystem or file access API, or from an administratively imposed resouce limitation.
    /// Simple disk full, and out of quota, have their own errors
    FileTooLarge,
    /// Loop in the filesystem or IO subsystem; often too many levels of symbolic links.
    /// 
    /// There was a loop (or excessively long chain) resolbing a filesystem object or file IO object.
    /// 
    /// On Unic this is usually the result of a symbolic link loop; or, of exceeding the system specific limit on the depth of symlink traversal.
    FilesystemLoop,
    /// Filesystem quota was exceeded.
    FilesystemQuotaExceeded,
    /// The remote host is not reachable.
    HostUnreachable,
    /// This operation was interrupted.
    /// 
    /// Interrrupted operations can typically be retried.
    Interrupted,
    /// Data not valid for the operation were encountered.
    /// 
    /// Unlike [`InvalidInput`], this typically means that the operation parameters were valid, however teh error was caused by malformed input data.
    /// 
    /// For example, a function that reads a file into a string will error with `InvalidData` if the file's contents are not valid UTF-8.
    /// 
    /// [`InvalidInput`]: ErrorKind::InvalidInput
    InvalidData,
    /// A filename was invalid
    /// 
    /// This error can also be caused if it exceeded the filename length limit.
    InvalidFilename,
    /// A parameter was incorrect.
    InvalidInput,
    /// The file system object is, unexpectedly, a directory
    /// 
    /// A directory was specified when a non-directory was expected.
    IsADirectory,
    /// The system's networking is down.
    NetworkDown,
    /// The network containing the remote host is not reachable.
    NetworkUnreachable,
    /// A filesystem object is, unexpectedly, not a directory.
    /// 
    /// For example, a filesystem path was specified where one of the intermediate directory components was, in fact, a plain file.
    NotADirectory,
    /// The network operation failed because it was not connected yet.
    NotConnected,
    /// An entity was not found, often a file.
    NotFound,
    /// Seek on unseekable file.
    /// 
    /// Seeking was attempted on an open file handle which is not suitable for seeking - for example, on Unix, a named pipe with `File::open`.
    NotSeekable,
    /// The operation lacked the necessary privileges to complete.
    PermissionDenied,
    /// The filesystem or storage medium is read-only, but a write operaton was attempted.
    ReadOnlyFilesystem,
    /// resource is busy.
    ResourceBusy,
    /// Stale network file handle.
    /// 
    /// With some network filesystems, notably NFS, an open file (or directory) can be invalidated by problems wit hthe network or server.
    StaleNetworkFileHandle,
    /// The underlying storage (typically a filesystem) is full.
    /// 
    /// This does not include out of quota erros.
    StorageFull,
    /// The I/O operation's timout expired, causing it to be cancelled.
    Timeout,
    /// Too many (hard) links to the same filesystem object.
    /// 
    /// The filesystem does not support making so many hardlinks to the same file.
    TooManyLinks,
    /// The operation needs to block to complete, but the blocking operation was requested to not occur.
    WouldBlock,
    /// An error returned when an operation could not be completed because a call to [`write`] returned [`OK(0)`].
    /// 
    /// This typically means that an operation could only succeed if it wrote a particular number of byte, but only a smaller number of bytes could be written.
    /// 
    /// [`write`]: crate::io::Write::write
    /// [`Ok(0)`]: Ok
    WriteZero,

    /// This operation is unsupported on this platform.
    /// 
    /// This means that the operation can never succeed.
    Unsupported,

    // Error kinds which are primarily categorisations for OS error codes should be added above

    /// An error returned when an operation could not be completed because an "end of ifle" was reached prematurely.
    /// 
    /// This typically means that an operaton could only succeed if it read a partivclar number of bytes, but only a smaller number of bytes could be read.
    UnexpectedEof,

    /// An opertion could not be completed, because it failed to allocate enough memory
    OutOfMemory,

    // "Unusual" error kinds which do not correspond simple to (sets of) OS error codes, should be added just above this comment.
    // `Other` and `Uncategorised` should remain at the end

    /// A custom error that does not fall under any otehr I/O error kind.
    /// 
    /// This can be used to contruct your own [`Error`]s that do not match any [`ErrorKind`].
    Other,

    /// Any I/O error that's not part of this list.
    /// 
    /// Errors are `Uncategorized` now may move to a different or a new [`ErrorKind`] variant in the future. It is not recommeded to match an error against `Uncategorized`; 
    /// use a widlcard match (`_`) instead.
    Uncategorized
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        use ErrorKind::*;
        match *self {
            AddrInUse               => "address in use",
            AddrNotAvailable        => "address not available",
            AlreadyExists           => "entity already exists",
            ArgumentListTooLong     => "argument list too long",
            BrokenPipe              => "boken pipe",
            ConnectionAborted       => "connection aborted",
            ConnectionRefused       => "connection refused,",
            ConnectionReset         => "connection reset",
            CrossesDevices          => "cross-device link or rename",
            Deadlock                => "deadlock",
            DirectoryNotEmpty       => "directory not empty",
            ExecutableFileBusy      => "executable file busy",
            FileTooLarge            => "file too large",
            FilesystemLoop          => "filesystem loop or indirection limit (e.g. symlink loop)",
            FilesystemQuotaExceeded => "filesystem quota exceeded",
            HostUnreachable         => "host unreachable",
            Interrupted             => "operation interrupted",
            InvalidData             => "invalid data",
            InvalidFilename         => "invalid filename",
            InvalidInput            => "invalid input parameter",
            IsADirectory            => "is a directory",
            NetworkDown             => "network down",
            NetworkUnreachable      => "network unreachable",
            NotADirectory           => "not a directory",
            NotConnected            => "not connected",
            NotFound                => "entity not found",
            NotSeekable             => "seek on unseekable file",
            PermissionDenied        => "permission denied",
            ReadOnlyFilesystem      => "read-only filesystem or storage medium",
            ResourceBusy            => "resource busy",
            StaleNetworkFileHandle  => "stale network file handle",
            StorageFull             => "no storage space",
            Timeout                 => "timed out",
            TooManyLinks            => "too many links",
            WouldBlock              => "operaton would block",
            WriteZero               => "write zero",
            Unsupported             => "unsupported",
            UnexpectedEof           => "unexpected end of file",
            OutOfMemory             => "out of memory",
            Other                   => "other error",
            Uncategorized           => "uncategorized error",
        }
    }
}

impl fmt::Display for ErrorKind {
    /// Shows a human-readable description of the `ErrorKind`.
    /// 
    /// THs is similar to `impl Display for Error`, but doesn't require first converting to Error
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<ErrorKind> for Error {
    /// Converts an [`ErrorKind`] into an [`Error`].
    /// 
    /// This conversion creates a new error with a simple representation of error kind.
    fn from(kind: ErrorKind) -> Self {
        Error { repr: Repr::new_simple(kind) }
    }
}

impl Error {
    /// Creates a new I/O error from a known kind of error as well as an arbitrary error payload.
    /// 
    /// This function is used to generically create I/O errors which do not originate from the OS itself.
    /// The `error` argument is an arbitrary payload which will be contained in this [`Error`].
    /// 
    /// If no extra payload is required, used the `From` conversion from `ErrorKind`.
    pub fn new<E>(kind: ErrorKind, error: E, alloc: UseAlloc) -> Error
    where
        E : Into<HeapPtr<dyn error::Error + Send + Sync>>,
    {
        Self::_new(kind, error.into(), alloc)
    }

    /// Creates a new I/O error from an arbitrary error payload.
    /// 
    /// This function is used to generically create I/O errors which do not originate from the OS itself.
    /// It is a shortcut for [`Error::new`] with [`ErrorKind::Other`].
    pub fn other<E>(error: E, alloc: UseAlloc) -> Error
    where
        E: Into<HeapPtr<dyn error::Error + Send + Sync>>
    {
        Self::_new(ErrorKind::Other, error.into(), alloc)
    }

    fn _new(kind: ErrorKind, error: HeapPtr<dyn error::Error + Send + Sync>, alloc: UseAlloc) -> Error {
        Error { repr: Repr::new_custom(HeapPtr::new(Custom{ kind, error }, alloc)) }
    }

    /// Creates a new I/O error from a known kind of error as well as a xonstant message.
    /// 
    /// This function does not allocate.
    /// 
    /// You should not use this directly, and instead use the `const_io_error!` macro: `io::const_io_error!(ErrorKind::Something, "some_message").
    pub(crate) const fn from_static_message(msg: &'static SimpleMessage) -> Error {
        Self { repr: Repr::new_simple_message(msg) }
    }

    /// Create a new instance of an [`Error`] from a particular OS error code.
    #[must_use]
    #[inline]
    pub fn from_raw_os_error(code: i32) -> Error {
        Error { repr: Repr::new_os(code) }
    }

    /// Returns the OS error that this error represents (if any).
    /// 
    /// If this [`Error`] was constructed via [`from_raw_os_error`], then this function will return [`Some`], otherwise it will return [`None`].
    ///
    /// [`from_raw_os_error`]: Error::from_raw_os_error
    #[must_use]
    #[inline]
    pub fn raw_os_error(&self) -> Option<i32> {
        match self.repr.data() {
            ErrorData::Os(i)            => Some(i),
            ErrorData::Simple(_)        => None,
            ErrorData::SimpleMessage(_) => None,
            ErrorData::Custom(_)        => None,
        }
    }

    /// Returns a reference to the inner error wrapped by this erro (if any).
    /// 
    /// If this [`Error`] was constructed via [`new`] or [`other`], then this function will return [`Some`], otherwise it will return [`None`]
    /// 
    /// [`new`]: Error::new
    /// [`other`]: Error::other
    pub fn get_ref(&self) -> Option<&(dyn error::Error + Send + Sync + 'static)> {
        match self.repr.data() {
            ErrorData::Os(_)            => None,
            ErrorData::Simple(_)        => None,
            ErrorData::SimpleMessage(_) => None,
            ErrorData::Custom(c)        => Some(&*c.error),
        }
    }

    /// Returns a mutable reference to the inner errorwrapped by this error (if any).
    /// 
    /// If this [`Error`] was constructed via [`new`] or [`other`], then this function will return [`Some`], otherwise it will return [`None`]
    /// 
    /// [`new`]: Error::new
    /// [`other`]: Error::other
    pub fn get_mut(&mut self) -> Option<&mut (dyn error::Error + Send + Sync + 'static)> {
        match self.repr.data_mut() {
            ErrorData::Os(_)            => None,
            ErrorData::Simple(_)        => None,
            ErrorData::SimpleMessage(_) => None,
            ErrorData::Custom(c)        => Some(&mut *c.error),
        }
    }

    /// Consumes the `Error`, returning its inner error (if any).
    /// 
    /// If this [`Error`] was constructed via [`new`] or [`other`], then this function will return [`Some`], otherwise it will return [`None`]
    /// 
    /// [`new`]: Error::new
    /// [`other`]: Error::other
    pub fn into_inner(self) -> Option<HeapPtr<dyn error::Error + Send + Sync>> {
        match self.repr.into_data() {
            ErrorData::Os(_)            => None,
            ErrorData::Simple(_)        => None,
            ErrorData::SimpleMessage(_) => None,
            ErrorData::Custom(c)        => Some(c.deref_move().error),
        }
    }

    /// Attempt to downgrade the inner error to `E` (if any).
    /// 
    /// If this [`Error`] was constructed via [`new`] or [`other`], then this function will attempt to perform downgrade on it, otherwise it will return [`Err`
    /// 
    /// If downgrade succeeds, it will return [`Ok`], otherwise it will also return [`Err`]
    /// 
    /// [`new`]: Error::new
    /// [`other`]: Error::other
    pub fn downcast<E>(self) -> result::Result<HeapPtr<E>, Self>
    where
        E : error::Error + Send + Sync + 'static
    {
        match self.repr.into_data() {
            ErrorData::Custom(b) if b.error.is::<E>() => {
                unsafe { Ok(b.deref_move().error.cast::<E>()) }
            },
            repr_data => Err(Self { repr: Repr::new(repr_data) }),
        }
    }

    /// Returns the corresponding [`ErrorKind`] for this error
    #[must_use]
    #[inline]
    pub fn kind(&self) -> ErrorKind {
        match self.repr.data() {
            ErrorData::Os(_)            => ErrorKind::Uncategorized,
            ErrorData::Simple(kind)     => kind,
            ErrorData::SimpleMessage(m) => m.kind,
            ErrorData::Custom(c)        => c.kind,
        }
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data() {
            ErrorData::Os(code) => f
                .debug_struct("Os")
                .field("code", &code)
                // TODO(jel)
                .field("kind", &"os")
                .field("message", &"unknown")
                .finish(),
            ErrorData::Simple(kind) => f
                .debug_tuple("Kind")
                .field(&kind)
                .finish(),
            ErrorData::SimpleMessage(msg) => f
                .debug_struct("Error")
                .field("kind", &msg.kind)
                .field("message", &msg.message)
                .finish(),
            ErrorData::Custom(c) => fmt::Debug::fmt(&c, f),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr.data() {
            ErrorData::Os(code) => {
                let detail = "unknown";
                write!(f, "{detail} (os error {code})")
            },
            ErrorData::Simple(kind) => write!(f, "{}", kind.as_str()),
            ErrorData::SimpleMessage(msg) => fmt::Display::fmt(&msg.message, f),
            ErrorData::Custom(ref c) => c.error.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.repr.data() {
            ErrorData::Os(..) => None,
            ErrorData::Simple(..) => None,
            ErrorData::SimpleMessage(..) => None,
            ErrorData::Custom(c) => c.error.source()
        }
    }
}

/// The internal representation.
/// 
/// This is a densely packed error representation.
/// 
/// Comceptually, it stores the dame data as a plain `ErrorData<_>` equivalent that could be used instead.
/// SPecifically, you can imagine it as an optimized version of the following enum
/// ```ignore (exposition-only)
/// enum ErrorData {
///     Os(i32),
///     Simple(ErrorKind),
///     SimpleMessage(&'static SimpleMessage),
///     Custom(HeapPtr<Error>)
/// }
/// ```
/// 
/// However, it packs this data into a 128bit non-zero value.
/// 
/// This optimization not only allows `io::Error` to occupy just 2 pointers, but improves `io::Result` as well, expecially for situations like `io::Result<()>` (which is not 126-bits),
/// or `io::Result<u64>` (which is now 192 bits).
/// 
/// # Layout
/// 
/// Tagged values are 128 bits, with the 2 least significant bits of the first pointer used for the tag. THis means there are 4 "variants".
/// 
/// - **Tag 0b00**: The first variant is equivalent to `ErrorData::SimpleMessage` and hods a `&'static SimpleMessage` directly.
/// 
///   `SimpleMessage` has an alignment >= 4 (which is requested with 1#[repr(align)]` and checked statically at the bottom of this file), 
///   which means every `&'static SimpleMessage` should have both tag bits as 0, meaning its tagged and untagged representation are equivalent
/// 
///   This means we can skip tagging it, which is necessary as this variant can be constructed from a `const fn`, which probably cannot tag pointers (or at least it would be very difficult)
/// 
/// - **Tag 0b01**: The other pointer variant holds the data for `ErrorData::Custom` and ther remaining 62 bits are used to store the pointer part of a `HeapPtr<Custom>`.
///   `Custom` also has an alignment of >= 4, so the bottom two bits are free to use for the tag
/// 
///   The only important thing to note is that `ptr::wrapping_add` and `ptr::wrapping_sub` are used to tag the pointer, rather than bitwise operations.
/// This should perserve the pointer's provenenace, which would otherwise be lost.
/// 
/// - **Tag 0b10**: Hold the data for ErroData::OS(i32)`.
///   We store the `i32` in the pointer's most significant 32 bits, an don't use the bits `2..32` for anything.
///   Using the top 32 bits is just to let use easily recover the `i32` code with the correct sign
/// 
/// - **Tag 0b11**: Holds the data for `ErroData::Simple(ErrorKind)`. This stores the `ErrorKind` in the top 32 bits as well, although it doesn't occupy nearly that many.
///   Most of the bits are unused here, but it's not like we need them for anything else yet.
/// 
/// # Use of `(NonNull<()>, u64)`
/// 
/// Everything is stored in a `(NonNull<()>, u64)`, which is off, but actually serves a purpose.
/// 
/// Comceptually, you might think of this more like
/// 
/// ```ignore (exposition-only)
/// union Repr {
///     // holds integer (Simple/OS) variants, and provides access to the tags
///     bits: NonZero64,
///     // Tag is 0, so this is stored untagged
///     msg: &'static SimpleMessage
///     // Tagged (offset) `HeapPtr<Custom>` pointer.
///     tagged_custom: NonNull<()>
/// }
/// ```
/// 
/// But there are a few problems with this:
/// 
/// 1. Union access is equivalent to a transmute, so this representation would require we transmute between inegers and pointer in at least one direction,
///    which may be UB (and even if not, it is likely harder for a compiler to reason about than explicit ptr->int operations)
/// 
/// 2. Even if all fields of a union have a niche, the union itself doesn't, althoguh this may change in the future.
/// This would make things like `io::Result<()>` and `io::Result<usize>` larger, which defeats part of the motivation of bitpacking.
/// 
/// Storing everything in a `(NonZeroUsize, u64)` (or some other integer) would be a bit more traditional for pointer tagging, but it would lose provenance information,
/// couldn't be constructed from a `const fn`, and would probably run into other issues as well.
/// 
/// The `NonNull<()>` seems like the only alternative, even if it's fairly off to use a pointer type to store something that may hold an integer, some of the time.

#[repr(transparent)]
struct Repr((NonNull<()>, u64), PhantomData<ErrorData<HeapPtr<Custom>>>);

unsafe impl Send for Repr {}
unsafe impl Sync for Repr {}

impl Repr {
    const TAG_MASK           : usize = 0b11;
    const TAG_SIMPLE_MESSAGE : usize = 0b00;
    const TAG_CUSTOM         : usize = 0b01;
    const TAG_OS             : usize = 0b10;
    const TAG_SIMPLE         : usize = 0b11;



    fn new(dat: ErrorData<HeapPtr<Custom>>) -> Self {
       match dat {
           ErrorData::Os(code) => Self::new_os(code),
           ErrorData::Simple(kind) => Self::new_simple(kind),
           ErrorData::SimpleMessage(simple_message) => Self::new_simple_message(simple_message),
           ErrorData::Custom(p) => Self::new_custom(p),
           
       }
    }

    fn new_custom(p: HeapPtr<Custom>) -> Self {
       let (ptr, layout) = unsafe { HeapPtr::leak(p).into_raw() };
       let ptr = ptr.as_ptr().cast::<u8>();
       /// Should only be possible if an allocator handed out a pointer with the wrong alignment
       debug_assert_eq!(ptr.addr() & Self::TAG_MASK, 0);
       // Note: Weknow `TAG_CUSTOM <= size_of::<Custom>` (static_asset at the end of the file), and both the start and end of the expression must be valid without address space wraparoung due to `HepPtr`'s semantics.
       //
       // This means if would be correct to implement this using `ptr::add` (rather than `ptr::wrapping_add`), but it's unclear this would give any benefit, so we just use `wrapping_add` instead.
       let tagged = ptr.wrapping_add(Self::TAG_CUSTOM).cast::<()>();
       // Safety: `TAG_CUSTOM + p` is the same as `TAG_CUSTOM | p`, because  `p`'s alignment means it isn't allowed to have any of the `TAG_BITS` set 
       // (you can verify that additional and bitwise-or are the same when the operations have no bits in common using a truth table)
       //
       // Then, `TAG_CUSTOM | p` is not zero, as that would require `TAG_CUSTOM` and `p` both to be zero, and neither is (as `p` came from a HeapPtr, and `TAG_CUSTOM` just.. isn't zero -- it's `0b01`).
       // Therefore, `TAG_CUSTOM + p` isn't zero and so `tagged` can't be, and the `new_unchecked` is safe
       let res = Self(unsafe{ (NonNull::new_unchecked(tagged) , core::mem::transmute(layout)) }, PhantomData);
       // quickly smoke-check we encoded the right thing (This generally will only run in tests)
       debug_assert!(matches!(res.data(), ErrorData::Custom(_)), "repr(custom) encoding failed");
       res
    }

    fn new_os(code: i32) -> Self {
        let untagged = ((code as usize) << 32) | Self::TAG_OS;
        // Safety: `TAG_OS`` is not zero, so the result of the `|` is not 0
        let res = Self((unsafe { NonNull::new_unchecked(core::ptr::invalid_mut(untagged)) }, 0), PhantomData);
        // quickly smoke-check we encoded the right thing (This generally will only run in tests)
        debug_assert!(matches!(res.data(), ErrorData::Os(c) if c == code), "repr(os) encoding failed for {code}");
        res
    }

    fn new_simple(kind: ErrorKind) -> Self {
        let untagged = ((kind as usize) << 32) | Self::TAG_SIMPLE;
        // Safety: `TAG_SIMPLE`` is not zero, so the result of the `|` is not 0
        let res = Self((unsafe { NonNull::new_unchecked(core::ptr::invalid_mut(untagged)) }, 0), PhantomData);
        // quickly smoke-check we encoded the right thing (This generally will only run in tests)
        debug_assert!(matches!(res.data(), ErrorData::Simple(k) if k == kind), "repr(os) encoding failed for {:?}", kind);
        res
    }

    const fn new_simple_message(m: &'static SimpleMessage) -> Self {
        // Safety: References are never null
        Self((unsafe { NonNull::new_unchecked(m as *const _ as *mut ()) }, 0), PhantomData)
    }

    fn data(&self) -> ErrorData<&Custom> {
        // Safety: We're a Repr, decode_repr is fine
        unsafe { decode_repr(self.0.0, self.0.1, |c, _| &*c) }
    }

    fn data_mut(&self) -> ErrorData<&mut Custom> {
        // Safety: We're a Repr, decode_repr is fine
        unsafe { decode_repr(self.0.0, self.0.1, |c, _| &mut *c) }
    }

    fn into_data(self) -> ErrorData<HeapPtr<Custom>> {
        // Safety: We're a Repr, decode_repr is fine
        unsafe { 
            decode_repr(self.0.0, self.0.1, |c, layout| 
                HeapPtr::from_raw_components(NonNull::new_unchecked(c), core::mem::transmute(layout))) 
        }
    }
}

 impl Drop for Repr {
    fn drop(&mut self) {
        // Safety: We're a Repr, decode_repr is fine.
        // The `HeapPtr` is safe because we're being dropped
        unsafe {
            decode_repr(self.0.0, self.0.1, |c, layout| 
                HeapPtr::from_raw_components(NonNull::new_unchecked(c), core::mem::transmute(layout)));
        }
    }
}

 unsafe fn decode_repr<C, F>(ptr: NonNull<()>, layout: u64, make_custom: F) -> ErrorData<C>
    where F: FnOnce(*mut Custom, u64) -> C
{
    let bits = ptr.as_ptr().addr();
    match bits & Repr::TAG_MASK {
        Repr::TAG_OS => {
            let code = ((bits as i64) >> 32) as i32;
            ErrorData::Os(code)
        },
        Repr::TAG_SIMPLE => {
            let kind_bits = (bits >> 32) as u32;
            let kind = kind_from_prim(kind_bits).unwrap_or_else(|| {
                debug_assert!(false, "Invalid io::error::repr bits: `Repr({:#018x})`", bits);
                // This means the `ptr` passed in was not valid, which violated the unsafe contract of `decode_repr`.
                //
                // Using this rather than unwrap meaning fully improves the code for callers which only care about one variant (usually `Custom`)
                core::hint::unreachable_unchecked();
            });
            ErrorData::Simple(kind)
        },
        Repr::TAG_SIMPLE_MESSAGE => ErrorData::SimpleMessage(&*ptr.cast::<SimpleMessage>().as_ptr()),
        Repr::TAG_CUSTOM => {
            // It would be correct for use to use `ptr::byte_sub` here (see the comment above the `wrapping_add` call in `new_custom` for why), but it isn't clear that it makes a difference, so we don't.
            let custom = ptr.as_ptr().cast::<u8>().wrapping_sub(Repr::TAG_CUSTOM).cast::<Custom>();
            ErrorData::Custom(make_custom(custom, layout))
        },
        // Can't happen
        _ => unreachable!()
    }
}

// This compiled to the same code as the check + transmute, but doesn't require unsafe, or  to har-code max ErrorKind or its size in a way the compiler couldn't verify.
#[inline]
fn kind_from_prim(ek: u32) -> Option<ErrorKind> {
    macro_rules! from_prim {
        ($prim:expr => $Enum:ident { $($Variant:ident),* $(,)? }) => {{
            // Force a compiler error if the list gets out of data
            const _: fn(e: $Enum) = |e: $Enum| match e {
                $($Enum::$Variant => ()),*
            };
            match $prim {
                $(v if v == ($Enum::$Variant as _) => Some($Enum::$Variant),)*
                _ => None,
            }
        }};
    }
    from_prim!(ek => ErrorKind {
        AddrInUse,
        AddrNotAvailable,
        AlreadyExists,
        ArgumentListTooLong,
        BrokenPipe,
        ConnectionAborted,
        ConnectionRefused,
        ConnectionReset,
        CrossesDevices,
        Deadlock,
        DirectoryNotEmpty,
        ExecutableFileBusy,
        FileTooLarge,
        FilesystemLoop,
        FilesystemQuotaExceeded,
        HostUnreachable,
        Interrupted,
        InvalidData,
        InvalidFilename,
        InvalidInput,
        IsADirectory,
        NetworkDown,
        NetworkUnreachable,
        NotADirectory,
        NotConnected,
        NotFound,
        NotSeekable,
        PermissionDenied,
        ReadOnlyFilesystem,
        ResourceBusy,
        StaleNetworkFileHandle,
        StorageFull,
        Timeout,
        TooManyLinks,
        WouldBlock,
        WriteZero,
        Unsupported,
        UnexpectedEof,
        OutOfMemory,
        Other,
        Uncategorized,
    })
}

fn _assert_error_is_sync_send() {
    fn _is_sync_send<T: Sync + Send>() {}
    _is_sync_send::<Error>();
}
 
// Some static checking to alert us if a change breaks any of the assumptions that our encoding relied on for correctness and soundness. (Some of these are a bit overly rhorough/cautious, admittedly).
//
// If any of these are hit on a platform that onca supports, we should likely just use an unpacked version there instead
macro_rules! static_assert {
    ($condition:expr) => {
        const _: () = assert!($condition);
    };
    (@usize_eq: $lhs:expr, $rhs:expr) => {
        const _ : [(); $lhs] = [(); $rhs];
    };
}

// The bitpacking we use requires pointers to be exactly 64 bits
static_assert!(@usize_eq: size_of::<NonNull<()>>(), 8);

// We also require pointers and usize to be the same size
static_assert!(@usize_eq: size_of::<NonNull<()>>(), size_of::<usize>());

// `SimpleMessage` needs to be a thin pointer
static_assert!(@usize_eq: size_of::<&'static SimpleMessage>(), 8);

// `Custom` needs to be a fat pointer
static_assert!(@usize_eq: size_of::<HeapPtr<Custom>>(), 16);

static_assert!((Repr::TAG_MASK + 1).is_power_of_two());

// And they must have sufficient alignment
static_assert!(align_of::<SimpleMessage>() >= Repr::TAG_MASK + 1);
static_assert!(align_of::<Custom>() >= Repr::TAG_MASK + 1);

static_assert!(@usize_eq: (Repr::TAG_MASK & Repr::TAG_SIMPLE_MESSAGE), Repr::TAG_SIMPLE_MESSAGE);
static_assert!(@usize_eq: (Repr::TAG_MASK & Repr::TAG_CUSTOM), Repr::TAG_CUSTOM);
static_assert!(@usize_eq: (Repr::TAG_MASK & Repr::TAG_OS), Repr::TAG_OS);
static_assert!(@usize_eq: (Repr::TAG_MASK & Repr::TAG_SIMPLE), Repr::TAG_SIMPLE);

// This is obviousle true (`TAG_CUSTOM` is `0b01`), but in `Repr::new_custom` we offset a pointer by this value, and expect it to both be with the same object, and not to wrap around the address space.
// See the comment in that function for further details.
//
// Actually, at the moment we use `ptr::wrapping_add`, not `ptr::add`, so this check isn't needed for that one, 
// although the assertion that we don't actually wrap around in that wrapping_add does simplify the safety reasoning elsewhere considerably.
static_assert!(size_of::<Custom>() >= Repr::TAG_CUSTOM);

// These two store a payload which is allowed to be zero, so they must be non-zero to preserve the `NonNull`'s range invarient
static_assert!(Repr::TAG_OS != 0);
static_assert!(Repr::TAG_SIMPLE != 0);
// We can't tag `SimpleMessage`s, the tag must be 0.
static_assert!(@usize_eq: Repr::TAG_SIMPLE_MESSAGE, 0);

// Check that the point of all of this still holds
//
// We'd check against `io::Error`, but *technically* it's allowed to vary as it's not `#[repr(transparnet)]`/`#[repr(C)]`.
// We could add that, but the `#[repr()]` would show up in rustdoc, which might be seen as a stable commitment
static_assert!(@usize_eq: size_of::<Repr>(), 16);
static_assert!(@usize_eq: size_of::<Option<Repr>>(), 16);
static_assert!(@usize_eq: size_of::<result::Result<(), Repr>>(), 16);
static_assert!(@usize_eq: size_of::<result::Result<usize, Repr>>(), 24);