// Adapted from libb2-sys.

extern crate walkdir;

use std::path::*;
use std::process::*;
use std::env;
use std::fs::{copy, create_dir_all, metadata, remove_dir_all};
use std::io::*;
use walkdir::WalkDir;

fn main() {
    let src_orig = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("libb2");
    let dst = PathBuf::from(&env::var("OUT_DIR").unwrap());

    // Copy all the source files before running ./configure. That keeps Cargo
    // from thinking files have "changed" when ./configure touches them, so it
    // doesn't redo the whole build every time.
    let src_copy = dst.join("libb2_src");
    if src_copy.exists() {
        println!("removing {:?}", src_copy);
        remove_dir_all(&src_copy).unwrap();
    }
    println!("copying {:?} to {:?}", src_orig, src_copy);
    for entry in WalkDir::new(&src_orig) {
        let entry = entry.unwrap();
        let entry_dest = src_copy.join(entry.path().strip_prefix(&src_orig).unwrap());
        if entry.file_type().is_dir() {
            create_dir_all(entry_dest).unwrap();
        } else {
            copy(entry.path(), entry_dest).unwrap();
        }
        // Also explicitly tell Cargo to rerun build.rs if any of the original
        // files change. This means that Cargo won't waste time rerunning
        // build.rs if we only make changes on the Rust side.
        println!("cargo:rerun-if-changed={}", entry.path().to_str().unwrap());
    }

    run(Command::new("./autogen.sh").current_dir(&src_copy));
    let mut configure_cmd = Command::new("./configure");
    configure_cmd.current_dir(&src_copy);
    configure_cmd.arg("--prefix");
    configure_cmd.arg(&dst);
    // We're statically linking. Skip building the shared libs.
    configure_cmd.arg("--enable-shared=no");
    if env::var_os("CARGO_FEATURE_NATIVE").is_some() {
        // This is the deafault for libb2, and we're just specifying it explicitly.
        configure_cmd.arg("--enable-native=yes");
    } else {
        // The "fat library" figures out at runtime what x86 SIMD
        // extensions it can use. Without this option, binaries build
        // on new machines might not run if copied to older machines.
        configure_cmd.arg("--enable-fat");
    }
    run(&mut configure_cmd);
    run(Command::new("make").arg("install").current_dir(&src_copy));

    println!("cargo:rustc-link-lib=static=b2");
    println!("cargo:rustc-link-search={}", dst.join("lib").display());
}

fn run(cmd: &mut Command) {
    println!("running: {:?}", cmd);
    let status = match cmd.status() {
        Ok(status) => status,
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            fail(&format!(
                "failed to execute command: {}\nnot installed?",
                e,
            ));
        }
        Err(e) => fail(&format!("failed to execute command: {}", e)),
    };
    if !status.success() {
        fail(&format!(
            "command did not execute successfully, got: {}",
            status
        ));
    }
}

fn fail(s: &str) -> ! {
    panic!("\n{}\n\nbuild script failed, must exit now", s)
}
