//! `blake2_c` is a safe Rust wrapper around the [C implementation of
//! BLAKE2](https://github.com/BLAKE2/BLAKE2). It exposes all the parameters
//! that Blake2 supports, like personalization and tree hashing.
//!
//! By default it builds the `ref` implementation, but if you use
//! `--features native` it will build the `sse` implementation. This gives
//! about an 8% speedup on my machine, but the resulting binary is probably
//! not portable, and it doesn't currently work on Windows.
//!
//! Originally based on [`libb2-sys`](https://github.com/cesarb/libb2-sys) by
//! @cmr and @cesarb and [`blake2-rfc`](https://github.com/cesarb/blake2-rfc)
//! by @cesarb.
//!
//! - [Docs](https://docs.rs/blake2_c)
//! - [Crate](https://crates.io/crates/blake2_c)
//! - [Repo](https://github.com/oconnor663/blake2_c.rs)

extern crate arrayvec;
extern crate constant_time_eq;

use std::fmt;
use std::mem;
use std::os::raw::c_void;
use arrayvec::{ArrayVec, ArrayString};
use constant_time_eq::constant_time_eq;

#[allow(warnings)]
mod sys;

/// An all-at-once convenience function for Blake2b-512.
pub fn blake2b_512(input: &[u8]) -> blake2b::Digest {
    blake2b::State::new(64).update(input).finalize()
}

/// An all-at-once convenience function for Blake2b-256.
pub fn blake2b_256(input: &[u8]) -> blake2b::Digest {
    blake2b::State::new(32).update(input).finalize()
}

/// An all-at-once convenience function for Blake2s-256.
pub fn blake2s_256(input: &[u8]) -> blake2s::Digest {
    blake2s::State::new(32).update(input).finalize()
}

