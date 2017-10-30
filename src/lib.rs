extern crate libb2_sys;
extern crate clear_on_drop;

pub const BLOCKBYTES: usize = libb2_sys::BLAKE2B_BLOCKBYTES as usize;
pub const OUTBYTES: usize = libb2_sys::BLAKE2B_OUTBYTES as usize;
pub const KEYBYTES: usize = libb2_sys::BLAKE2B_KEYBYTES as usize;
pub const SALTBYTES: usize = libb2_sys::BLAKE2B_SALTBYTES as usize;
pub const PERSONALBYTES: usize = libb2_sys::BLAKE2B_PERSONALBYTES as usize;

pub struct Blake2bBuilder {
    params: libb2_sys::blake2b_param,
    key: [u8; KEYBYTES as usize],
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
            key: [0; KEYBYTES],
            last_node: false,
        }
    }

    pub fn digest_length(&mut self, len: usize) {
        if len < 1 {
            panic!("Digest length must be at least 1.");
        }
        if len > OUTBYTES {
            panic!("Digest length must be at most {} bytes.", OUTBYTES);
        }
        self.params.digest_length = len as u8;
    }

    pub fn key(&mut self, key: &[u8]) {
        if key.len() > KEYBYTES {
            panic!("Key must be at most {} bytes.", KEYBYTES);
        }
        self.key = [0; KEYBYTES];
        self.key[..key.len()].copy_from_slice(key);
        self.params.key_length = key.len() as u8;
    }

    pub fn salt(&mut self, salt: &[u8]) {
        if salt.len() > SALTBYTES {
            panic!("Salt must be at most {} bytes.", SALTBYTES);
        }
        self.params.salt = [0; SALTBYTES];
        self.params.salt[..salt.len()].copy_from_slice(salt);
    }

    pub fn personal(&mut self, personal: &[u8]) {
        if personal.len() > PERSONALBYTES {
            panic!("Personalization must be at most {} bytes.", PERSONALBYTES);
        }
        self.params.personal = [0; SALTBYTES];
        self.params.personal[..personal.len()].copy_from_slice(personal);
    }

    pub fn fanout(&mut self, fanout: u8) {
        self.params.fanout = fanout;
    }

    pub fn max_depth(&mut self, depth: u8) {
        if depth == 0 {
            panic!("Max depth must be at least 1.");
        }
        self.params.depth = depth;
    }

    pub fn max_leaf_length(&mut self, len: u32) {
        self.params.leaf_length = len;
    }
}

impl Drop for Blake2bBuilder {
    fn drop(&mut self) {
        clear_on_drop::clear::Clear::clear(&mut self.key[..]);
    }
}
