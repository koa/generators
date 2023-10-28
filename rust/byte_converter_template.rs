//! Traits for (de)serialization of structs to byte vectors.
use byteorder::{ByteOrder, LittleEndian};

use crate::converting_receiver::BrickletError;

/// A trait to serialize the implementing type to a byte vector.
pub trait ToBytes {
    /// Serialize the implementing type to a byte vector.
    fn to_le_byte_vec(_: Self) -> Vec<u8>;
    fn write_to_slice(self, target: &mut [u8]);

    /// Try to serialize the implementing type to a byte vector. If the type is shorter than max_len, it will be padded with zero bytes. Currently this method is only used for strings. Other types use the standard implementation, which calls [`to_le_byte_vec`] without further checks or padding.
    /// # Errors
    /// Returns an InvalidArgument error if the type was too long.
    ///
    /// [`to_le_byte_vec`]: #ToBytes.to_le_byte_vec
    fn try_to_le_byte_vec(var: Self, _max_len: usize) -> Result<Vec<u8>, BrickletError>
    where
        Self: std::marker::Sized,
    {
        Ok(Self::to_le_byte_vec(var))
    }
    fn try_write_to_slice(self, _max_len: usize, target: &mut [u8]) -> Result<(), BrickletError>
    where
        Self: std::marker::Sized,
    {
        self.write_to_slice(target);
        Ok(())
    }
}

/// A trait to deserialize the implemeting type from a byte slice.
pub trait FromByteSlice {
    /// Deserialize the implementing type from a byte slice.
    fn from_le_byte_slice(bytes: &[u8]) -> Self;
    /// Returns how many bytes are expected to deserialize a instance of the implementing type. Currently this method is only used for strings.
    fn bytes_expected() -> usize;
}
impl<const N: usize> FromByteSlice for Box<[u8; N]> {
    fn from_le_byte_slice(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::bytes_expected());
        let mut ret = Box::new([0; N]);
        ret.clone_from_slice(bytes);
        ret
    }

    fn bytes_expected() -> usize {
        N
    }
}
impl<const N: usize> FromByteSlice for Box<[i16; N]> {
    fn from_le_byte_slice(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len() * 2, Self::bytes_expected());
        let mut ret = Box::new([0; N]);
        for i in 0..N {
            ret[i] = i16::from_le_byte_slice(&bytes[i * 2..i * 2 + 2]);
        }
        ret
    }

    fn bytes_expected() -> usize {
        N * 2
    }
}

impl<const N: usize> FromByteSlice for Box<[u16; N]> {
    fn from_le_byte_slice(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len() * 2, Self::bytes_expected());
        let mut ret = Box::new([0; N]);
        for i in 0..N {
            ret[i] = u16::from_le_byte_slice(&bytes[i * 2..i * 2 + 2]);
        }
        ret
    }

    fn bytes_expected() -> usize {
        N * 2
    }
}

impl<const N: usize> FromByteSlice for Box<[i32; N]> {
    fn from_le_byte_slice(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::bytes_expected());
        let mut ret = Box::new([0; N]);
        for i in 0..N {
            ret[i] = i32::from_le_byte_slice(&bytes[i * 4..i * 4 + 4]);
        }
        ret
    }

    fn bytes_expected() -> usize {
        N * 4
    }
}
impl<const N: usize> FromByteSlice for Box<[i64; N]> {
    fn from_le_byte_slice(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::bytes_expected());
        let mut ret = Box::new([0; N]);
        for i in 0..N {
            ret[i] = i64::from_le_byte_slice(&bytes[i * 8..i * 8 + 8]);
        }
        ret
    }

    fn bytes_expected() -> usize {
        N * 8
    }
}
impl<const N: usize> FromByteSlice for Box<[bool; N]> {
    fn from_le_byte_slice(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::bytes_expected());
        let mut ret = Box::new([false; N]);
        for (byte, elem) in bytes.into_iter().enumerate() {
            for i in 0..8 {
                if byte * 8 + i >= N {
                    break;
                }
                ret[byte * 8 + i] = (*elem & 1 << i) > 0;
            }
        }
        ret
    }

    fn bytes_expected() -> usize {
        (N + 7) / 8
    }
}
impl<const N: usize> FromByteSlice for Box<[char; N]> {
    fn from_le_byte_slice(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::bytes_expected());
        let mut ret = Box::new([0 as char; N]);
        for i in 0..N {
            ret[i] = bytes[i] as char;
        }
        ret
    }

    fn bytes_expected() -> usize {
        N
    }
}
/*
impl<const N: usize> FromByteSlice for [u8; N] {
    fn from_le_byte_slice(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), N);
        let mut ret = [0; N];
        ret.clone_from_slice(bytes);
        ret
    }

    fn bytes_expected() -> usize {
        N
    }
}
impl<const N: usize> ToBytes for [u8; N] {
    fn to_le_byte_vec(array: Self) -> Vec<u8> {
        array.to_vec()
    }

    fn write_to_slice(self, target: &mut [u8]) {
        target.copy_from_slice(&self);
    }
}
*/
impl ToBytes for () {
    fn to_le_byte_vec(_: ()) -> Vec<u8> {
        vec![]
    }

