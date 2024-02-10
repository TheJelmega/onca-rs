//! GUID
//! 
//! The implementation is based on [`RFC4122`](https://datatracker.ietf.org/doc/html/rfc4122)

use std::{
    fmt,
    mem, hash::Hasher,
};

use onca_common_macros::EnumDisplay;
use crate::{os, hashing::{MD5, Hasher128, SHA1, Hasher160}};


/// Guid variant.
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum GuidVariant {
    /// Reserved variant for NCS backward compatibility
    NcsBackCompat,
    /// Variant specified in RFC 4122
    Rfc4122,
    /// Reserved variant for Microsoft Corporation backward compatibility.
    MsBackCompat,
    // Reserved for future versions
    Reserved,
}

/// Guid version.
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum GuidVersion {
    /// Version 1: Time based version specified in RFC4122.
    #[display("Version 1: Time-based")]
    Version1,
    /// Version 2: DCE security version (with embedded POSIX UIDs).
    #[display("Version 2: DCE security")]
    Version2,
    /// Version 3: Named based version using MD5.
    #[display("Version 3: Name-based MD5")]
    Version3,
    /// Version 4: Randomly or pseudo-randomly generated version.
    #[display("Version 4: Random")]
    Version4,
    /// Version 5: Name based version using SHA-1.
    #[display("Version 5: Name-based SHA-1")]
    Version5,
    /// Version 6-16: Unknown.
    #[display("Version 6-16: Unknown")]
    Unknown,
}

/// Version of a Guid in the layout specified in RFC4122, with element in system endianess.
/// 
/// # Note
/// 
/// This version is not meant to be stored itself, please use [`Guid`] to store GUIDs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rfc4122Guid {
    /// The low field of the timestamp.
    pub time_low:                  u32,
    /// The middle field of the timestamp.
    pub time_mid:                  u16,
    /// The high field of the timestamp multiplexed with the version number.
    pub time_hi_and_version:       u16,
    /// The high field of the clock sequence multiplexed with the variant.
    pub clock_seq_hi_and_reserved: u8,
    /// THe low field of the clock sequence.
    pub clock_seq_low:             u8,
    /// The spatially unique node identifier.
    pub node:                      [u8; 6]
}

impl From<Guid> for Rfc4122Guid {
    fn from(guid: Guid) -> Self {
        let (high, low) = guid.as_high_low();
        Self {
            time_low: (high >> 32) as u32,
            time_mid: (high >> 16) as u16,
            time_hi_and_version: high as u16,
            clock_seq_hi_and_reserved: (low >> 56) as u8,
            clock_seq_low: (low >> 48) as u8,
            node: [guid.0[10], guid.0[11], guid.0[12], guid.0[13], guid.0[14], guid.0[15]],
        }
    }
}

/// Global Unique IDentifier, also know as a UUID (Universal Unique IDentifier).
/// 
/// Bytes are stored in a big-endian format, i.e the GUID
/// ```
/// 00112233-4455-6677-8899-aabbccddeeff
/// ```
/// will be stored as as
/// ```
/// [00, 11, 22, 33, 44, 55, 66, 77, 88, 99, aa, bb, cc, dd, ee, ff]
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct Guid([u8; 16]);

impl Guid {
    /// Nil Guid.
    pub const NIL: Guid = Guid([0; 16]);

    /// Create a new [`Guid`] from raw bytes.
    pub fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Create a new [`Guid`] from a u128.
    /// 
    /// The value will be stored as big-endian.
    pub fn new_u128(val: u128) -> Self {
        Self(val.to_be_bytes())
    }

    /// Create a new [`Guid`] from a high and low value.
    /// 
    /// The resulting Guid will be presented as `hhhh-hh-hh-ll-llllll`.
    /// 
    /// The value will be stored as big-endian.
    pub fn new_high_low(high: u64, low: u64) -> Self {
        let high = high.to_be();
        let low = low.to_be();
        Guid(unsafe { mem::transmute([high, low]) })
    }

    /// Create a [`Guid`] from a `8-4-4-4-12` representation.
    pub fn new_8_4_4_4_12(v0: u32, v1: u16, v2: u16, v3: u16, v4: u64) -> Self {
        let high = ((v0 as u64) << 32) | ((v1 as u64) << 16) | (v2 as u64);
        let low = ((v3 as u64) << 48) | (v4 & 0x0000_FFFF_FFFF_FFFF);
        Self::new_high_low(high, low)
    }

    /// Create a time-based [`Guid`]  (version 1)
    /// 
    /// # Note
    /// 
    /// There is a chance the underlying implementation may return a GUID with the PC's mac address included in it
    pub fn new_time_based() -> Self {
        Self(os::misc::create_v1_uuid())
    }

