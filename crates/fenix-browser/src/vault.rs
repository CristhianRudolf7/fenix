//! Owner-only password vault. No plaintext fallback, master password, or decrypted
//! entries are persisted. The vault is not exposed as an agent tool.
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use anyhow::{Context, Result, bail};
use argon2::Argon2;
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use std::path::Path;
use zeroize::Zeroizing;

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub profile: String,
    pub origin: String,
    pub username: String,
    pub password: String,
}
impl Drop for Entry {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.password.zeroize();
    }
}
#[derive(Serialize, Deserialize)]
struct Envelope {
    version: u8,
    salt: String,
    nonce: String,
    ciphertext: String,
}
fn key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; 32]>> {
    if password.chars().count() < 12 || password.len() > 1024 {
        bail!("Vault master password must contain at least 12 characters and at most 1024 bytes");
    }
    let mut key = Zeroizing::new([0; 32]);
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut *key)
        .map_err(|_| anyhow::anyhow!("Vault key derivation failed"))?;
    Ok(key)
}
pub fn load(path: &Path, password: &str) -> Result<Vec<Entry>> {
    if !path.exists() {
        key(password, &[0; 16])?;
        return Ok(vec![]);
    }
    if std::fs::metadata(path)?.len() > 1024 * 1024 {
        bail!("Vault size limit exceeded");
    }
    let envelope: Envelope = serde_json::from_slice(&std::fs::read(path)?)?;
    let salt = STANDARD.decode(envelope.salt)?;
    let nonce = STANDARD.decode(envelope.nonce)?;
    if envelope.version != 1 || salt.len() != 16 || nonce.len() != 12 {
        bail!("Unsupported vault format");
    }
    let key = key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&*key).unwrap();
    let plain = Zeroizing::new(
        cipher
            .decrypt(
                Nonce::from_slice(&nonce),
                STANDARD.decode(envelope.ciphertext)?.as_ref(),
            )
            .map_err(|_| anyhow::anyhow!("Incorrect master password or damaged vault"))?,
    );
    serde_json::from_slice(&plain).context("Invalid vault content")
}
pub fn save(path: &Path, password: &str, entries: &[Entry]) -> Result<()> {
    if entries.len() > 100 {
        bail!("Vault entry limit reached (100)");
    }
    let mut salt = [0; 16];
    let mut nonce = [0; 12];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);
    let key = key(password, &salt)?;
    let plain = Zeroizing::new(serde_json::to_vec(entries)?);
    let ciphertext = Aes256Gcm::new_from_slice(&*key)
        .unwrap()
        .encrypt(Nonce::from_slice(&nonce), plain.as_ref())
        .map_err(|_| anyhow::anyhow!("Vault encryption failed"))?;
    crate::write_json(
        path,
        &serde_json::to_value(Envelope {
            version: 1,
            salt: STANDARD.encode(salt),
            nonce: STANDARD.encode(nonce),
            ciphertext: STANDARD.encode(ciphertext),
        })?,
    )
}
