This is a safe wrapper around the high-performance [libb2 C
implementation](https://github.com/BLAKE2/libb2) of the Blake2 hash. It
exposes all the configuration options that Blake2 supports. It also uses
libb2's `--enable-fat` option by default, so that binaries take
advantage of x86 SIMD instructions, but only if they're supported at
runtime.

Based on [`libb2-sys`](https://github.com/cesarb/libb2-sys) by @cmr and
@cesarb and [`blake2-rfc`](https://github.com/cesarb/blake2-rfc) by
@cesarb.
