//! ## Algorithm
//!
//! Value-length quantity encoding is an implementation of variable-length integers.
//!
//! Each byte is encoded into 7 bits, with the highest bit of the byte used to indicate
//! if there are any further bytes to read. Reading will continue until the highest bit
//! is `1`, or will result in an error if the number is too large to fit in the desired
//! type.
//!
//! For example, the number `60000` (or `0xEA60` in hexadecimal):
//!
//! ```markdown
//!          11101010 01100000  [as u16]
//!       11  1010100  1100000  [as separated into 7-bit groups]
//!  1100000  1010100       11  [re-organized so least significant byte first]
//! 11100000 11010100 00000011  [as VLQ-encoded variable-length integer]
//! ```
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! vlq = { package = "vlq-rust", version = "0.2" }
//! ```
//!
//! Use `ReadVlqExt` and `WriteVlqExt` to get the `read_vlq` and `write_vlq` functions
//! on every `std::io::Read` and `std::io::Write` implementation.
//!
//! ## Example
//!
//! ```
//! # use vlq_rust as vlq;
//! use vlq::{ReadVlqExt, WriteVlqExt};
//!
//! let mut data = std::io::Cursor::new(vec![]);
//! data.write_vlq(std::u64::MAX).unwrap();
//! data.set_position(0);
//!
//! let x: u64 = data.read_vlq().unwrap();
//! assert_eq!(x, std::u64::MAX);
//!
//! let mut data = std::io::Cursor::new(vec![]);
//! data.write_vlq(std::i64::MIN).unwrap();
//! data.set_position(0);
//!
//! let x: i64 = data.read_vlq().unwrap();
//! assert_eq!(x, std::i64::MIN);
//! ```

pub mod fast;

/// Trait applied to all types that can be encoded as VLQ's.
pub trait Vlq: Sized {
    /// Read a variable-length quantity from a reader.
    fn from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self>;

    /// Write this variable-length quantity to a writer.
    fn to_writer<W: std::io::Write>(self, writer: &mut W) -> std::io::Result<()>;
}

pub trait ReadVlqExt<T> {
    /// Read a variable-length quantity.
    fn read_vlq(&mut self) -> std::io::Result<T>;
}

pub trait WriteVlqExt<T> {
    /// Write a variable-length quantity.
    fn write_vlq(&mut self, n: T) -> std::io::Result<()>;
}

impl<T: Vlq, R: ::std::io::Read> ReadVlqExt<T> for R {
    fn read_vlq(&mut self) -> std::io::Result<T> {
        T::from_reader(self)
    }
}

impl<T: Vlq, W: ::std::io::Write> WriteVlqExt<T> for W {
    fn write_vlq(&mut self, n: T) -> std::io::Result<()> {
        n.to_writer(self)
    }
}

/// Helper macro to implement the `Vlq` trait for a primitive type.
macro_rules! impl_primitive {
    ($ty:ty) => {
        impl_primitive!($ty, $ty);
    };
    ($ty:ty, $uty:ty) => {
        impl $crate::Vlq for $ty {
            fn from_reader<R: ::std::io::Read>(reader: &mut R) -> ::std::io::Result<Self> {
                let mut buf = [0; 1];
                let mut value: $uty = 0;
                let mut shift = 1 as $uty;

                loop {
                    reader.read_exact(&mut buf)?;

                    value = ((buf[0] & 0b0111_1111) as $uty)
                        .checked_mul(shift)
                        .and_then(|add| value.checked_add(add))
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                concat!("provided VLQ data too long to fit into ", stringify!($ty)),
                            )
                        })?;

                    if (buf[0] & 0b1000_0000) != 0 {
                        break;
                    }

                    shift <<= 7;
                }

                Ok(value as $ty)
            }

            fn to_writer<W: ::std::io::Write>(self, writer: &mut W) -> ::std::io::Result<()> {
                let mut n = self as $uty;
                let mut vlq_buf = [0u8; std::mem::size_of::<$ty>() * 8 / 7 + 1];
                let mut index = 0;

                while n >= 0x80 {
                    vlq_buf[index] = (n & 0b0111_1111) as u8;
                    index += 1;
                    n >>= 7;
                }

                vlq_buf[index] = 0b1000_0000 | n as u8;
                index += 1;
                writer.write_all(&vlq_buf[..index])?;
                Ok(())
            }
        }
    };
}

