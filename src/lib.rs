//! `blake2_c` is a safe Rust wrapper around the [C implementation of
//! BLAKE2](https://github.com/BLAKE2/BLAKE2). It exposes all the parameters
//! that Blake2 supports, like personalization and tree hashing.
//!
//! By default it builds the `ref` implementation, but if you use
//! `--features native` it will build the `sse` implementation. This gives
//! about an 8% speedup on my machine, but the resulting binary is probably
//! not portable.
//!
//! Originally based on [`libb2-sys`](https://github.com/cesarb/libb2-sys) by
//! @cmr and @cesarb and [`blake2-rfc`](https://github.com/cesarb/blake2-rfc)
//! by @cesarb.
//!
//! - [Docs](https://jacko.io/rustdoc/blake2_c)
//! - [Crate](https://crates.io/crates/blake2_c)
//! - [Repo](https://github.com/oconnor663/blake2_c.rs)

extern crate arrayvec;
extern crate constant_time_eq;
extern crate cty;

use std::fmt;
use std::mem;
use cty::c_void;
use arrayvec::{ArrayVec, ArrayString};
use constant_time_eq::constant_time_eq;

#[allow(warnings)]
mod sys;

#[cfg(test)]
mod test;

/// An all-at-once convenience function for Blake2b-512.
pub fn blake2b_512(input: &[u8]) -> Digest {
    blake2b::State::new(64).update(input).finalize()
}

/// An all-at-once convenience function for Blake2b-256.
pub fn blake2b_256(input: &[u8]) -> Digest {
    blake2b::State::new(32).update(input).finalize()
}

