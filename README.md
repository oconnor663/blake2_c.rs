# blake2_c.rs [![Travis build](https://travis-ci.org/oconnor663/blake2_c.rs.svg?branch=master)](https://travis-ci.org/oconnor663/blake2_c.rs) [![AppVeyor build](https://ci.appveyor.com/api/projects/status/9g5e5ji73197so2e/branch/master?svg=true)](https://ci.appveyor.com/project/oconnor663/blake2-c-rs/branch/master) [![crates.io](https://img.shields.io/crates/v/blake2_c.svg)](https://crates.io/crates/blake2_c) [![docs.rs](https://docs.rs/blake2_c/badge.svg)](https://docs.rs/blake2_c)

`blake2_c` is a safe Rust wrapper around the [C implementation of
BLAKE2](https://github.com/BLAKE2/BLAKE2). It exposes all the parameters
that Blake2 supports, like personalization and tree hashing.

By default it builds the `ref` implementation, but if you use
`--features native` it will build the `sse` implementation. This gives
about an 8% speedup on my machine, but the resulting binary is probably
not portable, and it doesn't currently work on Windows.

Originally based on [`libb2-sys`](https://github.com/cesarb/libb2-sys) by
@cmr and @cesarb and [`blake2-rfc`](https://github.com/cesarb/blake2-rfc)
by @cesarb.

- [Docs](https://docs.rs/blake2_c)
- [Crate](https://crates.io/crates/blake2_c)
- [Repo](https://github.com/oconnor663/blake2_c.rs)
