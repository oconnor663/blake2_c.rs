#![feature(test)]

extern crate blake2_c;
extern crate test;

use test::Bencher;

#[bench]
fn blake2b_100bytes(b: &mut Bencher) {
    b.iter(|| blake2_c::blake2b_512(&[0; 100]));
}

#[bench]
fn blake2b_1kb(b: &mut Bencher) {
    b.iter(|| blake2_c::blake2b_512(&[0; 1_000]));
}

#[bench]
fn blake2b_1mb(b: &mut Bencher) {
    b.iter(|| blake2_c::blake2b_512(&[0; 1_000_000]));
}

#[bench]
fn blake2s_100bytes(b: &mut Bencher) {
    b.iter(|| blake2_c::blake2s_256(&[0; 100]));
}

#[bench]
fn blake2s_1kb(b: &mut Bencher) {
    b.iter(|| blake2_c::blake2s_256(&[0; 1_000]));
}

#[bench]
fn blake2s_1mb(b: &mut Bencher) {
    b.iter(|| blake2_c::blake2s_256(&[0; 1_000_000]));
}
