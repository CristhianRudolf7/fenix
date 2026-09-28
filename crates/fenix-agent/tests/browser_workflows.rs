//! Real Chromium and agent dispatch; model decisions and university data are fixtures.
use fenix_agent::{Runtime, config::Settings};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wiremock::{
    Mock, MockServer, Request, ResponseTemplate,
    matchers::{body_string_contains, header, method, path, query_param},
};

const PROFILE: &str = "university-fixture";
const COOKIE: &str = "university_session=fixture-authenticated";

fn answer(text: &str) -> Value {
    json!({"id":"response","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":text}]}],"usage":{"input_tokens":12,"output_tokens":8,"total_tokens":20}})
}

fn call(id: &str, name: &str, mut args: Value) -> Value {
    if name.starts_with("browser_") {
        args["profile"] = json!(PROFILE);
    }
    json!({"id":"response","status":"completed","output":[{"type":"function_call","call_id":id,"name":name,"arguments":args.to_string()}],"usage":{"input_tokens":12,"output_tokens":8,"total_tokens":20}})
}

async fn finished(runtime: &Runtime, task: &str) -> Vec<Value> {
    tokio::time::timeout(Duration::from_secs(180), async {
        loop {
            let state = runtime.store.task(task).unwrap();
            assert_ne!(state.status, "failed", "Task failed: {state:?}");
            if state.status == "completed" {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("Agent workflow did not complete");
    let mut events = Vec::new();
    loop {
        let after = events
            .last()
            .and_then(|e: &Value| e["seq"].as_i64())
            .unwrap_or(0);
        let page = runtime.store.events(task, after, 100).unwrap();
        if page.is_empty() {
            break;
        }
        events.extend(page);
    }
    events
}

fn outcome<'a>(events: &'a [Value], id: &str) -> &'a Value {
    &events
        .iter()
        .find(|e| e["kind"] == "tool_result" && e["data"]["call_id"] == id)
        .unwrap_or_else(|| panic!("Missing tool outcome {id}"))["data"]
}

struct StreamingFile {
    url: String,
    bytes: Vec<u8>,
    requests: Arc<Mutex<usize>>,
    server: tokio::task::JoinHandle<()>,
}

impl Drop for StreamingFile {
    fn drop(&mut self) {
        self.server.abort();
    }
}

impl StreamingFile {
    async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/report.csv", listener.local_addr().unwrap());
        let chunk = b"course,grade\nsystems,18\n".repeat(1024);
        let bytes = chunk.repeat(100);
        let size = bytes.len();
        let requests = Arc::new(Mutex::new(0));
        let received = requests.clone();
        let server = tokio::spawn(async move {
            let mut jobs = tokio::task::JoinSet::new();
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let chunk = chunk.clone();
                let received = received.clone();
                jobs.spawn(async move {
                    let mut request = Vec::new();
                    while !request.ends_with(b"\r\n\r\n") {
                        let mut byte = [0];
                        if stream.read(&mut byte).await.unwrap_or(0) == 0 { return; }
                        request.push(byte[0]);
                        assert!(request.len() < 16384);
                    }
                    let request = String::from_utf8(request).unwrap();
                    assert!(request.starts_with("GET /report.csv "));
                    assert!(request.contains(COOKIE), "Download must use the authenticated session");
                    *received.lock().unwrap() += 1;
                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/csv\r\nContent-Disposition: attachment; filename=report.csv\r\nContent-Length: {size}\r\nConnection: close\r\n\r\n");
                    stream.write_all(response.as_bytes()).await.unwrap();
                    // One download takes 99 seconds, exceeding a browser_wait call.
                    for i in 0..100 {
                        if i > 0 { tokio::time::sleep(Duration::from_secs(1)).await; }
                        stream.write_all(&chunk).await.unwrap();
                    }
                    stream.shutdown().await.unwrap();
                });
            }
        });
        Self {
            url,
            bytes,
            requests,
            server,
        }
    }
}

