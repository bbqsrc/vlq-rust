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
//!  1100000  1010100       11  [re-organized so LSB first]
//! 01100000 01010100 10000011  [as VLQ-encoded variable-length integer]
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

pub trait ReadVlqExt<T> {
    fn read_vlq(&mut self) -> std::io::Result<T>;
}

pub trait WriteVlqExt<T> {
    fn write_vlq(&mut self, n: T) -> std::io::Result<()>;
}

macro_rules! impl_vlq {
    ($ty:ty, $cap:expr) => {
        impl_vlq!($ty, $ty, $cap);
    };
    ($ty:ty, $uty:ty, $cap:expr) => {
        impl<R: std::io::Read> $crate::ReadVlqExt<$ty> for R {
            fn read_vlq(&mut self) -> std::io::Result<$ty> {
                let mut vlq_buf = [0u8; $cap];
                let mut index = 0;

                {
                    let mut buf = [0; 1];

                    loop {
                        self.read_exact(&mut buf)?;
                        vlq_buf[index] = buf[0];
                        index += 1;
                        if (buf[0] & 0b1000_0000) != 0 {
                            break;
                        }
                    }
                }

                let mut iter = vlq_buf.iter().rev();
                let init = (iter.next().unwrap_or(&0) & 0b0111_1111) as $ty;
                iter.try_fold(init, |acc, cur| {
                    let acc = acc.checked_shl(7).ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            concat!("provided VLQ data too long to fit into ", stringify!($ty)),
                        )
                    })?;
                    Ok(acc | (cur & 0b0111_1111) as $ty)
                })
            }
        }

        impl<W: std::io::Write> $crate::WriteVlqExt<$ty> for W {
            fn write_vlq(&mut self, n: $ty) -> std::io::Result<()> {
                let mut n = n as $uty;
                let mut vlq_buf = [0u8; $cap];
                let mut index = 0;

                while n > 0 {
                    vlq_buf[index] = (n & 0b0111_1111) as u8;
                    index += 1;
                    n >>= 7;
                }

                match index {
                    0 => {
                        vlq_buf[index] = 0b1000_0000;
                        index += 1;
                    }
                    n => {
                        vlq_buf[n - 1] |= 0b1000_0000;
                    }
                }

                self.write_all(&vlq_buf[..index])
            }
        }
    };
}

impl_vlq!(i8, u8, 2);
impl_vlq!(i16, u16, 3);
impl_vlq!(i32, u32, 5);
impl_vlq!(i64, u64, 10);
impl_vlq!(i128, u128, 19);
impl_vlq!(isize, usize, std::mem::size_of::<usize>() / 7 + 1);
impl_vlq!(u8, 2);
impl_vlq!(u16, 3);
impl_vlq!(u32, 5);
impl_vlq!(u64, 10);
impl_vlq!(u128, 19);
impl_vlq!(usize, std::mem::size_of::<usize>() / 7 + 1);

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn roundtrip<T>(value: T) -> T
    where
        Vec<u8>: WriteVlqExt<T>,
        Cursor<Vec<u8>>: ReadVlqExt<T>,
    {
        let mut buf = vec![];
        buf.write_vlq(value).expect("successful write");
        Cursor::new(buf).read_vlq().expect("successful read")
    }

    #[test]
    fn test_smoke() {
        assert_eq!(std::u8::MAX, roundtrip(std::u8::MAX));
        assert_eq!(std::u8::MIN, roundtrip(std::u8::MIN));
        assert_eq!(std::i8::MAX, roundtrip(std::i8::MAX));
        assert_eq!(std::i8::MIN, roundtrip(std::i8::MIN));
        assert_eq!(std::u16::MAX, roundtrip(std::u16::MAX));
        assert_eq!(std::u16::MIN, roundtrip(std::u16::MIN));
        assert_eq!(std::i16::MAX, roundtrip(std::i16::MAX));
        assert_eq!(std::i16::MIN, roundtrip(std::i16::MIN));
        assert_eq!(std::u32::MAX, roundtrip(std::u32::MAX));
        assert_eq!(std::u32::MIN, roundtrip(std::u32::MIN));
        assert_eq!(std::i32::MAX, roundtrip(std::i32::MAX));
        assert_eq!(std::i32::MIN, roundtrip(std::i32::MIN));
        assert_eq!(std::u64::MAX, roundtrip(std::u64::MAX));
        assert_eq!(std::u64::MIN, roundtrip(std::u64::MIN));
        assert_eq!(std::i64::MAX, roundtrip(std::i64::MAX));
        assert_eq!(std::i64::MIN, roundtrip(std::i64::MIN));
        assert_eq!(std::u128::MAX, roundtrip(std::u128::MAX));
        assert_eq!(std::u128::MIN, roundtrip(std::u128::MIN));
        assert_eq!(std::i128::MAX, roundtrip(std::i128::MAX));
        assert_eq!(std::i128::MIN, roundtrip(std::i128::MIN));
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
