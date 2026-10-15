//! Owner-scoped native connectors. Secrets never form part of tool schemas or receipts.
mod database;
mod email;
mod google;
mod http;
mod oauth;
mod vault;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionInput {
    #[serde(default)]
    pub id: Option<String>,
    pub provider: String,
    pub name: String,
    #[serde(default)]
    pub credentials: Value,
    #[serde(default)]
    pub settings: Value,
    #[serde(default)]
    pub allow_write: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Connection {
    pub id: String,
    pub provider: String,
    pub name: String,
    pub credentials: Value,
    pub settings: Value,
    pub allow_write: bool,
    pub checked_at: Option<String>,
    pub state: String,
    #[serde(default)]
    pub service_checks: Value,
}
impl Connection {
    pub fn view(&self) -> Value {
        // Older Google records used "connected" after OAuth or a Drive-only
        // probe. They cannot attest to the new per-service checks.
        let state = if self.provider == "google"
            && self.state == "connected"
            && !self.service_checks.is_object()
        {
            "authorized"
        } else {
            &self.state
        };
        let mut settings = self.settings.clone();
        let mut service_checks = self.service_checks.clone();
        if self.provider == "google" {
            if let Some(checks) = service_checks.as_object_mut() {
                checks.remove("drive");
            }
            if let Some(services) = settings["services"].as_array_mut() {
                services.retain(|service| service != "drive");
            }
        }
        json!({"id":self.id,"provider":self.provider,"name":self.name,
            "allow_write":self.allow_write,"state":state,"checked_at":self.checked_at,
            "service_checks":service_checks,"granted_scopes":self.credentials["granted_scopes"],
            "settings":{"services":settings["services"],"path":self.settings["path"],"scopes":self.settings["scopes"],"imap_host":self.settings["imap_host"],"smtp_host":self.settings["smtp_host"],"smtp_starttls":self.settings["smtp_starttls"]}})
    }
}

pub struct Connectors {
    root: PathBuf,
    key: Vec<u8>,
    lock: Mutex<()>,
    oauth_lock: tokio::sync::Mutex<()>,
    discovery_cache: Mutex<std::collections::HashMap<String, (std::time::Instant, Value)>>,
    client: reqwest::Client,
    #[cfg(test)]
    test_origin: Option<String>,
}

pub fn catalog() -> Value {
    json!([
        {"id":"google","name":"Google Workspace","category":"office","auth":"oauth","services":["Docs","Sheets","Slides","Gmail","Calendar","Tasks","Contacts","Forms","Chat","Meet"],"service_options":google::service_options(),"operations":["workspace_schema","request","mail_list","mail_read","mail_send","mail_compose","calendar_list","test"]},
        {"id":"microsoft","name":"Microsoft 365","category":"office","auth":"oauth","services":["OneDrive","SharePoint","Outlook","Calendar","Excel"],"operations":["request","files_search","file_download","mail_list","mail_send","calendar_list"]},
        {"id":"slack","name":"Slack","category":"communication","auth":"oauth_or_token","operations":["request","channels","history","send"]},
        {"id":"github","name":"GitHub","category":"development","auth":"token","operations":["request","repositories","issues","create_issue"]},
        {"id":"sqlite","name":"SQLite","category":"data","auth":"local","operations":["query"]},
        {"id":"email","name":"Email · IMAP / SMTP","category":"communication","auth":"mail","services":["IMAP TLS","SMTP TLS","STARTTLS"],"operations":["mail_list","mail_read","mail_send"]},
        {"id":"telegram","name":"Telegram","category":"communication","auth":"existing","route":"/settings","tools":["notify_user"]},
        {"id":"whatsapp","name":"WhatsApp Web","category":"communication","auth":"browser","tools":["browser_connection","browser_open","browser_observe","browser_act"]},
        {"id":"web","name":"Web search","category":"research","auth":"builtin","tools":["web_search","web_read"]},
        {"id":"pdf","name":"PDF","category":"documents","auth":"builtin","tools":["document_read","document_extract","document_create"]},
        {"id":"office","name":"Documents, spreadsheets and presentations","category":"documents","auth":"builtin","tools":["document_create","document_read","document_edit"]}
    ])
}

impl Connectors {
    pub fn open(root: &Path) -> Result<Self> {
        let root = root.join("connectors");
        let root = fenix_execution::fs::private::create_private_dir(&root)?;
        let key = vault::key(&root)?;
        Ok(Self {
            root,
            key,
            lock: Mutex::new(()),
            oauth_lock: tokio::sync::Mutex::new(()),
            discovery_cache: Mutex::new(std::collections::HashMap::new()),
            #[cfg(test)]
            test_origin: None,
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(60))
                .user_agent("Fenix/0.3 native-connectors")
                .build()?,
        })
    }

