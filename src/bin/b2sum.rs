/// This is a trivial copy of the b2sum command line utility, mainly for
/// benchmarking.

extern crate libb2;

use std::io::stdin;
use std::io::prelude::*;

fn main() {
    let stdin = stdin();
    let mut stdin_lock = stdin.lock();
    let mut state = libb2::blake2b::State::new(64);
    // Using a big buffer like this is slightly more efficient than copy().
    let mut buf = [0; 65536];
    loop {
        let n = stdin_lock.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        state.update(&buf[..n]);
    }
    println!("{}", state.finalize().hex());
}
