//! Single-owner authentication in the agent server. No account routing or upstream proxy.
use anyhow::{Context, Result, ensure};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    Json, Router,
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use fenix_agent::Runtime;
use fenix_storage::Store;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{Mutex, Semaphore};

pub const SESSION_SECONDS: u64 = 7 * 24 * 60 * 60;
const SESSION_MILLIS: u64 = SESSION_SECONDS * 1000;
const OWNER_KIND: &str = "personal_auth";
const SESSION_KIND: &str = "personal_session";
#[derive(Clone, Serialize, Deserialize)]
struct Owner {
    email: String,
    display_name: String,
    password_hash: String,
    revision: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Session {
    revision: String,
    issued_at_ms: u64,
    expires_at_ms: u64,
}
impl Session {
    fn valid_at(&self, now: u64) -> bool {
        self.expires_at_ms.checked_sub(self.issued_at_ms) == Some(SESSION_MILLIS)
            && now >= self.issued_at_ms
            && now < self.expires_at_ms
    }
}
struct Auth {
    store: Arc<Store>,
    secure: bool,
    changes: Mutex<()>,
    attempts: Mutex<(Instant, u32)>,
    hashing: Semaphore,
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn token_id(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}
fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::encode_b64(uuid::Uuid::new_v4().as_bytes())
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| anyhow::anyhow!("{e}"))
}
fn verify(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok()
    })
}
fn cookie(request: &Request) -> Option<String> {
    request
        .headers()
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| part.trim().strip_prefix("fenix_session="))
        .filter(|token| token.len() == 64 && token.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(str::to_owned)
}
impl Auth {
    fn owner(&self) -> Result<Option<Owner>> {
        self.store.get(OWNER_KIND, "owner")
    }
    fn session(&self, token: &str, clock: u64) -> Option<Session> {
        let owner = self.owner().ok()??;
        let session: Session = self.store.get(SESSION_KIND, &token_id(token)).ok()??;
        (session.valid_at(clock) && session.revision == owner.revision).then_some(session)
    }
    fn cookie(&self, token: &str, age: u64) -> String {
        format!(
            "fenix_session={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age={age}{}",
            if self.secure { "; Secure" } else { "" }
        )
    }
}
/// Wrap the single runtime, preserving direct API and event delivery.
pub fn protect(app: Router, runtime: Arc<Runtime>, secure: bool) -> Router {
    let auth = Arc::new(Auth {
        store: runtime.store.clone(),
        secure,
        changes: Mutex::new(()),
        attempts: Mutex::new((Instant::now(), 0)),
        hashing: Semaphore::new(2),
    });
    let routes = Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/session", get(session))
        .route("/api/auth/profile", get(profile).put(update_profile))
        .route("/api/auth/password", post(password))
        .route(
            "/api/auth/google",
            get(|| async { Json(json!({"enabled":false,"registration":false})) }),
        )
        .with_state(auth.clone());
    app.merge(routes)
        .layer(axum::extract::DefaultBodyLimit::max(256 * 1024))
        .layer(middleware::from_fn_with_state(auth, require_session))
}
async fn require_session(State(auth): State<Arc<Auth>>, request: Request, next: Next) -> Response {
    let path = request.uri().path();
    let auth_response = path.starts_with("/api/auth/");
    if !path.starts_with("/api/")
        || (request.method() == axum::http::Method::GET && path == "/api/connectors/oauth/callback")
        || matches!(
            path,
            "/api/auth/login" | "/api/auth/session" | "/api/auth/google"
        )
    {
        let response = next.run(request).await;
        return if auth_response {
            no_cache(response)
        } else {
            response
        };
    }
    let Some(token) = cookie(&request) else {
        return unauthorized();
    };
    let Some(session) = auth.session(&token, now()) else {
        return unauthorized();
    };
    let response = next.run(request).await;
    let response = if auth_response {
        no_cache(response)
    } else {
        response
    };
    if !response
        .headers()
        .get(header::CONTENT_TYPE)
        .is_some_and(|value| {
            value
                .to_str()
                .is_ok_and(|value| value.starts_with("text/event-stream"))
        })
    {
        return response;
    }
    let (parts, body) = response.into_parts();
    let stream = futures_util::stream::unfold(
        (body.into_data_stream(), auth, token, session.expires_at_ms),
        |(mut source, auth, token, expires)| async move {
            loop {
                let clock = now();
                if clock >= expires || auth.session(&token, clock).is_none() {
                    return None;
                }
                tokio::select! {
                    chunk = source.next() => {
                    if auth.session(&token, now()).is_none() { return None; }
                    return chunk.map(|chunk| (chunk, (source, auth, token, expires)));
                },
                    _ = tokio::time::sleep(Duration::from_millis((expires-clock).min(1000))) => {},
                }
            }
        },
    );
    Response::from_parts(parts, Body::from_stream(stream))
}
fn no_cache(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}
fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({"error":"session_expired"})),
    )
        .into_response()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Login {
    email: String,
    password: String,
}
async fn login(State(auth): State<Arc<Auth>>, Json(input): Json<Login>) -> Response {
    if input.email.len() > 254 || input.password.len() > 1024 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let mut attempts = auth.attempts.lock().await;
    if attempts.0.elapsed() >= Duration::from_secs(60) {
        *attempts = (Instant::now(), 0);
    }
    if attempts.1 >= 10 {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    attempts.1 += 1;
    drop(attempts);
    let Ok(_permit) = auth.hashing.try_acquire() else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    let owner = match auth.owner() {
        Ok(Some(owner)) => owner,
        _ => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error":"owner_not_configured"})),
            )
                .into_response();
        }
    };
    let expected = owner.clone();
    let valid = tokio::task::spawn_blocking(move || {
        verify(&input.password, &expected.password_hash)
            && input.email.trim().eq_ignore_ascii_case(&expected.email)
    })
    .await
    .unwrap_or(false);
    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"invalid_credentials"})),
        )
            .into_response();
    }
    let _change = auth.changes.lock().await;
    if auth
        .owner()
        .ok()
        .flatten()
        .is_none_or(|current| current.revision != owner.revision)
    {
        return unauthorized();
    }
    let clock = now();
    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let session = Session {
        revision: owner.revision,
        issued_at_ms: clock,
        expires_at_ms: clock + SESSION_MILLIS,
    };
    if auth
        .store
        .put(SESSION_KIND, &token_id(&token), &session)
        .is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    *auth.attempts.lock().await = (Instant::now(), 0);
    (
        [(header::SET_COOKIE, auth.cookie(&token, SESSION_SECONDS))],
        Json(json!({"authenticated":true})),
    )
        .into_response()
}
async fn session(State(auth): State<Arc<Auth>>, request: Request) -> Response {
    let Some(token) = cookie(&request) else {
        return unauthorized();
    };
    let Some(session) = auth.session(&token, now()) else {
        return unauthorized();
    };
    match auth.owner() { Ok(Some(owner)) => Json(json!({"email":owner.email,"user_id":"owner","installation":"personal","expires_at_ms":session.expires_at_ms,"remaining_ms":session.expires_at_ms.saturating_sub(now())})).into_response(), _ => unauthorized() }
}
async fn logout(State(auth): State<Arc<Auth>>, request: Request) -> Response {
    if let Some(token) = cookie(&request)
        && auth.store.delete(SESSION_KIND, &token_id(&token)).is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    (
        [(header::SET_COOKIE, auth.cookie("", 0))],
        StatusCode::NO_CONTENT,
    )
        .into_response()
}
fn public_profile(owner: Owner) -> Value {
    json!({"email":owner.email,"display_name":owner.display_name,"google_connected":false,"google_sign_in_available":false,"password_available":true})
}
async fn profile(State(auth): State<Arc<Auth>>) -> Response {
    match auth.owner() {
        Ok(Some(owner)) => Json(public_profile(owner)).into_response(),
        _ => unauthorized(),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileInput {
    display_name: String,
}
async fn update_profile(
    State(auth): State<Arc<Auth>>,
    Json(input): Json<ProfileInput>,
) -> Response {
    if input.display_name.chars().count() > 120 || input.display_name.chars().any(char::is_control)
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let _change = auth.changes.lock().await;
    let Ok(Some(mut owner)) = auth.owner() else {
        return unauthorized();
    };
    owner.display_name = input.display_name.trim().into();
    if auth.store.put(OWNER_KIND, "owner", &owner).is_err() {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    Json(public_profile(owner)).into_response()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasswordInput {
    current_password: String,
    new_password: String,
}
async fn password(State(auth): State<Arc<Auth>>, Json(input): Json<PasswordInput>) -> Response {
    if input.current_password.len() > 1024
        || input.new_password.len() < 8
        || input.new_password.len() > 1024
        || input.new_password == input.current_password
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"invalid_password"})),
        )
            .into_response();
    }
    let Ok(_permit) = auth.hashing.try_acquire() else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    let _change = auth.changes.lock().await;
    let Ok(Some(mut owner)) = auth.owner() else {
        return unauthorized();
    };
    let old = owner.password_hash.clone();
    let result = tokio::task::spawn_blocking(move || {
        if !verify(&input.current_password, &old) {
            return Ok(None);
        }
        hash_password(&input.new_password).map(Some)
    })
    .await;
    match result {
        Ok(Ok(Some(hash))) => {
            owner.password_hash = hash;
            owner.revision = uuid::Uuid::new_v4().to_string();
            if auth.store.put(OWNER_KIND, "owner", &owner).is_err() {
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
            (
                [(header::SET_COOKIE, auth.cookie("", 0))],
                Json(json!({"changed":true,"sign_in_required":true})),
            )
                .into_response()
        }
        Ok(Ok(None)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"incorrect_password"})),
        )
            .into_response(),
        _ => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}
