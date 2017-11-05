extern crate arrayvec;
extern crate byteorder;
extern crate constant_time_eq;

use std::mem;
use arrayvec::{ArrayVec, ArrayString};
use constant_time_eq::constant_time_eq;

// The libb2-sys package on crates.io is out of date as of Nov 2017, and its
// build script is kind of wasteful, so we include our own bindings. If this
// turns out to conflict with another package, though, we can figure out how to
// share them.
mod sys;

pub fn blake2b_512(input: &[u8]) -> blake2b::Digest {
    blake2b::State::new(64).update(input).finalize()
}

pub fn blake2b_256(input: &[u8]) -> blake2b::Digest {
    blake2b::State::new(32).update(input).finalize()
}

pub fn blake2s_256(input: &[u8]) -> blake2s::Digest {
    blake2s::State::new(32).update(input).finalize()
}

pub mod blake2b {
    use super::*;

    pub const BLOCKBYTES: usize = sys::BLAKE2B_BLOCKBYTES as usize;
    pub const OUTBYTES: usize = sys::BLAKE2B_OUTBYTES as usize;
    pub const KEYBYTES: usize = sys::BLAKE2B_KEYBYTES as usize;
    pub const SALTBYTES: usize = sys::BLAKE2B_SALTBYTES as usize;
    pub const PERSONALBYTES: usize = sys::BLAKE2B_PERSONALBYTES as usize;

    // TODO: Clone, Debug
    pub struct Builder {
        params: sys::blake2b_param,
        key_block: [u8; BLOCKBYTES as usize],
        last_node: bool,
    }

    impl Builder {
        pub fn new() -> Self {
            Self {
                params: sys::blake2b_param {
                    digest_length: OUTBYTES as u8,
                    key_length: 0,
                    fanout: 1,
                    depth: 1,
                    leaf_length: 0,
                    node_offset: 0,
                    node_depth: 0,
                    inner_length: 0,
                    reserved: [0; 14],
                    salt: [0; SALTBYTES],
                    personal: [0; PERSONALBYTES],
                },
                // We don't currently attempt to zero the key bytes on drop. The
                // builder could get moved around in the stack in any case, and
                // drop wouldn't clear old bytes after a move. Callers who care
                // about this might want to look at clear_on_drop::clear_stack.
                key_block: [0; BLOCKBYTES],
                last_node: false,
            }
        }

        pub fn build(&self) -> State {
            let mut state;
            unsafe {
                state = State(mem::zeroed());
                sys::blake2b_init_param(&mut state.0, &self.params);
            }
            if self.last_node {
                state.0.last_node = 1;
            }
            if self.params.key_length > 0 {
                state.update(&self.key_block);
            }
            state
        }

        pub fn digest_length(&mut self, length: usize) -> &mut Self {
            if length == 0 || length > OUTBYTES {
                panic!("Bad digest length: {}", length);
            }
            self.params.digest_length = length as u8;
            self
        }

        /// An empty key is equivalent to having no key at all.
        pub fn key(&mut self, key: &[u8]) -> &mut Self {
            if key.len() > KEYBYTES {
                panic!("Bad key length: {}", key.len());
            }
            self.key_block = [0; BLOCKBYTES];
            self.key_block[..key.len()].copy_from_slice(key);
            self.params.key_length = key.len() as u8;
            self
        }

        pub fn fanout(&mut self, fanout: usize) -> &mut Self {
            if fanout > 255 {
                panic!("Bad fanout: {}", fanout);
            }
            self.params.fanout = fanout as u8;
            self
        }

        pub fn max_depth(&mut self, depth: usize) -> &mut Self {
            if depth == 0 || depth > 255 {
                panic!("Bad max depth: {}", depth);
            }
            self.params.depth = depth as u8;
            self
        }

        pub fn max_leaf_length(&mut self, length: u32) -> &mut Self {
            // NOTE: Tricky endianness issues, https://github.com/BLAKE2/libb2/issues/12.
            self.params.leaf_length = length.to_le();
            self
        }

        pub fn node_offset(&mut self, offset: u64) -> &mut Self {
            // NOTE: Tricky endianness issues, https://github.com/BLAKE2/libb2/issues/12.
            self.params.node_offset = offset.to_le();
            self
        }

        pub fn node_depth(&mut self, depth: usize) -> &mut Self {
            if depth > 255 {
                panic!("Bad node depth: {}", depth);
            }
            self.params.node_depth = depth as u8;
            self
        }

        pub fn inner_hash_length(&mut self, length: usize) -> &mut Self {
            if length > OUTBYTES {
                panic!("Bad inner hash length: {}", length);
            }
            self.params.inner_length = length as u8;
            self
        }

        pub fn salt(&mut self, salt: &[u8]) -> &mut Self {
            if salt.len() > SALTBYTES {
                panic!("Bad salt length: {}", salt.len());
            }
            self.params.salt = [0; SALTBYTES];
            self.params.salt[..salt.len()].copy_from_slice(salt);
            self
        }