#[tokio::test]
#[ignore = "Requires sandbox-capable Chromium; runs a 99-second authenticated download"]
async fn agent_authenticated_multistep_workflow_survives_wait_timeouts() {
    let started = Instant::now();
    let website = MockServer::start().await;
    let download = StreamingFile::start().await;
    Mock::given(method("GET")).and(path("/login"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"<!doctype html><h1>University login</h1><form method="post" action="/login"><input id="username" name="username" autocomplete="username"><input id="password" name="password" type="password"><button id="signin">Sign in</button></form>"#,
            "text/html")).mount(&website).await;
    Mock::given(method("POST"))
        .and(path("/login"))
        .and(body_string_contains("username=student"))
        .and(body_string_contains("password=fixture-password"))
        .respond_with(
            ResponseTemplate::new(303)
                .insert_header("Location", "/dashboard")
                .insert_header(
                    "Set-Cookie",
                    format!("{COOKIE}; HttpOnly; SameSite=Lax; Path=/; Max-Age=3600"),
                ),
        )
        .expect(1)
        .mount(&website)
        .await;
    Mock::given(method("GET")).and(path("/dashboard")).and(header("cookie", COOKIE))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            "<!doctype html><h1>Welcome student</h1><a id='catalog' href='/catalog'>Course catalog</a>", "text/html"))
        .mount(&website).await;
    Mock::given(method("GET")).and(path("/catalog")).and(header("cookie", COOKIE))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"<!doctype html><h1>Course catalog</h1><input id="query"><button id="search" onclick="fetch('/api/catalog?query='+encodeURIComponent(document.querySelector('#query').value)).then(r=>r.json()).then(v=>{document.querySelector('#results').textContent=v.result;document.querySelector('#first-course').hidden=false})">Search</button><p id="results"></p><a id="first-course" href="/course/1" hidden>First course</a>"#,
            "text/html")).mount(&website).await;
    Mock::given(method("GET"))
        .and(path("/api/catalog"))
        .and(header("cookie", COOKIE))
        .and(query_param("query", "sistemas 2026 ñ"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(2))
                .set_body_json(json!({"result":"Found 8 courses for sistemas 2026 ñ"})),
        )
        .expect(1)
        .mount(&website)
        .await;
    for course in 1..=8 {
        Mock::given(method("GET")).and(path(format!("/course/{course}"))).and(header("cookie", COOKIE))
            .respond_with(ResponseTemplate::new(200).set_body_raw(format!(
                "<!doctype html><h1>Course {course}</h1><p>Required credits: {course}</p><a id='next' href='/course/{}'>Next course</a><a id='report' href='{}'>Download report</a>", course + 1, download.url), "text/html"))
            .expect(1).mount(&website).await;
    }

    let dir = tempfile::tempdir().unwrap();
    let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let model = MockServer::start().await;
    let steps = Arc::new(Mutex::new(VecDeque::<Value>::new()));
    let script = steps.clone();
    let seen_download = Arc::new(Mutex::new(None::<String>));
    let observed_download = seen_download.clone();
    let download_link = Arc::new(Mutex::new(None::<String>));
    let observed_link = download_link.clone();
    let visible_download_url = regex::Regex::new(&format!(
        r"/api/browser/downloads/{PROFILE}/[0-9a-f-]{{36}}"
    ))
    .unwrap();
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(move |request: &Request| {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            // The real model can read a URL in a bounded preview or summary
            // transcript as well as in a structured function output. Only use
            // an exact URL actually present in a model request, never derive it
            // from a GUID or read it from the runtime's private journal.
            if let Some(found) = visible_download_url.find(&body.to_string()) {
                *observed_link.lock().unwrap() = Some(found.as_str().to_owned());
            }
            // Health checks and context summaries have no tools. They must not
            // consume an action from the scripted interactive workflow.
            if body.get("tools").is_none() {
                return ResponseTemplate::new(200).set_body_json(answer(
                    "Continue the authorized university workflow. Preserve completed course reads, use saved tool evidence, and download the report once. The agent has signed in with the authorized test credentials.",
                ));
            }
            let mut guid = observed_download.lock().unwrap();
            for input in body["input"].as_array().into_iter().flatten() {
                if input["type"] == "function_call_output"
                    && let Some(output) = input["output"].as_str()
                    && let Ok(value) = serde_json::from_str::<Value>(output)
                {
                    for event in value["events"].as_array().into_iter().flatten() {
                        if event["event"] == "Browser.downloadWillBegin" {
                            *guid = event["guid"].as_str().map(str::to_owned);
                        }
                    }
                    for file in value["downloads"]["files"].as_array().into_iter().flatten() {
                        *observed_link.lock().unwrap() = file["url"].as_str().map(str::to_owned);
                    }
                }
            }
            let mut response = script
                .lock()
                .unwrap()
                .pop_front()
                .expect("Unexpected model request");
            if response["output"][0]["type"] == "function_call" {
                let mut args: Value =
                    serde_json::from_str(response["output"][0]["arguments"].as_str().unwrap())
                        .unwrap();
                if args["download_id"] == "$download" {
                    args["download_id"] = json!(
                        guid.as_ref()
                            .expect("Agent did not receive the download ID")
                    );
                    response["output"][0]["arguments"] = json!(args.to_string());
                }
            } else if response["output"][0]["content"][0]["text"] == "$report" {
                response = answer(&format!(
                    "Verified 8 courses. [Download report]({})",
                    observed_link.lock().unwrap().as_ref().expect("The browser must provide the completed file's download URL")
                ));
            }
            ResponseTemplate::new(200).set_body_json(response)
        })
        .mount(&model)
        .await;
    let mut settings = Settings::initial(dir.path().to_owned());
    settings.base_url = Some(model.uri());
    runtime
        .configure(settings, Some("fixture-model-key".into()))
        .await
        .unwrap();
    let workers = runtime.start();
    let tools = [
        "browser_open",
        "browser_observe",
        "browser_act",
        "browser_wait",
        "browser_inspect",
        "browser_close",
        "browser_tabs",
    ];

    steps.lock().unwrap().extend([
        call("load-login", "tool_search", json!({"names":tools})),
        call(
            "login-page",
            "browser_open",
            json!({"url":format!("{}/login", website.uri())}),
        ),
        call(
            "login-ready",
            "browser_wait",
            json!({"condition":"visible","selector":"#password"}),
        ),
        call(
            "agent-username",
            "browser_act",
            json!({"action":"fill","selector":"#username","text":"student"}),
        ),
        call(
            "agent-password",
            "browser_act",
            json!({"action":"fill","selector":"#password","text":"fixture-password"}),
        ),
        call(
            "agent-signin",
            "browser_act",
            json!({"action":"click","selector":"#signin"}),
        ),
        call(
            "agent-signed-in",
            "browser_wait",
            json!({"condition":"text","text":"Welcome student"}),
        ),
        call("agent-login-close", "browser_close", json!({})),
        answer("Signed in and preserved the personal profile for future use."),
    ]);
    let first = runtime
        .submit(
            "Open the university and sign in using the synthetic test credentials.",
            "web",
        )
        .await
        .unwrap();
    let events = finished(&runtime, first["task_id"].as_str().unwrap()).await;
    assert_eq!(outcome(&events, "agent-password")["is_error"], false);
    assert_eq!(outcome(&events, "agent-signin")["is_error"], false);
    assert_eq!(
        outcome(&events, "agent-signed-in")["result"]["passed"],
        true
    );
    eprintln!("Agent completed login without owner handoff; checking the persisted session");
    let mut workflow = vec![
        call("load-workflow", "tool_search", json!({"names":tools})),
        call("profiles", "browser_tabs", json!({"action":"profiles"})),
        call(
            "restore-login",
            "browser_open",
            json!({"url":format!("{}/dashboard", website.uri())}),
        ),
        call(
            "dashboard-ready",
            "browser_wait",
            json!({"condition":"text","text":"Welcome student"}),
        ),
        call("dashboard-read", "browser_observe", json!({})),
        call(
            "catalog",
            "browser_act",
            json!({"action":"click","selector":"#catalog"}),
        ),
        call(
            "catalog-ready",
            "browser_wait",
            json!({"condition":"visible","selector":"#query"}),
        ),
        call(
            "search-fill",
            "browser_act",
            json!({"action":"fill","selector":"#query","text":"sistemas 2026 ñ"}),
        ),
        call(
            "search-submit",
            "browser_act",
            json!({"action":"click","selector":"#search"}),
        ),
        call(
            "search-ready",
            "browser_wait",
            json!({"condition":"text","text":"Found 8 courses"}),
        ),
        call("search-read", "browser_observe", json!({})),
    ];
    for course in 1..=8 {
        workflow.extend([
            call(
                &format!("course-{course}"),
                "browser_act",
                json!({"action":"click","selector":if course == 1 {"#first-course"} else {"#next"}}),
            ),
            call(
                &format!("course-{course}-ready"),
                "browser_wait",
                json!({"condition":"text","text":format!("Course {course}")}),
            ),
            call(
                &format!("course-{course}-read"),
                "browser_observe",
                json!({}),
            ),
        ]);
    }
    workflow.extend([
        call(
            "download-click",
            "browser_act",
            json!({"action":"click","selector":"#report"}),
        ),
        call("download-start", "browser_inspect", json!({})),
        call(
            "download-short-wait",
            "browser_wait",
            json!({"condition":"download","download_id":"$download","timeout_ms":1000}),
        ),
        call("download-progress", "browser_inspect", json!({})),
        call(
            "download-full-wait",
            "browser_wait",
            json!({"condition":"download","download_id":"$download","timeout_ms":30000}),
        ),
        call("download-progress-again", "browser_inspect", json!({})),
        call(
            "download-full-wait-2",
            "browser_wait",
            json!({"condition":"download","download_id":"$download","timeout_ms":30000}),
        ),
        call(
            "download-full-wait-3",
            "browser_wait",
            json!({"condition":"download","download_id":"$download","timeout_ms":30000}),
        ),
        call(
            "download-complete",
            "browser_wait",
            json!({"condition":"download","download_id":"$download","timeout_ms":30000}),
        ),
        call("download-inspect-complete", "browser_inspect", json!({})),
        call("close-profile", "browser_close", json!({})),
        answer("$report"),
    ]);
    steps.lock().unwrap().extend(workflow);
    let task = runtime.submit("Use the authorized university profile, search sistemas 2026 ñ, read all eight courses and download the report. Wait for completion without downloading it twice.", "web").await.unwrap();
    let task_id = task["task_id"].as_str().unwrap();
    let events = finished(&runtime, task_id).await;
    assert!(!events.iter().any(|e| e["kind"] == "loop_detected"));
    // The production-sized context need not compact this short fixture.
    // Dedicated context tests cover compaction at explicit test thresholds.
    let errors: Vec<_> = events
        .iter()
        .filter(|e| e["kind"] == "tool_result" && e["data"]["is_error"] == true)
        .map(|e| e["data"]["call_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        errors,
        Vec::<&str>::new(),
        "Unexpected tool errors: {}",
        json!(
            events
                .iter()
                .filter(|e| e["kind"] == "tool_result" && e["data"]["is_error"] == true)
                .map(|e| &e["data"])
                .collect::<Vec<_>>()
        )
    );
    let mut received = 0;
    for call in [
        "download-short-wait",
        "download-full-wait",
        "download-full-wait-2",
        "download-full-wait-3",
    ] {
        let result = &outcome(&events, call)["result"];
        assert_eq!(result["passed"], false);
        assert_eq!(result["timed_out"], true);
        let bytes = result["download"]["received_bytes"].as_u64().unwrap();
        assert!(
            bytes > received,
            "A slow download must expose real progress"
        );
        received = bytes;
    }
    assert_eq!(
        outcome(&events, "download-complete")["result"]["passed"],
        true
    );
    assert_eq!(
        outcome(&events, "close-profile")["result"]["profile_preserved"],
        true
    );
    assert!(
        outcome(&events, "search-read")["result"]["text"]
            .as_str()
            .unwrap()
            .contains("Found 8 courses")
    );
    for course in 1..=8 {
        assert!(
            outcome(&events, &format!("course-{course}-read"))["result"]["text"]
                .as_str()
                .unwrap()
                .contains(&format!("Required credits: {course}"))
        );
    }
    let guid = seen_download.lock().unwrap().clone().unwrap();
    let path = runtime.browser.download_path(PROFILE, &guid).unwrap();
    assert_eq!(std::fs::read(path).unwrap(), download.bytes);
    assert_eq!(*download.requests.lock().unwrap(), 1);
    assert_eq!(
        runtime.browser.downloads(PROFILE).unwrap()["files"][0]["bytes"],
        download.bytes.len()
    );
    assert_eq!(
        outcome(&events, "download-inspect-complete")["result"]["downloads"]["files"][0]["bytes"],
        download.bytes.len()
    );
    assert_eq!(
        download_link.lock().unwrap().as_deref(),
        Some(format!("/api/browser/downloads/{PROFILE}/{guid}").as_str())
    );
    let history = serde_json::to_string(&runtime.store.context(0).unwrap()).unwrap();
    // The agent now supplies the credential in its own tool call. Browser
    // observations must still omit input values and session cookies.
    for event in &events {
        if event["kind"] == "tool_result" {
            assert!(
                !event["data"]["result"]
                    .to_string()
                    .contains("fixture-password")
            );
        }
    }
    assert!(!history.contains(COOKIE));
    assert!(history.contains(&format!("/api/browser/downloads/{PROFILE}/{guid}")));
    assert!(steps.lock().unwrap().is_empty());
    website.verify().await;
    runtime.shutdown().await.unwrap();
    for worker in workers {
        worker.await.unwrap();
    }
    let calls = events
        .iter()
        .filter(|e| e["kind"] == "tool_started")
        .count();
    eprintln!(
        "Authenticated agent workflow: {calls} tool calls, {} bytes verified, one download request, {:.2}s total; model decisions were scripted",
        download.bytes.len(),
        started.elapsed().as_secs_f64()
    );
}
