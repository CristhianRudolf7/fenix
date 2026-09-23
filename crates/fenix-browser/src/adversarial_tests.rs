//! Opt-in real Chromium matrix. Oracles inspect actual page state, not tool acknowledgments.
use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct Fixture {
    _data: tempfile::TempDir,
    browser: Arc<Browser>,
    url: String,
    server: tokio::task::JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}
impl Fixture {
    async fn new() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                tokio::spawn(async move {
                    let mut buffer = vec![0; 8192];
                    let n = stream.read(&mut buffer).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&buffer[..n]);
                    let path = request.split_whitespace().nth(1).unwrap_or("/");
                    let (status, headers, body) = if path.starts_with("/frame") {
                        (
                            "200 OK",
                            "Content-Type: text/html; charset=utf-8\r\n",
                            include_str!("../tests/fixtures/frame.html")
                                .as_bytes()
                                .to_vec(),
                        )
                    } else if path == "/redirect" {
                        ("302 Found", "Location: /next\r\n", vec![])
                    } else if path == "/download" {
                        (
                            "200 OK",
                            "Content-Type: text/plain\r\nContent-Disposition: attachment; filename=fixture.txt\r\n",
                            b"Fenix download fixture\n".to_vec(),
                        )
                    } else if path == "/react.js" {
                        (
                            "200 OK",
                            "Content-Type: text/javascript\r\n",
                            std::fs::read(
                                Path::new(env!("CARGO_MANIFEST_DIR"))
                                    .join("../../target/browser-fixtures/controlled.js"),
                            )
                            .expect("Build the React fixture first"),
                        )
                    } else if path == "/next" {
                        (
                            "200 OK",
                            "Content-Type: text/html\r\n",
                            b"<!doctype html><title>Next page</title><h1>Next page</h1>".to_vec(),
                        )
                    } else if path == "/responsive" {
                        ("200 OK","Content-Type: text/html\r\n",br#"<!doctype html><meta name="viewport" content="width=device-width,initial-scale=1"><style>body{margin:0}main{display:grid;grid-template-columns:1fr 1fr}button{height:60px}@media(max-width:600px){main{grid-template-columns:1fr}}</style><main><button onclick="this.textContent='Clicked'">Responsive target</button><p>Second column</p></main>"#.to_vec())
                    } else {
                        (
                            "200 OK",
                            "Content-Type: text/html; charset=utf-8\r\n",
                            include_str!("../tests/fixtures/adversarial.html")
                                .replace("PORT", &port.to_string())
                                .into_bytes(),
                        )
                    };
                    let header = format!(
                        "HTTP/1.1 {status}\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(header.as_bytes()).await;
                    let _ = stream.write_all(&body).await;
                });
            }
        });
        let data = tempfile::tempdir().unwrap();
        let browser = Browser::open(data.path()).unwrap();
        Self {
            _data: data,
            browser,
            url: format!("http://127.0.0.1:{port}"),
            server,
        }
    }
    async fn call(&self, name: &str, args: Value) -> Result<Value> {
        self.browser
            .execute(name, &args, false, "adversarial", &CancellationToken::new())
            .await
    }
    async fn oracle(&self, expression: &str) -> Result<Value> {
        let sessions = self.browser.sessions.lock().await;
        let s = &sessions["test-adversarial"];
        let r = s
            .cdp
            .call(
                Some(&s.tabs[&s.active]),
                "Runtime.evaluate",
                json!({"expression":expression,"returnByValue":true,"awaitPromise":true}),
            )
            .await?;
        if r.get("exceptionDetails").is_some() {
            bail!("Fixture oracle failed: {}", r["exceptionDetails"]["text"])
        }
        Ok(r["result"]["value"].clone())
    }
    async fn reset(&self) -> Result<()> {
        let _ = self
            .call("browser_act", json!({"action":"dialog","accept":false}))
            .await;
        self.call("browser_open", json!({"url":format!("{}/",self.url)}))
            .await?;
        self.call("browser_wait", json!({"condition":"ready"}))
            .await?;
        Ok(())
    }
    async fn frame(&self, name: &str) -> Result<String> {
        fn find(v: &Value, name: &str) -> Option<String> {
            if v["frame"]["name"] == name {
                return v["frame"]["id"].as_str().map(str::to_owned);
            }
            v["childFrames"]
                .as_array()?
                .iter()
                .find_map(|v| find(v, name))
        }
        let state = self.call("browser_inspect", json!({})).await?;
        find(&state["frames"]["frameTree"], name).context("Fixture frame was not found")
    }
    async fn step(&self, mut step: Value) -> Result<()> {
        if let Some(script) = step["eval"].as_str() {
            let actual = self.oracle(script).await?;
            if let Some(expected) = step.get("equals") {
                ensure_eq(&actual, expected)?;
            }
            return Ok(());
        }
        if let Some(frame) = step["args"]["frame_name"].as_str().map(str::to_owned) {
            step["args"].as_object_mut().unwrap().remove("frame_name");
            step["args"]["frame"] = json!(self.frame(&frame).await?);
        }
        if let Some(selector) = step["args"]["to_selector"].as_str().map(str::to_owned) {
            step["args"].as_object_mut().unwrap().remove("to_selector");
            self.oracle("document.querySelector('#drag').scrollIntoView({block:'center'});true")
                .await?;
            let point=self.oracle(&format!("(()=>{{let r=document.querySelector({}).getBoundingClientRect();return {{x:r.x+20,y:r.y+20}}}})()",json!(selector))).await?;
            step["args"]["to_x"] = point["x"].clone();
            step["args"]["to_y"] = point["y"].clone();
        }
        if step["args"]["action"] == "upload" {
            let path = self._data.path().join("upload.txt");
            std::fs::write(&path, "fixture upload")?;
            step["args"]["path"] = json!(path);
        }
        let result = self
            .call(
                step["tool"].as_str().unwrap_or("browser_act"),
                step["args"].clone(),
            )
            .await;
        if step["error"] == true {
            if result.is_ok() {
                bail!("Expected rejection, but action reported success")
            };
            return Ok(());
        }
        let value = result?;
        if let Some(expected) = step.get("passed") {
            ensure_eq(&value["passed"], expected)?;
        }
        if let Some(expected) = step["result"].as_object() {
            for (key, expected) in expected {
                ensure_eq(&value[key], expected)?;
            }
        }
        Ok(())
    }
}
fn ensure_eq(actual: &Value, expected: &Value) -> Result<()> {
    if actual != expected {
        bail!("Observed {actual}; expected {expected}")
    }
    Ok(())
}
fn act(action: &str, selector: &str, text: &str) -> Value {
    json!({"args":{"action":action,"selector":selector,"text":text}})
}
fn value(selector: &str, expected: Value) -> Value {
    json!({"eval":format!("document.querySelector({}).value",json!(selector)),"equals":expected})
}
fn cases() -> Vec<(&'static str, Vec<Value>)> {
    vec![
        (
            "month_replace",
            vec![
                act("fill", "#month", "2027-08"),
                value("#month", json!("2027-08")),
            ],
        ),
        (
            "week_replace",
            vec![
                act("fill", "#week", "2027-W20"),
                value("#week", json!("2027-W20")),
            ],
        ),
        (
            "clear_date",
            vec![act("fill", "#date", ""), value("#date", json!(""))],
        ),
        (
            "clear_contenteditable",
            vec![
                act("fill", "#editor", ""),
                json!({"eval":"document.querySelector('#editor').textContent","equals":""}),
            ],
        ),
        (
            "textarea_crlf",
            vec![
                act("fill", "#textarea", "one\r\ntwo"),
                value("#textarea", json!("one\ntwo")),
            ],
        ),
        (
            "prevented_checkbox_rejected",
            vec![
                json!({"args":{"action":"check","selector":"#prevented"},"error":true}),
                json!({"eval":"document.querySelector('#prevented').checked","equals":false}),
            ],
        ),
        (
            "alert_can_be_dismissed",
            vec![
                json!({"args":{"action":"click","selector":"#alert"},"result":{"interrupted_by_dialog":true}}),
                json!({"tool":"browser_observe","args":{},"result":{"dialog":{"type":"alert"}}}),
                json!({"args":{"action":"dialog","accept":true}}),
                act("fill", "#text", "after dialog"),
                value("#text", json!("after dialog")),
            ],
        ),
        (
            "prompt_accepts_text",
            vec![
                act("click", "#prompt", ""),
                json!({"args":{"action":"dialog","accept":true,"text":"fixture answer"}}),
                json!({"eval":"document.querySelector('#result').textContent","equals":"fixture answer"}),
            ],
        ),
        (
            "cross_nested_frame_click",
            vec![
                json!({"args":{"action":"click","frame_name":"cross-nested","selector":"#frame-button"}}),
                json!({"eval":"new Promise(r=>setTimeout(()=>r(frameMessages.some(m=>m.frame==='?cross-nested'&&m.clicked==='clicked')),80))","equals":true}),
            ],
        ),
        (
            "scaled_frame_click",
            vec![
                json!({"eval":"document.querySelector('#outer').style.transform='scale(.7)';true"}),
                json!({"args":{"action":"click","frame_name":"outer","selector":"#frame-button"}}),
                json!({"eval":"document.querySelector('#outer').contentDocument.querySelector('output').textContent","equals":"clicked"}),
            ],
        ),
        (
            "covered_frame_rejected",
            vec![
                json!({"eval":"(()=>{let e=document.createElement('div');e.id='frame-cover';e.style='position:fixed;inset:0;z-index:999999;background:gray';document.body.append(e);return true})()"}),
                json!({"args":{"action":"click","frame_name":"outer","selector":"#frame-button"},"error":true}),
                json!({"eval":"document.querySelector('#outer').contentDocument.querySelector('output').textContent","equals":""}),
            ],
        ),
        (
            "react_controlled_form",
            vec![
                json!({"tool":"browser_wait","args":{"condition":"visible","selector":"#react-input"}}),
                act("fill", "#react-input", "React ✓"),
                json!({"args":{"action":"select","selector":"#react-select","value":"b"}}),
                act("check", "#react-checkbox", ""),
                act("click", "#react-submit", ""),
                json!({"eval":"JSON.parse(document.querySelector('#react-saved').textContent)","equals":{"text":"React ✓","choice":"b","checked":true}}),
            ],
        ),
        (
            "unicode_replace",
            vec![
                act("fill", "#text", "Árbol 👩🏽‍💻 漢字 e\u{301}"),
                value("#text", json!("Árbol 👩🏽‍💻 漢字 e\u{301}")),
            ],
        ),
        (
            "clear_text",
            vec![act("fill", "#text", ""), value("#text", json!(""))],
        ),
        (
            "email_replace",
            vec![
                act("fill", "#email", "new@example.test"),
                value("#email", json!("new@example.test")),
            ],
        ),
        (
            "number_replace",
            vec![
                act("fill", "#number", "45.5"),
                value("#number", json!("45.5")),
            ],
        ),
        (
            "date_replace",
            vec![
                act("fill", "#date", "2027-12-31"),
                value("#date", json!("2027-12-31")),
            ],
        ),
        (
            "time_replace",
            vec![
                act("fill", "#time", "09:15"),
                value("#time", json!("09:15")),
            ],
        ),
        (
            "datetime_replace",
            vec![
                act("fill", "#datetime", "2027-12-31T09:15"),
                value("#datetime", json!("2027-12-31T09:15")),
            ],
        ),
        (
            "color_replace",
            vec![
                act("fill", "#color", "#ff00aa"),
                value("#color", json!("#ff00aa")),
            ],
        ),
        (
            "invalid_date_rejected",
            vec![
                json!({"args":{"action":"fill","selector":"#date","text":"not a date"},"error":true}),
            ],
        ),
        (
            "range_text_rejected",
            vec![json!({"args":{"action":"fill","selector":"#range","text":"50"},"error":true})],
        ),
        (
            "readonly_rejected",
            vec![
                json!({"args":{"action":"fill","selector":"#readonly","text":"bad"},"error":true}),
                value("#readonly", json!("unchanged")),
            ],
        ),
        (
            "disabled_fieldset_rejected",
            vec![
                json!({"args":{"action":"fill","selector":"#disabled","text":"bad"},"error":true}),
                value("#disabled", json!("unchanged")),
            ],
        ),
        (
            "inert_rejected",
            vec![json!({"args":{"action":"fill","selector":"#inert","text":"bad"},"error":true})],
        ),
        (
            "noneditable_rejected",
            vec![
                json!({"args":{"action":"fill","selector":"#not-editor","text":"bad"},"error":true}),
            ],
        ),
        (
            "maxlength_not_silent",
            vec![
                json!({"args":{"action":"fill","selector":"#limited","text":"toolong"},"error":true}),
            ],
        ),
        (
            "textarea_multiline",
            vec![
                act("fill", "#textarea", "line 1\nline 2\n🌍"),
                value("#textarea", json!("line 1\nline 2\n🌍")),
            ],
        ),
        (
            "contenteditable_replace",
            vec![
                act("fill", "#editor", "new text"),
                json!({"eval":"document.querySelector('#editor').innerText","equals":"new text"}),
            ],
        ),
        (
            "type_at_caret",
            vec![
                act("fill", "#text", "ab"),
                json!({"args":{"action":"key","key":"ArrowLeft"}}),
                act("type", "#text", "X"),
                value("#text", json!("aXb")),
            ],
        ),
        (
            "key_targets_requested_input",
            vec![
                act("fill", "#text", "abc"),
                json!({"args":{"action":"key","selector":"#email","key":"Control+A"}}),
                json!({"args":{"action":"key","key":"Backspace"}}),
                value("#text", json!("abc")),
                value("#email", json!("")),
            ],
        ),
        (
            "enter_in_textarea",
            vec![
                act("fill", "#textarea", "one"),
                json!({"args":{"action":"key","key":"Enter"}}),
                act("type", "#textarea", "two"),
                value("#textarea", json!("one\ntwo")),
            ],
        ),
        (
            "literal_plus_key",
            vec![
                act("fill", "#text", ""),
                json!({"args":{"action":"key","key":"+"}}),
                value("#text", json!("+")),
            ],
        ),
        (
            "unicode_key",
            vec![
                act("fill", "#text", ""),
                json!({"args":{"action":"key","key":"ñ"}}),
                value("#text", json!("ñ")),
            ],
        ),
        (
            "focus_replacement_not_silent",
            vec![
                json!({"args":{"action":"fill","selector":"#replace","text":"lost"},"error":true}),
            ],
        ),
        (
            "checkbox_idempotent",
            vec![
                act("check", "#checkbox", ""),
                act("check", "#checkbox", ""),
                json!({"eval":"document.querySelector('#checkbox').checked","equals":true}),
                act("uncheck", "#checkbox", ""),
                json!({"eval":"document.querySelector('#checkbox').checked","equals":false}),
            ],
        ),
        (
            "radio_uncheck_rejected",
            vec![json!({"args":{"action":"uncheck","selector":"#radio"},"error":true})],
        ),
        (
            "select_enabled",
            vec![
                json!({"args":{"action":"select","selector":"#select","value":"b"}}),
                value("#select", json!("b")),
            ],
        ),
        (
            "disabled_option_rejected",
            vec![
                json!({"args":{"action":"select","selector":"#select","value":"disabled"},"error":true}),
                value("#select", json!("a")),
            ],
        ),
        (
            "disabled_optgroup_rejected",
            vec![
                json!({"args":{"action":"select","selector":"#select","value":"group-disabled"},"error":true}),
            ],
        ),
        (
            "hidden_file_upload",
            vec![
                json!({"args":{"action":"upload","selector":"#upload"}}),
                json!({"eval":"document.querySelector('#file-result').textContent","equals":"upload.txt:14"}),
            ],
        ),
        (
            "duplicate_selector_rejected",
            vec![json!({"args":{"action":"click","selector":".duplicate"},"error":true})],
        ),
        (
            "covered_click_rejected",
            vec![json!({"args":{"action":"click","selector":"#fully-covered"},"error":true})],
        ),
        (
            "partially_covered_click",
            vec![
                act("click", "#partially-covered", ""),
                json!({"eval":"counts.partial","equals":1}),
            ],
        ),
        (
            "clipped_large_click",
            vec![
                act("click", "#large", ""),
                json!({"eval":"counts.large","equals":1}),
            ],
        ),
        (
            "double_click",
            vec![
                act("double_click", "#double", ""),
                json!({"eval":"counts.double","equals":1}),
            ],
        ),
        (
            "hover",
            vec![
                act("hover", "#hover", ""),
                json!({"eval":"getComputedStyle(document.querySelector('#hover span')).display","equals":"inline"}),
            ],
        ),
        (
            "html_drag",
            vec![
                json!({"args":{"action":"drag","selector":"#drag","to_selector":"#drop"}}),
                json!({"eval":"counts.drop","equals":1}),
            ],
        ),
        (
            "shadow_fill",
            vec![
                act("fill", "#shadow-input", "Shadow ✓"),
                json!({"eval":"document.querySelector('#shadow').shadowRoot.querySelector('input').value","equals":"Shadow ✓"}),
            ],
        ),
        (
            "shadow_wait",
            vec![
                json!({"tool":"browser_wait","args":{"condition":"visible","selector":"#shadow-input","timeout_ms":250},"passed":true}),
            ],
        ),
        (
            "scoped_text_condition",
            vec![
                json!({"tool":"browser_check","args":{"condition":"text","selector":"#scope","text":"success"},"passed":false}),
            ],
        ),
        (
            "delayed_input",
            vec![
                act("click", "#delayed", ""),
                json!({"tool":"browser_wait","args":{"condition":"visible","selector":"#late","timeout_ms":2000},"passed":true}),
                act("fill", "#late", "ready"),
                value("#late", json!("ready")),
            ],
        ),
        (
            "nested_frame_click",
            vec![
                json!({"args":{"action":"click","frame_name":"nested","selector":"#frame-button"}}),
                json!({"eval":"document.querySelector('#outer').contentDocument.querySelector('iframe').contentDocument.querySelector('output').textContent","equals":"clicked"}),
            ],
        ),
        (
            "nested_frame_fill",
            vec![
                json!({"args":{"action":"fill","frame_name":"nested","selector":"#frame-input","text":"nested text"}}),
                json!({"eval":"document.querySelector('#outer').contentDocument.querySelector('iframe').contentDocument.querySelector('input').value","equals":"nested text"}),
            ],
        ),
        (
            "cross_site_frame_fill",
            vec![
                json!({"args":{"action":"fill","frame_name":"cross","selector":"#frame-input","text":"cross text"}}),
                json!({"eval":"new Promise(r=>setTimeout(()=>r(frameMessages.some(m=>m.frame==='?cross'&&m.value==='cross text')),80))","equals":true}),
            ],
        ),
        (
            "cross_site_frame_click",
            vec![
                json!({"args":{"action":"click","frame_name":"cross","selector":"#frame-button"}}),
                json!({"eval":"new Promise(r=>setTimeout(()=>r(frameMessages.some(m=>m.frame==='?cross'&&m.clicked==='clicked')),80))","equals":true}),
            ],
        ),
        (
            "agent_password_blocked",
            vec![
                json!({"args":{"action":"fill","selector":"#password","text":"bad"},"error":true}),
                value("#password", json!("fixture-secret")),
            ],
        ),
        (
            "navigation_back_forward",
            vec![
                act("click", "#next", ""),
                json!({"tool":"browser_wait","args":{"condition":"text","text":"Next page"}}),
                json!({"args":{"action":"back"}}),
                json!({"tool":"browser_wait","args":{"condition":"visible","selector":"#next"}}),
                json!({"args":{"action":"forward"}}),
                json!({"tool":"browser_wait","args":{"condition":"text","text":"Next page"}}),
            ],
        ),
        (
            "spa_back",
            vec![
                act("click", "#spa", ""),
                json!({"tool":"browser_check","args":{"condition":"url","text":"/route"},"passed":true}),
                json!({"args":{"action":"back"}}),
                json!({"eval":"location.pathname","equals":"/"}),
            ],
        ),
    ]
}
#[tokio::test]
#[ignore = "Requires sandbox-capable Chromium; controlled local fixtures only"]
async fn adversarial_browser_matrix() {
    let f = Fixture::new().await;
    let mut report = vec![];
    f.reset()
        .await
        .expect("Fixture must load before the matrix");
    {
        let sessions = f.browser.sessions.lock().await;
        let s = &sessions["test-adversarial"];
        let frames = s
            .cdp
            .call(Some(&s.tabs[&s.active]), "Page.getFrameTree", json!({}))
            .await
            .unwrap();
        let targets = s
            .cdp
            .call(None, "Target.getTargets", json!({}))
            .await
            .unwrap();
        std::fs::write(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/browser-frame-diagnostics.json"),
            json!({"frames":frames,"targets":targets}).to_string(),
        )
        .unwrap();
    }
    for (name, steps) in cases() {
        if std::env::var("FENIX_BROWSER_CASE").is_ok_and(|filter| !name.contains(&filter)) {
            continue;
        }
        let started = Instant::now();
        let result: Result<()> = async {
            f.reset().await?;
            for (i, step) in steps.into_iter().enumerate() {
                f.step(step)
                    .await
                    .with_context(|| format!("Step {}", i + 1))?;
            }
            Ok(())
        }
        .await;
        let error = result.err().map(|e| format!("{e:#}"));
        eprintln!(
            "{} {name}: {}",
            if error.is_some() { "FAIL" } else { "PASS" },
            error.as_deref().unwrap_or("verified")
        );
        report.push(json!({"case":name,"passed":error.is_none(),"error":error,"elapsed_ms":started.elapsed().as_millis()}));
    }
    f.browser.shutdown().await;
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/browser-adversarial-report.json");
    std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    let failed = report.iter().filter(|c| c["passed"] == false).count();
    assert_eq!(
        failed,
        0,
        "{failed} of {} browser scenarios failed; see target/browser-adversarial-report.json",
        report.len()
    );
}

