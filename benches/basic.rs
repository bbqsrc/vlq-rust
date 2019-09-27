#![feature(test)]

extern crate test;
use test::Bencher;

use std::io::Cursor;
use vlq_rust::{ReadVlqExt as _, Vlq, WriteVlqExt as _};

fn roundtrip<T: Vlq>(value: T) -> T {
    let mut buf = Vec::with_capacity(32);
    buf.write_vlq(value).expect("successful write");
    Cursor::new(buf).read_vlq().expect("successful read")
}

#[bench]
fn test_u8(b: &mut Bencher) {
    b.iter(|| roundtrip(std::u8::MAX));
}

#[bench]
fn test_u16(b: &mut Bencher) {
    b.iter(|| roundtrip(std::u16::MAX));
}

#[bench]
fn test_u32(b: &mut Bencher) {
    b.iter(|| roundtrip(std::u32::MAX));
}

#[bench]
fn test_u64(b: &mut Bencher) {
    b.iter(|| roundtrip(std::u64::MAX));
}

#[bench]
fn test_u128(b: &mut Bencher) {
    b.iter(|| roundtrip(std::u128::MAX));
}
