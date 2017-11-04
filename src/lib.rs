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
    pub fn update(&mut self, input: &[u8]) -> &mut Self {
        unsafe {
            libb2_sys::blake2b_update(&mut self.0, input.as_ptr(), input.len());
        }
        self
    }

    pub fn finalize(mut self) -> ArrayVec<[u8; OUTBYTES]> {
        let mut out = ArrayVec::new();
        unsafe {
            out.set_len(self.0.outlen as usize);
            libb2_sys::blake2b_final(&mut self.0, out.as_mut_ptr(), out.len());
        }
        out
    }
}

impl Default for Blake2bState {
    fn default() -> Self {
        Blake2bBuilder::new().build()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hex::FromHex;

    #[test]
    fn test_empty() {
        let hash = Blake2bState::default().finalize();
        let expected: Vec<u8> = FromHex::from_hex(
            "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce",
        ).unwrap();
        assert_eq!(&*hash, &*expected);
    }

    #[test]
    fn test_foo() {
        let mut state = Blake2bState::default();
        state.update(b"foo");
        let hash = state.finalize();
        let expected: Vec<u8> = FromHex::from_hex(
            "ca002330e69d3e6b84a46a56a6533fd79d51d97a3bb7cad6c2ff43b354185d6dc1e723fb3db4ae0737e120378424c714bb982d9dc5bbd7a0ab318240ddd18f8d",
        ).unwrap();
        assert_eq!(&*hash, &*expected);
    }

    #[test]
    fn test_foo_letter_by_letter() {
        let mut state = Blake2bState::default();
        state.update(b"f");
        state.update(b"o");
        state.update(b"o");
        let hash = state.finalize();
        let expected: Vec<u8> = FromHex::from_hex(
            "ca002330e69d3e6b84a46a56a6533fd79d51d97a3bb7cad6c2ff43b354185d6dc1e723fb3db4ae0737e120378424c714bb982d9dc5bbd7a0ab318240ddd18f8d",
        ).unwrap();
        assert_eq!(&*hash, &*expected);
    }

    #[test]
    fn test_all_parameters() {
        let mut state = Blake2bBuilder::new()
            .digest_length(16)
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
            .build();
        state.update(b"foo");
        let hash = state.finalize();
        let expected: Vec<u8> = FromHex::from_hex("920568b0c5873b2f0ab67bedb6cf1b2b").unwrap();
        assert_eq!(&*hash, &*expected);
    }
}