    pub fn list(&self) -> Result<Value> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Connector vault unavailable"))?;
        Ok(json!(
            self.read_all()?
                .iter()
                .filter(|c| c.provider != "postgresql")
                .map(Connection::view)
                .collect::<Vec<_>>()
        ))
    }

    pub fn describe(&self, id: &str, operation: &str) -> Result<Value> {
        let c = self.connection(id)?;
        let inputs = match operation {
            "workspace_schema" if c.provider == "google" => {
                json!({"service":"Service ID from service_options","method_id":"Optional full method ID (for example gmail.users.messages.get); omit to list methods","schema_name":"Optional named schema from a previous result","offset":"Method listing offset; follow next_offset"})
            }
            "mail_compose" if c.provider == "google" => google::mail_inputs(),
            "request" => {
                json!({"service":"Google service ID from service_options","method":"GET|POST|PUT|PATCH|DELETE","path":"API-relative path, for example /users/me/messages or /me/events","query":"Optional JSON object with query parameters","body":"Optional provider JSON body","dry_run":"Google only: optional boolean to preview locally without authentication or an API request","page_all":"Google GET only: bounded nextPageToken pagination","page_limit":"1–10 pages, default 5"})
            }
            "file_download" => {
                json!({"file_id":"ID from a previous search","destination":"New relative file path inside this task's workspace"})
            }
            "query" => {
                json!({"sql":"One SQL statement","parameters":"Optional array of bound scalar parameters; use ?1 for SQLite"})
            }
            "mail_list" => {
                json!({"q":"Gmail search expression","folder":"IMAP mailbox, default INBOX","limit":"Google: 1–100, default 25","page_token":"Google: nextPageToken","page_all":"Google: bounded pagination","page_limit":"1–10 pages"})
            }
            "mail_read" => {
                if c.provider == "google" {
                    json!({"message_id":"Gmail message ID","format":"metadata|full, default full"})
                } else {
                    json!({"uid":"Numeric IMAP message UID from mail_list","uid_validity":"Mailbox UID validity from mail_list","folder":"IMAP mailbox, default INBOX"})
                }
            }
            "mail_send" => match c.provider.as_str() {
                "google" => {
                    json!({"raw":"base64url RFC822 message","thread_id":"Optional existing Gmail thread ID; raw MIME must include matching reply headers"})
                }
                "microsoft" => {
                    json!({"body":"Microsoft Graph sendMail JSON: {message:{subject,body:{contentType:Text,content},toRecipients:[{emailAddress:{address}}]},saveToSentItems:true}"})
                }
                _ => {
                    json!({"to":"Recipient email address","subject":"Subject","text":"Plain text body"})
                }
            },
            "files_search" => json!({"q":"File search text"}),
            "history" | "send" => {
                json!({"channel":"Slack conversation ID","text":"Required for send"})
            }
            "issues" | "create_issue" => {
                json!({"owner":"Repository owner","repo":"Repository name","body":"For create_issue: title and body"})
            }
            "calendar_list" if c.provider == "google" => {
                json!({"calendar_id":"Default primary","time_min":"Optional RFC3339 start","time_max":"Optional RFC3339 end","limit":"1–100, default 25","page_token":"Optional nextPageToken","page_all":"Optional bounded pagination","page_limit":"1–10"})
            }
            "test" | "repositories" | "channels" | "calendar_list" => json!({}),
            _ => anyhow::bail!("Unknown operation"),
        };
        Ok(
            json!({"provider":c.provider,"operation":operation,"inputs":inputs,"allow_write":c.allow_write,"data_is_untrusted":true}),
        )
    }

    pub fn effect(&self, id: &str, operation: &str, args: &Value) -> Result<&'static str> {
        let c = self.connection(id)?;
        if c.provider == "google" && (operation == "workspace_schema" || args["dry_run"] == true) {
            return Ok("read");
        }
        if matches!(operation, "file_download") {
            return Ok("file_write");
        }
        if matches!(
            operation,
            "test"
                | "files_search"
                | "mail_list"
                | "mail_read"
                | "calendar_list"
                | "channels"
                | "history"
                | "repositories"
                | "issues"
        ) || operation == "query" && !c.allow_write
        {
            return Ok("read");
        }
        if operation == "request"
            && args["method"]
                .as_str()
                .unwrap_or("GET")
                .eq_ignore_ascii_case("GET")
            && c.provider != "slack"
        {
            return Ok("read");
        }
        Ok("external_write")
    }

    pub fn save(&self, mut input: ConnectionInput) -> Result<Value> {
        ensure!(
            [
                "google",
                "microsoft",
                "slack",
                "github",
                "sqlite",
                "email"
            ]
            .contains(&input.provider.as_str()),
            "Unknown configurable provider"
        );
        ensure!(
            !input.name.trim().is_empty() && input.name.len() <= 160,
            "Supply a connection name"
        );
        ensure!(
            input.credentials.is_null() || input.credentials.is_object(),
            "Credentials must be an object"
        );
        ensure!(
            input.settings.is_null() || input.settings.is_object(),
            "Settings must be an object"
        );
        ensure!(
            serde_json::to_vec(&input)?.len() <= 65_536,
            "Connection settings are too large"
        );
        if let Some(fields) = input.credentials.as_object() {
            ensure!(
                fields.keys().all(|k| [
                    "access_token",
                    "token",
                    "refresh_token",
                    "client_id",
                    "client_secret",
                    "url",
                    "username",
                    "password"
                ]
                .contains(&k.as_str())),
                "Unknown credential field"
            );
        }
        let _guard = self
            .lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Connector vault unavailable"))?;
        let mut all = self.read_all()?;
        let previous = input
            .id
            .as_ref()
            .map(|id| {
                all.iter()
                    .find(|c| &c.id == id)
                    .cloned()
                    .context("Connection not found")
            })
            .transpose()?;
        if input.provider == "google" {
            if input.settings.is_null() {
                input.settings = json!({});
            }
            if input.settings.get("services").is_none() {
                if let Some(services) = previous.as_ref().and_then(|c| c.settings.get("services")) {
                    input.settings["services"] = services.clone();
                } else if previous.is_none() {
                    input.settings["services"] = json!(["gmail"]);
                }
            }
            google::validate_settings(&input.settings)?;
        }
        if let Some(old) = &previous {
            ensure!(
                old.provider == input.provider,
                "A connection cannot change providers"
            );
        }
        let mut credentials = previous
            .as_ref()
            .map(|c| c.credentials.clone())
            .unwrap_or(json!({}));
        if input.credentials["access_token"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
        {
            credentials["refresh_token"] = Value::Null;
            credentials["expires_at"] = Value::Null;
            credentials["granted_scopes"] = Value::Null;
        }
        if let Some(fields) = input.credentials.as_object() {
            for (key, value) in fields {
                if value.as_str().is_some_and(|s| !s.is_empty()) || value.is_number() {
                    credentials[key] = value.clone();
                }
            }
        }
        credentials["pending_oauth"] = Value::Null;
        let connection = Connection {
            id: input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            provider: input.provider,
            name: input.name.trim().into(),
            credentials,
            settings: input.settings,
            allow_write: input.allow_write,
            checked_at: None,
            state: "unverified".into(),
            service_checks: Value::Null,
        };
        all.retain(|c| c.id != connection.id);
        all.push(connection.clone());
        self.write_all(&all)?;
        Ok(connection.view())
    }

    pub fn forget(&self, id: &str) -> Result<()> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Connector vault unavailable"))?;
        let mut all = self.read_all()?;
        ensure!(all.iter().any(|c| c.id == id), "Connection not found");
        all.retain(|c| c.id != id);
        self.write_all(&all)
    }

    pub(crate) fn connection(&self, id: &str) -> Result<Connection> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Connector vault unavailable"))?;
        self.read_all()?
            .into_iter()
            .find(|c| c.id == id && c.provider != "postgresql")
            .context("Connection not found in this account")
    }

    pub(crate) fn update(&self, connection: &Connection) -> Result<()> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Connector vault unavailable"))?;
        let mut all = self.read_all()?;
        let target = all
            .iter_mut()
            .find(|c| c.id == connection.id)
            .context("Connection was disconnected")?;
        *target = connection.clone();
        self.write_all(&all)
    }

    pub async fn test(&self, id: &str, workspace: &Path) -> Result<Value> {
        if self.connection(id)?.provider == "google" {
            return self.google_check(id, workspace).await;
        }
        let result = self.invoke(id, "test", &json!({}), workspace).await;
        let mut c = self.connection(id)?;
        c.checked_at = Some(chrono::Utc::now().to_rfc3339());
        c.state = if result.is_ok() {
            "connected"
        } else {
            "needs_attention"
        }
        .into();
        self.update(&c)?;
        result?;
        Ok(c.view())
    }

    pub async fn invoke(
        &self,
        id: &str,
        operation: &str,
        args: &Value,
        workspace: &Path,
    ) -> Result<Value> {
        let c = self.connection(id)?;
        if c.provider == "google" {
            if operation == "workspace_schema" {
                return self.google_schema(args).await;
            }
            if operation == "test" {
                return self.google_check(id, workspace).await;
            }
            if operation == "mail_compose" {
                return self.google_compose(c, args, workspace).await;
            }
            if args["page_all"] == true && args["dry_run"] != true {
                return self.google_pages(c, operation, args, workspace).await;
            }
        }
        if c.provider == "email" {
            return email::execute(c, operation, args.clone()).await;
        }
        if c.provider == "sqlite" {
            return database::execute(c, operation, args.clone(), workspace.to_owned()).await;
        }
        self.http_call(c, operation, args, workspace).await
    }
}
