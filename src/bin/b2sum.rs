/// This is a trivial copy of the b2sum command line utility, mainly for
/// benchmarking.

extern crate libb2;

use std::io::{copy, stdin};

fn main() {
    let mut state = libb2::blake2b::State::new(64);
    copy(&mut stdin(), &mut state).unwrap();
    println!("{}", state.finalize().hex());
}
