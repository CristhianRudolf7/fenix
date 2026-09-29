//! Google identity stays at the gateway; Drive consent remains a separate flow.
use super::*;
use axum::http::HeaderMap;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

pub(super) struct GoogleAuth {
    config: Option<Config>,
    pending: Mutex<HashMap<String, Pending>>,
    exchanges: tokio::sync::Semaphore,
}
struct Config {
    client_id: String,
    client_secret: String,
    redirect: String,
    token_endpoint: String,
    userinfo_endpoint: String,
}
struct Pending {
    browser: String,
    verifier: String,
    created: Instant,
    register: bool,
    owner: Option<String>,
}
impl GoogleAuth {
    pub(super) fn from_env(origin: Option<&str>, port: u16) -> Self {
        let nonempty = |name| std::env::var(name).ok().filter(|v| !v.trim().is_empty());
        let config = nonempty("FENIX_GOOGLE_CLIENT_ID")
            .zip(nonempty("FENIX_GOOGLE_CLIENT_SECRET"))
            .map(|(client_id, client_secret)| Config {
                client_id,
                client_secret,
                token_endpoint: "https://oauth2.googleapis.com/token".into(),
                userinfo_endpoint: "https://openidconnect.googleapis.com/v1/userinfo".into(),
                redirect: format!(
                    "{}/api/auth/google/callback",
                    origin
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("http://127.0.0.1:{port}"))
                ),
            });
        Self {
            config,
            pending: Mutex::new(HashMap::new()),
            exchanges: tokio::sync::Semaphore::new(4),
        }
    }
}
fn failure(code: &str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({"error":code}))).into_response()
}
pub(super) async fn status(State(g): State<Arc<Gateway>>) -> Response {
    let response = Json(json!({"enabled": g.google.config.is_some(), "registration":true}));
    ([(header::CACHE_CONTROL, "no-store")], response).into_response()
}
fn browser_cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|s| s.trim().strip_prefix("fenix_google="))
}
fn binding_cookie(g: &Gateway, value: &str, age: u64) -> String {
    format!(
        "fenix_google={value}; HttpOnly; SameSite=Strict; Path=/api/auth/google; Max-Age={age}{}",
        if g.secure { "; Secure" } else { "" }
    )
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Start {
    #[serde(default)]
    register: bool,
}
pub(super) async fn start(
    State(g): State<Arc<Gateway>>,
    headers: HeaderMap,
    Json(input): Json<Start>,
) -> Response {
    let Some(config) = &g.google.config else {
        return failure("google_unavailable");
    };
    let mut request = Request::new(Body::empty());
    *request.headers_mut() = headers.clone();
    let owner = g
        .account(cookie(&request).map(str::to_owned))
        .await
        .map(|a| a.id);
    let mut pending = g.google.pending.lock().await;
    pending.retain(|_, p| p.created.elapsed() < Duration::from_secs(600));
    // Repeated clicks from one browser replace its pending request.
    if let Some(previous) = browser_cookie(&headers) {
        pending.retain(|_, p| p.browser != previous);
    }
    if pending.len() >= 512 {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    let state = token();
    let browser = token();
    let verifier = token();
    let mut url = reqwest::Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap();
    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", &config.redirect)
        .append_pair("response_type", "code")
        .append_pair("scope", "openid email profile")
        .append_pair("prompt", "select_account")
        .append_pair("state", &state)
        .append_pair("code_challenge_method", "S256")
        .append_pair(
            "code_challenge",
            &URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())),
        );
    pending.insert(
        state,
        Pending {
            browser: browser.clone(),
            verifier,
            created: Instant::now(),
            register: input.register,
            owner,
        },
    );
    (
        [
            (header::SET_COOKIE, binding_cookie(&g, &browser, 600)),
            (header::CACHE_CONTROL, "no-store".into()),
        ],
        Json(json!({"url":url.as_str()})),
    )
        .into_response()
}

