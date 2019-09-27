#![allow(clippy::cast_lossless)]

/// Decoding bit depth by prefix in bits:
///
/// 0xxx_xxxx: 1 byte
/// 10xx_xxxx: 2 bytes
/// 110x_xxxx: 3 bytes
/// 1110_xxxx: 4 bytes
/// 1111_0xxx: 5 bytes
/// 1111_10xx: 6 bytes
/// 1111_110x: 7 bytes
/// 1111_1110: 8 bytes
/// 1111_1111: 9 bytes
#[inline(always)]
fn decode_len(n: u8) -> u8 {
    match n {
        i if i & 0b1000_0000 != 0b1000_0000 => 1,
        i if i & 0b1100_0000 != 0b1100_0000 => 2,
        i if i & 0b1110_0000 != 0b1110_0000 => 3,
        i if i & 0b1111_0000 != 0b1111_0000 => 4,
        i if i & 0b1111_1000 != 0b1111_1000 => 5,
        i if i & 0b1111_1100 != 0b1111_1100 => 6,
        i if i & 0b1111_1110 != 0b1111_1110 => 7,
        i if i != 0b1111_1111 => 8,
        _ => 9,
    }
}

/// Encoding bit depth by length in bytes:
///
/// 1: 7 bits
/// 2: 14 (6 + 8) bits
/// 3: 21 (5 + 8 * 2) bits
/// 4: 28 (4 + 8 * 3) bits
/// 5: 35 (3 + 8 * 4) bits
/// 6: 42 (2 + 8 * 5) bits
/// 7: 49 (1 + 8 * 6) bits
/// 8: 56 (0 + 8 * 7) bits
/// 9: 64 (0 + 8 * 8) bits
#[inline(always)]
fn encode_len(n: u64) -> u8 {
    for i in 1u64..=8 {
        if n < (1 << (7 * i)) {
            return i as u8;
        }
    }

    9
}

#[inline(always)]
fn prefix(len: u8, target: u8) -> u8 {
    match len {
        1 => target,
        2 => 0b1000_0000 | target,
        3 => 0b1100_0000 | target,
        4 => 0b1110_0000 | target,
        5 => 0b1111_0000 | target,
        6 => 0b1111_1000 | target,
        7 => 0b1111_1100 | target,
        8 => 0b1111_1110,
        _ => 0b1111_1111,
    }
}

#[inline(always)]
pub fn decode(n: FastVlq) -> u64 {
    let len = n.len();
    let n = n.0;
    match len {
        1 => n[0] as u64,
        2 => u64::from_le_bytes([n[1], n[0] & 0b0011_1111, 0, 0, 0, 0, 0, 0]),
        3 => u64::from_le_bytes([n[2], n[1], n[0] & 0b0001_1111, 0, 0, 0, 0, 0]),
        4 => u64::from_le_bytes([n[3], n[2], n[1], n[0] & 0b0000_1111, 0, 0, 0, 0]),
        5 => u64::from_le_bytes([n[4], n[3], n[2], n[1], n[0] & 0b0000_0111, 0, 0, 0]),
        6 => u64::from_le_bytes([n[5], n[4], n[3], n[2], n[1], n[0] & 0b0000_0011, 0, 0]),
        7 => u64::from_le_bytes([n[6], n[5], n[4], n[3], n[2], n[1], n[0] & 0b0000_0001, 0]),
        8 => u64::from_le_bytes([n[7], n[6], n[5], n[4], n[3], n[2], n[1], 0]),
        _ => u64::from_le_bytes([n[8], n[7], n[6], n[5], n[4], n[3], n[2], n[1]]),
    }
}

#[derive(Debug, Clone, Copy)]
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

