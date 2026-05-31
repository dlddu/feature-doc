//! Envelope encryption: round-trip, tamper detection, wrong-KEK rejection.

use featuredoc::crypto::{open, seal, Envelope};

#[test]
fn seal_then_open_round_trips() {
    let kek = [3u8; 32];
    let secret = b"sk-ant-api03-super-secret-value";
    let env = seal(&kek, secret).unwrap();

    // The ciphertext must not contain the plaintext bytes.
    assert!(!contains(&env.ciphertext, secret));

    let recovered = open(&kek, &env).unwrap();
    assert_eq!(recovered, secret);
}

#[test]
fn tampered_ciphertext_is_rejected() {
    let kek = [5u8; 32];
    let mut env = seal(&kek, b"hello-world-secret").unwrap();
    env.ciphertext[0] ^= 0xFF;
    assert!(open(&kek, &env).is_err());
}

#[test]
fn wrong_kek_cannot_open() {
    let env = seal(&[1u8; 32], b"another-secret-key").unwrap();
    assert!(open(&[2u8; 32], &env).is_err());
}

#[test]
fn distinct_seals_use_distinct_nonces() {
    let kek = [9u8; 32];
    let a = seal(&kek, b"same-input").unwrap();
    let b = seal(&kek, b"same-input").unwrap();
    // Random per-record nonces => identical plaintext seals to different ciphertext.
    assert_ne!(a.ciphertext, b.ciphertext);
    assert_ne!(a.nonce, b.nonce);
}

#[test]
fn malformed_nonce_is_rejected() {
    let env = Envelope {
        ciphertext: vec![0; 16],
        nonce: vec![0; 5],
        wrapped_dek: vec![0; 48],
        dek_nonce: vec![0; 12],
    };
    assert!(open(&[0u8; 32], &env).is_err());
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}
