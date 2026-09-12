//! Optional Telegram transport. Pair one owner locally; reuse the same runtime queue.
use crate::Runtime;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};

#[derive(Clone, Serialize, Deserialize)]
pub struct TelegramSettings {
    pub enabled: bool,
    pub username: String,
    pub owner_id: Option<i64>,
    pub pairing_code: Option<String>,
    pub offset: i64,
    pub last_error: Option<String>,
}
pub struct TelegramClient {
    http: reqwest::Client,
    base_url: String,
    token: String,
}
impl TelegramClient {
    pub fn new(token: String) -> Result<Self> {
        Self::with_base_url(token, "https://api.telegram.org".into())
    }
    pub fn with_base_url(token: String, base_url: String) -> Result<Self> {
        if token.is_empty()
            || !token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b':' | b'_' | b'-'))
        {
            bail!("Invalid Telegram token format");
        }
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(35))
                .connect_timeout(Duration::from_secs(10))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
            base_url,
            token,
        })
    }
    pub async fn call(&self, method: &str, body: Value) -> Result<Value> {
        let url = format!("{}/bot{}/{method}", self.base_url, self.token);
        let mut response = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("Telegram connection failed"))?;
        if !response.status().is_success() {
            bail!("Telegram returned HTTP {}", response.status().as_u16());
        }
        let mut bytes = vec![];
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| anyhow::anyhow!("Telegram response failed"))?
        {
            if bytes.len() + chunk.len() > 1_048_576 {
                bail!("Telegram response exceeded the size limit");
            }
            bytes.extend_from_slice(&chunk);
        }
        let result: Value = serde_json::from_slice(&bytes)
            .map_err(|_| anyhow::anyhow!("Invalid Telegram response"))?;
        if result["ok"] != true {
            bail!("Telegram rejected the request");
        }
        Ok(result["result"].clone())
    }
    pub async fn send(&self, owner: i64, text: &str) -> Result<()> {
        // Keep comfortably below Telegram's UTF-16 length limit, no Markdown parsing.
        let mut part = String::new();
        let mut length = 0;
        for ch in text.chars() {
            let units = ch.len_utf16();
            if length + units > 3500 {
                self.call("sendMessage", json!({"chat_id":owner,"text":part}))
                    .await?;
                part.clear();
                length = 0;
            }
            part.push(ch);
            length += units;
        }
        if !part.is_empty() {
            self.call("sendMessage", json!({"chat_id":owner,"text":part}))
                .await?;
        }
        Ok(())
    }
}

impl Runtime {
    pub fn telegram_settings(&self) -> Result<Option<TelegramSettings>> {
        self.store.get("telegram", "settings")
    }

    pub async fn configure_telegram(&self, token: Option<String>, enabled: bool) -> Result<Value> {
        let _lock = self.telegram_lock.lock().await;
        let old = self.telegram_settings()?;
        if !enabled {
            if let Some(mut settings) = old {
                settings.enabled = false;
                settings.last_error = None;
                self.store.put("telegram", "settings", &settings)?;
                return Ok(json!(settings));
            }
            return Ok(json!({"enabled":false}));
        }
        let previous = self.store.get::<String>("telegram_secret", "token")?;
        let token = token
            .filter(|t| !t.is_empty())
            .or_else(|| previous.clone())
            .context("Telegram token is required to enable Telegram")?;
        let client = TelegramClient::new(token.clone())?;
        let me = client.call("getMe", json!({})).await?;
        let username = me["username"]
            .as_str()
            .context("Telegram bot has no username")?
            .to_owned();
        let same = previous.as_deref() == Some(token.as_str());
        let mut settings = if same {
            old.unwrap_or_else(|| new_settings(username.clone()))
        } else {
            new_settings(username.clone())
        };
        settings.enabled = true;
        settings.username = username;
        settings.last_error = None;
        self.redactor.add(&token);
        self.store.put("telegram_secret", "token", &token)?;
        self.store.put("telegram", "settings", &settings)?;
        Ok(json!(settings))
    }

