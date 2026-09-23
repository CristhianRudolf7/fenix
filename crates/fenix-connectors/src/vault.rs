use super::*;
use aes_gcm::{
    Aes256Gcm, KeyInit,
    aead::{Aead, AeadCore, OsRng},
};
use base64::{Engine, engine::general_purpose::STANDARD};

pub(crate) fn key(root: &Path) -> Result<Vec<u8>> {
    let path = root.join("key");
    if !path.exists() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        fenix_execution::fs::create(&path, &key)?;
    }
    let key = fenix_execution::fs::read(&path, 32)?;
    ensure!(key.len() == 32, "Invalid connector vault key");
    Ok(key)
}

impl Connectors {
    pub(crate) fn read_all(&self) -> Result<Vec<Connection>> {
        let path = self.root.join("accounts.enc");
        if !path.exists() {
            return Ok(vec![]);
        }
        let envelope: Value =
            serde_json::from_slice(&fenix_execution::fs::read(&path, 4_194_304)?)?;
        let nonce = STANDARD.decode(envelope["nonce"].as_str().context("Invalid vault")?)?;
        ensure!(nonce.len() == 12, "Invalid vault nonce");
        let ciphertext = STANDARD.decode(envelope["data"].as_str().context("Invalid vault")?)?;
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| anyhow::anyhow!("Invalid vault key"))?;
        let plain = cipher
            .decrypt(nonce.as_slice().into(), ciphertext.as_slice())
            .map_err(|_| anyhow::anyhow!("Cannot decrypt connector vault"))?;
        Ok(serde_json::from_slice(&plain)?)
    }
    pub(crate) fn write_all(&self, connections: &[Connection]) -> Result<()> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| anyhow::anyhow!("Invalid vault key"))?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let encrypted = cipher
            .encrypt(&nonce, serde_json::to_vec(connections)?.as_slice())
            .map_err(|_| anyhow::anyhow!("Cannot encrypt connector vault"))?;
        let bytes = serde_json::to_vec(
            &json!({"nonce":STANDARD.encode(nonce),"data":STANDARD.encode(encrypted)}),
        )?;
        let path = self.root.join("accounts.enc");
        if path.exists() {
            fenix_execution::fs::replace(
                &path,
                &fenix_execution::fs::read(&path, 4_194_304)?,
                &bytes,
            )?;
        } else {
            fenix_execution::fs::create(&path, &bytes)?;
        }
        Ok(())
    }
}
