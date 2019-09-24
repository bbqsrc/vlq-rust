use std::fmt;

/// The maximum capacity that can be stored inline in the Vlq representation.
const CAP: usize = 23;

#[derive(Debug)]
pub struct VlqError;

#[derive(Debug, Clone, Copy, Default)]
pub struct Vlq {
    len: u8,
    buf: [u8; CAP],
}

/// Format the vlq number using a binary representation for debugging purposes.
pub fn bin(vlq: &Vlq) -> Bin<'_> {
    Bin(vlq)
}

pub struct Bin<'a>(&'a [u8]);

impl fmt::Display for Bin<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.0 {
            write!(fmt, "{:08b}", b)?;
        }

        Ok(())
    }
}

impl std::ops::Deref for Vlq {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buf[..(self.len as usize)]
    }
}

#[derive(Debug)]
pub enum TryFromVlqError {
    NumberTooLarge,
}

pub trait ReadVlqExt<T>: std::io::Read {
    fn read_vlq(&mut self) -> std::io::Result<T>;
}

pub trait WriteVlqExt<T>: std::io::Write {
    fn write_vlq(&mut self, n: T) -> std::io::Result<()>;
}

macro_rules! impl_vlq {
    ($ty:ty) => {
        impl_vlq!($ty, $ty);
    };
    ($ty:ty, $uty:ty) => {
        impl std::convert::TryFrom<Vlq> for $ty {
            type Error = TryFromVlqError;

            fn try_from(vlq: Vlq) -> Result<$ty, Self::Error> {
                let mut iter = vlq.iter().rev();
                let init = (iter.next().unwrap_or(&0) & 0b0111_1111) as $ty;
                iter.try_fold(init, |acc, cur| {
                    let acc = acc.checked_shl(7).ok_or(TryFromVlqError::NumberTooLarge)?;
                    Ok(acc | (cur & 0b0111_1111) as $ty)
                })
            }
        }

        impl From<$ty> for Vlq {
            fn from(n: $ty) -> Self {
                let mut n = n as $uty;
                let mut vlq_buf = [0u8; CAP];
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

                Vlq {
                    len: index as u8,
                    buf: vlq_buf,
                }
            }
        }

        impl<R: std::io::Read> $crate::ReadVlqExt<$ty> for R {
            fn read_vlq(&mut self) -> std::io::Result<$ty> {
                use std::convert::TryFrom;
                let mut vlq_buf = [0u8; CAP];
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

                let vlq = Vlq {
                    len: index as u8,
                    buf: vlq_buf,
                };

                <$ty>::try_from(vlq).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        concat!("provided VLQ data too long to fit into ", stringify!($ty)),
                    )
                })
            }
        }

        impl<W: std::io::Write> $crate::WriteVlqExt<$ty> for W {
            fn write_vlq(&mut self, n: $ty) -> std::io::Result<()> {
                self.write_all(&*Vlq::from(n))
            }
        }
    };
}

impl_vlq!(i8, u8);
impl_vlq!(i16, u16);
impl_vlq!(i32, u32);
impl_vlq!(i64, u64);
impl_vlq!(i128, u128);
impl_vlq!(isize, usize);
impl_vlq!(u8);
impl_vlq!(u16);
impl_vlq!(u32);
impl_vlq!(u64);
impl_vlq!(u128);
impl_vlq!(usize);

#[cfg(test)]
mod tests {
    use super::ReadVlqExt;
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn invalid_empty_vlq() {
        let garbage = Vlq::default();
        let y = u8::try_from(garbage).unwrap();
        assert_eq!(y, 0);
    }

    #[test]
    fn read() {
        let vlq = Vlq::from(std::u64::MAX);
        let mut data = std::io::Cursor::new(&*vlq);
        let x: u64 = data.read_vlq().unwrap();
        assert_eq!(x, std::u64::MAX);

        let vlq = Vlq::from(std::i64::MIN);
        let mut data = std::io::Cursor::new(&*vlq);
        let x: i64 = data.read_vlq().unwrap();
        assert_eq!(x, std::i64::MIN);
    }

    #[test]
    fn write() {
        let mut data = std::io::Cursor::new(vec![]);
        data.write_vlq(std::u64::MAX).unwrap();
        assert_eq!(data.into_inner(), &*Vlq::from(std::u64::MAX));

        let mut data = std::io::Cursor::new(vec![]);
        data.write_vlq(std::i64::MAX).unwrap();
        assert_eq!(data.into_inner(), &*Vlq::from(std::i64::MAX));
    }

    #[test]
    fn it_works() {
        let x = std::u8::MAX;
        let y = u8::try_from(Vlq::from(x)).unwrap();
        assert_eq!(x, y);

        let x = std::i8::MIN;
        let y = i8::try_from(Vlq::from(x)).unwrap();
        assert_eq!(x, y);

        let x = std::u64::MAX;
        let y = u64::try_from(Vlq::from(x)).unwrap();
        assert_eq!(x, y);

        let x = std::i64::MIN;
        let y = i64::try_from(Vlq::from(x)).unwrap();
        assert_eq!(x, y);

        let x = std::u128::MAX;
        let y = u128::try_from(Vlq::from(x)).unwrap();
        assert_eq!(x, y);

        let x = std::i128::MIN;
        let y = i128::try_from(Vlq::from(x)).unwrap();
        assert_eq!(x, y);
    }
}