        pub fn personal(&mut self, personal: &[u8]) -> &mut Self {
            if personal.len() > PERSONALBYTES {
                panic!("Bad personalization length: {}", personal.len());
            }
            self.params.personal = [0; PERSONALBYTES];
            self.params.personal[..personal.len()].copy_from_slice(personal);
            self
        }

        pub fn last_node(&mut self, last: bool) -> &mut Self {
            self.last_node = last;
            self
        }
    }

    // TODO: Clone, Debug
    pub struct State(sys::blake2b_state);

    impl State {
        /// Create a new hash state with the given digest length. For all the other
        /// Blake2 parameters, including keying, use a builder instead.
        pub fn new(digest_length: usize) -> Self {
            if digest_length == 0 || digest_length > OUTBYTES {
                panic!("Bad digest length: {}", digest_length);
            }
            let mut state;
            unsafe {
                state = State(mem::zeroed());
                sys::blake2b_init(&mut state.0, digest_length);
            }
            state
        }

        pub fn update(&mut self, input: &[u8]) -> &mut Self {
            unsafe {
                sys::blake2b_update(&mut self.0, input.as_ptr(), input.len());
            }
            self
        }

        /// Return the finalized hash. `finalize` takes `&mut self` so that you can
        /// chain method calls together easily, but calling it more than once on
        /// the same instance will give you a garbage result.
        pub fn finalize(&mut self) -> Digest {
            let mut bytes = ArrayVec::new();
            unsafe {
                bytes.set_len(self.0.outlen as usize);
                sys::blake2b_final(&mut self.0, bytes.as_mut_ptr(), bytes.len());
            }
            Digest { bytes }
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

    /// Holds a Blake2 hash. Supports constant-time equality, for cases where
    /// Blake2 is being used as a MAC. Althought digest lengths can vary at
    /// runtime, this type uses a statically-allocated ArrayVec. It could support
    /// `no_std`, though that's not yet implemented.
    #[derive(Clone, Debug)]
    pub struct Digest {
        pub bytes: ArrayVec<[u8; OUTBYTES]>,
    }

    impl Digest {
        pub fn hex(&self) -> ArrayString<[u8; 2 * OUTBYTES]> {
            use std::fmt::Write;
            let mut hexdigest = ArrayString::new();
            for &b in &self.bytes {
                write!(&mut hexdigest, "{:02x}", b).expect("too many bytes");
            }
            hexdigest
        }
    }

    impl PartialEq for Digest {
        fn eq(&self, other: &Digest) -> bool {
            constant_time_eq(&self.bytes, &other.bytes)
        }
    }

    impl Eq for Digest {}
}

pub mod blake2s {
    use super::*;

    pub const BLOCKBYTES: usize = sys::BLAKE2S_BLOCKBYTES as usize;
    pub const OUTBYTES: usize = sys::BLAKE2S_OUTBYTES as usize;
    pub const KEYBYTES: usize = sys::BLAKE2S_KEYBYTES as usize;
    pub const SALTBYTES: usize = sys::BLAKE2S_SALTBYTES as usize;
    pub const PERSONALBYTES: usize = sys::BLAKE2S_PERSONALBYTES as usize;

    // TODO: Clone, Debug
    pub struct Builder {
        params: sys::blake2s_param,
        key_block: [u8; BLOCKBYTES as usize],
        last_node: bool,
    }

    impl Builder {
        pub fn new() -> Self {
            Self {
                params: sys::blake2s_param {
                    digest_length: OUTBYTES as u8,
                    key_length: 0,
                    fanout: 1,
                    depth: 1,
                    leaf_length: 0,
                    node_offset: [0; 6],
                    node_depth: 0,
                    inner_length: 0,
                    salt: [0; SALTBYTES],
                    personal: [0; PERSONALBYTES],
                },
                // We don't currently attempt to zero the key bytes on drop. The
                // builder could get moved around in the stack in any case, and
                // drop wouldn't clear old bytes after a move. Callers who care
                // about this might want to look at clear_on_drop::clear_stack.
                key_block: [0; BLOCKBYTES],
                last_node: false,
            }
        }

        pub fn build(&self) -> State {
            let mut state;
            unsafe {
                state = State(mem::zeroed());
                sys::blake2s_init_param(&mut state.0, &self.params);
            }
            if self.last_node {
                state.0.last_node = 1;
            }
            if self.params.key_length > 0 {
                state.update(&self.key_block);
            }
            state
        }

        pub fn digest_length(&mut self, length: usize) -> &mut Self {
            if length == 0 || length > OUTBYTES {
                panic!("Bad digest length: {}", length);
            }
            self.params.digest_length = length as u8;
            self
        }

        /// An empty key is equivalent to having no key at all.
        pub fn key(&mut self, key: &[u8]) -> &mut Self {
            if key.len() > KEYBYTES {
                panic!("Bad key length: {}", key.len());
            }
            self.key_block = [0; BLOCKBYTES];
            self.key_block[..key.len()].copy_from_slice(key);
            self.params.key_length = key.len() as u8;
            self
        }

