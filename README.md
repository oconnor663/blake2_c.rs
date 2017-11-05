# blake2_c

This is a safe Rust wrapper around the [C implementation of
BLAKE2](https://github.com/BLAKE2/BLAKE2). It exposes all the parameters
that Blake2 supports, like personalization and tree hashing.

By default it builds the `ref` implementation, but if you use
`--features native` it will build the `sse` implementation. This gives
about an 8% speedup on my machine, but the resulting binary is probably
not portable, and it doesn't currently work on Windows.

Originally based on [`libb2-sys`](https://github.com/cesarb/libb2-sys)
by @cmr and @cesarb and [`blake2-rfc`](https://github.com/cesarb/blake2-rfc)
by @cesarb.
