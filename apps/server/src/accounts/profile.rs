//! Account settings stay in the host registry, never in a tenant-supplied path.
use super::*;

fn public_profile(account: &Account) -> serde_json::Value {
    json!({"email":account.email,"display_name":account.display_name,"google_connected":account.google_subject.is_some(),"password_available":account.password_login})
}
fn failure(status: StatusCode, code: &str) -> Response {
    (status, Json(json!({"error":code}))).into_response()
}
pub(super) async fn get(State(g): State<Arc<Gateway>>, request: Request) -> Response {
    match g.account(cookie(&request).map(str::to_owned)).await {
        Some(account) => Json(public_profile(&account)).into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProfileInput {
    display_name: String,
}

// Use the same advisory lock as the provisioning CLI; reload after acquiring it.
fn change(
    path: PathBuf,
    expected: Account,
    name: Option<String>,
    hash: Option<String>,
) -> Result<Account> {
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path.with_extension("admin.lock"))?;
    lock.lock()?;
    let mut registry = RegistrySource::File(path.clone()).load()?;
    let account = registry
        .accounts
        .iter_mut()
        .find(|a| a.id == expected.id)
        .context("Account unavailable")?;
    ensure!(
        !account.disabled
            && account.password_hash == expected.password_hash
            && account.auth_epoch == expected.auth_epoch,
        "Account authentication changed"
    );
    if let Some(name) = name {
        account.display_name = name;
    }
    // A changed password hash already invalidates every session. The execution
    // epoch belongs to controller credentials and must not change here.
    if let Some(hash) = hash {
        account.password_hash = hash;
    }
    let result = account.clone();
    registry::validate(&registry)?;
    private_write(&path, &serde_json::to_vec_pretty(&registry)?)?;
    Ok(result)
}
pub(super) async fn update(
    State(g): State<Arc<Gateway>>,
    headers: axum::http::HeaderMap,
    Json(input): Json<ProfileInput>,
) -> Response {
    let mut request = Request::new(Body::empty());
    *request.headers_mut() = headers;
    let Some(account) = g.account(cookie(&request).map(str::to_owned)).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let name = input.display_name.trim().to_string();
    if name.chars().count() > 80 || name.chars().any(char::is_control) {
        return failure(StatusCode::BAD_REQUEST, "invalid_name");
    }
    let RegistrySource::File(path) = &g.registry else {
        return StatusCode::NOT_IMPLEMENTED.into_response();
    };
    let path = path.clone();
    match tokio::task::spawn_blocking(move || change(path, account, Some(name), None)).await {
        Ok(Ok(account)) => Json(public_profile(&account)).into_response(),
        _ => failure(StatusCode::SERVICE_UNAVAILABLE, "profile_save_failed"),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PasswordInput {
    current_password: String,
    new_password: String,
}
pub(super) async fn password(
    State(g): State<Arc<Gateway>>,
    headers: axum::http::HeaderMap,
    Json(input): Json<PasswordInput>,
) -> Response {
    let mut request = Request::new(Body::empty());
    *request.headers_mut() = headers;
    let Some(account) = g.account(cookie(&request).map(str::to_owned)).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    if input.current_password.len() > 1024
        || !(8..=1024).contains(&input.new_password.len())
        || input.current_password == input.new_password
    {
        return failure(StatusCode::BAD_REQUEST, "invalid_password");
    }
    {
        let mut attempts = g.attempts.lock().await;
        attempts.retain(|_, (_, at)| at.elapsed() < Duration::from_secs(300));
        let entry = attempts
            .entry(format!("password:{}", account.id))
            .or_insert((0, Instant::now()));
        if entry.0 >= 5 {
            return failure(StatusCode::TOO_MANY_REQUESTS, "try_later");
        }
        entry.0 += 1;
    }
    let Ok(_permit) = g.hashing.try_acquire() else {
        return failure(StatusCode::TOO_MANY_REQUESTS, "try_later");
    };
    let RegistrySource::File(path) = &g.registry else {
        return StatusCode::NOT_IMPLEMENTED.into_response();
    };
    let path = path.clone();
    match tokio::task::spawn_blocking(move || -> Result<bool> {
        if !verify(&input.current_password, &account.password_hash) {
            return Ok(false);
        }
        let hash = hash_password(&input.new_password)?;
        change(path, account, None, Some(hash))?;
        Ok(true)
    })
    .await
    {
        Ok(Ok(true)) => (
            [(header::SET_COOKIE, g.cookie("", 0))],
            Json(json!({"changed":true,"sign_in_required":true})),
        )
            .into_response(),
        Ok(Ok(false)) => failure(StatusCode::BAD_REQUEST, "incorrect_password"),
        _ => failure(StatusCode::SERVICE_UNAVAILABLE, "password_save_failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;
    #[tokio::test]
    async fn profile_is_owner_scoped_and_password_change_revokes_sessions_without_stopping_workers()
    -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("accounts.json");
        let owner = Account {
            id: uuid::Uuid::new_v4().to_string(),
            email: "owner@example.test".into(),
            display_name: String::new(),
            google_subject: None,
            signup_available: false,
            password_login: true,
            password_hash: hash_password("old-password")?,
            upstream: "http://127.0.0.1:4500".into(),
            upstream_token: token(),
            runtime_port: 4173,
            disabled: false,
            auth_epoch: 4,
        };
        let other = Account {
            id: uuid::Uuid::new_v4().to_string(),
            email: "other@example.test".into(),
            upstream: "http://127.0.0.1:4501".into(),
            upstream_token: token(),
            ..owner.clone()
        };
        private_write(
            &path,
            &serde_json::to_vec(&Registry {
                accounts: vec![owner.clone(), other.clone()],
            })?,
        )?;
        let mut sessions = Sessions::open(dir.path().join("sessions.json"))?;
        for key in ["first", "second"] {
            sessions.insert(key.into(), Session::new(&owner, now_millis()), now_millis())?;
        }
        sessions.insert(
            "other".into(),
            Session::new(&other, now_millis()),
            now_millis(),
        )?;
        let app = router_from_source(
            RegistrySource::File(path.clone()),
            4173,
            None,
            sessions,
            None,
        );
        let req = |url: &str, method: &str, body: &str, key: &str| {
            Request::builder()
                .uri(url)
                .method(method)
                .header("host", "localhost:4173")
                .header("content-type", "application/json")
                .header("cookie", format!("fenix_session={key}"))
                .body(Body::from(body.to_owned()))
                .unwrap()
        };
        assert_eq!(
            app.clone()
                .oneshot(req("/api/auth/profile", "GET", "", "unknown"))
                .await?
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let response = app
            .clone()
            .oneshot(req(
                "/api/auth/profile",
                "PUT",
                r#"{"display_name":"Cristhian"}"#,
                "first",
            ))
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), 4096).await?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(
            value,
            json!({"email":"owner@example.test","display_name":"Cristhian","google_connected":false,"password_available":true})
        );
        assert!(RegistrySource::File(path.clone()).load()?.accounts[1] == other);
        assert_eq!(
            app.clone()
                .oneshot(req(
                    "/api/auth/password",
                    "POST",
                    r#"{"current_password":"wrong","new_password":"replacement-password"}"#,
                    "first"
                ))
                .await?
                .status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            app.clone()
                .oneshot(req(
                    "/api/auth/password",
                    "POST",
                    r#"{"current_password":"old-password","new_password":"replacement-password"}"#,
                    "first"
                ))
                .await?
                .status(),
            StatusCode::OK
        );
        for key in ["first", "second"] {
            assert_eq!(
                app.clone()
                    .oneshot(req("/api/auth/session", "GET", "", key))
                    .await?
                    .status(),
                StatusCode::UNAUTHORIZED
            );
        }
        assert_eq!(
            app.clone()
                .oneshot(req("/api/auth/session", "GET", "", "other"))
                .await?
                .status(),
            StatusCode::OK
        );
        let saved = RegistrySource::File(path).load()?;
        assert_eq!(saved.accounts[0].auth_epoch, 4);
        assert!(verify(
            "replacement-password",
            &saved.accounts[0].password_hash
        ));
        assert_eq!(
            app.clone()
                .oneshot(req("/api/auth/register", "POST", "{}", "other"))
                .await?
                .status(),
            StatusCode::NOT_FOUND
        );
        Ok(())
    }
}
