extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    // Compile the .c files, depending on whether --features=native is active.
    let header_path;
    if env::var_os("CARGO_FEATURE_NATIVE").is_some() {
        cc::Build::new()
            .file("./BLAKE2/sse/blake2b.c")
            .file("./BLAKE2/sse/blake2s.c")
            // MSVC doens't seem to have an equivalent for -march=native.
            .flag_if_supported("-march=native")
            .compile("blake2");
        header_path = "./BLAKE2/sse/blake2.h";
    } else {
        cc::Build::new()
            .file("./BLAKE2/ref/blake2b-ref.c")
            .file("./BLAKE2/ref/blake2s-ref.c")
            .compile("blake2");
        header_path = "./BLAKE2/ref/blake2.h";
    }

    // Generate bindings.rs from blake2.h. We use the header file from ref/ or
    // sse/ as appropriate, though currently they're identical.
    let mut builder = bindgen::Builder::default()
        .header(header_path)
        .constified_enum_module("blake2b_constant")
        .constified_enum_module("blake2s_constant")
        // If we don't blacklist this max_align_t we get a test failure:
        // https://github.com/rust-lang-nursery/rust-bindgen/issues/550.
        .blacklist_type("max_align_t");
    if cfg!(windows) {
        // Layout tests aren't working on Windows.
        // https://github.com/rust-lang-nursery/rust-bindgen/issues/1009#issuecomment-342041560.
        builder = builder.layout_tests(false);
    }
    let bindings = builder.generate().expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