/// Local provisioning only. Stop the agent first; no public registration endpoint.
pub fn configure_owner(data_dir: &Path, email: &str, password: &str) -> Result<()> {
    ensure!(
        email.len() <= 254 && email.contains('@') && !email.chars().any(char::is_whitespace),
        "Invalid email"
    );
    ensure!(
        (8..=1024).contains(&password.len()),
        "Password must contain 8–1024 bytes"
    );
    let data_dir = fenix_execution::fs::private::create_private_dir(data_dir)?;
    let _lock = fenix_execution::fs::private::Lock::try_exclusive(&data_dir.join("instance.lock"))?;
    let store = Store::open(&data_dir.join("fenix.sqlite3"))?;
    let previous: Option<Owner> = store.get(OWNER_KIND, "owner")?;
    let owner = Owner {
        email: email.trim().to_lowercase(),
        display_name: previous.map(|owner| owner.display_name).unwrap_or_default(),
        password_hash: hash_password(password)?,
        revision: uuid::Uuid::new_v4().to_string(),
    };
    store.put(OWNER_KIND, "owner", &owner)
}
pub fn configure_from_stdin() -> Result<()> {
    use std::io::Read;
    let mut args = std::env::args().skip(2);
    let email = args
        .next()
        .context("Usage: fenix owner EMAIL --password-stdin [--data-dir PATH]")?;
    let mut directory = None;
    let mut stdin = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--password-stdin" => stdin = true,
            "--data-dir" => {
                directory = Some(PathBuf::from(
                    args.next().context("Missing data directory")?,
                ))
            }
            _ => anyhow::bail!("Unknown owner option"),
        }
    }
    ensure!(
        stdin,
        "Pass --password-stdin; passwords are not accepted in command-line arguments"
    );
    let mut password = String::new();
    std::io::stdin().take(1026).read_to_string(&mut password)?;
    let password = password.trim_end_matches(['\r', '\n']);
    let directory = match directory {
        Some(path) => path,
        None => fenix_execution::fs::layout::default_data_dir()?,
    };
    configure_owner(&directory, &email, password)?;
    println!(
        "Configured the personal owner. Start Fenix and sign in with your email and password."
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request as HttpRequest;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    fn request(path: &str, method: &str, body: Value, cookie: Option<&str>) -> HttpRequest<Body> {
        let mut request = HttpRequest::builder()
            .uri(path)
            .method(method)
            .header("host", "127.0.0.1:4173")
            .header("content-type", "application/json");
        if let Some(cookie) = cookie {
            request = request.header(header::COOKIE, cookie);
        }
        request.body(Body::from(body.to_string())).unwrap()
    }
    async fn setup() -> (tempfile::TempDir, Arc<Runtime>, Router) {
        let directory = tempfile::tempdir().unwrap();
        configure_owner(directory.path(), "owner@example.test", "fixture-password").unwrap();
        let runtime = Runtime::open(directory.path(), Some(&directory.path().join("skills")))
            .await
            .unwrap();
        let app = crate::personal_router(runtime.clone(), 4173, None).unwrap();
        (directory, runtime, app)
    }
    async fn login_cookie(app: &Router) -> String {
        let response = app
            .clone()
            .oneshot(request(
                "/api/auth/login",
                "POST",
                json!({"email":"owner@example.test","password":"fixture-password"}),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
        assert!(
            cookie.contains("Max-Age=604800")
                && cookie.contains("HttpOnly")
                && cookie.contains("SameSite=Strict")
        );
        cookie.split(';').next().unwrap().to_owned()
    }
    #[test]
    fn seven_days_are_exact_and_do_not_slide_or_accept_future_issuance() {
        let session = Session {
            revision: "fixture".into(),
            issued_at_ms: 1000,
            expires_at_ms: 1000 + SESSION_MILLIS,
        };
        assert!(session.valid_at(1000));
        assert!(session.valid_at(session.expires_at_ms - 1));
        assert!(!session.valid_at(session.expires_at_ms));
        assert!(!session.valid_at(999));
        let malformed = Session {
            expires_at_ms: session.expires_at_ms + 1,
            ..session
        };
        assert!(!malformed.valid_at(1000));
    }
    #[tokio::test]
    async fn private_api_requires_login_and_expiration_survives_router_restart() {
        let (_directory, runtime, app) = setup().await;
        let response = app
            .clone()
            .oneshot(request("/api/status", "GET", json!(null), None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let cookie = login_cookie(&app).await;
        let key = token_id(cookie.strip_prefix("fenix_session=").unwrap());
        let original: Session = runtime.store.get(SESSION_KIND, &key).unwrap().unwrap();
        for app in [
            app,
            crate::personal_router(runtime.clone(), 4173, None).unwrap(),
        ] {
            let response = app
                .oneshot(request("/api/status", "GET", json!(null), Some(&cookie)))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let session: Session = runtime.store.get(SESSION_KIND, &key).unwrap().unwrap();
            assert_eq!(session.expires_at_ms, original.expires_at_ms);
        }
        let directory = runtime.data_dir.clone();
        drop(runtime);
        let runtime = Runtime::open(&directory, Some(&directory.join("skills")))
            .await
            .unwrap();
        let restarted = crate::personal_router(runtime.clone(), 4173, None).unwrap();
        assert_eq!(
            restarted
                .oneshot(request("/api/status", "GET", json!(null), Some(&cookie)))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        let expired = Session {
            revision: original.revision,
            issued_at_ms: now() - SESSION_MILLIS - 1,
            expires_at_ms: now() - 1,
        };
        runtime.store.put(SESSION_KIND, &key, &expired).unwrap();
        let app = crate::personal_router(runtime, 4173, None).unwrap();
        assert_eq!(
            app.oneshot(request("/api/status", "GET", json!(null), Some(&cookie)))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    #[tokio::test]
    async fn profile_logout_and_password_change_work_without_account_routing() {
        let (_directory, _runtime, app) = setup().await;
        let first = login_cookie(&app).await;
        let second = login_cookie(&app).await;
        let response = app
            .clone()
            .oneshot(request(
                "/api/auth/profile",
                "PUT",
                json!({"display_name":"Owner"}),
                Some(&first),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["display_name"], "Owner");
        assert!(body.get("password_hash").is_none());
        assert_eq!(
            app.clone()
                .oneshot(request("/api/auth/logout", "POST", json!({}), Some(&first)))
                .await
                .unwrap()
                .status(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            app.clone()
                .oneshot(request("/api/status", "GET", json!(null), Some(&first)))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(app.clone().oneshot(request("/api/auth/password","POST",json!({"current_password":"fixture-password","new_password":"changed-password"}),Some(&second))).await.unwrap().status(),StatusCode::OK);
        assert_eq!(
            app.clone()
                .oneshot(request("/api/status", "GET", json!(null), Some(&second)))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            app.clone()
                .oneshot(request(
                    "/api/auth/login",
                    "POST",
                    json!({"email":"owner@example.test","password":"changed-password"}),
                    None
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            app.oneshot(request("/api/auth/register", "POST", json!({}), None))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
}

#[cfg(test)]
mod stream_tests {
    use super::*;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    #[tokio::test]
    async fn logout_closes_an_already_open_event_stream_without_stopping_the_runtime() {
        let dir = tempfile::tempdir().unwrap();
        configure_owner(dir.path(), "owner@example.test", "fixture-password").unwrap();
        let rt = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
            .await
            .unwrap();
        let app = crate::personal_router(rt.clone(), 4173, None).unwrap();
        let req = |path: &str, method: &str, body: &str, cookie: Option<&str>| {
            let mut request = axum::http::Request::builder()
                .uri(path)
                .method(method)
                .header("host", "127.0.0.1:4173")
                .header("content-type", "application/json");
            if let Some(cookie) = cookie {
                request = request.header(header::COOKIE, cookie)
            }
            request.body(Body::from(body.to_owned())).unwrap()
        };
        let response = app
            .clone()
            .oneshot(req(
                "/api/auth/login",
                "POST",
                r#"{"email":"owner@example.test","password":"fixture-password"}"#,
                None,
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
        let response = app
            .clone()
            .oneshot(req("/api/events", "GET", "", Some(&cookie)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body();
        let read = tokio::spawn(async move { body.collect().await });
        assert_eq!(
            app.oneshot(req("/api/auth/logout", "POST", "{}", Some(&cookie)))
                .await
                .unwrap()
                .status(),
            StatusCode::NO_CONTENT
        );
        tokio::time::timeout(Duration::from_secs(3), read)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(!rt.stop.is_cancelled());
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;
    use tower::ServiceExt;
    #[tokio::test]
    async fn login_obeys_origin_checks_and_no_owner_is_a_closed_installation() {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
            .await
            .unwrap();
        let app = crate::personal_router(rt, 4173, None).unwrap();
        let request = |path: &str, origin: &str| {
            axum::http::Request::builder()
                .uri(path)
                .method("POST")
                .header("host", "127.0.0.1:4173")
                .header(header::ORIGIN, origin)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"owner@example.test","password":"fixture-password"}"#,
                ))
                .unwrap()
        };
        assert_eq!(
            app.clone()
                .oneshot(request("/api/auth/login", "https://other.test"))
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            app.clone()
                .oneshot(request("/api/auth/login", "http://127.0.0.1:4173"))
                .await
                .unwrap()
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        let google = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/auth/google")
                    .header("host", "127.0.0.1:4173")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(google.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            app.oneshot(
                axum::http::Request::builder()
                    .uri("/api/skills")
                    .header("host", "127.0.0.1:4173")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap()
            .status(),
            StatusCode::UNAUTHORIZED
        );
    }
}