#[inline(always)]
pub fn encode(n: u64) -> FastVlq {
    let len = encode_len(n);
    let mut out_buf = [0u8; 9];

    match len {
        1 => {
            out_buf[0] = n as u8;
        }
        2 => {
            let buf = (n as u16).to_be_bytes();
            (&mut out_buf[..2]).copy_from_slice(&buf[..2]);
            out_buf[0] = prefix(2, buf[0]);
        }
        3 => {
            let buf = ((n as u32) << 8).to_be_bytes();
            (&mut out_buf[..3]).copy_from_slice(&buf[..3]);
            out_buf[0] = prefix(3, buf[0]);
        }
        4 => {
            let buf = (n as u32).to_be_bytes();
            (&mut out_buf[..4]).copy_from_slice(&buf[..4]);
            out_buf[0] = prefix(4, buf[0]);
        }
        5 => {
            let buf = (n << (8 * 3)).to_be_bytes();
            (&mut out_buf[..5]).copy_from_slice(&buf[..5]);
            out_buf[0] = prefix(5, buf[0]);
        }
        6 => {
            let buf = (n << (8 * 2)).to_be_bytes();
            (&mut out_buf[..6]).copy_from_slice(&buf[..6]);
            out_buf[0] = prefix(6, buf[0]);
        }
        7 => {
            let buf = (n << 8).to_be_bytes();
            (&mut out_buf[..7]).copy_from_slice(&buf[..7]);
            out_buf[0] = prefix(7, buf[0]);
        }
        8 => {
            let buf = n.to_be_bytes();
            (&mut out_buf[..8]).copy_from_slice(&buf[..8]);
            out_buf[0] = prefix(8, buf[0]);
        }
        _ => {
            (&mut out_buf[1..]).copy_from_slice(&n.to_be_bytes());
            out_buf[0] = 0xFF;
        }
    };

    FastVlq(out_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_decode() {
        assert_eq!(decode_len(0b0000_0000), 1, "min for 1");
        assert_eq!(decode_len(0b0111_1111), 1, "max for 1");
        assert_eq!(decode_len(0b1000_0000), 2, "min for 2");
        assert_eq!(decode_len(0b1011_1111), 2, "max for 2");
        assert_eq!(decode_len(0b1100_0000), 3, "min for 3");
        assert_eq!(decode_len(0b1100_1111), 3, "max for 3");
        assert_eq!(decode_len(0b1110_0000), 4, "min for 4");
        assert_eq!(decode_len(0b1110_1111), 4, "max for 4");
        assert_eq!(decode_len(0b1111_0000), 5, "min for 5");
        assert_eq!(decode_len(0b1111_0111), 5, "max for 5");
        assert_eq!(decode_len(0b1111_1000), 6, "min for 6");
        assert_eq!(decode_len(0b1111_1011), 6, "max for 6");
        assert_eq!(decode_len(0b1111_1100), 7, "min for 7");
        assert_eq!(decode_len(0b1111_1101), 7, "max for 7");
        assert_eq!(decode_len(0b1111_1110), 8, "min for 8");
        assert_eq!(decode_len(0b1111_1111), 9, "min for 9");
    }

    #[test]
    fn check_encode() {
        assert_eq!(encode_len(std::u64::MIN), 1);
        assert_eq!(encode_len(0x7F), 1, "max for 1");
        assert_eq!(encode_len(0x80), 2, "min for 2");
        assert_eq!(encode_len(0x3FFF), 2, "max for 2");
        assert_eq!(encode_len(0x4000), 3, "min for 3");
        assert_eq!(encode_len(0x1F_FFFF), 3, "max for 3");
        assert_eq!(encode_len(0x20_0000), 4, "min for 4");
        assert_eq!(encode_len(0x0FFF_FFFF), 4, "max for 4");
        assert_eq!(encode_len(0x1000_0000), 5, "min for 5");
        assert_eq!(encode_len(0x07_FFFF_FFFF), 5, "max for 5");
        assert_eq!(encode_len(0x08_0000_0000), 6, "min for 6");
        assert_eq!(encode_len(0x03FF_FFFF_FFFF), 6, "max for 6");
        assert_eq!(encode_len(0x0400_0000_0000), 7, "min for 7");
        assert_eq!(encode_len(0x01_FFFF_FFFF_FFFF), 7, "max for 7");
        assert_eq!(encode_len(0x02_0000_0000_0000), 8, "min for 8");
        assert_eq!(encode_len(0x00FF_FFFF_FFFF_FFFF), 8, "max for 8");
        assert_eq!(encode_len(0x0100_0000_0000_0000), 9, "min for 9");
        assert_eq!(encode_len(std::u64::MAX), 9);
        assert_eq!(encode_len(0x3FFE), 2);
    }

    #[test]
    fn round_trip() {
        assert_eq!(decode(encode(std::u64::MIN)), std::u64::MIN);
        assert_eq!(decode(encode(0x7F)), 0x7F, "max for 1");
        assert_eq!(decode(encode(0x80)), 0x80, "min for 2");
        assert_eq!(decode(encode(0x3FFF)), 0x3FFF, "max for 2");
        assert_eq!(decode(encode(0x4000)), 0x4000, "min for 3");
        assert_eq!(decode(encode(0x1F_FFFF)), 0x1F_FFFF, "max for 3");
        assert_eq!(decode(encode(0x20_0000)), 0x20_0000, "min for 4");
        assert_eq!(decode(encode(0x0FFF_FFFF)), 0x0FFF_FFFF, "max for 4");
        assert_eq!(decode(encode(0x1000_0000)), 0x1000_0000, "min for 5");
        assert_eq!(decode(encode(0x07_FFFF_FFFF)), 0x07_FFFF_FFFF, "max for 5");
        assert_eq!(decode(encode(0x08_0000_0000)), 0x08_0000_0000, "min for 6");
        assert_eq!(
            decode(encode(0x03FF_FFFF_FFFF)),
            0x03FF_FFFF_FFFF,
            "max for 6"
        );
        assert_eq!(
            decode(encode(0x0400_0000_0000)),
            0x0400_0000_0000,
            "min for 7"
        );
        assert_eq!(
            decode(encode(0x01_FFFF_FFFF_FFFF)),
            0x01_FFFF_FFFF_FFFF,
            "max for 7"
        );
        assert_eq!(
            decode(encode(0x02_0000_0000_0000)),
            0x02_0000_0000_0000,
            "min for 8"
        );
        assert_eq!(
            decode(encode(0x00FF_FFFF_FFFF_FFFF)),
            0x00FF_FFFF_FFFF_FFFF,
            "max for 8"
        );
        assert_eq!(
            decode(encode(0x0100_0000_0000_0000)),
            0x0100_0000_0000_0000,
            "min for 9"
        );
        assert_eq!(decode(encode(std::u64::MAX)), std::u64::MAX);
        assert_eq!(decode(encode(std::i64::MIN as u64)) as i64, std::i64::MIN);

        assert_eq!(0, decode(encode(0x0)), "0");
        assert_eq!(1, decode(encode(0x1)), "1");
        assert_eq!(0x100, decode(encode(0x100)), "2");
        assert_eq!(0xFF220022, decode(encode(0xFF220022)), "3");
        assert_eq!(
            0x00FF_FFFF_FFFF_FFFF,
            decode(encode(0x00FF_FFFF_FFFF_FFFF)),
            "4"
        );
        assert_eq!(
            0x0100_0000_0000_0000,
            decode(encode(0x0100_0000_0000_0000)),
            "5"
        );
        assert_eq!(std::u64::MAX, decode(encode(std::u64::MAX)), "max");
    }
}