    /// Create a name-based [`Guid`]  using MD5 (version 3).
    /// 
    /// # Note
    /// 
    /// If there is no need for backwards compatibility, version 5 using SHA-1 should be prefered.
    pub fn new_name_md5(mut namespace: Guid, name: &str) -> Self {
        let mut hasher = MD5::new();
        hasher.write(&namespace.0);
        hasher.write(name.as_bytes());
        let hash = hasher.finish128();

        Self([
            // time_low
            hash[0], hash[1], hash[2], hash[3],
            // time_mid
            hash[4], hash[5],
            // time_hi_and_version
            hash[6] & 0x0F | (3 << 4), hash[7],
            // clock_seq_hi_and_reserved
            hash[8] & 0x3F | (0b10 << 6),
            // clock_seq_low
            hash[9],
            // node_id
            hash[10], hash[11], hash[12], hash[13], hash[14], hash[15],
        ])
    }
    
    /// Create a name-based [`Guid`]  using SHA-1 (version 5).
    pub fn new_name_sha1(mut namespace: Guid, name: &str) -> Self {
        let mut hasher = SHA1::new();
        hasher.write(&namespace.0);
        hasher.write(name.as_bytes());
        let hash = hasher.finish160();

        Self([
            // time_low
            hash[0], hash[1], hash[2], hash[3],
            // time_mid
            hash[4], hash[5],
            // time_hi_and_version
            hash[6] & 0x0F | (5 << 4), hash[7],
            // clock_seq_hi_and_reserved
            hash[8] & 0x3F | (0b10 << 6),
            // clock_seq_low
            hash[9],
            // node_id
            hash[10], hash[11], hash[12], hash[13], hash[14], hash[15],
        ])
    }

    /// Create a random [`Guid`]  (version 4).
    pub fn new_random() -> Guid {
        Guid(os::misc::create_v4_uuid())
    }

    /// Create a [`Guid`]  from a raw array
    /// 
    /// # SAFETY
    /// 
    /// The caller nees to ensure a valid [`Guid`]  value is passed.
    pub unsafe fn from_raw(raw: [u8; 16]) -> Guid {
        Self(raw)
    }
    
    /// Get the [`Guid`] as a [`u128`].
    pub fn as_u128(self) -> u128 {
        u128::from_be_bytes(self.0)
    }

    /// Get the [`Guid`] as a pair of low and high [`u64`]'s.
    pub fn as_high_low(self) -> (u64, u64) {
        let (high, low): (u64, u64) = unsafe { mem::transmute(self.0) };
        (high.to_be(), low.to_be())
    }

    /// Get the [`Guid`] as its `8-4-4-4-12` representation.
    pub fn as_8_4_4_4_12(self) -> (u32, u16, u16, u16, u64) {
        let (high, low) = self.as_high_low();
        let v0 = (high >> 32) as u32;
        let v1 = (high >> 16) as u16;
        let v2 = high as u16;
        let v3 = (low >> 48) as u16;
        let v4 = low & 0x0000_FFFF_FFFF_FFFF;
        (v0, v1, v2, v3, v4)
    }

    /// Get the variant from the [`Guid`].
    pub fn get_variant(&self) -> GuidVariant {
        let variant = self.0[8] >> 5;
        match variant {
            0b000 |
            0b001 |
            0b010 |
            0b011 => GuidVariant::NcsBackCompat,
            0b100 |
            0b101 => GuidVariant::Rfc4122,
            0b110 => GuidVariant::MsBackCompat,
            0b111 => GuidVariant::Reserved,
            _ => unreachable!()
        }
    }

    /// Get the version from the [`Guid`].
    pub fn get_version(&self) -> GuidVersion {
        let version = self.0[6] >> 4;
        match version {
            1 => GuidVersion::Version1,
            2 => GuidVersion::Version2,
            3 => GuidVersion::Version3,
            4 => GuidVersion::Version4,
            5 => GuidVersion::Version5,
            _ => GuidVersion::Unknown,
        }
    }

    /// Check if the [`Guid`] is valid, i.e. not {00000000-0000-0000-0000-000000000000}.
    pub fn is_valid(&self) -> bool {
        *self != Self::default()
    }

    /// Parse a string into a Guid
    /// 
    /// Supported formats:
    /// - "00000000000000000000000000000000" <- 32 hexadecimal digits
    /// - "00000000-0000-0000-0000-000000000000" <- 32 hexadecimal digits separated by hyphens
    /// - "{00000000-0000-0000-0000-000000000000}" <- 32 hexadecimal digits separated by hyphens, enclosed by braces
    /// - "(00000000-0000-0000-0000-000000000000)" <- 32 hexadecimal digits separated by hyphens, enclosed by parentheses
    pub fn parse(s: &str) -> Option<Self> {
        if s.starts_with('(') {
            let parens: &[_] = &['(', ')'];
            let s = s.trim_matches(parens);
            Self::parse_dashed(s)
        } else if s.starts_with('{') {
            let parens: &[_] = &['{', '}'];
            let s = s.trim_matches(parens);
            Self::parse_dashed(s)
        } else if s.contains('-') {
            Self::parse_dashed(s)
        } else if s.len() == 32 {
            u128::from_str_radix(s, 16).map_or(None, |val| Some(Guid::new_u128(val)))
        } else {
            None
        }
    }

