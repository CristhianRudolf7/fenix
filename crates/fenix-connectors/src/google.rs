//! Fénix-owned Workspace metadata, bounded reads and per-service diagnostics.
use super::*;
use std::collections::{BTreeSet, HashSet};
use std::time::{Duration, Instant};

struct Service {
    id: &'static str,
    label: &'static str,
    version: &'static str,
    base: &'static str,
    discovery: &'static str,
    read: &'static str,
    write: &'static str,
}
const SERVICES: &[Service] = &[
    Service {
        id: "docs",
        label: "Docs",
        version: "v1",
        base: "https://docs.googleapis.com/v1",
        discovery: "https://docs.googleapis.com/$discovery/rest?version=v1",
        read: "documents.readonly",
        write: "documents",
    },
    Service {
        id: "sheets",
        label: "Sheets",
        version: "v4",
        base: "https://sheets.googleapis.com/v4",
        discovery: "https://sheets.googleapis.com/$discovery/rest?version=v4",
        read: "spreadsheets.readonly",
        write: "spreadsheets",
    },
    Service {
        id: "slides",
        label: "Slides",
        version: "v1",
        base: "https://slides.googleapis.com/v1",
        discovery: "https://slides.googleapis.com/$discovery/rest?version=v1",
        read: "presentations.readonly",
        write: "presentations",
    },
    Service {
        id: "gmail",
        label: "Gmail",
        version: "v1",
        base: "https://gmail.googleapis.com/gmail/v1",
        discovery: "https://gmail.googleapis.com/$discovery/rest?version=v1",
        read: "gmail.readonly",
        write: "gmail.modify",
    },
    Service {
        id: "calendar",
        label: "Calendar",
        version: "v3",
        base: "https://www.googleapis.com/calendar/v3",
        discovery: "https://www.googleapis.com/discovery/v1/apis/calendar/v3/rest",
        read: "calendar.readonly",
        write: "calendar.events calendar.calendarlist.readonly calendar.settings.readonly",
    },
    Service {
        id: "tasks",
        label: "Tasks",
        version: "v1",
        base: "https://tasks.googleapis.com/tasks/v1",
        discovery: "https://tasks.googleapis.com/$discovery/rest?version=v1",
        read: "tasks.readonly",
        write: "tasks",
    },
    Service {
        id: "people",
        label: "Contacts",
        version: "v1",
        base: "https://people.googleapis.com/v1",
        discovery: "https://people.googleapis.com/$discovery/rest?version=v1",
        read: "contacts.readonly",
        write: "contacts",
    },
    Service {
        id: "forms",
        label: "Forms",
        version: "v1",
        base: "https://forms.googleapis.com/v1",
        discovery: "https://forms.googleapis.com/$discovery/rest?version=v1",
        read: "forms.body.readonly forms.responses.readonly",
        write: "forms.body forms.responses.readonly",
    },
    Service {
        id: "chat",
        label: "Chat",
        version: "v1",
        base: "https://chat.googleapis.com/v1",
        discovery: "https://chat.googleapis.com/$discovery/rest?version=v1",
        read: "chat.spaces.readonly chat.messages.readonly",
        write: "chat.spaces.readonly chat.messages",
    },
    Service {
        id: "meet",
        label: "Meet",
        version: "v2",
        base: "https://meet.googleapis.com/v2",
        discovery: "https://meet.googleapis.com/$discovery/rest?version=v2",
        read: "meetings.space.readonly",
        write: "meetings.space.readonly meetings.space.created",
    },
];
fn service(id: &str) -> Result<&'static Service> {
    SERVICES
        .iter()
        .find(|s| s.id == id)
        .context("Unsupported Google service")
}
pub(crate) fn base(id: &str) -> Result<&'static str> {
    Ok(service(id)?.base)
}
pub(crate) fn service_options() -> Value {
    json!(
        SERVICES
            .iter()
            .map(|s| json!({"id":s.id,"name":s.label}))
            .collect::<Vec<_>>()
    )
}
pub(crate) fn selected(settings: &Value) -> Result<Vec<String>> {
    // Old vaults retain their original service surface until explicitly reconfigured.
    let Some(services) = settings.get("services") else {
        return Ok(["docs", "sheets", "slides", "gmail", "calendar"]
            .map(str::to_owned)
            .to_vec());
    };
    let values = services
        .as_array()
        .context("Google services must be an array")?;
    ensure!(
        !values.is_empty() && values.len() <= SERVICES.len(),
        "Select at least one supported Google service"
    );
    let mut result = Vec::new();
    for value in values {
        let id = value
            .as_str()
            .context("Google service IDs must be strings")?;
        if id == "drive" {
            continue; // Ignore the retired service in previously saved accounts.
        }
        service(id)?;
        ensure!(!result.iter().any(|s| s == id), "Duplicate Google service");
        result.push(id.to_owned());
    }
    ensure!(
        !result.is_empty(),
        "Select a supported Google service; Drive has been removed"
    );
    Ok(result)
}
pub(crate) fn validate_settings(settings: &Value) -> Result<()> {
    ensure!(
        !settings["services"]
            .as_array()
            .is_some_and(|services| services.iter().any(|service| service == "drive")),
        "Google Drive is not supported"
    );
    selected(settings)?;
    if let Some(scopes) = settings.get("scopes") {
        let scopes = scopes
            .as_str()
            .context("Google scopes must be a space-separated string")?;
        ensure!(
            !scopes
                .split_whitespace()
                .any(|scope| scope.starts_with("https://www.googleapis.com/auth/drive")),
            "Google Drive is not supported"
        );
        ensure!(
            scopes.len() <= 8192
                && scopes
                    .split_whitespace()
                    .all(|s| s.starts_with("https://www.googleapis.com/auth/")
                        || s == "https://mail.google.com/"
                        || s == "openid"),
            "Use Google OAuth scope URLs"
        );
    }
    Ok(())
}
pub(crate) fn scopes(c: &Connection) -> Result<String> {
    if let Some(custom) = c.settings["scopes"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
    {
        ensure!(
            !custom
                .split_whitespace()
                .any(|scope| scope.starts_with("https://www.googleapis.com/auth/drive")),
            "Remove obsolete Drive scopes before reconnecting"
        );
        return Ok(custom.to_owned());
    }
    let mut scopes = BTreeSet::from(["https://www.googleapis.com/auth/userinfo.email".to_owned()]);
    for id in selected(&c.settings)? {
        let s = service(&id)?;
        for scope in (if c.allow_write { s.write } else { s.read }).split_whitespace() {
            scopes.insert(format!("https://www.googleapis.com/auth/{scope}"));
        }
    }
    Ok(scopes.into_iter().collect::<Vec<_>>().join(" "))
}
pub(crate) fn ensure_selected(c: &Connection, id: &str) -> Result<()> {
    ensure!(
        id == "identity" || selected(&c.settings)?.iter().any(|s| s == id),
        "Select this Google service in Integrations and authorize its permissions first"
    );
    Ok(())
}
fn collect_methods(node: &Value, output: &mut Vec<Value>) {
    if let Some(methods) = node["methods"].as_object() {
        for method in methods.values() {
            output.push(json!({"id":method["id"],"http_method":method["httpMethod"],"path":method["path"],"description":method["description"]}));
        }
    }
    if let Some(resources) = node["resources"].as_object() {
        for child in resources.values() {
            collect_methods(child, output);
        }
    }
}
fn find_method<'a>(node: &'a Value, id: &str) -> Option<&'a Value> {
    if let Some(methods) = node["methods"].as_object() {
        if let Some(method) = methods.values().find(|m| m["id"] == id) {
            return Some(method);
        }
    }
    node["resources"]
        .as_object()?
        .values()
        .find_map(|n| find_method(n, id))
}
fn refs(value: &Value, names: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(name) = map.get("$ref").and_then(Value::as_str) {
                names.insert(name.into());
            }
            for child in map.values() {
                refs(child, names);
            }
        }
        Value::Array(values) => {
            for child in values {
                refs(child, names);
            }
        }
        _ => {}
    }
}
impl Connectors {
    pub(crate) async fn google_schema(&self, args: &Value) -> Result<Value> {
        let s = service(
            args["service"]
                .as_str()
                .context("Google service required")?,
        )?;
        let cached = self
            .discovery_cache
            .lock()
            .map_err(|_| anyhow::anyhow!("Discovery cache unavailable"))?
            .get(s.id)
            .filter(|(time, _)| time.elapsed() < Duration::from_secs(21600))
            .map(|(_, value)| value.clone());
        let document = if let Some(doc) = cached {
            doc
        } else {
            #[allow(unused_mut)]
            let mut url = reqwest::Url::parse(s.discovery)?;
            #[cfg(test)]
            if let Some(origin) = &self.test_origin {
                let target = reqwest::Url::parse(origin)?;
                url.set_scheme(target.scheme()).unwrap();
                url.set_host(target.host_str())?;
                url.set_port(target.port()).unwrap();
            }
            let response = self
                .client
                .get(url)
                .send()
                .await
                .context("Google Discovery unavailable")?;
            ensure!(
                response.status().is_success(),
                "Google Discovery returned HTTP {}",
                response.status().as_u16()
            );
            let doc: Value = serde_json::from_slice(
                &crate::http::bounded_bytes(response, 4 * 1024 * 1024).await?,
            )?;
            ensure!(
                doc["name"] == s.id && doc["version"] == s.version,
                "Unexpected Discovery service/version"
            );
            self.discovery_cache
                .lock()
                .map_err(|_| anyhow::anyhow!("Discovery cache unavailable"))?
                .insert(s.id.into(), (Instant::now(), doc.clone()));
            doc
        };
        let mut output = json!({"service":s.id,"source":s.discovery,"revision":document["revision"],"api_base":s.base,"data_is_untrusted":true,"note":"Schemas describe APIs, not account permissions. request paths are relative to api_base; remove a version prefix already present in api_base."});
        if let Some(name) = args["schema_name"].as_str() {
            output["schema"] = document["schemas"]
                .get(name)
                .context("Unknown schema name")?
                .clone();
        } else if let Some(id) = args["method_id"].as_str() {
            let method = find_method(&document, id).context("Unknown Google method ID")?;
            output["method"] = method.clone();
            output["common_parameters"] = document["parameters"].clone();
            let mut schemas = serde_json::Map::new();
            for key in ["request", "response"] {
                if let Some(name) = method[key]["$ref"].as_str() {
                    if let Some(value) = document["schemas"].get(name) {
                        schemas.insert(name.into(), value.clone());
                    }
                }
            }
            output["schemas"] = Value::Object(schemas);
        } else {
            let mut methods = Vec::new();
            collect_methods(&document, &mut methods);
            methods.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
            let offset = args["offset"].as_u64().unwrap_or(0) as usize;
            output["total"] = json!(methods.len());
            output["methods"] = json!(methods.iter().skip(offset).take(50).collect::<Vec<_>>());
            output["next_offset"] = if offset.saturating_add(50) < methods.len() {
                json!(offset + 50)
            } else {
                Value::Null
            };
        }
        let mut names = BTreeSet::new();
        refs(&output, &mut names);
        output["referenced_schemas"] = json!(names);
        ensure!(
            serde_json::to_vec(&output)?.len() <= 256 * 1024,
            "Schema result too large; request a specific schema_name"
        );
        Ok(output)
    }

