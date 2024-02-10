macro_rules! create_index_handle {
    ($doc:meta, $example:meta, $iden:ident => $ty:ty) => {
        #[$doc]
        /// 
        /// Stores an index and a lifetime.
        /// 
        /// The handle is the size of the provided unsigned integer, with N bits of it storing the index and the remaining bits storing a lifetime.
        #[$example]
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $iden<const N: usize>($ty);

        impl<const N: usize> $iden<N> {
            pub const ID_BITS:       usize = N;
            pub const LIFETIME_BITS: usize = core::mem::size_of::<$ty>() * 8 - N;
        
            pub const MAX_ID:        $ty = (1 << N) - 1;
            pub const MAX_LIFETIME:  $ty = (1 << Self::LIFETIME_BITS) - 1;
        
            /// Create a new handle.
            pub fn new(index: $ty, lifetime: $ty) -> Self {
                assert!(index <= Self::MAX_ID);
                assert!(lifetime <= Self::MAX_LIFETIME);
            
                Self(index | (lifetime << Self::LIFETIME_BITS))
            }
        
            /// Get the index of the handle.
            pub fn index(self) -> $ty {
                self.0 & Self::MAX_ID
            }
        
            /// Get the lifetime of the handle.
            pub fn lifetime(self) -> $ty {
                self.0 >> N
            }
        }
    };
}

create_index_handle!{
    doc = "8-bit index handle.",
    doc = "e.g. IndexHandle8<6> can store 2^6 (64) unique values, and 2^2 (4) lifetime values.",
    IndexHandle8 => u8
}
create_index_handle!{
    doc = "16-bit index handle.",
    doc = "e.g. IndexHandle16<10> can store 2^10 (1024) unique values, and 2^6 (64) lifetime values.",
    IndexHandle16 => u16
}
create_index_handle!{
    doc = "32-bit index handle.",
    doc = "e.g. IndexHandle32<10> can store 2^20 (1'048'576) unique values, and 2^12 (4096) lifetime values.",
    IndexHandle32 => u32
}
create_index_handle!{
    doc = "64-bit index handle.",
    doc = "e.g. IndexHandle64<10> can store 2^48 (281'474'976'710'656) unique values, and 2^16 (65536) lifetime values.",
    IndexHandle64 => u64
}