/// An all-at-once convenience function for Blake2s-256.
pub fn blake2s_256(input: &[u8]) -> Digest {
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
        $init_param_fn:path,
        $update_fn:path,
        $finalize_fn:path,
        $node_offset_max:expr,
        $xof_length_type:ty,
    } => {
#[$moddoc]
pub mod $name {
    use super::*;

    /// The size of an input block, mostly an implementation detail.
    pub const BLOCKBYTES: usize = $blockbytes;
    /// The maximum digest length.
    pub const OUTBYTES: usize = $outbytes;
    /// The maximum secret key length.
    pub const KEYBYTES: usize = $keybytes;
    /// The maximum salt length.
    pub const SALTBYTES: usize = $saltbytes;
    /// The maximum personalization length.
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
            // Zeroing the params helps us avoid dealing with the `reserved`
            // field difference between 32-bit and 64-bit, and it's safe
            // because the struct is plain old data.
            let mut params: $param_type = unsafe { mem::zeroed() };
            params.digest_length = OUTBYTES as u8;
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
            let ret = unsafe {
                state = State(mem::zeroed());
                $init_param_fn(&mut state.0, &self.params)
            };
            // Errors from init should be impossible in the current C
            // implementation, but we check them in case that changes.
            assert_eq!(ret, 0, "Blake2 init returned an error");
            // Assert that outlen gets set, since we rely on this later.
            debug_assert_eq!(self.params.digest_length as usize, state.0.outlen);
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
            assert!(1 <= length && length <= OUTBYTES, "Bad digest length: {}", length);
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
            assert!(key.len() <= KEYBYTES, "Bad key length: {}", key.len());
            self.key_block = [0; BLOCKBYTES];
            self.key_block[..key.len()].copy_from_slice(key);
            self.params.key_length = key.len() as u8;
            self
        }

        /// From 0 (meaning unlimited) to 255. The default is 1 (meaning
        /// sequential).
        pub fn fanout(&mut self, fanout: usize) -> &mut Self {
            assert!(fanout <= 255, "Bad fanout: {}", fanout);
            self.params.fanout = fanout as u8;
            self
        }

        /// From 1 (the default, meaning sequential) to 255 (meaning
        /// unlimited).
        pub fn max_depth(&mut self, depth: usize) -> &mut Self {
            assert!(1 <= depth && depth <= 255, "Bad max depth: {}", depth);
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
            assert!(offset <= $node_offset_max, "Bad node offset: {}", offset);
            // The version of "blake2.h" we're using includes the xof_length
            // param from BLAKE2X, which occupies the high bits of node_offset.
            // NOTE: Tricky endianness issues, https://github.com/BLAKE2/libb2/issues/12.
            self.params.node_offset = (offset as u32).to_le();
            self.params.xof_length = ((offset >> 32) as $xof_length_type).to_le();
            self
        }

        /// From 0 (the default, meaning leaf or sequential) to 255.
        pub fn node_depth(&mut self, depth: usize) -> &mut Self {
            assert!(depth <= 255, "Bad node depth: {}", depth);
            self.params.node_depth = depth as u8;
            self
        }

        /// From 0 (the default, meaning sequential) to `OUTBYTES`.
        pub fn inner_hash_length(&mut self, length: usize) -> &mut Self {
            assert!(length <= OUTBYTES, "Bad inner hash length: {}", length);
            self.params.inner_length = length as u8;
            self
        }

        /// At most `SALTBYTES` bytes. Shorter salts are padded with null
        /// bytes. An empty salt is equivalent to having no salt at all.
        pub fn salt(&mut self, salt: &[u8]) -> &mut Self {
            assert!(salt.len() <= SALTBYTES, "Bad salt length: {}", salt.len());
            self.params.salt = [0; SALTBYTES];
            self.params.salt[..salt.len()].copy_from_slice(salt);
            self
        }

        /// At most `PERSONALBYTES` bytes. Shorter personalizations are padded
        /// with null bytes. An empty personalization is equivalent to having
        /// no personalization at all.
        pub fn personal(&mut self, personal: &[u8]) -> &mut Self {
            assert!(personal.len() <= PERSONALBYTES, "Bad personalization length: {}", personal.len());
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
            Builder::new().digest_length(digest_length).build()
        }

        /// Write input to the hash. You can call `update` any number of times.
        pub fn update(&mut self, input: &[u8]) -> &mut Self {
            // Errors from update should be impossible in the current C
            // implementation, but we check them in case that changes.
            let ret = unsafe {
                $update_fn(&mut self.0, input.as_ptr() as *const c_void, input.len())
            };
            assert_eq!(ret, 0, "Blake2 update returned an error");
            self
        }

        /// Return the final hash. `finalize` takes `&mut self` so that you can
        /// chain method calls together easily, but calling it more than once
        /// on the same state will panic.
        pub fn finalize(&mut self) -> Digest {
            let mut bytes = ArrayVec::new();
            let ret = unsafe {
                bytes.set_len(self.0.outlen);
                $finalize_fn(&mut self.0, bytes.as_mut_ptr() as *mut c_void, bytes.len())
            };
            // The current C implementation sets a finalize flag, and calling
            // finalize a second time is an error.
            assert_eq!(ret, 0, "Blake2 finalize returned an error");
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
    sys::blake2s_init_param,
    sys::blake2s_update,
    sys::blake2s_final,
    ((1 << 48) - 1),
    u16,
}

/// A finalized Blake2 hash.
///
/// `Digest` supports constant-time equality checks, for cases where Blake2 is
/// being used as a MAC. It uses an
/// [`ArrayVec`](https://docs.rs/arrayvec/0.4.6/arrayvec/struct.ArrayVec.html)
/// to hold various digest lengths without needing to allocate on the heap. It
/// could support `no_std`, though that's not yet implemented.
#[derive(Clone, Debug)]
pub struct Digest {
    // blake2b::OUTBYTES is the largest possible digest length for either algorithm.
    pub bytes: ArrayVec<[u8; blake2b::OUTBYTES]>,
}

impl Digest {
    /// Convert the digest to a hexadecimal string. Because we know the maximum
    /// length of the string in advance (`2 * OUTBYTES`), we can use an
    /// [`ArrayString`](https://docs.rs/arrayvec/0.4.6/arrayvec/struct.ArrayString.html)
    /// to avoid allocating.
    pub fn hex(&self) -> ArrayString<[u8; 2 * blake2b::OUTBYTES]> {
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
