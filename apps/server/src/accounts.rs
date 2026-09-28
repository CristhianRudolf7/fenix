//! Host-side authentication and routing. Tenant processes never receive this registry.
use anyhow::{Context, Result, ensure};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    Json, Router,
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    io::{Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
mod profile;
mod registry;
mod sessions;
use registry::RegistrySource;
use sessions::{SESSION_SECONDS, Session, Sessions, now_millis};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub display_name: String,
    pub password_hash: String,
    pub upstream: String,
    pub upstream_token: String,
    #[serde(default = "runtime_port")]
    pub runtime_port: u16,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub auth_epoch: u64,
}
fn runtime_port() -> u16 {
    4173
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Registry {
    pub accounts: Vec<Account>,
}
struct Gateway {
    registry: RegistrySource,
    slots: Mutex<HashMap<String, Arc<tokio::sync::Semaphore>>>,
    global_slots: Arc<tokio::sync::Semaphore>,
    sessions: Mutex<Sessions>,
    attempts: Mutex<HashMap<String, (u32, Instant)>>,
    hashing: tokio::sync::Semaphore,
    client: reqwest::Client,
    secure: bool,
}
fn token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
fn hash_password(password: &str) -> Result<String> {
    let bytes = *uuid::Uuid::new_v4().as_bytes();
    let salt = SaltString::encode_b64(&bytes).map_err(|e| anyhow::anyhow!("{e}"))?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| anyhow::anyhow!("{e}"))
}
fn verify(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|h| {
        Argon2::default()
            .verify_password(password.as_bytes(), &h)
            .is_ok()
    })
}
fn private_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    std::fs::rename(tmp, path)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::File::open(parent)?.sync_all()?;
    Ok(())
}
/// Deliberately local-only: there is no account creation HTTP route.
pub fn admin() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(2).collect();
    let _registry_lock = if let Some(registry_path) = args.get(1) {
        let lock_path = Path::new(registry_path).with_extension("admin.lock");
        if !lock_path.exists() {
            fenix_execution::fs::create(&lock_path, b"")?;
        }
        let lock = fenix_execution::fs::open(&lock_path)?;
        lock.lock()?;
        Some(lock)
    } else {
        None
    };
    if matches!(
        args.first().map(String::as_str),
        Some("disable" | "enable" | "passwd")
    ) {
        ensure!(
            args.len() == 3,
            "Usage: fenix accounts disable|enable|passwd REGISTRY EMAIL"
        );
        let path = PathBuf::from(&args[1]);
        let mut registry = RegistrySource::File(path.clone()).load()?;
        let account = registry
            .accounts
            .iter_mut()
            .find(|a| a.email == args[2].trim().to_lowercase())
            .context("Account not found")?;
        account.auth_epoch = account
            .auth_epoch
            .checked_add(1)
            .context("Account revision exhausted")?;
        match args[0].as_str() {
            "disable" => account.disabled = true,
            "enable" => account.disabled = false,
            _ => {
                let mut password = String::new();
                std::io::stdin().take(1025).read_to_string(&mut password)?;
                let password = password.trim_end_matches(['\r', '\n']);
                ensure!(
                    (7..=1024).contains(&password.len()),
                    "Password length must be 7–1024 bytes"
                );
                account.password_hash = hash_password(password)?;
            }
        }
        private_write(&path, &serde_json::to_vec_pretty(&registry)?)?;
        println!(
            "Account updated. Gateway authorization reloads on the next request; execution leases are revoked by the host service."
        );
        return Ok(());
    }
    ensure!(
        args.len() == 5 && args[0] == "create",
        "Usage: fenix accounts create REGISTRY USERS_ROOT EMAIL UPSTREAM (password on stdin)"
    );
    let path = PathBuf::from(&args[1]);
    let root = PathBuf::from(&args[2]);
    let email = args[3].trim().to_lowercase();
    ensure!(email.contains('@') && email.len() <= 254, "Invalid email");
    let url = reqwest::Url::parse(&args[4])?;
    ensure!(
        url.scheme() == "http"
            && url.host_str() == Some("127.0.0.1")
            && url.port().is_some_and(|port| port > 0)
            && url.path() == "/"
            && url.query().is_none()
            && url.fragment().is_none()
            && url.username().is_empty()
            && url.password().is_none(),
        "Upstream must be a loopback HTTP origin"
    );
    let mut registry: Registry = if path.exists() {
        RegistrySource::File(path.clone()).load()?
    } else {
        Registry::default()
    };
    ensure!(
        !registry
            .accounts
            .iter()
            .any(|a| a.email == email || a.upstream == args[4]),
        "Account or upstream already exists"
    );
    let mut password = String::new();
    std::io::stdin().take(1025).read_to_string(&mut password)?;
    let password = password.trim_end_matches(['\r', '\n']);
    ensure!(
        (7..=1024).contains(&password.len()),
        "Password length must be 7–1024 bytes"
    );
    let id = uuid::Uuid::new_v4().to_string();
    let candidate = Account {
        id: id.clone(),
        email: email.clone(),
        display_name: String::new(),
        password_hash: hash_password(password)?,
        upstream: args[4].clone(),
        upstream_token: token(),
        runtime_port: 4173,
        disabled: false,
        auth_epoch: 0,
    };
    registry.accounts.push(candidate);
    registry::validate(&registry)?;
    let directory = root.join(&id);
    std::fs::create_dir_all(&root)?;
    std::fs::create_dir(&directory)?;
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
    for name in [
        "projects",
        "workspace",
        "memory",
        "skills",
        "artifacts",
        "sessions",
        "state",
    ] {
        std::fs::create_dir(directory.join(name))?;
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    private_write(&path, &serde_json::to_vec_pretty(&registry)?)?;
    println!("Created account {id}; directory: {}", directory.display());
    Ok(())
}
impl Gateway {
    async fn account(&self, key: Option<String>) -> Option<Account> {
        let key = key?;
        let registry = self.registry.load().ok()?;
        let sessions = self.sessions.lock().await;
        let session = sessions.get(&key, now_millis())?;
        registry.accounts.into_iter().find(|account| {
            !account.disabled
                && account.id == session.account_id
                && account.password_hash == session.password_hash
                && account.auth_epoch == session.auth_epoch
        })
    }
    fn cookie(&self, value: &str, age: u64) -> String {
        format!(
            "fenix_session={value}; HttpOnly; SameSite=Strict; Path=/; Max-Age={age}{}",
            if self.secure { "; Secure" } else { "" }
        )
    }
}
fn cookie(request: &Request) -> Option<&str> {
    request
        .headers()
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| part.trim().strip_prefix("fenix_session="))
}
#[derive(Deserialize)]
struct Login {
    email: String,
    password: String,
}
async fn login(State(g): State<Arc<Gateway>>, Json(input): Json<Login>) -> Response {
    let email = input.email.trim().to_lowercase();
    if email.len() > 254 || input.password.len() > 1024 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    {
        let mut attempts = g.attempts.lock().await;
        attempts.retain(|_, (_, at)| at.elapsed() < Duration::from_secs(300));
        if attempts.len() >= 1000 && !attempts.contains_key(&email) {
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        }
        let entry = attempts.entry(email.clone()).or_insert((0, Instant::now()));
        if entry.0 >= 5 {
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        }
        entry.0 += 1;
    }
    let Ok(_permit) = g.hashing.try_acquire() else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    let Ok(registry) = g.registry.load() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let account = registry
        .accounts
        .iter()
        .find(|a| !a.disabled && a.email == email)
        .cloned();
    let Some(hash) = account
        .as_ref()
        .or(registry.accounts.first())
        .map(|a| a.password_hash.clone())
    else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let valid = tokio::task::spawn_blocking(move || verify(&input.password, &hash))
        .await
        .unwrap_or(false);
    let Some(account) =
        account.filter(|a| valid && g.registry.load().is_ok_and(|r| r.accounts.contains(a)))
    else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"Invalid email or password"})),
        )
            .into_response();
    };
    g.attempts.lock().await.remove(&email);
    let token = token();
    let mut sessions = g.sessions.lock().await;
    let now = now_millis();
    if sessions.full(now) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    if sessions
        .insert(token.clone(), Session::new(&account, now), now)
        .is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    (
        [(header::SET_COOKIE, g.cookie(&token, SESSION_SECONDS))],
        Json(json!({"email":email,"user_id":account.id})),
    )
        .into_response()
}
async fn session(State(g): State<Arc<Gateway>>, request: Request) -> Response {
    match g.account(cookie(&request).map(str::to_owned)).await {
        Some(account) => Json(json!({"email":account.email,"user_id":account.id})).into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}
async fn logout(State(g): State<Arc<Gateway>>, request: Request) -> Response {
    if let Some(key) = cookie(&request)
        && g.sessions.lock().await.remove(key).is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    (
        [(header::SET_COOKIE, g.cookie("", 0))],
        StatusCode::NO_CONTENT,
    )
        .into_response()
}
async fn proxy(State(g): State<Arc<Gateway>>, request: Request) -> Response {
    let session_key = cookie(&request).map(str::to_owned);
    let Some(account) = g.account(session_key.clone()).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Ok(global_permit) = g.global_slots.clone().try_acquire_owned() else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    let slots = g
        .slots
        .lock()
        .await
        .entry(account.id.clone())
        .or_insert_with(|| Arc::new(tokio::sync::Semaphore::new(16)))
        .clone();
    let Ok(account_permit) = slots.try_acquire_owned() else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    let (parts, body) = request.into_parts();
    let Ok(Ok(bytes)) = tokio::time::timeout(
        Duration::from_secs(15),
        axum::body::to_bytes(body, 256 * 1024),
    )
    .await
    else {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    };
    let path = parts
        .uri
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or("/");
    let mut upstream = g
        .client
        .request(
            parts.method,
            format!("{}{path}", account.upstream.trim_end_matches('/')),
        )
        .header(header::HOST, format!("127.0.0.1:{}", account.runtime_port))
        .header("x-fenix-upstream", &account.upstream_token)
        .body(bytes);
    for name in [
        header::CONTENT_TYPE,
        header::ACCEPT,
        header::HeaderName::from_static("last-event-id"),
    ] {
        if let Some(value) = parts.headers.get(&name) {
            upstream = upstream.header(name, value);
        }
    }
    if g.account(session_key.clone()).await.as_ref() != Some(&account) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match tokio::time::timeout(Duration::from_secs(60), upstream.send()).await {
        Ok(Ok(response)) => {
            let mut builder = Response::builder().status(response.status());
            for name in [header::CONTENT_TYPE, header::CONTENT_DISPOSITION] {
                if let Some(value) = response.headers().get(&name) {
                    builder = builder.header(name, value);
                }
            }
            // Poll authorization even when SSE is quiet. Pin the complete account
            // binding, not a mutable registry index that could refer to someone else.
            let source = response.bytes_stream();
            let stream = futures_util::stream::unfold(
                (
                    source,
                    g,
                    session_key,
                    account,
                    global_permit,
                    account_permit,
                ),
                |(mut source, gateway, key, expected, global, account_slot)| async move {
                    loop {
                        if gateway.account(key.clone()).await.as_ref() != Some(&expected) {
                            return None;
                        }
                        tokio::select! {
                            chunk = source.next() => {
                                let chunk = chunk?;
                                if gateway.account(key.clone()).await.as_ref() != Some(&expected) { return None; }
                                return Some((chunk, (source, gateway, key, expected, global, account_slot)));
                            }
                            _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                        }
                    }
                },
            );
            builder
                .body(Body::from_stream(stream))
                .expect("Valid proxy response")
        }
        _ => (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error":"Your isolated agent is unavailable"})),
        )
            .into_response(),
    }
}
#[cfg(test)]
fn router(registry: Registry, port: u16, public: Option<String>, sessions: Sessions) -> Router {
    router_with_market(registry, port, public, sessions, None)
}
async fn market_owner(
    State(g): State<Arc<Gateway>>,
    mut request: Request,
    next: middleware::Next,
) -> Response {
    let Some(account) = g.account(cookie(&request).map(str::to_owned)).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    request
        .extensions_mut()
        .insert(crate::skill_guard::Owner(account.id));
    next.run(request).await
}
#[cfg(test)]
fn router_with_market(
    registry: Registry,
    port: u16,
    public: Option<String>,
    sessions: Sessions,
    market: Option<Arc<crate::skill_guard::Service>>,
) -> Router {
    router_from_source(
        RegistrySource::Static(registry),
        port,
        public,
        sessions,
        market,
    )
}
fn router_from_source(
    registry: RegistrySource,
    port: u16,
    public: Option<String>,
    sessions: Sessions,
    market: Option<Arc<crate::skill_guard::Service>>,
) -> Router {
    let g = Arc::new(Gateway {
        registry,
        slots: Mutex::new(HashMap::new()),
        global_slots: Arc::new(tokio::sync::Semaphore::new(256)),
        sessions: Mutex::new(sessions),
        attempts: Mutex::new(HashMap::new()),
        hashing: tokio::sync::Semaphore::new(2),
        client: reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("HTTP client"),
        secure: public.is_some(),
    });
    let app = Router::new()
        .route("/", get(crate::assets::index))
        .route("/chat", get(crate::assets::index))
        .route("/projects", get(crate::assets::index))
        .route("/memory", get(crate::assets::index))
        .route("/skills", get(crate::assets::index))
        .route("/design", get(crate::assets::index))
        .route("/browser", get(crate::assets::index))
        .route("/integrations", get(crate::assets::index))
        .route("/schedules", get(crate::assets::index))
        .route("/activity", get(crate::assets::index))
        .route("/settings", get(crate::assets::index))
        .route("/profile", get(crate::assets::index))
        .route("/assets/{*path}", get(crate::assets::asset))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/session", get(session))
        .route("/api/auth/profile", get(profile::get).put(profile::update))
        .route("/api/auth/password", post(profile::password))
        .route(
            "/api/connectors/oauth/callback",
            get(crate::connectors::oauth_landing).post(proxy),
        )
        // Unknown auth routes (including registration) deliberately remain 404.
        .route(
            "/api/auth/{*path}",
            axum::routing::any(|| async { StatusCode::NOT_FOUND }),
        )
        .route("/api/{*path}", axum::routing::any(proxy))
        .with_state(g.clone());
    let app = if let Some(market) = market {
        app.merge(
            market
                .routes()
                .layer(middleware::from_fn_with_state(g, market_owner)),
        )
    } else {
        app
    };
    app.layer(axum::extract::DefaultBodyLimit::max(256 * 1024))
        .layer(middleware::from_fn(move |request, next| {
            crate::local_guard(request, next, port, public.clone())
        }))
}
pub async fn serve() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(2).collect();
    ensure!(args.len() == 1, "Usage: fenix gateway REGISTRY");
    let registry = RegistrySource::File(PathBuf::from(&args[0])).load()?;
    ensure!(!registry.accounts.is_empty(), "No accounts provisioned");
    let public = std::env::var("FENIX_PUBLIC_ORIGIN").ok();
    if let Some(value) = &public {
        let url = reqwest::Url::parse(value)?;
        ensure!(
            url.scheme() == "https" && url.origin().ascii_serialization() == *value,
            "Invalid public origin"
        );
    }
    let port = std::env::var("FENIX_GATEWAY_PORT")
        .unwrap_or("4173".into())
        .parse::<u16>()?;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .context("Bind gateway")?;
    let sessions = Sessions::open(Path::new(&args[0]).with_extension("sessions.json"))?;
    let market_dir = Path::new(&args[0])
        .parent()
        .context("Registry needs a parent directory")?
        .join("skill-exchange");
    std::fs::create_dir_all(&market_dir)?;
    std::fs::set_permissions(&market_dir, std::fs::Permissions::from_mode(0o700))?;
    let config = std::env::var_os("FENIX_SENTINEL_CONFIG")
        .map(PathBuf::from)
        .or_else(|| {
            let path = market_dir.join("sentinel.json");
            path.is_file().then_some(path)
        });
    let market =
        crate::skill_guard::Service::hosted(&market_dir.join("market.sqlite"), config.as_deref())?;
    axum::serve(
        listener,
        router_from_source(
            RegistrySource::File(PathBuf::from(&args[0])),
            port,
            public,
            sessions,
            Some(market),
        ),
    )
    .with_graceful_shutdown(async {
        let mut sig = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM");
        tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = sig.recv() => {} }
    })
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;
    fn request(path: &str, method: &str, body: &str) -> Request {
        Request::builder()
            .uri(path)
            .method(method)
            .header("host", "localhost:4173")
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }
    #[tokio::test]
    async fn live_registry_revocation_closes_quiet_streams_and_cannot_resurrect_sessions() {
        let directory = tempfile::tempdir().unwrap();
        let registry_path = directory.path().join("accounts.json");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream = format!("http://{}", listener.local_addr().unwrap());
        let source = Router::new().route(
            "/api/quiet",
            get(|| async {
                let stream = futures_util::stream::once(async {
                    Ok::<_, std::io::Error>(axum::body::Bytes::from_static(b"ready\n"))
                })
                .chain(futures_util::stream::pending::<
                    Result<axum::body::Bytes, std::io::Error>,
                >());
                Response::builder()
                    .header(header::CONTENT_TYPE, "text/event-stream")
                    .body(Body::from_stream(stream))
                    .unwrap()
            }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, source).await.unwrap() });
        let mut registry = Registry {
            accounts: vec![Account {
                id: uuid::Uuid::new_v4().to_string(),
                email: "alice@example.test".into(),
                display_name: String::new(),
                password_hash: hash_password("test-password").unwrap(),
                upstream,
                upstream_token: token(),
                runtime_port: 4173,
                disabled: false,
                auth_epoch: 0,
            }],
        };
        private_write(&registry_path, &serde_json::to_vec(&registry).unwrap()).unwrap();
        let app = router_from_source(
            RegistrySource::File(registry_path.clone()),
            4173,
            None,
            Sessions::open(directory.path().join("sessions.json")).unwrap(),
            None,
        );
        let response = app
            .clone()
            .oneshot(request(
                "/api/auth/login",
                "POST",
                r#"{"email":"alice@example.test","password":"test-password"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let cookie = response.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        let authenticated = |path| {
            let mut request = request(path, "GET", "");
            request
                .headers_mut()
                .insert(header::COOKIE, cookie.parse().unwrap());
            request
        };
        let response = app
            .clone()
            .oneshot(authenticated("/api/quiet"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let mut stream = response.into_body().into_data_stream();
        assert_eq!(stream.next().await.unwrap().unwrap(), "ready\n");
        let mut waiting = Box::pin(stream.next());
        assert!(
            tokio::time::timeout(Duration::from_millis(30), &mut waiting)
                .await
                .is_err()
        );
        registry.accounts[0].disabled = true;
        registry.accounts[0].auth_epoch += 1;
        private_write(&registry_path, &serde_json::to_vec(&registry).unwrap()).unwrap();
        assert!(
            tokio::time::timeout(Duration::from_secs(2), waiting)
                .await
                .unwrap()
                .is_none()
        );
        registry.accounts[0].disabled = false;
        private_write(&registry_path, &serde_json::to_vec(&registry).unwrap()).unwrap();
        assert_eq!(
            app.clone()
                .oneshot(authenticated("/api/auth/session"))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        std::fs::write(&registry_path, b"corrupt").unwrap();
        assert_eq!(
            app.oneshot(authenticated("/api/auth/session"))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        server.abort();
    }

    #[tokio::test]
    async fn sessions_route_only_to_their_own_tenant() {
        let mut accounts = Vec::new();
        let mut servers = Vec::new();
        for id in ["alice", "bob"] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let upstream = format!("http://{}", listener.local_addr().unwrap());
            let expected = format!("secret-{id}");
            let app = Router::new().route(
                "/api/messages",
                get(move |request: Request| async move {
                    assert_eq!(request.headers()["x-fenix-upstream"], expected);
                    assert!(!request.headers().contains_key(header::COOKIE));
                    Json(json!({"owner":id}))
                }),
            );
            servers.push(tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            }));
            accounts.push(Account {
                id: id.into(),
                email: format!("{id}@example.test"),
                display_name: String::new(),
                password_hash: hash_password("test-password").unwrap(),
                upstream,
                upstream_token: format!("secret-{id}"),
                runtime_port: 4173,
                disabled: false,
                auth_epoch: 0,
            });
        }
        let directory = tempfile::tempdir().unwrap();
        let app = router(
            Registry { accounts },
            4173,
            None,
            Sessions::open(directory.path().join("sessions.json")).unwrap(),
        );
        for id in ["alice", "bob"] {
            let response = app
                .clone()
                .oneshot(request(
                    "/api/auth/login",
                    "POST",
                    &json!({"email":format!("{id}@example.test"),"password":"test-password"})
                        .to_string(),
                ))
                .await
                .unwrap();
            let cookie = response.headers()[header::SET_COOKIE]
                .to_str()
                .unwrap()
                .split(';')
                .next()
                .unwrap()
                .to_owned();
            let mut req = request("/api/messages?user_id=other-user", "GET", "");
            req.headers_mut()
                .insert(header::COOKIE, cookie.parse().unwrap());
            req.headers_mut()
                .insert("x-fenix-upstream", "attacker-controlled".parse().unwrap());
            let response = app.clone().oneshot(req).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let bytes = axum::body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap();
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["owner"],
                id
            );
        }
        let mut cross_site = request("/api/auth/login", "POST", "{}");
        cross_site
            .headers_mut()
            .insert(header::ORIGIN, "https://evil.example".parse().unwrap());
        assert_eq!(
            app.oneshot(cross_site).await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
        for server in servers {
            server.abort();
        }
    }
    #[tokio::test]
    async fn oauth_landing_is_public_but_exchange_requires_the_owner_session() {
        let directory = tempfile::tempdir().unwrap();
        let app = router(
            Registry { accounts: vec![] },
            4173,
            None,
            Sessions::open(directory.path().join("sessions.json")).unwrap(),
        );
        let mut landing = request(
            "/api/connectors/oauth/callback?state=untrusted&code=untrusted",
            "GET",
            "",
        );
        landing
            .headers_mut()
            .insert("sec-fetch-site", "cross-site".parse().unwrap());
        let response = app.clone().oneshot(landing).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert!(
            response.headers()["content-security-policy"]
                .to_str()
                .unwrap()
                .contains("script-src 'nonce-")
        );
        let html = axum::body::to_bytes(response.into_body(), 32_000)
            .await
            .unwrap();
        assert!(!String::from_utf8_lossy(&html).contains("untrusted"));
        let response = app
            .clone()
            .oneshot(request(
                "/api/connectors/oauth/callback",
                "POST",
                r#"{"state":"untrusted","code":"untrusted"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let mut cross_site = request("/api/connectors/oauth/callback", "POST", "{}");
        cross_site
            .headers_mut()
            .insert("sec-fetch-site", "cross-site".parse().unwrap());
        assert_eq!(
            app.oneshot(cross_site).await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
    }
    #[tokio::test]
    async fn authentication_and_no_registration() {
        let registry = Registry {
            accounts: vec![Account {
                id: "one".into(),
                email: "one@example.test".into(),
                display_name: String::new(),
                password_hash: hash_password("test-password").unwrap(),
                upstream: "http://127.0.0.1:9".into(),
                upstream_token: token(),
                runtime_port: 4173,
                disabled: false,
                auth_epoch: 0,
            }],
        };
        let directory = tempfile::tempdir().unwrap();
        let app = router(
            registry,
            4173,
            None,
            Sessions::open(directory.path().join("sessions.json")).unwrap(),
        );
        assert_eq!(
            app.clone()
                .oneshot(request("/api/messages", "GET", ""))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            app.clone()
                .oneshot(request("/api/auth/register", "POST", "{}"))
                .await
                .unwrap()
                .status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            app.clone()
                .oneshot(request(
                    "/api/auth/login",
                    "POST",
                    r#"{"email":"one@example.test","password":"wrong"}"#
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let response = app
            .clone()
            .oneshot(request(
                "/api/auth/login",
                "POST",
                r#"{"email":"one@example.test","password":"test-password"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let cookie = response.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        let mut req = request("/api/auth/session", "GET", "");
        req.headers_mut()
            .insert(header::COOKIE, cookie.parse().unwrap());
        assert_eq!(
            app.clone().oneshot(req).await.unwrap().status(),
            StatusCode::OK
        );
        let mut req = request("/api/auth/logout", "POST", "{}");
        req.headers_mut()
            .insert(header::COOKIE, cookie.parse().unwrap());
        assert_eq!(
            app.clone().oneshot(req).await.unwrap().status(),
            StatusCode::NO_CONTENT
        );
        let mut req = request("/api/auth/session", "GET", "");
        req.headers_mut()
            .insert(header::COOKIE, cookie.parse().unwrap());
        assert_eq!(
            app.oneshot(req).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
    }
    #[tokio::test]
    async fn persistent_login_survives_restart_and_account_reordering_but_not_logout_or_password_change()
     {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sessions.json");
        let alice = Account {
            id: "alice".into(),
            email: "alice@example.test".into(),
            display_name: String::new(),
            password_hash: hash_password("test-password").unwrap(),
            upstream: "http://127.0.0.1:9".into(),
            upstream_token: token(),
            runtime_port: 4173,
            disabled: false,
            auth_epoch: 0,
        };
        let mut bob = alice.clone();
        bob.id = "bob".into();
        bob.email = "bob@example.test".into();
        let registry = Registry {
            accounts: vec![alice.clone(), bob.clone()],
        };
        let make = |registry| router(registry, 4173, None, Sessions::open(path.clone()).unwrap());
        let app = make(registry.clone());
        let response = app
            .oneshot(request(
                "/api/auth/login",
                "POST",
                r#"{"email":"alice@example.test","password":"test-password"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let header = response.headers()[header::SET_COOKIE].to_str().unwrap();
        assert!(header.contains("Max-Age=604800"));
        assert!(header.contains("HttpOnly; SameSite=Strict; Path=/"));
        let cookie = header.split(';').next().unwrap().to_owned();
        let with_cookie = |path, method, body| {
            let mut req = request(path, method, body);
            req.headers_mut()
                .insert(header::COOKIE, cookie.parse().unwrap());
            req
        };
        let original = std::fs::read(&path).unwrap();
        let reordered = Registry {
            accounts: vec![bob, alice],
        };
        let restarted = make(reordered.clone());
        let response = restarted
            .clone()
            .oneshot(with_cookie("/api/auth/session", "GET", ""))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["user_id"],
            "alice"
        );
        assert_eq!(std::fs::read(&path).unwrap(), original);
        // An expired durable session is rejected by the HTTP boundary after restart.
        let mut expired: serde_json::Value = serde_json::from_slice(&original).unwrap();
        let key = cookie.strip_prefix("fenix_session=").unwrap();
        let deadline = now_millis() - 1;
        expired["sessions"][key]["issued_at_ms"] = json!(deadline - SESSION_SECONDS * 1000);
        expired["sessions"][key]["expires_at_ms"] = json!(deadline);
        private_write(&path, &serde_json::to_vec(&expired).unwrap()).unwrap();
        assert_eq!(
            make(reordered.clone())
                .oneshot(with_cookie("/api/auth/session", "GET", ""))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        private_write(&path, &original).unwrap();
        let mut changed = reordered.clone();
        changed.accounts[1].password_hash = hash_password("replacement-password").unwrap();
        assert_eq!(
            make(changed)
                .oneshot(with_cookie("/api/auth/session", "GET", ""))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            make(Registry { accounts: vec![] })
                .oneshot(with_cookie("/api/auth/session", "GET", ""))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            restarted
                .oneshot(with_cookie("/api/auth/logout", "POST", "{}"))
                .await
                .unwrap()
                .status(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            make(reordered)
                .oneshot(with_cookie("/api/auth/session", "GET", ""))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
}

#[cfg(test)]
mod marketplace_tests {
    use super::*;
    use tower::ServiceExt;
    #[tokio::test]
    async fn central_market_uses_authenticated_account_not_supplied_headers() {
        let dir = tempfile::tempdir().unwrap();
        let accounts: Vec<_> = ["alice", "bob"]
            .into_iter()
            .map(|id| Account {
                id: id.into(),
                email: format!("{id}@test.invalid"),
                display_name: String::new(),
                password_hash: "hash".into(),
                upstream: "http://127.0.0.1:1".into(),
                upstream_token: "unused".into(),
                runtime_port: 4173,
                disabled: false,
                auth_epoch: 0,
            })
            .collect();
        let mut sessions = Sessions::open(dir.path().join("sessions.json")).unwrap();
        for account in &accounts {
            sessions
                .insert(
                    account.id.clone(),
                    Session::new(account, now_millis()),
                    now_millis(),
                )
                .unwrap();
        }
        let market =
            crate::skill_guard::Service::hosted(&dir.path().join("market.sqlite"), None).unwrap();
        let app = router_with_market(Registry { accounts }, 4173, None, sessions, Some(market));
        let request = |owner: &str, method: &str, path: &str, body: serde_json::Value| {
            Request::builder()
                .method(method)
                .uri(path)
                .header("host", "localhost:4173")
                .header("content-type", "application/json")
                .header("cookie", format!("fenix_session={owner}"))
                .header("x-owner-id", "alice")
                .body(Body::from(body.to_string()))
                .unwrap()
        };
        let response=app.clone().oneshot(request("alice","POST","/api/skill-exchange/publications",json!({"bundle":{"name":"example","files":{"SKILL.md":"description: Example\n\nSummarize notes."},"executable":[]},"category":"office"}))).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), 16384)
                .await
                .unwrap(),
        )
        .unwrap();
        let id = value["id"].as_str().unwrap();
        for (owner, expected) in [
            ("alice", StatusCode::OK),
            ("bob", StatusCode::NOT_FOUND),
            ("unknown", StatusCode::UNAUTHORIZED),
        ] {
            let response = app
                .clone()
                .oneshot(request(
                    owner,
                    "GET",
                    &format!("/api/skill-exchange/publications/{id}"),
                    json!({}),
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), expected, "owner {owner}");
        }
    }
}
