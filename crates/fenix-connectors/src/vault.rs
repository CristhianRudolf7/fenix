use super::*;
use aes_gcm::{
    Aes256Gcm, KeyInit,
    aead::{Aead, AeadCore, OsRng},
};
use base64::{Engine, engine::general_purpose::STANDARD};

pub(crate) fn key(root: &Path) -> Result<Vec<u8>> {
    let path = root.join("key");
    let files = fenix_execution::fs::private::PrivateDirectory::open(root)?;
    if !path.exists() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        files.create(Path::new("key"), &key, false)?;
    }
    let key = files.read(Path::new("key"), 32)?;
    fenix_execution::fs::private::secure(&path)?;
    ensure!(key.len() == 32, "Invalid connector vault key");
    Ok(key)
}

impl Connectors {
    pub(crate) fn read_all(&self) -> Result<Vec<Connection>> {
        let path = self.root.join("accounts.enc");
        if !path.exists() {
            return Ok(vec![]);
        }
        let envelope: Value = serde_json::from_slice(
            &fenix_execution::fs::private::PrivateDirectory::open(&self.root)?
                .read(Path::new("accounts.enc"), 4_194_304)?,
        )?;
        fenix_execution::fs::private::secure(&path)?;
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
        let files = fenix_execution::fs::private::PrivateDirectory::open(&self.root)?;
        if path.try_exists()? {
            files.replace(
                Path::new("accounts.enc"),
                &files.read(Path::new("accounts.enc"), 4_194_304)?,
                &bytes,
            )?;
        } else {
            files.create(Path::new("accounts.enc"), &bytes, false)?;
        }
        fenix_execution::fs::private::secure(&path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_vault_reopens_with_the_same_key_and_keeps_credentials_encrypted() {
        let directory = tempfile::tempdir().unwrap();
        let connectors = Connectors::open(directory.path()).unwrap();
        connectors
            .save(ConnectionInput {
                id: None,
                provider: "github".into(),
                name: "Owner".into(),
                credentials: json!({"token":"fixture-private-token"}),
                settings: json!({}),
                allow_write: false,
            })
            .unwrap();
        let before = connectors.key.clone();
        drop(connectors);
        let reopened = Connectors::open(directory.path()).unwrap();
        assert_eq!(reopened.key, before);
        assert_eq!(
            reopened.read_all().unwrap()[0].credentials["token"],
            "fixture-private-token"
        );
        let path = directory.path().join("connectors/accounts.enc");
        assert!(
            !std::fs::read_to_string(&path)
                .unwrap()
                .contains("fixture-private-token")
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for path in [path, directory.path().join("connectors/key")] {
                assert_eq!(
                    std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                    0o600
                );
            }
        }
    }
}