    pub async fn receive_telegram(
        &self,
        update: &Value,
        settings: &mut TelegramSettings,
    ) -> Result<()> {
        let Some(message) = update.get("message") else {
            return Ok(());
        };
        let Some(sender) = message["from"]["id"].as_i64() else {
            return Ok(());
        };
        if message["chat"]["type"] != "private" || message["chat"]["id"].as_i64() != Some(sender) {
            return Ok(());
        }
        let text = message["text"].as_str();
        if settings.owner_id.is_none() {
            let Some(code) = &settings.pairing_code else {
                return Ok(());
            };
            if text != Some(format!("/start {code}").as_str()) {
                return Ok(());
            }
            settings.owner_id = Some(sender);
            settings.pairing_code = None;
            self.store.put("telegram", "settings", settings)?;
            self.notify(None,"Telegram conectado. Esta es la misma conversación que aparece en la web. Envía mensajes de texto para continuar.")?;
            return Ok(());
        }
        if settings.owner_id != Some(sender) {
            return Ok(());
        }
        if let Some(text) = text {
            if text == "/start" {
                self.notify(None,"Fénix está conectado a tu conversación continua. Puedes escribir aquí o desde la web.")?;
            } else {
                self.submit(text, "telegram").await?;
            }
        } else {
            self.notify(None,"Por ahora sólo recibo texto. Los archivos, imágenes y audios quedan para una versión posterior.")?;
        }
        Ok(())
    }

    pub(crate) async fn telegram_loop(self: Arc<Self>) {
        loop {
            if self.stop.is_cancelled() {
                break;
            }
            let result = self.telegram_cycle().await;
            if let Err(error) = result
                && let Ok(Some(mut settings)) = self.telegram_settings()
            {
                settings.last_error = Some(self.redactor.text(&error.to_string()));
                let _ = self.store.put("telegram", "settings", &settings);
            }
            tokio::select! {_=self.stop.cancelled()=>break,_=tokio::time::sleep(Duration::from_secs(2))=>{}}
        }
    }
    async fn telegram_cycle(&self) -> Result<()> {
        let Some(snapshot) = self.telegram_settings()?.filter(|s| s.enabled) else {
            return Ok(());
        };
        let token = self
            .store
            .get::<String>("telegram_secret", "token")?
            .context("Telegram token missing")?;
        let client = TelegramClient::new(token.clone())?;
        let updates = tokio::select! {
            _=self.stop.cancelled()=>return Ok(()),
            result=client.call("getUpdates",json!({"offset":snapshot.offset,"timeout":5,"allowed_updates":["message"]}))=>result?,
        };
        let _lock = self.telegram_lock.lock().await;
        let Some(mut settings) = self.telegram_settings()?.filter(|s| s.enabled) else {
            return Ok(());
        };
        if self
            .store
            .get::<String>("telegram_secret", "token")?
            .as_deref()
            != Some(token.as_str())
        {
            return Ok(());
        }
        for update in updates
            .as_array()
            .context("Telegram updates were not an array")?
        {
            let id = update["update_id"]
                .as_i64()
                .context("Telegram update ID missing")?;
            if id < settings.offset {
                continue;
            }
            // Claim before dispatch. Do not replay potentially applied effects after a crash.
            settings.offset = id + 1;
            self.store.put("telegram", "settings", &settings)?;
            if let Err(error) = self.receive_telegram(update, &mut settings).await {
                self.notify(
                    None,
                    &format!("Telegram: {}", self.redactor.text(&error.to_string())),
                )?;
            }
        }
        settings.last_error = None;
        self.store.put("telegram", "settings", &settings)?;
        if let Some(owner) = settings.owner_id {
            for (id, text) in self.store.deliveries()? {
                let sent = client.send(owner, &text).await;
                self.store.delivery_result(id, sent.is_ok())?;
                sent?;
            }
        }
        Ok(())
    }
}
fn new_settings(username: String) -> TelegramSettings {
    TelegramSettings {
        enabled: true,
        username,
        owner_id: None,
        pairing_code: Some(fenix_storage::id()),
        offset: 0,
        last_error: None,
    }
}
