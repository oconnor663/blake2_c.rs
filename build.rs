extern crate cc;

use std::env;

fn main() {
    if env::var_os("CARGO_FEATURE_NATIVE").is_some() {
        cc::Build::new()
            .file("./BLAKE2/sse/blake2b.c")
            .file("./BLAKE2/sse/blake2s.c")
            // MSVC doens't seem to have an equivalent for -march=native.
            .flag_if_supported("-march=native")
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
