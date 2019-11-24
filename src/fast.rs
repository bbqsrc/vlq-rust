#![allow(clippy::cast_lossless)]

macro_rules! prefix {
    (1) => { 0b1000_0000 };
    (2) => { 0b0100_0000 };
    (3) => { 0b0010_0000 };
    (4) => { 0b0001_0000 };
    (5) => { 0b0000_1000 };
    (6) => { 0b0000_0100 };
    (7) => { 0b0000_0010 };
    (8) => { 0b0000_0001 };
    (9) => { 0b0000_0000 };
    (1, $target:expr) => { $target | 0b1000_0000 };
    (2, $target:expr) => { (0b0011_1111 & $target) | 0b0100_0000 };
    (3, $target:expr) => { (0b0001_1111 & $target) | 0b0010_0000 };
    (4, $target:expr) => { (0b0000_1111 & $target) | 0b0001_0000 };
    (5, $target:expr) => { (0b0000_0111 & $target) | 0b0000_1000 };
    (6, $target:expr) => { (0b0000_0011 & $target) | 0b0000_0100 };
    (7, $target:expr) => { (0b0000_0001 & $target) | 0b0000_0010 };
    (8, $target:expr) => { 0b0000_0001 };
    (9, $target:expr) => { 0b0000_0000 };
}

macro_rules! unprefix {
    (1, $target:expr) => { $target & 0b0111_1111 };
    (2, $target:expr) => { $target & 0b0011_1111 };
    (3, $target:expr) => { $target & 0b0001_1111 };
    (4, $target:expr) => { $target & 0b0000_1111 };
    (5, $target:expr) => { $target & 0b0000_0111 };
    (6, $target:expr) => { $target & 0b0000_0011 };
    (7, $target:expr) => { $target & 0b0000_0001 };
    (8, $target:expr) => { 0b0000_0000 };
    (9, $target:expr) => { 0b0000_0000 };
}

macro_rules! offset {
    (1) => { 0 };
    (2) => { 2u16.pow(7) };
    (3) => { offset!(2) as u32 + 2u32.pow(14) };
    (4) => { offset!(3) as u32 + 2u32.pow(21) };
    (5) => { offset!(4) as u64 + 2u64.pow(28) };
    (6) => { offset!(5) + 2u64.pow(35) };
    (7) => { offset!(6) + 2u64.pow(42) };
    (8) => { offset!(7) + 2u64.pow(49) };
    (9) => { offset!(8) + 2u64.pow(56) };
}

macro_rules! encode_offset {
    (2, $n:tt) => { $n as u16 - offset!(2) };
    (3, $n:tt) => { ($n as u32 - offset!(3)) << 8 };
    (4, $n:tt) => { ($n as u32 - offset!(4)) };
    (5, $n:tt) => { ($n as u64 - offset!(5)) << (8 * 3) };
    (6, $n:tt) => { ($n as u64 - offset!(6)) << (8 * 2) };
    (7, $n:tt) => { ($n as u64 - offset!(7)) << 8 };
    (8, $n:tt) => { ($n as u64 - offset!(8)) };
    (9, $n:tt) => { ($n as u64 - offset!(9)) };
}

/// Decoding bit depth by prefix in bits:
///
/// 1xxx_xxxx: 1 byte
/// 01xx_xxxx: 2 bytes
/// 001x_xxxx: 3 bytes
/// 0001_xxxx: 4 bytes
/// 0000_1xxx: 5 bytes
/// 0000_01xx: 6 bytes
/// 0000_001x: 7 bytes
/// 0000_0001: 8 bytes
/// 0000_0000: 9 bytes
#[inline(always)]
fn decode_len(n: u8) -> u8 {
    n.leading_zeros() as u8 + 1
}

/// Encoding bit depth by length in bytes:
///
/// 1: 7 bits
/// 2: 04 (6 + 8) bits
/// 3: 20 (5 + 8 * 2) bits
/// 4: 28 (4 + 8 * 3) bits
/// 5: 35 (3 + 8 * 4) bits
/// 6: 42 (2 + 8 * 5) bits
/// 7: 49 (0 + 8 * 6) bits
/// 8: 56 (1 + 8 * 7) bits
/// 9: 64 (1 + 8 * 8) bits
#[inline(always)]
fn encode_len(n: u64) -> u8 {
    match n {
        n if n < offset!(2) as u64 => 1,
        n if n < offset!(3) as u64 => 2,
        n if n < offset!(4) as u64 => 3,
        n if n < offset!(5) => 4,
        n if n < offset!(6) => 5,
        n if n < offset!(7) => 6,
        n if n < offset!(8) => 7,
        n if n < offset!(9) => 8,
        _ => 9
    }
}