    fn write_to_slice(self, _target: &mut [u8]) {}
}

impl FromByteSlice for () {
    fn from_le_byte_slice(_: &[u8]) {}

    fn bytes_expected() -> usize {
        0
    }
}

impl ToBytes for bool {
    fn to_le_byte_vec(b: bool) -> Vec<u8> {
        vec![b as u8]
    }

    fn write_to_slice(self, target: &mut [u8]) {
        *(target.get_mut(0).expect("slice too small")) = self as u8;
    }
}

impl FromByteSlice for bool {
    fn from_le_byte_slice(bytes: &[u8]) -> bool {
        bytes[0] != 0
    }

    fn bytes_expected() -> usize {
        1
    }
}

impl ToBytes for u8 {
    fn to_le_byte_vec(num: u8) -> Vec<u8> {
        vec![num]
    }

    fn write_to_slice(self, target: &mut [u8]) {
        *(target.get_mut(0).expect("slice too small")) = self;
    }
}

impl FromByteSlice for u8 {
    fn from_le_byte_slice(bytes: &[u8]) -> u8 {
        bytes[0]
    }

    fn bytes_expected() -> usize {
        1
    }
}

impl ToBytes for i8 {
    fn to_le_byte_vec(num: i8) -> Vec<u8> {
        vec![num as u8]
    }
    fn write_to_slice(self, target: &mut [u8]) {
        *(target.get_mut(0).expect("slice too small")) = self as u8;
    }
}

impl FromByteSlice for i8 {
    fn from_le_byte_slice(bytes: &[u8]) -> i8 {
        bytes[0] as i8
    }

    fn bytes_expected() -> usize {
        1
    }
}

impl ToBytes for u16 {
    fn to_le_byte_vec(num: u16) -> Vec<u8> {
        let mut buf = vec![0; 2];
        LittleEndian::write_u16(&mut buf, num);
        buf
    }
    fn write_to_slice(self, target: &mut [u8]) {
        LittleEndian::write_u16(target, self);
    }
}

impl FromByteSlice for u16 {
    fn from_le_byte_slice(bytes: &[u8]) -> u16 {
        LittleEndian::read_u16(bytes)
    }

    fn bytes_expected() -> usize {
        2
    }
}

impl ToBytes for i16 {
    fn to_le_byte_vec(num: i16) -> Vec<u8> {
        let mut buf = vec![0; 2];
        LittleEndian::write_i16(&mut buf, num);
        buf
    }

    fn write_to_slice(self, target: &mut [u8]) {
        LittleEndian::write_i16(target, self);
    }
}

impl FromByteSlice for i16 {
    fn from_le_byte_slice(bytes: &[u8]) -> i16 {
        LittleEndian::read_i16(bytes)
    }

    fn bytes_expected() -> usize {
        2
    }
}

impl ToBytes for u32 {
    fn to_le_byte_vec(num: u32) -> Vec<u8> {
        let mut buf = vec![0; 4];
        LittleEndian::write_u32(&mut buf, num);
        buf
    }

    fn write_to_slice(self, target: &mut [u8]) {
        LittleEndian::write_u32(target, self);
    }
}

impl FromByteSlice for u32 {
    fn from_le_byte_slice(bytes: &[u8]) -> u32 {
        LittleEndian::read_u32(bytes)
    }

    fn bytes_expected() -> usize {
        4
    }
}

impl ToBytes for i32 {
    fn to_le_byte_vec(num: i32) -> Vec<u8> {
        let mut buf = vec![0; 4];
        LittleEndian::write_i32(&mut buf, num);
        buf
    }

    fn write_to_slice(self, target: &mut [u8]) {
        LittleEndian::write_i32(target, self);
    }
}

