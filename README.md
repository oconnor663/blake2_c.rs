# blake2_c.rs [![Travis build](https://travis-ci.org/oconnor663/blake2_c.rs.svg?branch=master)](https://travis-ci.org/oconnor663/blake2_c.rs) [![AppVeyor build](https://ci.appveyor.com/api/projects/status/9g5e5ji73197so2e/branch/master?svg=true)](https://ci.appveyor.com/project/oconnor663/blake2-c-rs/branch/master) [![crates.io](https://img.shields.io/crates/v/blake2_c.svg)](https://crates.io/crates/blake2_c) [![docs.rs](https://docs.rs/blake2_c/badge.svg)](https://docs.rs/blake2_c)

**Deprecation:** The [`blake2b_simd`](https://github.com/oconnor663/blake2b_simd)
crate has better performance than this one, with the added benefit of being
pure Rust. This library probably won't be developed any further.
`blake2b_simd` doesn't currently support BLAKE2s, but if there are any
callers that need it I can add support or fork the crate, so let me know.

`blake2_c` is a safe Rust wrapper around the [C implementation of
BLAKE2](https://github.com/BLAKE2/BLAKE2). It exposes all the parameters
that BLAKE2 supports, like personalization and tree hashing.

By default this crate links against the portable ["ref"
implementation](https://github.com/BLAKE2/BLAKE2/tree/master/ref), but if
you turn on the `native` feature it will link against the ["sse"
implementation](https://github.com/BLAKE2/BLAKE2/tree/master/sse), which
uses SIMD instructions if your processor supports them. That gives about an
8% speedup on my machine, but the resulting binary is probably not
portable.

This crate supports `no_std`. The `std` feature is on by default, to
provide implementations of `std::io::Write`, but it can be [disabled in the
caller's `Cargo.toml`](http://doc.crates.io/manifest.html#rules) using
`default-features = false`.

Originally based on [`libb2-sys`](https://github.com/cesarb/libb2-sys) by
@cmr and @cesarb and [`blake2-rfc`](https://github.com/cesarb/blake2-rfc)
by @cesarb.

- [Docs](https://docs.rs/blake2_c)
- [Crate](https://crates.io/crates/blake2_c)
- [Repo](https://github.com/oconnor663/blake2_c.rs)