    fn parse_dashed(s: &str) -> Option<Self> {
        let mut bytes = [0; 16];
        for (idx, nibble) in s.bytes().filter(|val| *val != b'-').enumerate() {
            let byte_idx = idx / 2;
            let upper = idx & 0x1 == 0;

            let nibble = if nibble >= b'0' && nibble <= b'9' {
                nibble as u8 - b'0'
            } else if nibble >= b'A' && nibble <= b'F' {
                10 + nibble as u8 - b'A'
            } else if nibble >= b'a' && nibble <= b'f' {
                10 + nibble as u8 - b'a'
            } else {
                return None;
            };
            bytes[byte_idx] |= nibble << (upper as usize * 4);
        }
        /// SAFETY: Guid is parsed as expected
        Some(unsafe { Self::from_raw(bytes) })
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{:02X}{:02x}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
            self.0[0], self.0[1], self.0[2], self.0[3],
            self.0[4], self.0[5],
            self.0[6], self.0[7],   
            self.0[8], self.0[9],
            self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15]
        )
    }
}

impl From<Rfc4122Guid> for Guid {
    fn from(guid: Rfc4122Guid) -> Self {
        let low = [guid.clock_seq_hi_and_reserved, guid.clock_seq_low, guid.node[0], guid.node[1], guid.node[2], guid.node[3], guid.node[4], guid.node[5]];
        let low = u64::from_be_bytes(low);
        let high = ((guid.time_low as u64) << 32) | ((guid.time_mid as u64) << 16) | (guid.time_hi_and_version as u64);
        Self::new_high_low(high, low)
    }
}

impl From<Guid> for u128 {
    fn from(value: Guid) -> Self {
        u128::from_be_bytes(value.0)
    }
}

impl From<&str> for Guid {
    fn from(value: &str) -> Self {
        Self::parse(value).expect("Invalid guid format")
    }
}


#[cfg(test)]
mod test {
    use crate::guid::Rfc4122Guid;

    use super::Guid;

    
    #[test]
    pub fn guid_test() {
        let guid0 = Guid::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);

        let guid1 = Guid::new_u128(0x00112233445566778899AABBCCDDEEFF);
        assert_eq!(guid0, guid1);
        let guid_u128: u128 = guid0.into();
        assert_eq!(guid_u128, 0x00112233445566778899AABBCCDDEEFF);

        let guid2 = Guid::new_high_low(0x11223344556677, 0x8899AABBCCDDEEFF);
        assert_eq!(guid0, guid2);
        let (high, low) = guid2.as_high_low();
        assert_eq!(high, 0x11223344556677);
        assert_eq!(low, 0x8899AABBCCDDEEFF);

        let guid3 = Guid::new_8_4_4_4_12(0x00112233, 0x4455, 0x6677, 0x8899, 0xAABBCCDDEEFF);
        assert_eq!(guid0, guid3);
        let (v0, v1, v2, v3, v4) = guid3.as_8_4_4_4_12();
        assert_eq!(v0, 0x00112233);
        assert_eq!(v1, 0x4455);
        assert_eq!(v2, 0x6677);
        assert_eq!(v3, 0x8899);
        assert_eq!(v4, 0xAABBCCDDEEFF);

        let expected_parse_guid = Guid::new([0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);

        let parsed = Guid::parse("0123456789ABCDEF8899AABBCCDDEEFF");
        assert_eq!(parsed, Some(expected_parse_guid));

        let parsed = Guid::parse("01234567-89AB-CDEF-8899-AABBCCDDEEFF");
        assert_eq!(parsed, Some(expected_parse_guid));

        let parsed = Guid::parse("(01234567-89AB-CDEF-8899-AABBCCDDEEFF)");
        assert_eq!(parsed, Some(expected_parse_guid));

        let parsed = Guid::parse("{01234567-89AB-CDEF-8899-AABBCCDDEEFF}");
        assert_eq!(parsed, Some(expected_parse_guid));

        
        let formatted = format!("{}", guid0);
        assert_eq!(formatted, "00112233-4455-6677-8899-AABBCCDDEEFF");


        let rfc_guid0 = Rfc4122Guid {
            time_low: 0x00112233,
            time_mid: 0x4455,
            time_hi_and_version: 0x6677,
            clock_seq_hi_and_reserved: 0x88,
            clock_seq_low: 0x99,
            node: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
        };
        let guid_from_rfc: Guid = rfc_guid0.into();
        assert_eq!(guid0, guid_from_rfc);

        let rfc_guid1:Rfc4122Guid = guid_from_rfc.into();
        assert_eq!(rfc_guid0, rfc_guid1);
    }
}