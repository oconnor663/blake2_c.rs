#! /usr/bin/env python3

# This script generates sys.rs, which is checked in.

from pathlib import Path
from subprocess import run

root = Path(__file__).parent / ".."
ref_header = root / "BLAKE2/ref/blake2.h"
sse_header = root / "BLAKE2/sse/blake2.h"

# Make sure that the two headers are identical, since we're using the same
# generated file in both modes.
assert ref_header.open().read() == sse_header.open().read()

command = ["bindgen", str(ref_header)]
command += ["--constified-enum-module", "blake2b_constant"]
command += ["--constified-enum-module", "blake2s_constant"]
# If we don't blacklist this symbol we get a test failure:
# https://github.com/rust-lang-nursery/rust-bindgen/issues/550.
command += ["--blacklist-type", "max_align_t"]
with (root / "src/sys.rs").open("w") as output:
    run(command, stdout=output, check=True)
