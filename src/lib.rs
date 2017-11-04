extern crate arrayvec;
extern crate byteorder;
extern crate clear_on_drop;
extern crate hex;
extern crate libb2_sys;

use arrayvec::ArrayVec;

pub const BLOCKBYTES: usize = libb2_sys::BLAKE2B_BLOCKBYTES as usize;
pub const OUTBYTES: usize = libb2_sys::BLAKE2B_OUTBYTES as usize;
pub const KEYBYTES: usize = libb2_sys::BLAKE2B_KEYBYTES as usize;
pub const SALTBYTES: usize = libb2_sys::BLAKE2B_SALTBYTES as usize;
pub const PERSONALBYTES: usize = libb2_sys::BLAKE2B_PERSONALBYTES as usize;

// TODO: Clone, Debug
pub struct Blake2bBuilder {
    params: libb2_sys::blake2b_param,
    key_block: [u8; BLOCKBYTES as usize],
    last_node: bool,
}

impl Blake2bBuilder {
    pub fn new() -> Self {
        Self {
            params: libb2_sys::blake2b_param {
                digest_length: OUTBYTES as u8,
                key_length: 0,
                fanout: 1,
                depth: 1,
                leaf_length: 0,
                node_offset: 0,
                node_depth: 0,
                inner_length: 0,
                reserved: [0; 14],
                salt: [0; 16],
                personal: [0; 16],
            },
            key_block: [0; BLOCKBYTES],
            last_node: false,
        }
    }

    pub fn build(&self) -> Blake2bState {
        let mut state;
        unsafe {
            state = Blake2bState(std::mem::zeroed());
            libb2_sys::blake2b_init_param(&mut state.0, &self.params);
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

impl Drop for Blake2bBuilder {
    fn drop(&mut self) {
        clear_on_drop::clear::Clear::clear(&mut self.key_block[..]);
    }
}

// TODO: Clone, Debug
pub struct Blake2bState(libb2_sys::blake2b_state);

impl Blake2bState {
    /// Create a new hash state with the given digest length. For all the other
    /// Blake2 parameters, including keying, use a builder instead.
    pub fn new(digest_length: usize) -> Self {
        if digest_length == 0 || digest_length > OUTBYTES {
            panic!("Bad digest length: {}", digest_length);
        }
        let mut state;
        unsafe {
            state = Blake2bState(std::mem::zeroed());
            libb2_sys::blake2b_init(&mut state.0, digest_length);
        }
        state
    }

    pub fn update(&mut self, input: &[u8]) -> &mut Self {
        unsafe {
            libb2_sys::blake2b_update(&mut self.0, input.as_ptr(), input.len());
        }
        self
    }

    /// Return the bytes of the final hash. `finalize` takes `&mut self` for
    /// convenience, but calling it more than once on the same instance is a
    /// logic error.
    // TODO: Return a wrapped type that can constant-time-eq and to_hex itself.
    pub fn finalize(&mut self) -> ArrayVec<[u8; OUTBYTES]> {
        let mut out = ArrayVec::new();
        unsafe {
            out.set_len(self.0.outlen as usize);
            libb2_sys::blake2b_final(&mut self.0, out.as_mut_ptr(), out.len());
        }
        out
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hex::ToHex;

    #[test]
    fn test_empty() {
        let hash = Blake2bState::new(16).finalize().to_hex();
        assert_eq!("cae66941d9efbd404e4d88758ea67670", hash);

        // Make sure the builder gives the same answer.
        let hash2 = Blake2bBuilder::new()
            .digest_length(16)
            .build()
            .finalize()
            .to_hex();
        assert_eq!("cae66941d9efbd404e4d88758ea67670", hash2);
    }

    #[test]
    fn test_foo() {
        let hash = Blake2bState::new(16).update(b"foo").finalize().to_hex();
        assert_eq!("04136e24f85d470465c3db66e58ed56c", hash);

        // Make sure feeding one byte at a time gives the same answer.
        let hash2 = Blake2bState::new(16)
            .update(b"f")
            .update(b"o")
            .update(b"o")
            .finalize()
            .to_hex();
        assert_eq!("04136e24f85d470465c3db66e58ed56c", hash2);
    }

    #[test]
    fn test_large_input() {
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
            let hash = Blake2bState::new(answer.len() / 2)
                .update(&input)
                .finalize()
                .to_hex();
            assert_eq!(answer, hash);
        }
    }

    #[test]
    fn test_all_parameters() {
        let hash = Blake2bBuilder::new()
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
            .to_hex();
        assert_eq!("8cf9408d6c57cb17802e24821631a881dc", hash);
    }
}
