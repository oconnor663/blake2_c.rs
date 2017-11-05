// Mostly copied from libb2-sys.

#![allow(bad_style)]

extern crate libc;

pub const BLAKE2S_BLOCKBYTES: libc::c_uint = 64;
pub const BLAKE2S_OUTBYTES: libc::c_uint = 32;
pub const BLAKE2S_KEYBYTES: libc::c_uint = 32;
pub const BLAKE2S_SALTBYTES: libc::c_uint = 8;
pub const BLAKE2S_PERSONALBYTES: libc::c_uint = 8;

pub const BLAKE2B_BLOCKBYTES: libc::c_uint = 128;
pub const BLAKE2B_OUTBYTES: libc::c_uint = 64;
pub const BLAKE2B_KEYBYTES: libc::c_uint = 64;
pub const BLAKE2B_SALTBYTES: libc::c_uint = 16;
pub const BLAKE2B_PERSONALBYTES: libc::c_uint = 16;

#[repr(C)]
pub struct blake2s_param {
    pub digest_length: u8,
    pub key_length: u8,
    pub fanout: u8,
    pub depth: u8,
    pub leaf_length: u32,
    pub node_offset: [u8; 6usize],
    pub node_depth: u8,
    pub inner_length: u8,
    pub salt: [u8; 8usize],
    pub personal: [u8; 8usize],
}

#[repr(C)]
pub struct blake2s_state {
    pub h: [u32; 8usize],
    pub t: [u32; 2usize],
    pub f: [u32; 2usize],
    pub buf: [u8; 128usize],
    pub buflen: u32,
    pub outlen: u8,
    pub last_node: u8,
}

#[repr(C)]
pub struct blake2b_param {
    pub digest_length: u8,
    pub key_length: u8,
    pub fanout: u8,
    pub depth: u8,
    pub leaf_length: u32,
    pub node_offset: u64,
    pub node_depth: u8,
    pub inner_length: u8,
    pub reserved: [u8; 14usize],
    pub salt: [u8; 16usize],
    pub personal: [u8; 16usize],
}

#[repr(C)]
pub struct blake2b_state {
    pub h: [u64; 8usize],
    pub t: [u64; 2usize],
    pub f: [u64; 2usize],
    pub buf: [u8; 256usize],
    pub buflen: u32,
    pub outlen: u8,
    pub last_node: u8,
}

extern "C" {
    pub fn blake2s_init(S: *mut blake2s_state, outlen: libc::size_t) -> libc::c_int;
    pub fn blake2s_init_param(S: *mut blake2s_state, P: *const blake2s_param) -> libc::c_int;
    pub fn blake2s_update(
        S: *mut blake2s_state,
        _in: *const u8,
        inlen: libc::size_t,
    ) -> libc::c_int;
    pub fn blake2s_final(S: *mut blake2s_state, out: *mut u8, outlen: libc::size_t) -> libc::c_int;
    pub fn blake2b_init(S: *mut blake2b_state, outlen: libc::size_t) -> libc::c_int;
    pub fn blake2b_init_param(S: *mut blake2b_state, P: *const blake2b_param) -> libc::c_int;
    pub fn blake2b_update(
        S: *mut blake2b_state,
        _in: *const u8,
        inlen: libc::size_t,
    ) -> libc::c_int;
    pub fn blake2b_final(S: *mut blake2b_state, out: *mut u8, outlen: libc::size_t) -> libc::c_int;
}