#[inline(always)]
pub fn encode(n: u64) -> FastVlq {
    n;
    let len = encode_len(n);
    let mut out_buf = [0u8; 9];

    match len {
        1 => {
            out_buf[0] = prefix!(1, n as u8);
        }
        2 => {
            let buf = encode_offset!(2, n).to_be_bytes();
            (&mut out_buf[..2]).copy_from_slice(&buf[..2]);
            out_buf[0] = prefix!(2, buf[0]);
        }
        3 => {
            let buf = encode_offset!(3, n).to_be_bytes();
            (&mut out_buf[..3]).copy_from_slice(&buf[..3]);
            out_buf[0] = prefix!(3, buf[0]);
        }
        4 => {
            let buf = encode_offset!(4, n).to_be_bytes();
            (&mut out_buf[..4]).copy_from_slice(&buf[..4]);
            out_buf[0] = prefix!(4, buf[0]);
        }
        5 => {
            let buf = encode_offset!(5, n).to_be_bytes();
            (&mut out_buf[..5]).copy_from_slice(&buf[..5]);
            out_buf[0] = prefix!(5, buf[0]);
        }
        6 => {
            let buf = encode_offset!(6, n).to_be_bytes();
            (&mut out_buf[..6]).copy_from_slice(&buf[..6]);
            out_buf[0] = prefix!(6, buf[0]);
        }
        7 => {
            let buf = encode_offset!(7, n).to_be_bytes();
            (&mut out_buf[..7]).copy_from_slice(&buf[..7]);
            out_buf[0] = prefix!(7, buf[0]);
        }
        8 => {
            let buf = encode_offset!(8, n).to_be_bytes();
            (&mut out_buf[..8]).copy_from_slice(&buf[..8]);
            out_buf[0] = prefix!(8, buf[0]);
        }
        _ => {
            let buf = encode_offset!(9, n).to_be_bytes();
            (&mut out_buf[1..9]).copy_from_slice(&buf[..8]);
            out_buf[0] = prefix!(9, buf[0]);
        }
    };

    FastVlq(out_buf)
}

