//! Envelope encryption for credentials at rest (AC4.3a) plus a `SecretString`
//! whose `Debug`/`Display` never reveal the wrapped value (AC4.3c).
//!
//! Scheme: each secret gets a freshly generated 256-bit data-encryption key
//! (DEK). The secret is sealed with the DEK using AES-256-GCM; the DEK itself
//! is then wrapped with the key-encryption key (KEK) that lives only in
//! process memory (injected from a k8s Secret). Only the wrapped DEK and the
//! ciphertext are persisted — never the plaintext, never the bare DEK.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// A secret value that refuses to print itself. Use this for any plaintext
/// credential held transiently in memory so an accidental `{:?}` in a log or
/// error can never leak it.
#[derive(Clone)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        SecretString(value.into())
    }

    /// Escape hatch for the single call site that actually needs the bytes
    /// (encryption, or the outbound provider request). Named loudly on purpose.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretString(REDACTED)")
    }
}

impl std::fmt::Display for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("REDACTED")
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        SecretString(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    // Note: the underlying aes-gcm error is opaque by design and carries no
    // plaintext, so it is safe to surface. We still keep the message generic.
    #[error("cryptographic operation failed")]
    Aead,
    #[error("malformed key material")]
    BadKey,
}

/// The key-encryption key. 32 bytes, loaded once at startup.
#[derive(Clone)]
pub struct Kek([u8; 32]);

impl Kek {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Kek(bytes)
    }

    fn cipher(&self) -> Aes256Gcm {
        Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.0))
    }
}

impl std::fmt::Debug for Kek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Kek(REDACTED)")
    }
}

/// The persisted shape of an enveloped secret. None of these fields reveal the
/// plaintext: `dek_wrapped`/`ciphertext` are AES-GCM outputs, `fingerprint` is
/// a one-way hash, `masked` shows only a non-sensitive tail.
#[derive(Debug, Clone)]
pub struct Envelope {
    pub dek_wrapped: Vec<u8>,
    pub dek_nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub ciphertext_nonce: Vec<u8>,
    /// SHA-256 of the plaintext, hex, first 16 chars — lets us detect
    /// duplicate registrations without storing the value.
    pub fingerprint: String,
    /// e.g. `sk-ant-…last4` — safe to show in the UI.
    pub masked: String,
}

fn random_nonce() -> [u8; 12] {
    let mut n = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut n);
    n
}

fn random_key() -> [u8; 32] {
    let mut k = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut k);
    k
}

pub fn fingerprint(plaintext: &str) -> String {
    let digest = Sha256::digest(plaintext.as_bytes());
    hex::encode(digest)[..16].to_string()
}

/// Show only enough of a credential to recognise it: keep any provider prefix
/// up to the first run of value, then the last 4 characters.
pub fn mask(plaintext: &str) -> String {
    let len = plaintext.chars().count();
    if len <= 8 {
        return "•".repeat(len.max(4));
    }
    let prefix: String = plaintext.chars().take(7).collect();
    let suffix: String = plaintext.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("{prefix}••••••{suffix}")
}

/// Seal a plaintext secret into an `Envelope`. The plaintext is never copied
/// anywhere it could outlive this call.
pub fn seal(kek: &Kek, plaintext: &SecretString) -> Result<Envelope, CryptoError> {
    let pt = plaintext.expose();

    let dek = random_key();
    let dek_cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&dek));

    let ciphertext_nonce = random_nonce();
    let ciphertext = dek_cipher
        .encrypt(Nonce::from_slice(&ciphertext_nonce), pt.as_bytes())
        .map_err(|_| CryptoError::Aead)?;

    // Wrap the DEK with the KEK.
    let dek_nonce = random_nonce();
    let dek_wrapped = kek
        .cipher()
        .encrypt(Nonce::from_slice(&dek_nonce), dek.as_slice())
        .map_err(|_| CryptoError::Aead)?;

    Ok(Envelope {
        dek_wrapped,
        dek_nonce: dek_nonce.to_vec(),
        ciphertext,
        ciphertext_nonce: ciphertext_nonce.to_vec(),
        fingerprint: fingerprint(pt),
        masked: mask(pt),
    })
}

/// Recover the plaintext from an `Envelope`. Returns a `SecretString` so the
/// recovered value is, again, un-loggable by default.
pub fn open(kek: &Kek, env: &Envelope) -> Result<SecretString, CryptoError> {
    let dek = kek
        .cipher()
        .decrypt(Nonce::from_slice(&env.dek_nonce), env.dek_wrapped.as_slice())
        .map_err(|_| CryptoError::Aead)?;
    if dek.len() != 32 {
        return Err(CryptoError::BadKey);
    }
    let dek_cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&dek));
    let pt = dek_cipher
        .decrypt(
            Nonce::from_slice(&env.ciphertext_nonce),
            env.ciphertext.as_slice(),
        )
        .map_err(|_| CryptoError::Aead)?;
    let s = String::from_utf8(pt).map_err(|_| CryptoError::Aead)?;
    Ok(SecretString::new(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_kek() -> Kek {
        Kek::from_bytes([7u8; 32])
    }

    #[test]
    fn roundtrip_recovers_plaintext() {
        let kek = test_kek();
        let secret = SecretString::new("sk-ant-supersecretvalue1234");
        let env = seal(&kek, &secret).unwrap();
        let opened = open(&kek, &env).unwrap();
        assert_eq!(opened.expose(), secret.expose());
    }

    #[test]
    fn ciphertext_never_contains_plaintext() {
        let kek = test_kek();
        let secret = SecretString::new("sk-ant-supersecretvalue1234");
        let env = seal(&kek, &secret).unwrap();
        let needle = b"supersecret";
        assert!(!env.ciphertext.windows(needle.len()).any(|w| w == needle));
        assert!(!env.dek_wrapped.windows(needle.len()).any(|w| w == needle));
    }

    #[test]
    fn each_seal_uses_fresh_dek_and_nonce() {
        let kek = test_kek();
        let secret = SecretString::new("sk-ant-supersecretvalue1234");
        let a = seal(&kek, &secret).unwrap();
        let b = seal(&kek, &secret).unwrap();
        // Same plaintext, different ciphertext/wrapping every time.
        assert_ne!(a.ciphertext, b.ciphertext);
        assert_ne!(a.dek_wrapped, b.dek_wrapped);
        // ...but a stable fingerprint, so duplicates are detectable.
        assert_eq!(a.fingerprint, b.fingerprint);
    }

    #[test]
    fn wrong_kek_cannot_open() {
        let env = seal(&test_kek(), &SecretString::new("sk-ant-abcdefghijklmnop")).unwrap();
        let other = Kek::from_bytes([9u8; 32]);
        assert!(open(&other, &env).is_err());
    }

    #[test]
    fn secret_debug_and_display_are_redacted() {
        let s = SecretString::new("sk-ant-topsecret");
        assert_eq!(format!("{s:?}"), "SecretString(REDACTED)");
        assert_eq!(format!("{s}"), "REDACTED");
        assert!(!format!("{s:?} {s}").contains("topsecret"));
    }

    #[test]
    fn mask_keeps_only_prefix_and_tail() {
        let m = mask("sk-ant-abcdefghijklmnop4242");
        assert!(m.starts_with("sk-ant-"));
        assert!(m.ends_with("4242"));
        assert!(!m.contains("abcdefghij"));
    }
}
