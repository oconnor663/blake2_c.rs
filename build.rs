// File copied from libb2-sys.

use std::path::*;
use std::process::*;
use std::env;
use std::io::*;

fn main() {
    let src = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("libb2");
    let dst = PathBuf::from(&env::var("OUT_DIR").unwrap());

    // `make` will automatically rerun `./configure` if the timestamps imply
    // that it needs to.
    if !src.join("Makefile").is_file() {
        let mut configure_cmd = Command::new("./configure");
        configure_cmd.current_dir(&src);
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
    }
    run(Command::new("make").current_dir(&src));
    run(Command::new("make").arg("install").current_dir(&src));

    println!("cargo:rustc-flags=-l static=b2");
    println!("cargo:rustc-flags=-L {}", dst.join("lib").display());
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
