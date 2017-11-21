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

# The align tests run into several failures:
#   - max_align_t always seems to fail
#     https://github.com/rust-lang-nursery/rust-bindgen/issues/550.
#   - *_state (but not *_param) has a different size on 32-bit platforms.
#   - Alignment is different on Windows.
#     https://github.com/rust-lang-nursery/rust-bindgen/issues/1009#issuecomment-342041560
# We can solve the first two with blacklisting and dynamically generating the
# bindings, but not the third one. The only solution I know of right now is to
# disable tests :(
command = ["bindgen", str(ref_header), "--no-layout-tests",
           "--ctypes-prefix=::cty"]
with (root / "src/sys.rs").open("w") as output:
    run(command, stdout=output, check=True)
