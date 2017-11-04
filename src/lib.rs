extern crate byteorder;
extern crate clear_on_drop;
extern crate libb2_sys;

pub const BLOCKBYTES: usize = libb2_sys::BLAKE2B_BLOCKBYTES as usize;
pub const OUTBYTES: usize = libb2_sys::BLAKE2B_OUTBYTES as usize;
pub const KEYBYTES: usize = libb2_sys::BLAKE2B_KEYBYTES as usize;
pub const SALTBYTES: usize = libb2_sys::BLAKE2B_SALTBYTES as usize;
pub const PERSONALBYTES: usize = libb2_sys::BLAKE2B_PERSONALBYTES as usize;

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
        let mut inner: libb2_sys::blake2b_state;
        unsafe {
            inner = std::mem::uninitialized();
            libb2_sys::blake2b_init_param(&mut inner, &self.params);
            // TODO: key block
        }
        Blake2bState { inner }
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

pub struct Blake2bState {
    inner: libb2_sys::blake2b_state,
}
