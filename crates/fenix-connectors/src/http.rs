use super::*;
use sha2::{Digest, Sha256};

pub(crate) async fn bounded_bytes(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>> {
    ensure!(
        response
            .content_length()
            .is_none_or(|size| size <= limit as u64),
        "Provider response exceeds the artifact limit"
    );
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        ensure!(
            bytes.len() + chunk.len() <= limit,
            "Provider response exceeds the artifact limit"
        );
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn segment(value: &Value, key: &str) -> Result<String> {
    let s = value[key].as_str().context("Resource ID is required")?;
    ensure!(
        !s.is_empty()
            && s.len() < 512
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.!:".contains(&b)),
        "Invalid resource ID"
    );
    Ok(s.into())
}

fn endpoint(provider: &str, service: &str, path: &str) -> Result<reqwest::Url> {
    ensure!(
        path.starts_with('/')
            && !path.starts_with("//")
            && !path.contains(['\\', '?', '#'])
            && !path.split('/').any(|s| s == ".." || s == ".")
            && !path.contains('%'),
        "Use an API-relative path without traversal, query or encoded separators"
    );
    let base = match (provider, service) {
        ("google", "identity") => "https://www.googleapis.com/oauth2/v2",
        ("google", service) => crate::google::base(service)?,
        ("microsoft", _) => "https://graph.microsoft.com/v1.0",
        ("slack", _) => "https://slack.com/api",
        ("github", _) => "https://api.github.com",
        _ => anyhow::bail!("Unknown provider service"),
    };
    Ok(reqwest::Url::parse(&format!("{base}{path}"))?)
}

impl Connectors {
    pub(crate) async fn http_call(
        &self,
        c: Connection,
        operation: &str,
        args: &Value,
        workspace: &Path,
    ) -> Result<Value> {
        let mut service = args["service"].as_str().unwrap_or("drive").to_owned();
        let mut method = "GET".to_owned();
        let mut query = args["query"].clone();
        let mut body = args["body"].clone();
        let mut download = false;
        let mut upload = None;
        let path = match (c.provider.as_str(), operation) {
            ("google", "test") => {
                service = "drive".into();
                query = json!({"fields":"user(displayName,emailAddress)"});
                "/about".into()
            }
            ("microsoft", "test") => "/me".into(),
            ("github", "test") => "/user".into(),
            ("slack", "test") => {
                method = "POST".into();
                "/auth.test".into()
            }
            ("google", "drive_search") => {
                service = "drive".into();
                query = json!({"q":args["q"].as_str().unwrap_or("trashed = false"),"pageSize":args["limit"].as_u64().unwrap_or(25).clamp(1,100),"fields":"nextPageToken,files(id,name,mimeType,modifiedTime,size,webViewLink)","supportsAllDrives":true,"includeItemsFromAllDrives":true});
                if let Some(page) = args.get("page_token") {
                    query["pageToken"] = page.clone();
                }
                "/files".into()
            }
            ("google", "drive_download") => {
                download = true;
                service = "drive".into();
                query = json!({"alt":"media","supportsAllDrives":true});
                format!("/files/{}", segment(args, "file_id")?)
            }
            ("google", "drive_export") => {
                download = true;
                service = "drive".into();
                query = json!({"mimeType":args["mime_type"].as_str().unwrap_or("application/pdf")});
                format!("/files/{}/export", segment(args, "file_id")?)
            }
            ("google", "drive_upload") => {
                service = "drive".into();
                method = "POST".into();
                let relative = args["source"]
                    .as_str()
                    .context("Workspace-relative source required")?;
                let source = crate::database::workspace_path(workspace, relative, false)?;
                let bytes = fenix_execution::fs::read(&source, 25 * 1024 * 1024)?;
                let name = args["name"]
                    .as_str()
                    .or_else(|| source.file_name().and_then(|s| s.to_str()))
                    .context("File name required")?;
                ensure!(!name.is_empty() && name.len() <= 1000, "Invalid file name");
                body = json!({"name":name});
                if args.get("folder_id").is_some() {
                    body["parents"] = json!([segment(args, "folder_id")?]);
                }
                let content_type = args["mime_type"]
                    .as_str()
                    .unwrap_or("application/octet-stream");
                let _ = lettre::message::header::ContentType::parse(content_type)
                    .context("Invalid MIME type")?;
                ensure!(!content_type.contains(['\r', '\n']), "Invalid MIME type");
                let boundary = format!("fenix_{}", uuid::Uuid::new_v4().simple());
                let mut multipart = format!("--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{}\r\n--{boundary}\r\nContent-Type: {content_type}\r\n\r\n", serde_json::to_string(&body)?).into_bytes();
                multipart.extend_from_slice(&bytes);
                multipart.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
                upload = Some((boundary, multipart));
                query = json!({"uploadType":"multipart","supportsAllDrives":true,"fields":"id,name,mimeType,size,webViewLink,parents"});
                "/files".into()
            }
            ("google", "mail_list") => {
                service = "gmail".into();
                query = json!({"q":args["q"].as_str().unwrap_or(""),"maxResults":args["limit"].as_u64().unwrap_or(25).clamp(1,100)});
                if let Some(page) = args.get("page_token") {
                    query["pageToken"] = page.clone();
                }
                "/users/me/messages".into()
            }
            ("google", "mail_read") => {
                service = "gmail".into();
                let format = args["format"].as_str().unwrap_or("full");
                ensure!(
                    ["full", "metadata"].contains(&format),
                    "Use full or metadata Gmail format"
                );
                query = json!({"format":format});
                format!("/users/me/messages/{}", segment(args, "message_id")?)
            }
            ("google", "mail_send") => {
                service = "gmail".into();
                method = "POST".into();
                body = json!({"raw":args["raw"]});
                if let Some(thread) = args.get("thread_id") {
                    body["threadId"] = thread.clone();
                }
                "/users/me/messages/send".into()
            }
            ("google", "calendar_list") => {
                service = "calendar".into();
                query = json!({"maxResults":args["limit"].as_u64().unwrap_or(25).clamp(1,100),"singleEvents":true,"orderBy":"startTime"});
                for (input, output) in [
                    ("time_min", "timeMin"),
                    ("time_max", "timeMax"),
                    ("page_token", "pageToken"),
                ] {
                    if let Some(value) = args.get(input) {
                        query[output] = value.clone();
                    }
                }
                let calendar = args["calendar_id"].as_str().unwrap_or("primary");
                ensure!(
                    !calendar.is_empty()
                        && calendar.len() < 512
                        && calendar
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"-_.@".contains(&b)),
                    "Invalid calendar ID"
                );
                format!("/calendars/{calendar}/events")
            }
            ("microsoft", "files_search") => {
                let q = args["q"].as_str().context("Search text required")?;
                ensure!(
                    q.chars()
                        .all(|ch| ch.is_alphanumeric() || " -_".contains(ch)),
                    "Use letters, digits, spaces, hyphens for file search"
                );
                format!("/me/drive/root/search(q='{q}')")
            }
            ("microsoft", "file_download") => {
                download = true;
                format!("/me/drive/items/{}/content", segment(args, "file_id")?)
            }
            ("microsoft", "mail_list") => "/me/messages".into(),
            ("microsoft", "mail_send") => {
                method = "POST".into();
                body = args["body"].clone();
                "/me/sendMail".into()
            }
            ("microsoft", "calendar_list") => "/me/events".into(),
            ("slack", "channels") => "/conversations.list".into(),
            ("slack", "history") => {
                query = json!({"channel":segment(args,"channel")?,"limit":50});
                "/conversations.history".into()
            }
            ("slack", "send") => {
                method = "POST".into();
                body = json!({"channel":segment(args,"channel")?,"text":args["text"]});
                "/chat.postMessage".into()
            }
            ("github", "repositories") => "/user/repos".into(),
            ("github", "issues" | "create_issue") => {
                if operation == "create_issue" {
                    method = "POST".into();
                    body = args["body"].clone();
                }
                format!(
                    "/repos/{}/{}/issues",
                    segment(args, "owner")?,
                    segment(args, "repo")?
                )
            }
            (_, "request") => {
                method = args["method"].as_str().unwrap_or("GET").to_uppercase();
                args["path"]
                    .as_str()
                    .context("API-relative path required")?
                    .to_owned()
            }
            _ => anyhow::bail!("Unsupported operation for this provider"),
        };
        ensure!(
            ["GET", "POST", "PUT", "PATCH", "DELETE"].contains(&method.as_str()),
            "Unsupported HTTP method"
        );
        for key in ["dry_run", "page_all"] {
            if let Some(value) = args.get(key) {
                ensure!(value.is_boolean(), "{key} must be a boolean");
            }
        }
        let preview = c.provider == "google" && args["dry_run"] == true;
        if c.provider == "google" {
            crate::google::ensure_selected(&c, &service)?;
        }
        let read = if c.provider == "slack" {
            matches!(
                path.as_str(),
                "/auth.test"
                    | "/conversations.list"
                    | "/conversations.history"
                    | "/conversations.info"
                    | "/conversations.replies"
                    | "/users.info"
                    | "/users.list"
                    | "/search.messages"
            )
        } else {
            method == "GET"
        };
        ensure!(
            preview || read || c.allow_write,
            "Enable write access for this connection first"
        );
        let mut url = endpoint(&c.provider, &service, &path)?;
        if upload.is_some() {
            url = reqwest::Url::parse("https://www.googleapis.com/upload/drive/v3/files")?;
        }
        ensure!(
            query.is_null() || query.is_object(),
            "Query must be an object"
        );
        if let Some(query) = query.as_object() {
            for (key, value) in query {
                let values = value
                    .as_array()
                    .map(Vec::as_slice)
                    .unwrap_or(std::slice::from_ref(value));
                for value in values {
                    ensure!(
                        value.is_string() || value.is_number() || value.is_boolean(),
                        "Query values must be scalars or arrays of scalars"
                    );
                    url.query_pairs_mut().append_pair(
                        key,
                        &value
                            .as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| value.to_string()),
                    );
                }
            }
        }
        if preview {
            return Ok(
                json!({"dry_run":true,"executed":false,"method":method,"url":url.as_str(),"body":body,"allow_write":c.allow_write,"note":"Local request preview only; provider schema, scopes and resource permissions have not been validated."}),
            );
        }
        #[cfg(test)]
        if let Some(origin) = &self.test_origin {
            let target = reqwest::Url::parse(origin)?;
            url.set_scheme(target.scheme()).unwrap();
            url.set_host(target.host_str())?;
            url.set_port(target.port()).unwrap();
        }
        let token = self.bearer(&c.id).await?;
        let mut request = self
            .client
            .request(method.parse()?, url)
            .bearer_auth(&token)
            .header("accept", "application/json");
        if c.provider == "github" {
            request = request.header("X-GitHub-Api-Version", "2022-11-28");
        }
        if let Some((boundary, bytes)) = upload {
            request = request
                .header(
                    "content-type",
                    format!("multipart/related; boundary={boundary}"),
                )
                .body(bytes);
        } else if !body.is_null() && method != "GET" {
            request = request.json(&body);
        }
        let mut response = request.send().await.map_err(|_|anyhow::anyhow!("Provider connection failed; the result of a write may be unknown. Inspect before retrying."))?;
        // Graph sends a short-lived, preauthenticated download URL. Never forward the bearer token.
        if download && c.provider == "microsoft" && response.status() == reqwest::StatusCode::FOUND
        {
            let destination = reqwest::Url::parse(
                response
                    .headers()
                    .get("location")
                    .context("Missing download URL")?
                    .to_str()?,
            )?;
            let host = destination.host_str().unwrap_or_default();
            ensure!(
                destination.scheme() == "https"
                    && (host.ends_with(".sharepoint.com")
                        || host.ends_with(".1drv.com")
                        || host.ends_with(".storage.live.com"))
                    && destination.username().is_empty()
                    && destination.password().is_none(),
                "Unexpected download host"
            );
            response = self
                .client
                .get(destination)
                .send()
                .await
                .map_err(|_| anyhow::anyhow!("Download failed"))?;
        }
        ensure!(
            response.status().is_success(),
            "Provider returned HTTP {}; reconnect for 401, check permissions for 403, respect rate limits for 429. Writes are not retried automatically.",
            response.status().as_u16()
        );
        let bytes = bounded_bytes(
            response,
            if download {
                25 * 1024 * 1024
            } else {
                3 * 1024 * 1024
            },
        )
        .await?;
        if download {
            let destination = args["destination"]
                .as_str()
                .context("A workspace-relative destination is required")?;
            let path = crate::database::workspace_path(workspace, destination, true)?;
            fenix_execution::fs::create(&path, &bytes)?;
            return Ok(
                json!({"path":path,"bytes":bytes.len(),"sha256":format!("{:x}",Sha256::digest(&bytes)),"verified":true}),
            );
        }
        if bytes.is_empty() {
            return Ok(json!({"ok":true}));
        }
        let clean = String::from_utf8(bytes)?.replace(&token, "[REDACTED]");
        let value: Value = serde_json::from_str(&clean)
            .context("Provider returned non-JSON content; use a file download operation")?;
        ensure!(
            c.provider != "slack" || value["ok"] == true,
            "Slack rejected the operation; check token scopes and channel membership"
        );
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn api_paths_cannot_redirect_credentials() {
        for path in [
            "//evil.test",
            "/../token",
            "/%2e%2e/token",
            "/x?token=a",
            "/x\\y",
        ] {
            assert!(endpoint("google", "drive", path).is_err());
        }
        assert_eq!(
            endpoint("google", "drive", "/files").unwrap().host_str(),
            Some("www.googleapis.com")
        );
    }
    #[tokio::test]
    async fn providers_use_native_endpoints_check_errors_and_keep_downloads_private() {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{header, method, path},
        };
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("workspace");
        std::fs::create_dir(&workspace).unwrap();
        let mut connectors = Connectors::open(&dir.path().join("state")).unwrap();
        connectors.test_origin = Some(server.uri());
        for (provider, operation, route, response) in [
            (
                "google",
                "drive_search",
                "/drive/v3/files",
                json!({"files":[{"id":"f1","name":"report"}]}),
            ),
            (
                "microsoft",
                "mail_list",
                "/v1.0/me/messages",
                json!({"value":[{"id":"m1"}]}),
            ),
            (
                "slack",
                "channels",
                "/api/conversations.list",
                json!({"ok":true,"channels":[{"id":"C1"}]}),
            ),
            (
                "github",
                "repositories",
                "/user/repos",
                json!([{"name":"repo"}]),
            ),
        ] {
            Mock::given(method("GET"))
                .and(path(route))
                .and(header("authorization", "Bearer fixture-private-key"))
                .respond_with(ResponseTemplate::new(200).set_body_json(response.clone()))
                .expect(1)
                .mount(&server)
                .await;
            let c = connectors
                .save(ConnectionInput {
                    id: None,
                    provider: provider.into(),
                    name: provider.into(),
                    credentials: json!({"access_token":"fixture-private-key"}),
                    settings: json!({}),
                    allow_write: false,
                })
                .unwrap();
            assert_eq!(
                connectors
                    .invoke(c["id"].as_str().unwrap(), operation, &json!({}), &workspace)
                    .await
                    .unwrap(),
                response
            );
            if provider == "google" {
                Mock::given(method("GET"))
                    .and(path("/drive/v3/files/f1/export"))
                    .respond_with(
                        ResponseTemplate::new(200).set_body_bytes(b"%PDF-fixture".to_vec()),
                    )
                    .mount(&server)
                    .await;
                let args = json!({"file_id":"f1","destination":"reports/result.pdf"});
                let downloaded = connectors
                    .invoke(c["id"].as_str().unwrap(), "drive_export", &args, &workspace)
                    .await
                    .unwrap();
                assert_eq!(downloaded["verified"], true);
                assert_eq!(
                    std::fs::read(workspace.join("reports/result.pdf")).unwrap(),
                    b"%PDF-fixture"
                );
                assert!(
                    connectors
                        .invoke(
                            c["id"].as_str().unwrap(),
                            "drive_export",
                            &json!({"file_id":"f1","destination":"../escape.pdf"}),
                            &workspace
                        )
                        .await
                        .is_err()
                );
            }
            if provider == "slack" {
                assert!(
                    connectors
                        .invoke(
                            c["id"].as_str().unwrap(),
                            "request",
                            &json!({"method":"GET","path":"/chat.postMessage"}),
                            &workspace
                        )
                        .await
                        .is_err()
                );
            }
            if provider == "github" {
                assert!(
                    connectors
                        .invoke(
                            c["id"].as_str().unwrap(),
                            "create_issue",
                            &json!({"owner":"owner","repo":"repo","body":{"title":"Test"}}),
                            &workspace
                        )
                        .await
                        .is_err()
                );
            }
        }
        server.verify().await;
    }
}