impl_primitive!(i8, u8);
impl_primitive!(i16, u16);
impl_primitive!(i32, u32);
impl_primitive!(i64, u64);
impl_primitive!(i128, u128);
impl_primitive!(isize, usize);
impl_primitive!(u8);
impl_primitive!(u16);
impl_primitive!(u32);
impl_primitive!(u64);
impl_primitive!(u128);
impl_primitive!(usize);

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn roundtrip<T: Vlq>(value: T) -> T {
        let mut buf = vec![];
        buf.write_vlq(value).expect("successful write");
        Cursor::new(buf).read_vlq().expect("successful read")
    }

    #[test]
    fn test_smoke() {
        assert_eq!(std::u8::MAX, roundtrip(std::u8::MAX));
        assert_eq!(std::u8::MIN, roundtrip(std::u8::MIN));
        assert_eq!(0u8, roundtrip(0u8));

        assert_eq!(std::i8::MAX, roundtrip(std::i8::MAX));
        assert_eq!(std::i8::MIN, roundtrip(std::i8::MIN));
        assert_eq!(0i8, roundtrip(0i8));
        assert_eq!(-1i8, roundtrip(-1i8));

        assert_eq!(std::u16::MAX, roundtrip(std::u16::MAX));
        assert_eq!(std::u16::MIN, roundtrip(std::u16::MIN));
        assert_eq!(0u16, roundtrip(0u16));

        assert_eq!(std::i16::MAX, roundtrip(std::i16::MAX));
        assert_eq!(std::i16::MIN, roundtrip(std::i16::MIN));
        assert_eq!(0i16, roundtrip(0i16));
        assert_eq!(-1i16, roundtrip(-1i16));

        assert_eq!(std::u32::MAX, roundtrip(std::u32::MAX));
        assert_eq!(std::u32::MIN, roundtrip(std::u32::MIN));
        assert_eq!(0u32, roundtrip(0u32));

        assert_eq!(std::i32::MAX, roundtrip(std::i32::MAX));
        assert_eq!(std::i32::MIN, roundtrip(std::i32::MIN));
        assert_eq!(0i32, roundtrip(0i32));
        assert_eq!(-1i32, roundtrip(-1i32));

        assert_eq!(std::u64::MAX, roundtrip(std::u64::MAX));
        assert_eq!(std::u64::MIN, roundtrip(std::u64::MIN));
        assert_eq!(0u64, roundtrip(0u64));

        assert_eq!(std::i64::MAX, roundtrip(std::i64::MAX));
        assert_eq!(std::i64::MIN, roundtrip(std::i64::MIN));
        assert_eq!(0i64, roundtrip(0i64));
        assert_eq!(-1i64, roundtrip(-1i64));

        assert_eq!(std::u128::MAX, roundtrip(std::u128::MAX));
        assert_eq!(std::u128::MIN, roundtrip(std::u128::MIN));
        assert_eq!(0u128, roundtrip(0u128));

        assert_eq!(std::i128::MAX, roundtrip(std::i128::MAX));
        assert_eq!(std::i128::MIN, roundtrip(std::i128::MIN));
        assert_eq!(0i128, roundtrip(0i128));
        assert_eq!(-1i128, roundtrip(-1i128));
    }

    #[test]
    fn read_write() {
        let mut data = std::io::Cursor::new(vec![]);
        data.write_vlq(std::u64::MAX).unwrap();
        data.set_position(0);

        let x: u64 = data.read_vlq().unwrap();
        assert_eq!(x, std::u64::MAX);

        let mut data = std::io::Cursor::new(vec![]);
        data.write_vlq(std::i64::MIN).unwrap();
        data.set_position(0);

        let x: i64 = data.read_vlq().unwrap();
        assert_eq!(x, std::i64::MIN);
    }
}