    pub(crate) async fn google_check(&self, id: &str, workspace: &Path) -> Result<Value> {
        let c = self.connection(id)?;
        let mut checks = serde_json::Map::new();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
        for name in selected(&c.settings)? {
            let probe = match name.as_str() {
                "gmail" => Some(("/users/me/profile", json!({}))),
                "calendar" => Some(("/users/me/calendarList", json!({"maxResults":1}))),
                "tasks" => Some(("/users/@me/lists", json!({"maxResults":1}))),
                "people" => Some((
                    "/people/me/connections",
                    json!({"personFields":"names","pageSize":1}),
                )),
                "chat" => Some(("/spaces", json!({"pageSize":1}))),
                "meet" => Some(("/conferenceRecords", json!({"pageSize":1}))),
                _ => None,
            };
            let result = if let Some((path, query)) = probe {
                let args = json!({"service":name,"method":"GET","path":path,"query":query});
                match tokio::time::timeout_at(
                    deadline,
                    self.http_call(c.clone(), "request", &args, workspace),
                )
                .await
                {
                    Ok(Ok(_)) => {
                        json!({"status":"passed","detail":"Read probe passed; write access and other resources remain untested."})
                    }
                    Ok(Err(error)) => json!({"status":"failed","detail":error.to_string()}),
                    Err(_) => json!({"status":"failed","detail":"Connection check timed out."}),
                }
            } else {
                json!({"status":"not_checked","detail":"Requires a document, spreadsheet, presentation or form ID. Verify a user-selected resource with a read request."})
            };
            checks.insert(name, result);
        }
        // Reload so a token refreshed by a probe is not overwritten by the old copy.
        let mut latest = self.connection(id)?;
        ensure!(
            latest.settings == c.settings && latest.allow_write == c.allow_write,
            "Connection changed during checks; check it again"
        );
        latest.state = if checks.values().any(|v| v["status"] == "failed") {
            "needs_attention"
        } else if !checks.values().any(|v| v["status"] == "passed") {
            "unverified"
        } else if checks.values().any(|v| v["status"] == "not_checked") {
            "partially_checked"
        } else {
            "connected"
        }
        .into();
        latest.checked_at = Some(chrono::Utc::now().to_rfc3339());
        latest.service_checks = Value::Object(checks);
        self.update(&latest)?;
        Ok(latest.view())
    }

