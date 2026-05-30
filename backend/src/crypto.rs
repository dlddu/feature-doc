//! Envelope encryption for credential material (AC4.3).
//!
//! Each secret gets a fresh random 256-bit data-encryption key (DEK). The secret
//! is sealed with the DEK under AES-256-GCM; the DEK itself is then wrapped with
//! the process key-encryption key (KEK). Only the wrapped DEK and ciphertext are
//! persisted — the plaintext secret and the plaintext DEK never touch disk, and
//! the DEK is scrubbed from memory after sealing.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::RngCore;

use crate::error::AppError;

/// The persisted result of [`seal`]: everything needed to later [`open`] the secret,
/// none of which reveals it without the KEK.
pub struct Envelope {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub wrapped_dek: Vec<u8>,
    pub dek_nonce: Vec<u8>,
}

/// Seals `plaintext` under a fresh DEK, wrapping that DEK with `kek`.
pub fn seal(kek: &[u8; 32], plaintext: &[u8]) -> Result<Envelope, AppError> {
    let mut dek = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut dek);
    let mut nonce = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let mut dek_nonce = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut dek_nonce);

    let data_cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&dek));
    let ciphertext = data_cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|_| AppError::internal("seal: data encryption failed"))?;

    let kek_cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(kek));
    let wrapped_dek = kek_cipher
        .encrypt(Nonce::from_slice(&dek_nonce), dek.as_ref())
        .map_err(|_| AppError::internal("seal: dek wrap failed"))?;

    // Best-effort scrub of the plaintext DEK now that it is wrapped.
    dek.iter_mut().for_each(|b| *b = 0);

    Ok(Envelope {
        ciphertext,
        nonce: nonce.to_vec(),
        wrapped_dek,
        dek_nonce: dek_nonce.to_vec(),
    })
}

/// Recovers the plaintext from an [`Envelope`], unwrapping the DEK with `kek`.
/// Any tampering (GCM tag mismatch) or wrong KEK yields an error, never garbage.
pub fn open(kek: &[u8; 32], env: &Envelope) -> Result<Vec<u8>, AppError> {
    if env.nonce.len() != 12 || env.dek_nonce.len() != 12 {
        return Err(AppError::internal("open: malformed nonce"));
    }
    let kek_cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(kek));
    let mut dek = kek_cipher
        .decrypt(Nonce::from_slice(&env.dek_nonce), env.wrapped_dek.as_ref())
        .map_err(|_| AppError::internal("open: dek unwrap failed"))?;
    if dek.len() != 32 {
        return Err(AppError::internal("open: bad dek length"));
    }

    let data_cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&dek));
    let plaintext = data_cipher
        .decrypt(Nonce::from_slice(&env.nonce), env.ciphertext.as_ref())
        .map_err(|_| AppError::internal("open: data decryption failed"))?;

    dek.iter_mut().for_each(|b| *b = 0);
    Ok(plaintext)
}