#[tokio::test]
#[ignore = "Requires sandbox-capable Chromium and the built React fixture"]
async fn browser_navigation_resources_and_responsive_layout() {
    let f = Fixture::new().await;
    f.reset().await.unwrap();
    let observed = f.call("browser_observe", json!({})).await.unwrap();
    let elements = observed["elements"].as_array().unwrap();
    assert!(
        elements
            .iter()
            .any(|e| e["disabled"] == true && e["tag"] == "input")
    );
    assert!(elements.iter().any(|e| e["readonly"] == true));
    assert!(!observed.to_string().contains("fixture-secret"));
    let reference = elements.iter().find(|e| e["name"] == "Count").unwrap()["ref"].clone();
    f.call("browser_act", json!({"action":"click","ref":reference}))
        .await
        .unwrap();
    assert_eq!(f.oracle("counts.click").await.unwrap(), 1);
    f.reset().await.unwrap();
    assert!(
        f.call("browser_act", json!({"action":"click","ref":reference}))
            .await
            .is_err()
    );

    f.call("browser_open", json!({"url":format!("{}/redirect",f.url)}))
        .await
        .unwrap();
    f.call(
        "browser_wait",
        json!({"condition":"text","text":"Next page"}),
    )
    .await
    .unwrap();
    assert_eq!(f.oracle("location.pathname").await.unwrap(), "/next");
    f.reset().await.unwrap();
    f.call(
        "browser_act",
        json!({"action":"click","selector":"#download"}),
    )
    .await
    .unwrap();
    let download = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let files = f.browser.downloads("test-adversarial").unwrap();
            if let Some(file) = files["files"].as_array().unwrap().first() {
                break file.clone();
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .unwrap();
    f.call(
        "browser_wait",
        json!({"condition":"download","download_id":download["id"]}),
    )
    .await
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(
            f.browser
                .download_path("test-adversarial", download["id"].as_str().unwrap())
                .unwrap()
        )
        .unwrap(),
        "Fenix download fixture\n"
    );

    let original = f.call("browser_tabs", json!({})).await.unwrap()["active"].clone();
    f.call("browser_act", json!({"action":"click","selector":"#popup"}))
        .await
        .unwrap();
    let tabs = f.call("browser_tabs", json!({})).await.unwrap();
    assert_eq!(tabs["tabs"].as_array().unwrap().len(), 2);
    let popup = tabs["tabs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["tab"] != original)
        .unwrap()["tab"]
        .clone();
    f.call("browser_tabs", json!({"action":"select","tab":popup}))
        .await
        .unwrap();
    f.call(
        "browser_wait",
        json!({"condition":"text","text":"Next page"}),
    )
    .await
    .unwrap();
    let other_frame = f.frame("outer").await;
    assert!(
        other_frame.is_err(),
        "Frames from another tab must not appear in the active tab"
    );
    f.call("browser_close", json!({"tab":popup})).await.unwrap();
    assert_eq!(
        f.call("browser_tabs", json!({})).await.unwrap()["tabs"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    f.call(
        "browser_open",
        json!({"url":format!("{}/responsive",f.url)}),
    )
    .await
    .unwrap();
    f.call("browser_wait", json!({"condition":"ready"}))
        .await
        .unwrap();
    for (width, height, dpr, mobile) in [
        (390, 844, 3, true),
        (768, 1024, 2, false),
        (1280, 800, 1, false),
        (1920, 1080, 1, false),
    ] {
        f.call(
            "browser_resize",
            json!({"width":width,"height":height,"dpr":dpr,"mobile":mobile,"touch":mobile}),
        )
        .await
        .unwrap();
        assert_eq!(
            f.call("browser_check", json!({"condition":"no_overflow"}))
                .await
                .unwrap()["passed"],
            true
        );
        let actual=f.oracle("({width:innerWidth,height:innerHeight,dpr:devicePixelRatio,touch:navigator.maxTouchPoints>0,columns:getComputedStyle(document.querySelector('main')).gridTemplateColumns.split(' ').length})").await.unwrap();
        assert_eq!(
            actual,
            json!({"width":width,"height":height,"dpr":dpr,"touch":mobile,"columns":if width<=600 {1}else{2}})
        );
        let capture = f
            .call("browser_inspect", json!({"action":"capture"}))
            .await
            .unwrap();
        let png = f
            .browser
            .artifact("test-adversarial", capture["artifact"].as_str().unwrap())
            .unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(
            u32::from_be_bytes(png[16..20].try_into().unwrap()),
            (width * dpr) as u32
        );
        assert_eq!(
            u32::from_be_bytes(png[20..24].try_into().unwrap()),
            (height * dpr) as u32
        );
        f.call("browser_act", json!({"action":"click","selector":"button"}))
            .await
            .unwrap();
        assert_eq!(
            f.oracle("document.querySelector('button').textContent")
                .await
                .unwrap(),
            "Clicked"
        );
    }
    assert!(
        f.call("browser_resize", json!({"width":0,"height":0}))
            .await
            .is_err()
    );
    f.call(
        "browser_resize",
        json!({"width":390,"height":844,"dpr":3,"mobile":true,"touch":true}),
    )
    .await
    .unwrap();
    let new_tab = f
        .call(
            "browser_tabs",
            json!({"action":"new","url":format!("{}/responsive",f.url)}),
        )
        .await
        .unwrap()["active"]
        .clone();
    f.call("browser_wait", json!({"condition":"ready"}))
        .await
        .unwrap();
    assert_eq!(
        f.oracle("({width:innerWidth,dpr:devicePixelRatio,touch:navigator.maxTouchPoints>0})")
            .await
            .unwrap(),
        json!({"width":390,"dpr":3,"touch":true})
    );
    f.call("browser_close", json!({"tab":new_tab}))
        .await
        .unwrap();

    // A cancellation already requested must never close the live profile.
    let cancelled = CancellationToken::new();
    cancelled.cancel();
    for _ in 0..20 {
        assert!(
            f.browser
                .execute(
                    "browser_close",
                    &json!({}),
                    false,
                    "adversarial",
                    &cancelled
                )
                .await
                .is_err()
        );
        assert_eq!(
            f.browser.status().await.unwrap()["sessions"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
    let cancelled = CancellationToken::new();
    let stop = cancelled.clone();
    let timer = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;
        stop.cancel();
    });
    let start = Instant::now();
    assert!(
        f.browser
            .execute(
                "browser_wait",
                &json!({"condition":"visible","selector":"#never-created","timeout_ms":30000}),
                false,
                "adversarial",
                &cancelled
            )
            .await
            .is_err()
    );
    assert!(start.elapsed() < Duration::from_secs(2));
    timer.await.unwrap();
    assert!(f.call("browser_observe", json!({})).await.is_ok());
    eprintln!(
        "Resource workflow RSS: {} MiB",
        f.browser.status().await.unwrap()["sessions"][0]["rss_sum_mib"]
    );
    f.browser.shutdown().await;
    assert!(
        f.browser.status().await.unwrap()["sessions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(!f.browser.directory("test-adversarial").exists());
}

#[tokio::test]
#[ignore = "Requires Chromium and internet access to Selenium's public test form"]
async fn public_selenium_form_smoke() {
    let f = Fixture::new().await;
    f.call(
        "browser_open",
        json!({"url":"https://www.selenium.dev/selenium/web/web-form.html"}),
    )
    .await
    .unwrap();
    f.call(
        "browser_wait",
        json!({"condition":"visible","selector":"#my-text-id","timeout_ms":15000}),
    )
    .await
    .unwrap();
    f.call(
        "browser_act",
        json!({"action":"fill","selector":"#my-text-id","text":"Fenix public test ✓"}),
    )
    .await
    .unwrap();
    f.call(
        "browser_act",
        json!({"action":"fill","selector":"textarea","text":"First line\nSecond line"}),
    )
    .await
    .unwrap();
    f.call(
        "browser_act",
        json!({"action":"select","selector":"select","value":"2"}),
    )
    .await
    .unwrap();
    f.call(
        "browser_act",
        json!({"action":"fill","selector":"input[list]","text":"Seattle"}),
    )
    .await
    .unwrap();
    f.call(
        "browser_act",
        json!({"action":"check","selector":"#my-check-2"}),
    )
    .await
    .unwrap();
    assert!(
        f.call(
            "browser_act",
            json!({"action":"fill","selector":"input[readonly]","text":"rejected"})
        )
        .await
        .is_err()
    );
    f.call(
        "browser_act",
        json!({"action":"click","selector":"button[type=submit]"}),
    )
    .await
    .unwrap();
    f.call(
        "browser_wait",
        json!({"condition":"text","text":"Received!","timeout_ms":15000}),
    )
    .await
    .unwrap();
    assert_eq!(
        f.oracle("Object.fromEntries(new URLSearchParams(location.search))['my-text']")
            .await
            .unwrap(),
        "Fenix public test ✓"
    );
    assert_eq!(
        f.oracle("Object.fromEntries(new URLSearchParams(location.search))['my-textarea']")
            .await
            .unwrap(),
        "First line\r\nSecond line"
    );
    assert_eq!(
        f.oracle("Object.fromEntries(new URLSearchParams(location.search))['my-select']")
            .await
            .unwrap(),
        "2"
    );
    f.browser.shutdown().await;
}

#[tokio::test]
#[ignore = "Requires sandbox-capable Chromium"]
async fn download_results_survive_event_eviction_and_cancellation_is_terminal() {
    let f = Fixture::new().await;
    f.reset().await.unwrap();
    f.call(
        "browser_act",
        json!({"action":"click","selector":"#download"}),
    )
    .await
    .unwrap();
    let id = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let state = f.call("browser_inspect", json!({})).await.unwrap();
            if let Some(id) = state["downloads"]["transfers"][0]["id"].as_str() {
                break id.to_owned();
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .unwrap();
    let wait = json!({"condition":"download","download_id":id});
    assert_eq!(
        f.call("browser_wait", wait.clone()).await.unwrap()["passed"],
        true
    );
    let canceled = uuid::Uuid::new_v4().to_string();
    {
        let sessions = f.browser.sessions.lock().await;
        let session = sessions.get("test-adversarial").unwrap();
        session.cdp.events.lock().unwrap().clear();
        session.cdp.downloads.lock().unwrap().observe(&json!({"method":"Browser.downloadProgress","params":{"guid":canceled,"state":"canceled","receivedBytes":20}}));
    }
    let result = f.call("browser_wait", wait.clone()).await.unwrap();
    assert_eq!(result["passed"], true);
    assert!(result["download"]["file"]["url"].is_string());
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        f.call(
            "browser_wait",
            json!({"condition":"download","download_id":canceled,"timeout_ms":30000}),
        ),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(result["terminal"], true);
    assert_eq!(result["passed"], false);
    assert_eq!(result["download"]["state"], "canceled");
    f.call("browser_close", json!({})).await.unwrap();
    f.reset().await.unwrap();
    assert_eq!(
        f.call("browser_wait", wait).await.unwrap()["passed"],
        true,
        "Completed file evidence must survive a browser restart"
    );
    f.browser.shutdown().await;
}

#[tokio::test]
#[ignore = "Requires sandbox-capable Chromium"]
async fn owner_vault_fills_the_explicit_tab_without_touching_the_active_tab() {
    let f = Fixture::new().await;
    f.browser.profile("owner", false).await.unwrap();
    let run = |name: &'static str, mut args: Value| {
        let browser = f.browser.clone();
        async move {
            args["profile"] = json!("owner");
            browser
                .execute(name, &args, true, "owner", &CancellationToken::new())
                .await
                .unwrap()
        }
    };
    run("browser_open", json!({"url":f.url})).await;
    run("browser_wait", json!({"condition":"ready"})).await;
    let first = run("browser_tabs", json!({})).await["active"]
        .as_str()
        .unwrap()
        .to_owned();
    let second = run("browser_tabs", json!({"action":"new","url":f.url})).await["active"]
        .as_str()
        .unwrap()
        .to_owned();
    run("browser_wait", json!({"condition":"ready"})).await;
    f.browser.credentials(json!({"profile":"owner","tab":first,"action":"save","master":"synthetic-master-password","password":"saved-owner-secret"})).await.unwrap();
    f.browser.credentials(json!({"profile":"owner","tab":first,"action":"fill","master":"synthetic-master-password"})).await.unwrap();
    {
        let sessions = f.browser.sessions.lock().await;
        let session = &sessions["owner"];
        assert_eq!(session.active, second);
        for (tab, expected) in [(&first, true), (&second, false)] {
            let result = session.cdp.call(Some(&session.tabs[tab]),"Runtime.evaluate",json!({"expression":"document.querySelector('input[type=password]').value === 'saved-owner-secret'","returnByValue":true})).await.unwrap();
            assert_eq!(result["result"]["value"], expected);
        }
    }
    assert!(f.browser.credentials(json!({"profile":"owner","tab":"closed-tab","action":"fill","master":"synthetic-master-password"})).await.is_err());
    f.browser.shutdown().await;
}