    pub(crate) async fn google_pages(
        &self,
        c: Connection,
        operation: &str,
        args: &Value,
        workspace: &Path,
    ) -> Result<Value> {
        ensure!(
            matches!(operation, "request" | "mail_list" | "calendar_list"),
            "Pagination is only available for Google list reads"
        );
        ensure!(
            operation != "request"
                || args["method"]
                    .as_str()
                    .unwrap_or("GET")
                    .eq_ignore_ascii_case("GET"),
            "Pagination never repeats writes"
        );
        let limit = args
            .get("page_limit")
            .map(|v| v.as_u64().context("page_limit must be an integer"))
            .transpose()?
            .unwrap_or(5);
        ensure!(
            (1..=10).contains(&limit),
            "page_limit must be between 1 and 10"
        );
        if let Some(fields) = args["query"]["fields"].as_str() {
            ensure!(
                fields.contains("nextPageToken") || fields == "*",
                "Include nextPageToken in paginated field projections"
            );
        }
        let mut input = args.clone();
        input["page_all"] = json!(false);
        let mut pages = Vec::new();
        let mut seen = HashSet::new();
        let mut bytes = 0;
        let mut next = args["query"]["pageToken"]
            .as_str()
            .or(args["page_token"].as_str())
            .map(str::to_owned);
        if let Some(token) = &next {
            seen.insert(token.clone());
        }
        let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
        let mut stop = "complete";
        for _ in 0..limit {
            let response = tokio::time::timeout_at(
                deadline,
                self.http_call(c.clone(), operation, &input, workspace),
            )
            .await;
            let page = match response {
                Ok(Ok(page)) => page,
                Ok(Err(error)) if pages.is_empty() => return Err(error),
                Ok(Err(_)) => {
                    stop = "provider_error";
                    break;
                }
                Err(_) => {
                    stop = "time_limit";
                    break;
                }
            };
            let page_bytes = serde_json::to_vec(&page)?.len();
            if bytes + page_bytes > 3 * 1024 * 1024 {
                stop = "byte_limit";
                break;
            }
            bytes += page_bytes;
            next = page["nextPageToken"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_owned);
            pages.push(page);
            let Some(token) = &next else {
                break;
            };
            if !seen.insert(token.clone()) {
                stop = "repeated_page_token";
                break;
            }
            if operation == "request" {
                if input["query"].is_null() {
                    input["query"] = json!({});
                }
                input["query"]["pageToken"] = json!(token);
            } else {
                input["page_token"] = json!(token);
            }
            stop = "page_limit";
        }
        let complete = next.is_none() && matches!(stop, "complete" | "page_limit");
        Ok(
            json!({"pages":pages,"complete":complete,"stop_reason":if complete { "complete" } else { stop },"next_page_token":next,"bytes":bytes.min(3 * 1024 * 1024)}),
        )
    }
}

mod mail;
pub(crate) use mail::mail_inputs;
#[cfg(test)]
mod tests;
