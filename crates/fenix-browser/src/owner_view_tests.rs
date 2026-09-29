use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn owner_view_does_not_select_unknown_or_reserved_profiles() {
    let dir = tempfile::tempdir().unwrap();
    let browser = Browser::open(dir.path()).unwrap();
    for name in [
        "../other-user",
        "missing",
        "test-other-task",
        "integration-whatsapp",
    ] {
        assert!(browser.open_view(Some(name)).await.is_err());
    }
    assert!(
        browser.status().await.unwrap()["profiles"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
#[ignore = "Requires sandbox-capable local Chromium"]
async fn real_owner_view_reuses_tabs_and_login_without_sharing_accounts() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}/?page=login#form", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut buffer = [0; 8192];
                let n = stream.read(&mut buffer).await.unwrap_or(0);
                let signed_in =
                    String::from_utf8_lossy(&buffer[..n]).contains("owner_session=saved");
                let body = if signed_in {
                    "<!doctype html><h1>Welcome back</h1>"
                } else {
                    r#"<!doctype html><input id="password" type="password"><button id="login" onclick="if(document.querySelector('#password').value==='synthetic-secret'){document.cookie='owner_session=saved; Max-Age=86400; SameSite=Lax';localStorage.setItem('session','saved');document.querySelector('h1').textContent='Signed in'}">Sign in</button><h1>Sign in first</h1><output id="frame-result"></output><iframe style="position:absolute;left:0;top:100px;border:0" srcdoc="<input oninput='parent.document.querySelector(&quot;#frame-result&quot;).textContent=this.value'>"></iframe>"#
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    });
    let dir = tempfile::tempdir().unwrap();
    let browser = Browser::open(dir.path()).unwrap();
    let view = browser.open_view(None).await.unwrap();
    assert_eq!(view["profile"], "personal");
    assert_eq!(view["tabs"][0]["url"], "about:blank");
    let cancel = CancellationToken::new();
    let run = |tool: &'static str, mut args: Value| {
        let browser = browser.clone();
        let cancel = cancel.clone();
        async move {
            args["profile"] = json!("personal");
            browser
                .execute(tool, &args, true, "owner", &cancel)
                .await
                .unwrap()
        }
    };
    run("browser_open", json!({"url":address})).await;
    run("browser_wait", json!({"condition":"ready"})).await;
    run("browser_act", json!({"action":"click","x":50,"y":118})).await;
    run(
        "browser_act",
        json!({"action":"type","text":"Frame input works"}),
    )
    .await;
    assert_eq!(
        run(
            "browser_check",
            json!({"condition":"text","text":"Frame input works"})
        )
        .await["passed"],
        true
    );
    run(
        "browser_act",
        json!({"action":"click","selector":"#password"}),
    )
    .await;
    run(
        "browser_act",
        json!({"action":"type","text":"synthetic-secret"}),
    )
    .await;
    run("browser_act", json!({"action":"click","selector":"#login"})).await;
    assert_eq!(
        run(
            "browser_check",
            json!({"condition":"text","text":"Signed in"})
        )
        .await["passed"],
        true
    );
    browser.profile("personal", true).await.unwrap();
    let reopened = browser.open_view(None).await.unwrap();
    assert_eq!(view["active"], reopened["active"]);
    assert_eq!(reopened["tabs"].as_array().unwrap().len(), 1);
    assert_eq!(reopened["tabs"][0]["url"], address);
    assert_eq!(
        browser.status().await.unwrap()["profiles"][0]["allow_agent"],
        true
    );
    let agent_tabs = browser
        .execute(
            "browser_tabs",
            &json!({"profile":"personal"}),
            false,
            "test",
            &cancel,
        )
        .await
        .unwrap();
    assert!(
        !agent_tabs["tabs"][0]["url"]
            .as_str()
            .unwrap()
            .contains("page=login")
    );
    let jpeg = browser.view_frame("personal", None, true).await.unwrap();
    assert!(jpeg.starts_with(&[0xff, 0xd8]));
    let png = browser.screenshot("personal", None).await.unwrap();
    assert!(png.starts_with(b"\x89PNG"));
    // Owner frames do not silently satisfy the agent's restore-observation guard.
    browser
        .sessions
        .lock()
        .await
        .get_mut("personal")
        .unwrap()
        .needs_observation = true;
    browser.view_frame("personal", None, true).await.unwrap();
    let blocked = browser
        .execute(
            "browser_act",
            &json!({"profile":"personal","action":"reload"}),
            false,
            "test",
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(blocked["action_executed"], false);
    run("browser_resize", json!({"width":390,"height":600,"dpr":1})).await;
    assert_eq!(
        browser.status().await.unwrap()["profiles"][0]["allow_agent"],
        false
    );
    run("browser_close", json!({})).await;
    browser.open_view(None).await.unwrap();
    run("browser_open", json!({"url":address})).await;
    run("browser_wait", json!({"condition":"ready"})).await;
    assert_eq!(
        run(
            "browser_check",
            json!({"condition":"text","text":"Welcome back"})
        )
        .await["passed"],
        true
    );
    let other_dir = tempfile::tempdir().unwrap();
    let other = Browser::open(other_dir.path()).unwrap();
    assert!(other.view_frame("personal", None, true).await.is_err());
    other.open_view(None).await.unwrap();
    other
        .execute(
            "browser_open",
            &json!({"profile":"personal","url":address}),
            true,
            "owner",
            &cancel,
        )
        .await
        .unwrap();
    other
        .execute(
            "browser_wait",
            &json!({"profile":"personal","condition":"ready"}),
            true,
            "owner",
            &cancel,
        )
        .await
        .unwrap();
    let result = other
        .execute(
            "browser_check",
            &json!({"profile":"personal","condition":"text","text":"Sign in first"}),
            true,
            "owner",
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(result["passed"], true);
    other
        .execute(
            "browser_close",
            &json!({"profile":"personal"}),
            true,
            "owner",
            &cancel,
        )
        .await
        .unwrap();
    run("browser_close", json!({})).await;
    server.abort();
}
