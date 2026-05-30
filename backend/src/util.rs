//! Small shared helpers: time + cryptographically-random opaque tokens.

use rand::RngCore;

/// Current wall-clock time as unix epoch seconds.
pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 256 bits of OS randomness, hex-encoded. Used for session ids and OAuth state nonces.
pub fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}