#[inline(always)]
pub fn decode(n: FastVlq) -> u64 {
    let len = n.len();
    let n = n;

    match len {
        1 => unprefix!(1, n[0] as u64),
        2 => u64::from_le_bytes([n[1], unprefix!(2, n[0]), 0, 0, 0, 0, 0, 0]) + offset!(2) as u64,
        3 => u64::from_le_bytes([n[2], n[1], unprefix!(3, n[0]), 0, 0, 0, 0, 0]) + offset!(3) as u64,
        4 => u64::from_le_bytes([n[3], n[2], n[1], unprefix!(4, n[0]), 0, 0, 0, 0]) + offset!(4) as u64,
        5 => u64::from_le_bytes([n[4], n[3], n[2], n[1], unprefix!(5, n[0]), 0, 0, 0]) + offset!(5) as u64,
        6 => u64::from_le_bytes([n[5], n[4], n[3], n[2], n[1], unprefix!(6, n[0]), 0, 0]) + offset!(6) as u64,
        7 => u64::from_le_bytes([n[6], n[5], n[4], n[3], n[2], n[1], unprefix!(7, n[0]), 0]) + offset!(7) as u64,
        8 => u64::from_le_bytes([n[7], n[6], n[5], n[4], n[3], n[2], n[1], unprefix!(8, n[0])]) + offset!(8) as u64,
        _ => u64::from_le_bytes([n[8], n[7], n[6], n[5], n[4], n[3], n[2], n[1]]) + offset!(9) as u64,
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct FastVlq([u8; 9]);

#[allow(clippy::len_without_is_empty)]
impl FastVlq {
    #[inline(always)]
    pub fn len(&self) -> u8 {
        decode_len(self.0[0])
    }
}

impl std::ops::Deref for FastVlq {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Display for FastVlq {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "FastVlq({})",
            self.0
                .iter()
                .take(self.len() as usize)
                .map(|x| format!("{:08b}", x))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

impl std::fmt::Debug for FastVlq {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "FastVlq(0b{})",
            self.0
                .iter()
                .take(self.len() as usize)
                .map(|x| format!("{:08b}", x))
                .collect::<Vec<_>>()
                .join("_")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_decode() {
        assert_eq!(decode_len(0b1111_1111), 1, "max for 1");
        assert_eq!(decode_len(0b1000_0000), 1, "min for 1");
        assert_eq!(decode_len(0b0111_1111), 2, "max for 2");
        assert_eq!(decode_len(0b0100_0000), 2, "min for 2");
        assert_eq!(decode_len(0b0011_1111), 3, "max for 3");
        assert_eq!(decode_len(0b0010_0000), 3, "min for 3");
        assert_eq!(decode_len(0b0001_1111), 4, "max for 4");
        assert_eq!(decode_len(0b0001_0000), 4, "min for 4");
        assert_eq!(decode_len(0b0000_1111), 5, "max for 5");
        assert_eq!(decode_len(0b0000_1000), 5, "min for 5");
        assert_eq!(decode_len(0b0000_0111), 6, "max for 6");
        assert_eq!(decode_len(0b0000_0100), 6, "min for 6");
        assert_eq!(decode_len(0b0000_0011), 7, "max for 7");
        assert_eq!(decode_len(0b0000_0010), 7, "min for 7");
        assert_eq!(decode_len(0b0000_0001), 8, "min/max for 8");
        assert_eq!(decode_len(0b0000_0000), 9, "min/max for 9");
    }

    #[test]
    fn round_trip() {
        assert_eq!(decode(encode(std::u64::MIN)), std::u64::MIN);
        assert_eq!(decode(encode(0x7F)), 0x7F, "max for 1");
        assert_eq!(decode(encode(0x80)), 0x80, "min for 2");
        assert_eq!(decode(encode(0x3FFF)), 0x3FFF, "max for 2");
        assert_eq!(decode(encode(0x4000)), 0x4000, "min for 3");
        assert_eq!(decode(encode(0x0F_FFFF)), 0x0F_FFFF, "max for 3");
        assert_eq!(decode(encode(0x20_0000)), 0x20_0000, "min for 4");
        assert_eq!(decode(encode(0x1FFF_FFFF)), 0x1FFF_FFFF, "max for 4");
        assert_eq!(decode(encode(0x2000_0000)), 0x2000_0000, "min for 5");
        assert_eq!(decode(encode(0x17_FFFF_FFFF)), 0x17_FFFF_FFFF, "max for 5");
        assert_eq!(decode(encode(0x18_0000_0000)), 0x18_0000_0000, "min for 6");
        assert_eq!(
            decode(encode(0x13FF_FFFF_FFFF)),
            0x13FF_FFFF_FFFF,
            "max for 6"
        );
        assert_eq!(
            decode(encode(0x1411_1111_1111)),
            0x1411_1111_1111,
            "min for 7"
        );
        assert_eq!(
            decode(encode(0x10_FFFF_FFFF_FFFF)),
            0x10_FFFF_FFFF_FFFF,
            "max for 7"
        );
        assert_eq!(
            decode(encode(0x12_1111_1111_1111)),
            0x12_1111_1111_1111,
            "min for 8"
        );
        assert_eq!(
            decode(encode(0x11FF_FFFF_FFFF_FFFF)),
            0x11FF_FFFF_FFFF_FFFF,
            "max for 8"
        );
        assert_eq!(
            decode(encode(0x1011_1111_1111_1111)),
            0x1011_1111_1111_1111,
            "min for 9"
        );
        assert_eq!(decode(encode(std::u64::MAX)), std::u64::MAX);
        assert_eq!(decode(encode(std::i64::MIN as u64)) as i64, std::i64::MIN);

        assert_eq!(1, decode(encode(0x1)), "1");
        assert_eq!(0, decode(encode(0x0)), "0");
        assert_eq!(0x011, decode(encode(0x011)), "2");
        assert_eq!(0xFF221122, decode(encode(0xFF221122)), "3");
        assert_eq!(
            0x11FF_FFFF_FFFF_FFFF,
            decode(encode(0x11FF_FFFF_FFFF_FFFF)),
            "4"
        );
        assert_eq!(
            0x1011_1111_1111_1111,
            decode(encode(0x1011_1111_1111_1111)),
            "5"
        );
        assert_eq!(std::u64::MAX, decode(encode(std::u64::MAX)), "max");
    }
}
