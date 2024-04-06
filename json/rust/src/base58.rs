//! Parses Base58 encoded brick and bricklet uids.
use std::{
    fmt::{
        Debug,
        Display,
        Formatter
    },
    error::Error,
    str::FromStr
};

use byteorder::{ByteOrder, LittleEndian};
use const_str::to_char_array;

use crate::byte_converter::{FromByteSlice, ToBytes};

const ALPHABET: [char; 58] = to_char_array!("123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ");

const ERROR_INVALID_CHAR: &str = "UID contains an invalid character";
const ERROR_TOO_BIG: &str = "UID is too big to fit into a u64";
const ERROR_EMPTY: &str = "UID is empty or a value that mapped to zero";

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
pub struct Uid(u32);

impl Uid {
    #[inline]
    pub(crate) fn zero() -> Uid {
        Uid(0)
    }
}

impl From<u32> for Uid {
    fn from(value: u32) -> Self {
        Uid(value)
    }
}

impl From<Uid> for u32 {
    fn from(value: Uid) -> Self {
        value.0
    }
}

impl FromStr for Uid {
    type Err = Base58Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "0" {
            Ok(Uid(0))
        } else {
            Ok(Self(s.base58_to_u32()?))
        }
    }
}

impl Display for Uid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&u32_to_base58(self.0))
    }
}

impl Debug for Uid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0 {
            f.write_str("0")
        } else {
            f.write_str(&u32_to_base58(self.0))
        }
    }
}

impl ToBytes for Uid {
    fn write_to_slice(self, target: &mut [u8]) {
        LittleEndian::write_u32(target, self.0);
    }
}

impl FromByteSlice for Uid {
    fn from_le_byte_slice(bytes: &[u8]) -> Uid {
        Uid(LittleEndian::read_u32(bytes))
    }

    fn bytes_expected() -> usize {
        4
    }
}

#[cfg(feature = "serde")]
mod serde {
    use std::{fmt::Formatter, str::FromStr};

    use serde::{
        de::{Error, Visitor},
        Deserialize, Deserializer, Serialize, Serializer,
    };

    use crate::base58::Uid;

    struct UidVisitor;

    impl<'de> Visitor<'de> for UidVisitor {
        type Value = Uid;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a string a tinkerforge uid in base58 format")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Uid::from_str(v).map_err(|error| E::custom(format!("Cannot parse {v} as uid: {error}")))
        }
    }

    impl<'de> Deserialize<'de> for Uid {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(UidVisitor)
        }
    }

    impl Serialize for Uid {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_string())
        }
    }
}

///Error type of Base58 parser.
#[derive(Debug, Copy, Clone)]
pub enum Base58Error {
    ///Is returned if the parse finds an invalid character. Contains the character and it's index in the string.
    InvalidCharacter,
    UidTooBig,
    UidEmpty,
}

impl Display for Base58Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            Base58Error::InvalidCharacter => write!(f, "{}", ERROR_INVALID_CHAR),
            Base58Error::UidTooBig => write!(f, "{}", ERROR_TOO_BIG),
            Base58Error::UidEmpty => write!(f, "{}", ERROR_EMPTY),
        }
    }
}

impl Error for Base58Error {
    fn description(&self) -> &str {
        match *self {
            Base58Error::InvalidCharacter => ERROR_INVALID_CHAR,
            Base58Error::UidTooBig => ERROR_TOO_BIG,
            Base58Error::UidEmpty => ERROR_EMPTY,
        }
    }
}

///A trait which adds Base58 parsing capabilities to strings. The alphabet used is "123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ".
pub trait Base58 {
    /// Parse this string as Base58 encoded uid. Returns an error if a character is found, that is not part of the used alphabet.
    fn base58_to_u32(&self) -> Result<u32, Base58Error>;
}

impl Base58 for str {
    fn base58_to_u32(&self) -> Result<u32, Base58Error> {
        let mut result_u64: u64 = 0;
        for character in self.chars() {
            match ALPHABET.iter().enumerate().find(|(_, c)| **c == character).map(|(i, _)| i) {
                None => return Err(Base58Error::InvalidCharacter),
                Some(i) => {
                    result_u64 = result_u64
                        .checked_mul(ALPHABET.len() as u64)
                        .ok_or(Base58Error::UidTooBig)?
                        .checked_add(i as u64)
                        .ok_or(Base58Error::UidTooBig)?;
                }
            }
        }

        let result = if result_u64 > u32::max_value().into() {
            let value1 = result_u64 & 0xFF_FF_FF_FF;
            let value2 = (result_u64 >> 32) & 0xFF_FF_FF_FF;
            ((value1 & 0x00_00_0F_FF)
                | (value1 & 0x0F_00_00_00) >> 12
                | (value2 & 0x00_00_00_3F) << 16
                | (value2 & 0x00_0F_00_00) << 6
                | (value2 & 0x3F_00_00_00) << 2) as u32
        } else {
            result_u64 as u32
        };
        if result == 0 {
            Err(Base58Error::UidEmpty)
        } else {
            Ok(result)
        }
    }
}

impl Base58 for String {
    fn base58_to_u32(&self) -> Result<u32, Base58Error> {
        self.as_str().base58_to_u32()
    }
}

pub fn u32_to_base58(mut id: u32) -> Box<str> {
    let radix = ALPHABET.len() as u32;
    // u32::MAX needs 6 digits
    let mut buffer = [0 as char; 6];
    let mut ptr = 0;
    while id > 0 {
        let digit = id % radix;
        buffer[ptr] = ALPHABET[digit as usize];
        id = id / radix;
        ptr += 1;
    }
    buffer[..ptr].iter().rev().collect::<String>().into_boxed_str()
}

#[cfg(test)]
mod test {
    use crate::base58::{u32_to_base58, Base58};

    #[test]
    fn test_parse_address() {
        assert_eq!(130221, "EHc".base58_to_u32().unwrap());
        assert_eq!(130221, "111111111111111111111111111111111111111111111111EHc".base58_to_u32().unwrap());
        assert_eq!(u32::MAX, "7xwQ9g".base58_to_u32().unwrap());
    }

    #[test]
    fn test_format_address() {
        assert_eq!("EHc", &u32_to_base58(130221).to_string());
        assert_eq!("7xwQ9g", &u32_to_base58(u32::MAX).to_string());
    }
}