        pub fn fanout(&mut self, fanout: usize) -> &mut Self {
            if fanout > 255 {
                panic!("Bad fanout: {}", fanout);
            }
            self.params.fanout = fanout as u8;
            self
        }

        pub fn max_depth(&mut self, depth: usize) -> &mut Self {
            if depth == 0 || depth > 255 {
                panic!("Bad max depth: {}", depth);
            }
            self.params.depth = depth as u8;
            self
        }

        pub fn max_leaf_length(&mut self, length: u32) -> &mut Self {
            // NOTE: Tricky endianness issues, https://github.com/BLAKE2/libb2/issues/12.
            self.params.leaf_length = length.to_le();
            self
        }

        pub fn node_offset(&mut self, offset: u64) -> &mut Self {
            use byteorder::{ByteOrder, LittleEndian};
            if offset > ((1 << 48) - 1) {
                panic!("Bad node offset: {}", offset);
            }
            let mut buf = [0; 8];
            LittleEndian::write_u64(&mut buf, offset);
            self.params.node_offset[..].copy_from_slice(&buf[..6]);
            self
        }

        pub fn node_depth(&mut self, depth: usize) -> &mut Self {
            if depth > 255 {
                panic!("Bad node depth: {}", depth);
            }
            self.params.node_depth = depth as u8;
            self
        }

        pub fn inner_hash_length(&mut self, length: usize) -> &mut Self {
            if length > OUTBYTES {
                panic!("Bad inner hash length: {}", length);
            }
            self.params.inner_length = length as u8;
            self
        }

        pub fn salt(&mut self, salt: &[u8]) -> &mut Self {
            if salt.len() > SALTBYTES {
                panic!("Bad salt length: {}", salt.len());
            }
            self.params.salt = [0; SALTBYTES];
            self.params.salt[..salt.len()].copy_from_slice(salt);
            self
        }

        pub fn personal(&mut self, personal: &[u8]) -> &mut Self {
            if personal.len() > PERSONALBYTES {
                panic!("Bad personalization length: {}", personal.len());
            }
            self.params.personal = [0; PERSONALBYTES];
            self.params.personal[..personal.len()].copy_from_slice(personal);
            self
        }

        pub fn last_node(&mut self, last: bool) -> &mut Self {
            self.last_node = last;
            self
        }
    }

    // TODO: Clone, Debug
    pub struct State(sys::blake2s_state);

    impl State {
        /// Create a new hash state with the given digest length. For all the other
        /// Blake2 parameters, including keying, use a builder instead.
        pub fn new(digest_length: usize) -> Self {
            if digest_length == 0 || digest_length > OUTBYTES {
                panic!("Bad digest length: {}", digest_length);
            }
            let mut state;
            unsafe {
                state = State(mem::zeroed());
                sys::blake2s_init(&mut state.0, digest_length);
            }
            state
        }

        pub fn update(&mut self, input: &[u8]) -> &mut Self {
            unsafe {
                sys::blake2s_update(&mut self.0, input.as_ptr(), input.len());
            }
            self
        }

        /// Return the finalized hash. `finalize` takes `&mut self` so that you can
        /// chain method calls together easily, but calling it more than once on
        /// the same instance will give you a garbage result.
        pub fn finalize(&mut self) -> Digest {
            let mut bytes = ArrayVec::new();
            unsafe {
                bytes.set_len(self.0.outlen as usize);
                sys::blake2s_final(&mut self.0, bytes.as_mut_ptr(), bytes.len());
            }
            Digest { bytes }
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

    /// Holds a Blake2 hash. Supports constant-time equality, for cases where
    /// Blake2 is being used as a MAC. Althought digest lengths can vary at
    /// runtime, this type uses a statically-allocated ArrayVec. It could support
    /// `no_std`, though that's not yet implemented.
    #[derive(Clone, Debug)]
    pub struct Digest {
        pub bytes: ArrayVec<[u8; OUTBYTES]>,
    }

    impl Digest {
        pub fn hex(&self) -> ArrayString<[u8; 2 * OUTBYTES]> {
            use std::fmt::Write;
            let mut hexdigest = ArrayString::new();
            for &b in &self.bytes {
                write!(&mut hexdigest, "{:02x}", b).expect("too many bytes");
            }
            hexdigest
        }
    }

    impl PartialEq for Digest {
        fn eq(&self, other: &Digest) -> bool {
            constant_time_eq(&self.bytes, &other.bytes)
        }
    }

    impl Eq for Digest {}
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
            .max_leaf_length(4)
            .node_offset(5)
            .node_depth(6)
            .inner_hash_length(7)
            .last_node(true)
            .build()
            .update(b"foo")
            .finalize()
            .hex();
        assert_eq!("8cf9408d6c57cb17802e24821631a881dc", &*hash);
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
            .max_leaf_length(4)
            .node_offset(5)
            .node_depth(6)
            .inner_hash_length(7)
            .last_node(true)
            .build()
            .update(b"foo")
            .finalize()
            .hex();
        assert_eq!("a6a418a8c97d2c7801b3c8c86aa243aa75", &*hash);
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
}