// The cross-site GET serves no account data. Its same-origin POST receives the
// Strict browser-binding cookie, so no relaxation of the normal session is needed.
pub(super) async fn landing() -> Response {
    let nonce = token();
    let html = include_str!("../../resources/google-login.html").replace("{{nonce}}", &nonce);
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8".to_string()),
      (header::CACHE_CONTROL, "no-store".into()),
      (header::REFERRER_POLICY, "no-referrer".into()),
      (header::CONTENT_SECURITY_POLICY, format!("default-src 'none'; script-src 'nonce-{nonce}'; style-src 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'"))], html).into_response()
}
#[derive(Deserialize)]
pub(super) struct Callback {
    state: String,
    code: Option<String>,
    error: Option<String>,
}
#[derive(Deserialize)]
struct Identity {
    sub: String,
    email: String,
    email_verified: bool,
    #[serde(default)]
    name: String,
    hd: Option<String>,
}
impl Identity {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.email_verified && !self.sub.is_empty() && self.sub.len() <= 255,
            "google_identity_invalid"
        );
        ensure!(
            self.email.contains('@')
                && self.email.len() <= 254
                && !self.email.chars().any(char::is_control),
            "google_identity_invalid"
        );
        Ok(())
    }
    fn authoritative(&self) -> bool {
        self.email.to_lowercase().ends_with("@gmail.com")
            || self.hd.as_ref().is_some_and(|s| !s.is_empty())
    }
}
#[derive(Deserialize)]
struct Tokens {
    access_token: String,
}
async fn identify(g: &Gateway, config: &Config, pending: &Pending, code: &str) -> Result<Identity> {
    let tokens: Tokens = g
        .client
        .post(&config.token_endpoint)
        .timeout(Duration::from_secs(20))
        .form(&[
            ("client_id", config.client_id.as_str()),
            ("client_secret", config.client_secret.as_str()),
            ("redirect_uri", config.redirect.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("code_verifier", pending.verifier.as_str()),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    // Only a token obtained by this server with its own client and PKCE verifier
    // is accepted. Never accept a browser-supplied profile, token or decoded JWT.
    let identity: Identity = g
        .client
        .get(&config.userinfo_endpoint)
        .timeout(Duration::from_secs(15))
        .bearer_auth(tokens.access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    identity.validate()?;
    Ok(identity)
}
fn resolve(
    path: PathBuf,
    identity: Identity,
    register: bool,
    owner: Option<String>,
    ready: Vec<String>,
) -> Result<Account> {
    identity.validate()?;
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path.with_extension("admin.lock"))?;
    lock.lock()?;
    let mut registry = RegistrySource::File(path.clone()).load()?;
    let email = identity.email.trim().to_lowercase();
    let by_subject = registry
        .accounts
        .iter()
        .position(|a| a.google_subject.as_deref() == Some(identity.sub.as_str()));
    let by_email = registry
        .accounts
        .iter()
        .position(|a| a.email == email && !a.signup_available);
    let index = if let Some(i) = by_subject {
        i
    } else if let Some(i) = by_email {
        let account = &registry.accounts[i];
        ensure!(account.google_subject.is_none(), "google_account_conflict");
        ensure!(
            identity.authoritative() || owner.as_deref() == Some(account.id.as_str()),
            "google_link_required"
        );
        i
    } else {
        ensure!(register, "google_registration_required");
        registry
            .accounts
            .iter()
            .position(|a| a.signup_available && !a.disabled && ready.contains(&a.id))
            .context("google_capacity_unavailable")?
    };
    let account = &mut registry.accounts[index];
    ensure!(!account.disabled, "google_account_unavailable");
    if let Some(owner) = owner {
        ensure!(account.id == owner, "google_account_conflict");
    }
    if account.signup_available {
        account.email = email;
        account.display_name = identity
            .name
            .chars()
            .filter(|c| !c.is_control())
            .take(80)
            .collect();
        account.signup_available = false;
        account.password_login = false;
        // Invalidate any accidental session issued before this tenant was assigned.
        account.password_hash = hash_password(&token())?;
    }
    account.google_subject = Some(identity.sub);
    let result = account.clone();
    registry::validate(&registry)?;
    private_write(&path, &serde_json::to_vec_pretty(&registry)?)?;
    Ok(result)
}
pub(super) async fn finish(
    State(g): State<Arc<Gateway>>,
    headers: HeaderMap,
    Json(input): Json<Callback>,
) -> Response {
    let Some(config) = &g.google.config else {
        return failure("google_unavailable");
    };
    let Ok(_permit) = g.google.exchanges.try_acquire() else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    let pending = {
        let mut entries = g.google.pending.lock().await;
        let valid = entries.get(&input.state).is_some_and(|p| {
            p.created.elapsed() < Duration::from_secs(600)
                && browser_cookie(&headers) == Some(p.browser.as_str())
        });
        if !valid {
            return failure("google_expired");
        }
        entries.remove(&input.state).unwrap()
    };
    if let Some(owner) = &pending.owner {
        let mut request = Request::new(Body::empty());
        *request.headers_mut() = headers.clone();
        if !g
            .account(cookie(&request).map(str::to_owned))
            .await
            .is_some_and(|a| &a.id == owner)
        {
            return failure("google_expired");
        }
    }
    if input.error.is_some() {
        return failure("google_cancelled");
    }
    let Some(code) = input.code.filter(|c| !c.is_empty() && c.len() <= 8192) else {
        return failure("google_failed");
    };
    let identity = match identify(&g, config, &pending, &code).await {
        Ok(v) => v,
        Err(_) => return failure("google_failed"),
    };
    let RegistrySource::File(path) = &g.registry else {
        return failure("google_unavailable");
    };
    let path = path.clone();
    // Admission uses already provisioned, isolated runtimes. A public auth route
    // never gains Docker/root privileges or hands out another user's workspace.
    let mut ready = Vec::new();
    if pending.register
        && let Ok(registry) = g.registry.load()
    {
        for slot in registry
            .accounts
            .iter()
            .filter(|a| a.signup_available && !a.disabled)
            .take(8)
        {
            if g.client
                .get(format!(
                    "{}/api/status",
                    slot.upstream.trim_end_matches('/')
                ))
                .header("x-fenix-upstream", &slot.upstream_token)
                .timeout(Duration::from_secs(2))
                .send()
                .await
                .is_ok_and(|r| r.status().is_success())
            {
                ready.push(slot.id.clone());
            }
        }
    }
    let account = match tokio::task::spawn_blocking(move || {
        resolve(path, identity, pending.register, pending.owner, ready)
    })
    .await
    {
        Ok(Ok(account)) => account,
        Ok(Err(error)) => {
            let code = error.to_string();
            return failure(match code.as_str() {
                "google_link_required"
                | "google_registration_required"
                | "google_capacity_unavailable"
                | "google_account_conflict"
                | "google_account_unavailable" => &code,
                _ => "google_failed",
            });
        }
        Err(_) => return failure("google_failed"),
    };
    if !g
        .registry
        .load()
        .is_ok_and(|r| r.accounts.contains(&account))
    {
        return failure("google_account_unavailable");
    }
    let key = token();
    let now = now_millis();
    let mut sessions = g.sessions.lock().await;
    if sessions.full(now) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    if sessions
        .insert(key.clone(), Session::new(&account, now), now)
        .is_err()
    {
        return failure("google_failed");
    }
    let mut response = Json(json!({"redirect":"/chat"})).into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        g.cookie(&key, SESSION_SECONDS).parse().unwrap(),
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        binding_cookie(&g, "", 0).parse().unwrap(),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{header as match_header, method, path},
    };
    fn account(email: &str, port: u16, reserve: bool) -> Account {
        Account {
            id: uuid::Uuid::new_v4().to_string(),
            email: email.into(),
            display_name: String::new(),
            google_subject: None,
            signup_available: reserve,
            password_login: !reserve,
            password_hash: hash_password("test-password").unwrap(),
            upstream: format!("http://127.0.0.1:{port}"),
            upstream_token: token(),
            runtime_port: 4173,
            disabled: false,
            auth_epoch: 0,
        }
    }
    fn identity(email: &str, sub: &str) -> Identity {
        Identity {
            sub: sub.into(),
            email: email.into(),
            email_verified: true,
            name: "Test User".into(),
            hd: None,
        }
    }
    fn write(path: &Path, accounts: Vec<Account>) {
        private_write(path, &serde_json::to_vec(&Registry { accounts }).unwrap()).unwrap();
    }
    #[test]
    fn registration_claims_a_ready_private_tenant_once_and_relogin_uses_subject() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("accounts.json");
        let existing = account("existing@gmail.com", 4500, false);
        let slot = account("slot@signup.invalid", 4501, true);
        write(&file, vec![existing.clone(), slot.clone()]);
        assert!(
            resolve(
                file.clone(),
                identity("new@gmail.com", "new-sub"),
                false,
                None,
                vec![slot.id.clone()]
            )
            .is_err()
        );
        assert!(
            resolve(
                file.clone(),
                identity("new@gmail.com", "new-sub"),
                true,
                None,
                vec![]
            )
            .is_err()
        );
        let created = resolve(
            file.clone(),
            identity("new@gmail.com", "new-sub"),
            true,
            None,
            vec![slot.id.clone()],
        )
        .unwrap();
        assert_eq!(created.id, slot.id);
        assert_eq!(created.upstream, slot.upstream);
        assert!(!created.signup_available);
        assert!(!created.password_login);
        assert_ne!(created.password_hash, slot.password_hash);
        let again = resolve(
            file.clone(),
            identity("changed@gmail.com", "new-sub"),
            true,
            None,
            vec![],
        )
        .unwrap();
        assert_eq!(again.id, created.id);
        let saved = RegistrySource::File(file.clone()).load().unwrap();
        assert!(saved.accounts[0] == existing);
        assert!(
            resolve(
                file,
                identity("another@gmail.com", "another-sub"),
                true,
                None,
                vec![slot.id]
            )
            .is_err()
        );
    }
    #[test]
    fn linking_rejects_unverified_email_conflicting_identity_and_disabled_accounts() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("accounts.json");
        let original = account("owner@example.com", 4500, false);
        write(&file, vec![original.clone()]);
        let mut unverified = identity("owner@example.com", "one");
        unverified.email_verified = false;
        assert!(resolve(file.clone(), unverified, false, None, vec![]).is_err());
        assert!(
            resolve(
                file.clone(),
                identity("owner@example.com", "one"),
                false,
                None,
                vec![]
            )
            .is_err()
        );
        let linked = resolve(
            file.clone(),
            identity("owner@example.com", "one"),
            false,
            Some(original.id.clone()),
            vec![],
        )
        .unwrap();
        assert_eq!(linked.password_hash, original.password_hash);
        assert!(
            resolve(
                file.clone(),
                identity("owner@example.com", "two"),
                false,
                Some(original.id),
                vec![]
            )
            .is_err()
        );
        let mut disabled = linked;
        disabled.disabled = true;
        write(&file, vec![disabled]);
        assert!(
            resolve(
                file,
                identity("owner@example.com", "one"),
                false,
                None,
                vec![]
            )
            .is_err()
        );
    }
    #[tokio::test]
    async fn oauth_binds_browser_consumes_state_and_uses_server_obtained_identity() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"access_token":"mock-access"})),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET")).and(path("/userinfo")).and(match_header("authorization", "Bearer mock-access"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"sub":"google-one","email":"owner@gmail.com","email_verified":true,"name":"Owner"}))).expect(1).mount(&server).await;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("accounts.json");
        let owner = account("owner@gmail.com", 4500, false);
        write(&file, vec![owner.clone()]);
        let g = Arc::new(Gateway {
            registry: RegistrySource::File(file),
            slots: Mutex::new(HashMap::new()),
            global_slots: Arc::new(tokio::sync::Semaphore::new(2)),
            sessions: Mutex::new(Sessions::open(dir.path().join("sessions.json")).unwrap()),
            attempts: Mutex::new(HashMap::new()),
            hashing: tokio::sync::Semaphore::new(2),
            client: reqwest::Client::new(),
            secure: false,
            google: GoogleAuth {
                config: Some(Config {
                    client_id: "test-client".into(),
                    client_secret: "test-secret".into(),
                    redirect: "http://127.0.0.1:4173/api/auth/google/callback".into(),
                    token_endpoint: format!("{}/token", server.uri()),
                    userinfo_endpoint: format!("{}/userinfo", server.uri()),
                }),
                pending: Mutex::new(HashMap::new()),
                exchanges: tokio::sync::Semaphore::new(4),
            },
        });
        let started = start(
            State(g.clone()),
            HeaderMap::new(),
            Json(Start { register: false }),
        )
        .await;
        let binding = started.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_string();
        let body = axum::body::to_bytes(started.into_body(), 8192)
            .await
            .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let url = reqwest::Url::parse(value["url"].as_str().unwrap()).unwrap();
        let query: HashMap<_, _> = url
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        assert_eq!(query["scope"], "openid email profile");
        assert_eq!(query["code_challenge_method"], "S256");
        let state = query["state"].clone();
        let callback = || {
            Json(Callback {
                state: state.clone(),
                code: Some("server-code".into()),
                error: None,
            })
        };
        assert_eq!(
            finish(State(g.clone()), HeaderMap::new(), callback())
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, binding.parse().unwrap());
        let result = finish(State(g.clone()), headers.clone(), callback()).await;
        assert_eq!(result.status(), StatusCode::OK);
        let session = result
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .find_map(|h| {
                h.to_str()
                    .ok()
                    .and_then(|s| s.strip_prefix("fenix_session="))
                    .map(|s| s.split(';').next().unwrap().to_string())
            })
            .unwrap();
        assert_eq!(g.account(Some(session)).await.unwrap().id, owner.id);
        assert_eq!(
            finish(State(g.clone()), headers, callback()).await.status(),
            StatusCode::BAD_REQUEST
        );
        for (expired, denied) in [(true, false), (false, true)] {
            let state = token();
            g.google.pending.lock().await.insert(
                state.clone(),
                Pending {
                    browser: "test-binding".into(),
                    verifier: token(),
                    created: Instant::now() - Duration::from_secs(if expired { 601 } else { 0 }),
                    register: false,
                    owner: None,
                },
            );
            let mut headers = HeaderMap::new();
            headers.insert(header::COOKIE, "fenix_google=test-binding".parse().unwrap());
            let response = finish(
                State(g.clone()),
                headers,
                Json(Callback {
                    state: state.clone(),
                    code: Some("unused".into()),
                    error: denied.then(|| "access_denied".into()),
                }),
            )
            .await;
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            if denied {
                assert!(!g.google.pending.lock().await.contains_key(&state));
            }
        }
        let requests = server.received_requests().await.unwrap();
        let form = String::from_utf8_lossy(&requests[0].body);
        assert!(form.contains("code_verifier="));
        assert!(form.contains("client_id=test-client"));
    }
    #[test]
    fn concurrent_registrations_cannot_claim_the_same_tenant() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("accounts.json");
        let slot = account("slot@signup.invalid", 4510, true);
        write(&file, vec![slot.clone()]);
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let handles: Vec<_> = (0..2)
            .map(|n| {
                let file = file.clone();
                let id = slot.id.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    resolve(
                        file,
                        identity(&format!("user{n}@gmail.com"), &format!("subject{n}")),
                        true,
                        None,
                        vec![id],
                    )
                    .is_ok()
                })
            })
            .collect();
        assert_eq!(
            handles
                .into_iter()
                .map(|h| usize::from(h.join().unwrap()))
                .sum::<usize>(),
            1
        );
    }
    #[tokio::test]
    async fn public_landing_does_not_allow_cross_site_post_or_enable_unconfigured_login() {
        use tower::ServiceExt;
        let dir = tempfile::tempdir().unwrap();
        let app = super::super::router(
            Registry::default(),
            4173,
            None,
            Sessions::open(dir.path().join("sessions.json")).unwrap(),
        );
        let request = Request::builder()
            .uri("/api/auth/google/callback")
            .header("host", "127.0.0.1:4173")
            .header("sec-fetch-site", "cross-site")
            .body(Body::empty())
            .unwrap();
        let landing = app.clone().oneshot(request).await.unwrap();
        assert_eq!(landing.status(), StatusCode::OK);
        assert_eq!(landing.headers()[header::CACHE_CONTROL], "no-store");
        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/google/callback")
            .header("host", "127.0.0.1:4173")
            .header("origin", "https://untrusted.example")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"state":"x"}"#))
            .unwrap();
        assert_eq!(
            app.oneshot(request).await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
    }
}
