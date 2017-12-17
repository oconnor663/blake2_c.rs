extern crate cc;

use std::env;

fn main() {
    if env::var_os("CARGO_FEATURE_AVX2").is_some() {
        cc::Build::new()
            .file("./BLAKE2/avx2/blake2b.c")
            // GCC and Clang
            .flag_if_supported("-march=native")
            // MSVC
            .flag_if_supported("/arch:AVX2")
            .compile("blake2");
    } else if env::var_os("CARGO_FEATURE_NATIVE").is_some() {
        cc::Build::new()
            .file("./BLAKE2/sse/blake2b.c")
            .file("./BLAKE2/sse/blake2s.c")
            // GCC and Clang
            .flag_if_supported("-march=native")
            // MSVC
            .flag_if_supported("/arch:AVX")
            .compile("blake2");
    } else {
        cc::Build::new()
            .file("./BLAKE2/ref/blake2b-ref.c")
            .file("./BLAKE2/ref/blake2s-ref.c")
            .compile("blake2");
    }

    // We'd like to use bindgen here at compile time, as per the bindgen docs
    // (https://rust-lang-nursery.github.io/rust-bindgen/tutorial-3.html),
    // rather than checking in the generated file. Unfortunately, that requires
    // libclang to be installed, which causes problems on AppVeyor and probably
    // plenty of users' machines. It also makes the build much slower.
}