impl FromByteSlice for i32 {
    fn from_le_byte_slice(bytes: &[u8]) -> i32 {
        LittleEndian::read_i32(bytes)
    }

    fn bytes_expected() -> usize {
        4
    }
}

impl ToBytes for u64 {
    fn to_le_byte_vec(num: u64) -> Vec<u8> {
        let mut buf = vec![0; 8];
        LittleEndian::write_u64(&mut buf, num);
        buf
    }

    fn write_to_slice(self, target: &mut [u8]) {
        LittleEndian::write_u64(target, self);
    }
}

impl FromByteSlice for u64 {
    fn from_le_byte_slice(bytes: &[u8]) -> u64 {
        LittleEndian::read_u64(bytes)
    }

    fn bytes_expected() -> usize {
        8
    }
}

impl ToBytes for i64 {
    fn to_le_byte_vec(num: i64) -> Vec<u8> {
        let mut buf = vec![0; 8];
        LittleEndian::write_i64(&mut buf, num);
        buf
    }

    fn write_to_slice(self, target: &mut [u8]) {
        LittleEndian::write_i64(target, self);
    }
}

impl FromByteSlice for i64 {
    fn from_le_byte_slice(bytes: &[u8]) -> i64 {
        LittleEndian::read_i64(bytes)
    }

    fn bytes_expected() -> usize {
        8
    }
}

impl ToBytes for char {
    fn to_le_byte_vec(c: char) -> Vec<u8> {
        vec![c as u8]
    }

    fn write_to_slice(self, target: &mut [u8]) {
        *(target.get_mut(0).expect("slice too small")) = self as u8;
    }
}

impl FromByteSlice for char {
    fn from_le_byte_slice(bytes: &[u8]) -> char {
        bytes[0] as char
    }

    fn bytes_expected() -> usize {
        1
    }
}

impl ToBytes for String {
    fn to_le_byte_vec(s: String) -> Vec<u8> {
        s.into_bytes()
    }

    fn write_to_slice(self, target: &mut [u8]) {
        target.copy_from_slice(&self.into_bytes());
    }

    fn try_to_le_byte_vec(s: String, max_len: usize) -> Result<Vec<u8>, BrickletError> {
        if s.chars().any(|c| c as u32 > 255) {
            return Err(BrickletError::InvalidParameter);
        }
        let bytes: Vec<u8> = s.chars().map(|c| c as u8).collect();
        if bytes.len() > max_len {
            Err(BrickletError::InvalidParameter)
        } else {
            let mut result = vec![0u8; max_len];
            result[0..bytes.len()].copy_from_slice(&bytes);
            Ok(result)
        }
    }

    fn try_write_to_slice(self, max_len: usize, target: &mut [u8]) -> Result<(), BrickletError>
    where
        Self: Sized,
    {
        let bytes = self.into_bytes();
        if bytes.len() > max_len {
            Err(BrickletError::InvalidParameter)
        } else {
            target[0..bytes.len()].copy_from_slice(&bytes);
            Ok(())
        }
    }
}

impl FromByteSlice for String {
    fn from_le_byte_slice(bytes: &[u8]) -> String {
        bytes
            .into_iter()
            .filter(|&&b| b != 0)
            .map(|&b| b as char)
            .collect()
    }

    fn bytes_expected() -> usize {
        1
    }
}

impl ToBytes for f32 {
    fn to_le_byte_vec(num: f32) -> Vec<u8> {
        let mut buf = vec![0; 4];
        LittleEndian::write_f32(&mut buf, num);
        buf
    }

    fn write_to_slice(self, target: &mut [u8]) {
        LittleEndian::write_f32(target, self);
    }
}

impl FromByteSlice for f32 {
    fn from_le_byte_slice(bytes: &[u8]) -> f32 {
        LittleEndian::read_f32(bytes)
    }

    fn bytes_expected() -> usize {
        4
    }
}

impl ToBytes for f64 {
    fn to_le_byte_vec(num: f64) -> Vec<u8> {
        let mut buf = vec![0; 8];
        LittleEndian::write_f64(&mut buf, num);
        buf
    }

    fn write_to_slice(self, target: &mut [u8]) {
        LittleEndian::write_f64(target, self);
    }
}

impl FromByteSlice for f64 {
    fn from_le_byte_slice(bytes: &[u8]) -> f64 {
        LittleEndian::read_f64(bytes)
    }

    fn bytes_expected() -> usize {
        8
    }
}
