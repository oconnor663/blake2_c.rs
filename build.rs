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
        run(
            Command::new("./configure")
                .arg("--prefix")
                .arg(&dst)
                // We're statically linking. Skip building the shared libs.
                .arg("--enable-shared=no")
                // The "fat library" figures out at runtime what x86 SIMD
                // extensions it can use. Without this option, binaries build
                // on new machines might not run if copied to older machines.
                .arg("--enable-fat")
                .current_dir(&src),
        );
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