macro_rules! blake2_impl {
    {
        $name:ident,
        $moddoc:meta,
        $blockbytes:expr,
        $outbytes:expr,
        $keybytes:expr,
        $saltbytes:expr,
        $personalbytes:expr,
        $param_type:path,
        $state_type:path,
        $init_fn:path,
        $init_param_fn:path,
        $update_fn:path,
        $finalize_fn:path,
        $node_offset_max:expr,
        $xof_length_type:ty,
    } => {
#[$moddoc]
pub mod $name {
    use super::*;

    pub const BLOCKBYTES: usize = $blockbytes;
    pub const OUTBYTES: usize = $outbytes;
    pub const KEYBYTES: usize = $keybytes;
    pub const SALTBYTES: usize = $saltbytes;
    pub const PERSONALBYTES: usize = $personalbytes;

    /// A builder for `State` that lets you set all the various Blake2
    /// parameters.
    ///
    /// Apart from `digest_length`, all of these parameters are just associated
    /// data for the hash. They help you guarantee that hashes used for
    /// different applications will never collide. For all the details, see
    /// [the Blake2 spec](https://blake2.net/blake2.pdf).
    ///
    /// Most of the builder methods will panic if their input is too large or
    /// too small, as defined by the spec.
    #[derive(Clone)]
    pub struct Builder {
        params: $param_type,
        key_block: [u8; BLOCKBYTES as usize],
        last_node: bool,
    }

    impl Builder {
        /// Create a new `Builder` with all the default paramters. For example,
        /// `Builder::new().build()` would give the same state as
        /// `State::new(OUTBYTES)`.
        pub fn new() -> Self {
            let mut params: $param_type = unsafe { mem::zeroed() };
            params.fanout = 1;
            params.depth = 1;
            Self {
                params,
                // We don't currently attempt to zero the key bytes on drop. The
                // builder could get moved around in the stack in any case, and
                // drop wouldn't clear old bytes after a move. Callers who care
                // about this might want to look at clear_on_drop::clear_stack.
                key_block: [0; BLOCKBYTES],
                last_node: false,
            }
        }

        /// Create a `State` instance with all the parameters from this
        /// `Builder`.
        pub fn build(&self) -> State {
            let mut state;
            unsafe {
                state = State(mem::zeroed());
                $init_param_fn(&mut state.0, &self.params);
            }
            if self.last_node {
                state.0.last_node = 1;
            }
            if self.params.key_length > 0 {
                state.update(&self.key_block);
            }
            state
        }

        /// Set the length of the final hash. This is associated data too, so
        /// changing the length will give a totally different hash. The maximum
        /// digest length is `OUTBYTES`.
        pub fn digest_length(&mut self, length: usize) -> &mut Self {
            if length == 0 || length > KEYBYTES {
                panic!("Bad digest length: {}", length);
            }
            self.params.digest_length = length as u8;
            self
        }

        /// Use a secret key, so that Blake2 acts as a MAC. The maximum key
        /// length is `KEYBYTES`. An empty key is equivalent to having no key
        /// at all. Also note that neither `Builder` nor `State` zeroes out
        /// their memory on drop, so callers who worry about keys sticking
        /// around in memory need to zero their own stacks. See for example the
        /// [`clear_on_drop`](https://crates.io/crates/clear_on_drop) crate.
        pub fn key(&mut self, key: &[u8]) -> &mut Self {
            if key.len() > KEYBYTES {
                panic!("Bad key length: {}", key.len());
            }
            self.key_block = [0; BLOCKBYTES];
            self.key_block[..key.len()].copy_from_slice(key);
            self.params.key_length = key.len() as u8;
            self
        }

        /// From 0 (meaning unlimited) to 255. The default is 1 (meaning
        /// sequential).
        pub fn fanout(&mut self, fanout: usize) -> &mut Self {
            if fanout > 255 {
                panic!("Bad fanout: {}", fanout);
            }
            self.params.fanout = fanout as u8;
            self
        }

        /// From 1 (the default, meaning sequential) to 255 (meaning
        /// unlimited).
        pub fn max_depth(&mut self, depth: usize) -> &mut Self {
            if depth == 0 || depth > 255 {
                panic!("Bad max depth: {}", depth);
            }
            self.params.depth = depth as u8;
            self
        }

        /// From 0 (the default, meaning unlimited or sequential) to `2^32 - 1`.
        pub fn max_leaf_length(&mut self, length: u32) -> &mut Self {
            // NOTE: Tricky endianness issues, https://github.com/BLAKE2/libb2/issues/12.
            self.params.leaf_length = length.to_le();
            self
        }

        /// From 0 (the default, meaning first, leftmost, leaf, or sequential)
        /// to `2^64 - 1` in Blake2b, or to `2^48 - 1` in Blake2s.
        pub fn node_offset(&mut self, offset: u64) -> &mut Self {
            if offset > $node_offset_max {
                panic!("Bad node offset: {}", offset);
            }
            // The version of "blake2.h" we're using includes the xof_length
            // param from BLAKE2X, which occupies the high bits of node_offset.
            // NOTE: Tricky endianness issues, https://github.com/BLAKE2/libb2/issues/12.
            self.params.node_offset = (offset as u32).to_le();
            self.params.xof_length = ((offset >> 32) as $xof_length_type).to_le();
            self
        }

        /// From 0 (the default, meaning leaf or sequential) to 255.
        pub fn node_depth(&mut self, depth: usize) -> &mut Self {
            if depth > 255 {
                panic!("Bad node depth: {}", depth);
            }
            self.params.node_depth = depth as u8;
            self
        }

        /// From 0 (the default, meaning sequential) to `OUTBYTES`.
        pub fn inner_hash_length(&mut self, length: usize) -> &mut Self {
            if length > OUTBYTES {
                panic!("Bad inner hash length: {}", length);
            }
            self.params.inner_length = length as u8;
            self
        }

        /// At most `SALTBYTES` bytes. Shorter salts are padded with null
        /// bytes. An empty salt is equivalent to having no salt at all.
        pub fn salt(&mut self, salt: &[u8]) -> &mut Self {
            if salt.len() > SALTBYTES {
                panic!("Bad salt length: {}", salt.len());
            }
            self.params.salt = [0; SALTBYTES];
            self.params.salt[..salt.len()].copy_from_slice(salt);
            self
        }

        /// At most `PERSONALBYTES` bytes. Shorter personalizations are padded
        /// with null bytes. An empty personalization is equivalent to having
        /// no personalization at all.
        pub fn personal(&mut self, personal: &[u8]) -> &mut Self {
            if personal.len() > PERSONALBYTES {
                panic!("Bad personalization length: {}", personal.len());
            }
            self.params.personal = [0; PERSONALBYTES];
            self.params.personal[..personal.len()].copy_from_slice(personal);
            self
        }

        /// Indicates the last node of a layer in tree-hashing modes.
        pub fn last_node(&mut self, last: bool) -> &mut Self {
            self.last_node = last;
            self
        }
    }

    impl fmt::Debug for Builder {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Builder {{ params: ")?;
            fmt::Debug::fmt(&self.params, f)?;
            write!(f, ", last_node: {}, key=<redacted> }}", self.last_node)
        }
    }

    /// Computes a Blake2 hash incrementally.
    #[derive(Clone)]
    pub struct State($state_type);

    impl State {
        /// Create a new hash state with the given digest length, and default
        /// values for all the other parameters. If you need to set other
        /// Blake2 parameters, including keying, use the `Builder` instead.
        pub fn new(digest_length: usize) -> Self {
            if digest_length == 0 || digest_length > OUTBYTES {
                panic!("Bad digest length: {}", digest_length);
            }
            let mut state;
            unsafe {
                state = State(mem::zeroed());
                $init_fn(&mut state.0, digest_length);
            }
            state
        }

        /// Write input to the hash. You can call `update` any number of times.
        pub fn update(&mut self, input: &[u8]) -> &mut Self {
            unsafe {
                $update_fn(&mut self.0, input.as_ptr() as *const c_void, input.len());
            }
            self
        }

        /// Return the final hash. `finalize` takes `&mut self` so that you can
        /// chain method calls together easily, but calling it more than once
        /// on the same state will give you a garbage result.
        pub fn finalize(&mut self) -> Digest {
            let mut bytes = ArrayVec::new();
            unsafe {
                bytes.set_len(self.0.outlen);
                $finalize_fn(&mut self.0, bytes.as_mut_ptr() as *mut c_void, bytes.len());
            }
            Digest { bytes }
        }
    }

    impl fmt::Debug for State {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "State {{ outlen: {}, ... }}", self.0.outlen)
        }
    }

    impl std::io::Write for State {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.update(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// A finalized Blake2 hash.
    ///
    /// `Digest` supports constant-time equality checks, for cases where Blake2
    /// is being used as a MAC. It uses an
    /// [`ArrayVec`](https://docs.rs/arrayvec/0.4.6/arrayvec/struct.ArrayVec.html)
    /// to hold various digest lengths without needing to allocate on the heap.
    /// It could support `no_std`, though that's not yet implemented.
    #[derive(Clone, Debug)]
    pub struct Digest {
        pub bytes: ArrayVec<[u8; OUTBYTES]>,
    }

    impl Digest {
        /// Convert the digest to a hexadecimal string. Because we know the
        /// maximum length of the string in advance (`2 * OUTBYTES`), we can
        /// use an
        /// [`ArrayString`](https://docs.rs/arrayvec/0.4.6/arrayvec/struct.ArrayString.html)
        /// to avoid allocating.
        pub fn hex(&self) -> ArrayString<[u8; 2 * OUTBYTES]> {
            use std::fmt::Write;
            let mut hexdigest = ArrayString::new();
            for &b in &self.bytes {
                write!(&mut hexdigest, "{:02x}", b).expect("too many bytes");
            }
            hexdigest
        }
    }

    /// This implementation is constant time, if the two digests are the same
    /// length.
    impl PartialEq for Digest {
        fn eq(&self, other: &Digest) -> bool {
            constant_time_eq(&self.bytes, &other.bytes)
        }
    }

    impl Eq for Digest {}
}
}} // end of blake2_impl!

blake2_impl! {
    blake2b,
    doc="The more common version of Blake2, optimized for 64-bit processors.",
    128,
    64,
    64,
    16,
    16,
    sys::blake2b_param,
    sys::blake2b_state,
    sys::blake2b_init,
    sys::blake2b_init_param,
    sys::blake2b_update,
    sys::blake2b_final,
    u64::max_value(),
    u32,
}

blake2_impl! {
    blake2s,
    doc="The less common version of Blake2, optimized for smaller processors.",
    64,
    32,
    32,
    8,
    8,
    sys::blake2s_param,
    sys::blake2s_state,
    sys::blake2s_init,
    sys::blake2s_init_param,
    sys::blake2s_update,
    sys::blake2s_final,
    ((1 << 48) - 1),
    u16,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_blake2b() {
        let hash = blake2b::State::new(16).finalize().hex();
        assert_eq!("cae66941d9efbd404e4d88758ea67670", &*hash);

        // Make sure the builder gives the same answer.
        let hash2 = blake2b::Builder::new()
            .digest_length(16)
            .build()
            .finalize()
            .hex();
        assert_eq!("cae66941d9efbd404e4d88758ea67670", &*hash2);
    }

    #[test]
    fn test_empty_blake2s() {
        let hash = blake2s::State::new(16).finalize().hex();
        assert_eq!("64550d6ffe2c0a01a14aba1eade0200c", &*hash);

        // Make sure the builder gives the same answer.
        let hash2 = blake2s::Builder::new()
            .digest_length(16)
            .build()
            .finalize()
            .hex();
        assert_eq!("64550d6ffe2c0a01a14aba1eade0200c", &*hash2);
    }

    #[test]
    fn test_foo_blake2b() {
        let hash = blake2b::State::new(16).update(b"foo").finalize().hex();
        assert_eq!("04136e24f85d470465c3db66e58ed56c", &*hash);

        // Make sure feeding one byte at a time gives the same answer.
        let hash2 = blake2b::State::new(16)
            .update(b"f")
            .update(b"o")
            .update(b"o")
            .finalize()
            .hex();
        assert_eq!("04136e24f85d470465c3db66e58ed56c", &*hash2);
    }

    #[test]
    fn test_foo_blake2s() {
        let hash = blake2s::State::new(16).update(b"foo").finalize().hex();
        assert_eq!("4447d20921efe4103c56a695dcaafa38", &*hash);

        // Make sure feeding one byte at a time gives the same answer.
        let hash2 = blake2s::State::new(16)
            .update(b"f")
            .update(b"o")
            .update(b"o")
            .finalize()
            .hex();
        assert_eq!("4447d20921efe4103c56a695dcaafa38", &*hash2);
    }

    #[test]
    fn test_large_input_blake2b() {
        let input = vec![0; 1_000_000];
        // Check several different digest lengths.
        let answers = &[
            "15",
            "b930",
            "459494",
            "93a83d45",
            "28e7fa6b489b7557",
            "6990ee96760194861181a9ddeadd4007",
            "0cbf381956ec0d36533b813283c85bc12142a0512ae86f59e0d4342af99010b6",
            "2b5e760175daa6f07397df9dce3b40aaa47ba59b513c15b523ffc2a086a2f9c05a0ac4251c869cca0f3b67478d3933c604705a0bf041030c2d7d0578e3f783",
            "9ef8b51be521c6e33abb22d6a69363902b6d7eb67ca1364ebc87a64d5a36ec5e749e5c9e7029a85b0008e46cff24281e87500886818dbe79dc8e094f119bbeb8",
        ];
        for &answer in answers {
            // While we're at it, test the std::io::Write interface.
            use std::io::Write;
            let mut state = blake2b::State::new(answer.len() / 2);
            state.write_all(&input).unwrap();
            let hash = state.finalize().hex();
            assert_eq!(answer, &*hash);
        }
    }

    #[test]
    fn test_large_input_blake2s() {
        let input = vec![0; 1_000_000];
        // Check several different digest lengths.
        let answers = &[
            "e3",
            "1c79",
            "6a2d52",
            "583d8010",
            "265882c701630caf",
            "658eed8bb2da916e98b5eba781322926",
            "a1a8bd1ccdb681cb8fa9373639a2e88dbb1bbfc52aea4a703233ea197e87bc",
            "cc07784ef067dd3e05f2d0720933ef177846b9719b1e0741c607aca3ff7a38ae",
        ];
        for &answer in answers {
            // While we're at it, test the std::io::Write interface.
            use std::io::Write;
            let mut state = blake2s::State::new(answer.len() / 2);
            state.write_all(&input).unwrap();
            let hash = state.finalize().hex();
            assert_eq!(answer, &*hash);
        }
    }

    #[test]
    fn test_all_parameters_blake2b() {
        let hash = blake2b::Builder::new()
            .digest_length(17)
            .key(b"bar")
            .salt(b"baz")
            .personal(b"bing")
            .fanout(2)
            .max_depth(3)
            .max_leaf_length(0x04050607)
            .node_offset(0x08090a0b0c0d0e0f)
            .node_depth(16)
            .inner_hash_length(17)
            .last_node(true)
            .build()
            .update(b"foo")
            .finalize()
            .hex();
        assert_eq!("0dea28da297ebeb1abb7fdd4c573887349", &*hash);
    }

    #[test]
    fn test_all_parameters_blake2s() {
        let hash = blake2s::Builder::new()
            .digest_length(17)
            .key(b"bar")
            .salt(b"baz")
            .personal(b"bing")
            .fanout(2)
            .max_depth(3)
            .max_leaf_length(0x04050607)
            .node_offset(0x08090a0b0c0d)
            .node_depth(16)
            .inner_hash_length(17)
            .last_node(true)
            .build()
            .update(b"foo")
            .finalize()
            .hex();
        assert_eq!("179b9a70409efca3310998dd8aacc0a5dd", &*hash);
    }

    #[test]
    fn test_one_off_functions() {
        assert_eq!(
            &*blake2b_512(b"abc").hex(),
            "ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923"
        );
        assert_eq!(
            &*blake2b_256(b"abc").hex(),
            "bddd813c634239723171ef3fee98579b94964e3bb1cb3e427262c8c068d52319"
        );
        assert_eq!(
            &*blake2s_256(b"abc").hex(),
            "508c5e8c327c14e2e1a72ba34eeb452f37458b209ed63a294d999b4c86675982"
        );
    }

    #[test]
    fn test_param_struct_size() {
        // These are part of the spec: https://blake2.net/blake2.pdf.
        assert_eq!(64, mem::size_of::<sys::blake2b_param>());
        assert_eq!(32, mem::size_of::<sys::blake2s_param>());
    }
}
