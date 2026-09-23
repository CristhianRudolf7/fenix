use super::*;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

fn endpoints(provider: &str) -> Result<(&'static str, &'static str, &'static str)> {
    Ok(match provider {
        "google" => (
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
            "https://www.googleapis.com/auth/drive.readonly https://www.googleapis.com/auth/userinfo.email",
        ),
        "microsoft" => (
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            "offline_access User.Read Files.Read Mail.Read Calendars.Read",
        ),
        "slack" => (
            "https://slack.com/oauth/v2/authorize",
            "https://slack.com/api/oauth.v2.access",
            "channels:read,channels:history",
        ),
        _ => anyhow::bail!("This provider uses a personal token or a local connection"),
    })
}

fn app(c: &Connection, field: &str) -> Result<String> {
    c.credentials[field]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            std::env::var(format!(
                "FENIX_{}_{}",
                c.provider.to_uppercase(),
                field.to_uppercase()
            ))
            .ok()
        })
        .with_context(|| {
            format!(
                "Configure the {} OAuth application ({field}) first",
                c.provider
            )
        })
}

impl Connectors {
    pub async fn oauth_start(&self, id: &str, redirect_uri: &str) -> Result<Value> {
        let _guard = self.oauth_lock.lock().await;
        let mut c = self.connection(id)?;
        let (authorize, _, scopes) = endpoints(&c.provider)?;
        let redirect = reqwest::Url::parse(redirect_uri)?;
        ensure!(
            redirect.scheme() == "https"
                || redirect.scheme() == "http"
                    && matches!(
                        redirect.host_str(),
                        Some("localhost" | "127.0.0.1" | "[::1]")
                    ),
            "OAuth requires HTTPS or localhost"
        );
        ensure!(
            redirect.path() == "/api/connectors/oauth/callback"
                && redirect.query().is_none()
                && redirect.fragment().is_none()
                && redirect.username().is_empty()
                && redirect.password().is_none(),
            "Invalid OAuth callback"
        );
        let client_id = app(&c, "client_id")?;
        let _ = app(&c, "client_secret")?;
        let state = uuid::Uuid::new_v4().to_string();
        let verifier = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        let mut url = reqwest::Url::parse(authorize)?;
        let scopes = match (c.provider.as_str(), c.allow_write) {
            ("google", true) => {
                "https://www.googleapis.com/auth/drive https://www.googleapis.com/auth/gmail.modify https://www.googleapis.com/auth/calendar https://www.googleapis.com/auth/userinfo.email"
            }
            ("google", false) => {
                "https://www.googleapis.com/auth/drive.readonly https://www.googleapis.com/auth/gmail.readonly https://www.googleapis.com/auth/calendar.readonly https://www.googleapis.com/auth/userinfo.email"
            }
            ("microsoft", true) => {
                "offline_access User.Read Files.ReadWrite Mail.ReadWrite Mail.Send Calendars.ReadWrite"
            }
            ("slack", true) => "channels:read,channels:history,chat:write",
            _ => scopes,
        };
        let google_scopes = if c.provider == "google" {
            Some(crate::google::scopes(&c)?)
        } else {
            None
        };
        let configured_scopes = google_scopes
            .as_deref()
            .or_else(|| c.settings["scopes"].as_str().filter(|s| !s.is_empty()))
            .unwrap_or(scopes);
        {
            let mut query = url.query_pairs_mut();
            query
                .append_pair("client_id", &client_id)
                .append_pair("redirect_uri", redirect_uri)
                .append_pair("state", &state)
                .append_pair("scope", configured_scopes);
            if c.provider != "slack" {
                query
                    .append_pair("response_type", "code")
                    .append_pair("code_challenge_method", "S256")
                    .append_pair(
                        "code_challenge",
                        &URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())),
                    );
            }
            if c.provider == "google" {
                query
                    .append_pair("access_type", "offline")
                    .append_pair("prompt", "consent");
            }
        }
        c.credentials["pending_oauth"] = json!({"state":state,"verifier":verifier,"redirect_uri":redirect_uri,"expires":chrono::Utc::now().timestamp()+600});
        self.update(&c)?;
        Ok(json!({"url":url.to_string()}))
    }

    pub async fn oauth_finish(&self, state: &str, code: &str) -> Result<String> {
        let _guard = self.oauth_lock.lock().await;
        ensure!(
            !state.is_empty() && !code.is_empty(),
            "OAuth response is incomplete"
        );
        let mut c = {
            let _lock = self
                .lock
                .lock()
                .map_err(|_| anyhow::anyhow!("Connector vault unavailable"))?;
            self.read_all()?
                .into_iter()
                .find(|c| c.credentials["pending_oauth"]["state"].as_str() == Some(state))
                .context("OAuth state is invalid or already used")?
        };
        let pending = c.credentials["pending_oauth"].take();
        self.update(&c)?; // Consume before exchanging. A callback cannot be replayed.
        ensure!(
            pending["expires"].as_i64().unwrap_or(0) > chrono::Utc::now().timestamp(),
            "OAuth request expired; reconnect"
        );
        let (_, token_url, _) = endpoints(&c.provider)?;
        let mut form = vec![
            ("client_id", app(&c, "client_id")?),
            ("client_secret", app(&c, "client_secret")?),
            ("code", code.into()),
            (
                "redirect_uri",
                pending["redirect_uri"]
                    .as_str()
                    .context("Missing callback")?
                    .into(),
            ),
        ];
        if c.provider != "slack" {
            form.push(("grant_type", "authorization_code".into()));
            form.push((
                "code_verifier",
                pending["verifier"].as_str().unwrap_or_default().into(),
            ));
        }
        let tokens = self.exchange(token_url, &form).await?;
        // A reconnect can select a different Google account. Never pair its new
        // access token with the previous account's refresh token or scope report.
        if c.provider == "google" {
            c.credentials["refresh_token"] = Value::Null;
            c.credentials["granted_scopes"] = Value::Null;
        }
        apply_tokens(&mut c, &tokens)?;
        c.state = if c.provider == "google" {
            "authorized"
        } else {
            "connected"
        }
        .into();
        c.service_checks = Value::Null;
        c.checked_at = if c.provider == "google" {
            None
        } else {
            Some(chrono::Utc::now().to_rfc3339())
        };
        let id = c.id.clone();
        self.update(&c)?;
        Ok(id)
    }

    async fn exchange(&self, url: &str, form: &[(&str, String)]) -> Result<Value> {
        #[allow(unused_mut)]
        let mut endpoint = reqwest::Url::parse(url)?;
        #[cfg(test)]
        if let Some(origin) = &self.test_origin {
            let target = reqwest::Url::parse(origin)?;
            endpoint.set_scheme(target.scheme()).unwrap();
            endpoint.set_host(target.host_str())?;
            endpoint.set_port(target.port()).unwrap();
        }
        let response = self
            .client
            .post(endpoint)
            .form(form)
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("OAuth server could not be reached"))?;
        ensure!(
            response.status().is_success(),
            "OAuth exchange failed (HTTP {}); reconnect the account",
            response.status().as_u16()
        );
        let bytes = crate::http::bounded_bytes(response, 65_536).await?;
        let value: Value = serde_json::from_slice(&bytes).context("Invalid OAuth response")?;
        ensure!(
            value.get("error").is_none() && value["ok"] != false,
            "Provider rejected OAuth authorization"
        );
        Ok(value)
    }

    pub(crate) async fn bearer(&self, id: &str) -> Result<String> {
        let _guard = self.oauth_lock.lock().await;
        let mut c = self.connection(id)?;
        if c.credentials["expires_at"]
            .as_i64()
            .is_some_and(|expires| expires <= chrono::Utc::now().timestamp() + 60)
        {
            let refresh = c.credentials["refresh_token"]
                .as_str()
                .context("Connection expired; reconnect the account")?
                .to_owned();
            let (_, url, _) = endpoints(&c.provider)?;
            let form = vec![
                ("grant_type", "refresh_token".into()),
                ("refresh_token", refresh),
                ("client_id", app(&c, "client_id")?),
                ("client_secret", app(&c, "client_secret")?),
            ];
            let tokens = self.exchange(url, &form).await?;
            apply_tokens(&mut c, &tokens)?;
            self.update(&c)?;
        }
        c.credentials["access_token"]
            .as_str()
            .or(c.credentials["token"].as_str())
            .filter(|t| !t.is_empty())
            .map(str::to_owned)
            .context("Connect or configure this account first")
    }
}

fn apply_tokens(c: &mut Connection, tokens: &Value) -> Result<()> {
    let access = tokens["access_token"]
        .as_str()
        .context("No access token returned")?;
    c.credentials["access_token"] = json!(access);
    if let Some(scopes) = tokens["scope"].as_str() {
        c.credentials["granted_scopes"] = json!(scopes.split_whitespace().collect::<Vec<_>>());
    }
    if let Some(refresh) = tokens["refresh_token"].as_str() {
        c.credentials["refresh_token"] = json!(refresh);
    }
    c.credentials["expires_at"] = tokens["expires_in"]
        .as_i64()
        .map(|seconds| json!(chrono::Utc::now().timestamp() + seconds))
        .unwrap_or(Value::Null);
    Ok(())
}
