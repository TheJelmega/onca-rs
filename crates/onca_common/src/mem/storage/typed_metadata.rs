use core::{
    ptr::Pointee,
    marker::Unsize
};


// Typed metadata, for type-safe APIs.
#[derive(Debug)]
pub struct TypedMetadata<T: ?Sized> {
    metadata: <T as Pointee>::Metadata,
    // Work around for https://github.com/rust-lang/rust/issues/111821
    //
    // rustc fails to realize that `Pointee::Metadata` is always `Sized`, which in case of cycles
    // may lead it to erronously reject a program due to use of a possible `!Sized` type for a non-last field.
    //
    // According to the issue, this should also work without this value and -Ztrait-solver=next
    //_sized: (),
}

impl <T: ?Sized> TypedMetadata<T> {
    /// Creates an instance from the given metadata
    pub const fn from_metadata(metadata: <T as Pointee>::Metadata) -> Self {
        Self { metadata, /*_sized: ()*/ }
    }

    pub const fn get(&self) -> <T as Pointee>::Metadata {
        self.metadata
    }

    pub const fn coerce<U: ?Sized>(&self) -> TypedMetadata<U> where
        T: Unsize<U>
    {
        let ptr: *const T = core::ptr::from_raw_parts(core::ptr::null(), self.metadata);
        let ptr: *const U = ptr;
        let (_, metadata) = ptr.to_raw_parts();

        TypedMetadata {
            metadata,
            //_sized: ()
        }
    }
}

impl<T> TypedMetadata<T> {
    /// Creates a new instance
    pub const fn new() -> Self {
        Self::from_metadata(())
    }
}

impl<T: ?Sized> Clone for TypedMetadata<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized> Copy for TypedMetadata<T> {}

impl<T> Default for TypedMetadata<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<usize> for TypedMetadata<[T]> {
    fn from(value: usize) -> Self {
        Self::from_metadata(value)
    }
}
