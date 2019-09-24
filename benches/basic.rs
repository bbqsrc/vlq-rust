#![feature(test)]

extern crate test;
use test::Bencher;

use std::convert::TryFrom;
use vlq_rust::Vlq;

#[bench]
fn test_u8(b: &mut Bencher) {
    b.iter(|| u8::try_from(Vlq::from(std::u8::MAX)));
}

#[bench]
fn test_u16(b: &mut Bencher) {
    b.iter(|| u16::try_from(Vlq::from(std::u16::MAX)));
}

#[bench]
fn test_u32(b: &mut Bencher) {
    b.iter(|| u32::try_from(Vlq::from(std::u32::MAX)));
}

#[bench]
fn test_u64(b: &mut Bencher) {
    b.iter(|| u64::try_from(Vlq::from(std::u64::MAX)));
}

#[bench]
fn test_u128(b: &mut Bencher) {
    b.iter(|| u128::try_from(Vlq::from(std::u128::MAX)));
}
