#![feature(test)]

extern crate test;
use test::Bencher;

use rand::prelude::*;
use std::io::Cursor;
use vlq_rust::{ReadVlqExt as _, Vlq, WriteVlqExt as _};

fn roundtrip<T: Vlq>(value: T) -> T {
    let mut buf = Vec::with_capacity(32);
    buf.write_vlq(value).expect("successful write");
    Cursor::new(buf).read_vlq().expect("successful read")
}

fn roundtrip_fast(value: u64) -> u64 {
    let encoded = vlq_rust::fast::encode(value);
    vlq_rust::fast::decode(encoded)
}

fn encode_fast(value: u64) {
    let encoded = vlq_rust::fast::encode(value);
}

fn decode_fast(encoded: vlq_rust::fast::FastVlq) -> u64 {
    vlq_rust::fast::decode(encoded)
}

#[bench]
fn test_fast_u8(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| roundtrip_fast(rng.gen::<u8>() as u64))
}

#[bench]
fn test_fast_u16(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| roundtrip_fast(rng.gen::<u16>() as u64))
}

#[bench]
fn test_fast_u32(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| roundtrip_fast(rng.gen::<u32>() as u64))
}

#[bench]
fn test_fast_u64(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| roundtrip_fast(rng.gen()))
}

#[bench]
fn test_fast_u8_encode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| encode_fast(rng.gen::<u8>() as u64))
}

#[bench]
fn test_fast_u16_encode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| encode_fast(rng.gen::<u16>() as u64))
}

#[bench]
fn test_fast_u32_encode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| encode_fast(rng.gen::<u32>() as u64))
}

#[bench]
fn test_fast_u64_encode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| encode_fast(rng.gen()))
}

#[bench]
fn test_fast_u8_decode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let value = vlq_rust::fast::encode(rng.gen::<u8>() as u64);
    b.iter(|| decode_fast(value))
}

#[bench]
fn test_fast_u16_decode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let value = vlq_rust::fast::encode(rng.gen::<u16>() as u64);
    b.iter(|| decode_fast(value))
}

#[bench]
fn test_fast_u32_decode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let value = vlq_rust::fast::encode(rng.gen::<u32>() as u64);
    b.iter(|| decode_fast(value))
}

#[bench]
fn test_fast_u64_decode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let value = vlq_rust::fast::encode(rng.gen::<u64>() as u64);
    b.iter(|| decode_fast(value))
}

#[bench]
fn test_u8(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| roundtrip(rng.gen::<u8>()));
}

#[bench]
fn test_u16(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| roundtrip(rng.gen::<u16>()));
}

#[bench]
fn test_u32(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| roundtrip(rng.gen::<u32>()));
}

#[bench]
fn test_u64(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| roundtrip(rng.gen::<u64>()));
}

#[bench]
fn test_u128(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(|| roundtrip(rng.gen::<u128>()));
}

#[bench]
fn test_u8_decode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut buf = Vec::with_capacity(32);
    buf.write_vlq(rng.gen::<u8>()).expect("successful write");
    let mut cursor = Cursor::new(buf);
    b.iter(|| {
        cursor.set_position(0);
        let _: u8 = cursor.read_vlq().expect("successful read");
    });
}

#[bench]
fn test_u16_decode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut buf = Vec::with_capacity(32);
    buf.write_vlq(rng.gen::<u16>()).expect("successful write");
    let mut cursor = Cursor::new(buf);
    b.iter(|| {
        cursor.set_position(0);
        let _: u16 = cursor.read_vlq().expect("successful read");
    });
}

#[bench]
fn test_u32_decode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut buf = Vec::with_capacity(32);
    buf.write_vlq(rng.gen::<u32>()).expect("successful write");
    let mut cursor = Cursor::new(buf);
    b.iter(|| {
        cursor.set_position(0);
        let _: u32 = cursor.read_vlq().expect("successful read");
    });
}

#[bench]
fn test_u64_decode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut buf = Vec::with_capacity(32);
    buf.write_vlq(rng.gen::<u64>()).expect("successful write");
    let mut cursor = Cursor::new(buf);
    b.iter(|| {
        cursor.set_position(0);
        let _: u64 = cursor.read_vlq().expect("successful read");
    });
}

#[bench]
fn test_u128_decode(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut buf = Vec::with_capacity(32);
    buf.write_vlq(rng.gen::<u128>()).expect("successful write");
    let mut cursor = Cursor::new(buf);
    b.iter(|| {
        cursor.set_position(0);
        let _: u128 = cursor.read_vlq().expect("successful read");
    });
}
