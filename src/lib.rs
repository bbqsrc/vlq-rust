pub trait ReadVlqExt<T>: std::io::Read {
    fn read_vlq(&mut self) -> std::io::Result<T>;
}

pub trait WriteVlqExt<T>: std::io::Write {
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

impl_vlq!(i8, u8, 1);
impl_vlq!(i16, u16, 3);
impl_vlq!(i32, u32, 5);
impl_vlq!(i64, u64, 10);
impl_vlq!(i128, u128, 19);
impl_vlq!(isize, usize, std::mem::size_of::<usize>() / 7 + 1);
impl_vlq!(u8, 1);
impl_vlq!(u16, 3);
impl_vlq!(u32, 5);
impl_vlq!(u64, 10);
impl_vlq!(u128, 19);
impl_vlq!(usize, std::mem::size_of::<usize>() / 7 + 1);

#[cfg(test)]
mod tests {
    use super::*;

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
