use super::*;

#[test]
fn test_empty_blake2b() {
    let hash = blake2b::State::new(blake2b::OUTBYTES).finalize().hex();
    assert_eq!(
        "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce",
        &*hash
    );

    // Make sure the builder gives the same answer.
    let hash2 = blake2b::Builder::new().build().finalize().hex();
    assert_eq!(
        "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce",
        &*hash2
    );
}

#[test]
fn test_empty_blake2s() {
    let hash = blake2s::State::new(blake2s::OUTBYTES).finalize().hex();
    assert_eq!(
        "69217a3079908094e11121d042354a7c1f55b6482ca1a51e1b250dfd1ed0eef9",
        &*hash
    );

    // Make sure the builder gives the same answer.
    let hash2 = blake2s::Builder::new().build().finalize().hex();
    assert_eq!(
        "69217a3079908094e11121d042354a7c1f55b6482ca1a51e1b250dfd1ed0eef9",
        &*hash2
    );
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

#[cfg(feature = "std")]
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

#[cfg(feature = "std")]
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

#[test]
fn test_constants_match() {
    // We copy the constant by value, so that they show up clearly in the
    // docs instead of being opaque references. Make sure their values still
    // match.
    let vendored = &[
        blake2b::BLOCKBYTES,
        blake2b::OUTBYTES,
        blake2b::KEYBYTES,
        blake2b::SALTBYTES,
        blake2b::PERSONALBYTES,
        blake2s::BLOCKBYTES,
        blake2s::OUTBYTES,
        blake2s::KEYBYTES,
        blake2s::SALTBYTES,
        blake2s::PERSONALBYTES,
    ];
    let original = &[
        sys::blake2b_constant_BLAKE2B_BLOCKBYTES as usize,
        sys::blake2b_constant_BLAKE2B_OUTBYTES as usize,
        sys::blake2b_constant_BLAKE2B_KEYBYTES as usize,
        sys::blake2b_constant_BLAKE2B_SALTBYTES as usize,
        sys::blake2b_constant_BLAKE2B_PERSONALBYTES as usize,
        sys::blake2s_constant_BLAKE2S_BLOCKBYTES as usize,
        sys::blake2s_constant_BLAKE2S_OUTBYTES as usize,
        sys::blake2s_constant_BLAKE2S_KEYBYTES as usize,
        sys::blake2s_constant_BLAKE2S_SALTBYTES as usize,
        sys::blake2s_constant_BLAKE2S_PERSONALBYTES as usize,
    ];
    assert_eq!(vendored, original);
}

#[test]
#[should_panic]
fn test_finalize_twice_panics_blake2b() {
    let mut state = blake2b::State::new(32);
    state.finalize();
    state.finalize();
}

#[test]
#[should_panic]
fn test_finalize_twice_panics_blake2s() {
    let mut state = blake2s::State::new(32);
    state.finalize();
    state.finalize();
}

#[cfg(feature = "std")]
#[test]
fn test_debug_repr() {
    // Check that the debug output has a "<none>" string when there's no key,
    // but not a "<redacted>" string.
    let mut builder = blake2b::Builder::new();
    assert!(format!("{:?}", builder).find("key=<none>").is_some());
    assert!(format!("{:?}", builder).find("key=<redacted>").is_none());

    // Check that the debug output has "<redacted>" after setting the key, and
    // also that it doesn't have the key bytes anywhere.
    builder.key(b"foo");
    assert!(format!("{:?}", builder).find("key=<none>").is_none());
    assert!(format!("{:?}", builder).find("key=<redacted>").is_some());
    assert!(format!("{:?}", builder).find("foo").is_none());
    assert!(format!("{:?}", builder).find("666f6f").is_none());
    assert!(format!("{:?}", builder).find("102, 111, 111").is_none());
}
