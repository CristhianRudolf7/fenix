use super::*;

impl Browser {
    pub(super) async fn act(
        &self,
        s: &Session,
        session: &str,
        frame: Option<&str>,
        args: &Value,
        owner: bool,
    ) -> Result<Value> {
        let action = text(args, "action", "");
        if matches!(action, "back" | "forward") {
            let h = s
                .cdp
                .call(Some(session), "Page.getNavigationHistory", json!({}))
                .await?;
            let index =
                h["currentIndex"].as_i64().unwrap_or(0) + if action == "back" { -1 } else { 1 };
            let entry = h["entries"]
                .get(index as usize)
                .context("No navigation history in that direction")?;
            s.cdp
                .call(
                    Some(session),
                    "Page.navigateToHistoryEntry",
                    json!({"entryId":entry["id"]}),
                )
                .await?;
        } else if action == "reload" {
            s.cdp.call(Some(session), "Page.reload", json!({})).await?;
        } else if action == "dialog" {
            s.cdp
                .call(
                    Some(session),
                    "Page.handleJavaScriptDialog",
                    json!({"accept":args["accept"].as_bool().unwrap_or(false)}),
                )
                .await?;
        } else {
            let targeted = args.get("ref").is_some() || args.get("selector").is_some();
            let script=include_str!("target.js").replace("TARGET", &json!({"ref":args["ref"],"selector":args["selector"],"scroll":targeted && !matches!(action,"scroll"|"upload")}).to_string());
            let mut target = self.eval(s, session, frame, script.clone()).await?;
            if targeted && !matches!(action, "scroll" | "upload") {
                let mut stable = false;
                for _ in 0..10 {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    let next = self.eval(s, session, frame, script.clone()).await?;
                    stable = ["x", "y", "width", "height"]
                        .iter()
                        .all(|key| (number(&next, key, 0.) - number(&target, key, 0.)).abs() < 0.5);
                    target = next;
                    if stable {
                        break;
                    }
                }
                if !stable {
                    bail!("Target is moving; wait for layout to settle");
                }
            }
            if let Some(error) = target["error"].as_str() {
                bail!("{error}");
            }
            if !owner && target["secret"] == true && matches!(action, "fill" | "type" | "key") {
                bail!("Password entry requires the owner's browser controls or credential vault");
            }
            if targeted
                && action != "upload"
                && (target["visible"] != true
                    || target["disabled"] == true
                    || target["covered"] == true)
            {
                bail!("Target is hidden, disabled or covered; observe or wait before retrying");
            }
            let mut x = number(args, "x", number(&target, "x", s.width as f64 / 2.));
            let mut y = number(args, "y", number(&target, "y", s.height as f64 / 2.));
            if let Some(frame) = frame {
                let tree = s
                    .cdp
                    .call(Some(session), "Page.getFrameTree", json!({}))
                    .await?;
                if tree["frameTree"]["frame"]["id"] != frame {
                    let owner = s
                        .cdp
                        .call(Some(session), "DOM.getFrameOwner", json!({"frameId":frame}))
                        .await?;
                    let model = s
                        .cdp
                        .call(
                            Some(session),
                            "DOM.getBoxModel",
                            json!({"backendNodeId":owner["backendNodeId"]}),
                        )
                        .await?;
                    x += model["model"]["content"][0].as_f64().unwrap_or(0.);
                    y += model["model"]["content"][1].as_f64().unwrap_or(0.);
                }
            }
            let mouse = |kind: &str, extra: Value| {
                let mut value = json!({"type":kind,"x":x,"y":y});
                value
                    .as_object_mut()
                    .unwrap()
                    .extend(extra.as_object().unwrap().clone());
                value
            };
            match action {
                "click" | "double_click" | "check" | "uncheck" => {
                    let perform = if matches!(action, "check" | "uncheck") {
                        let checked=self.eval(s,session,frame,"(()=>{const e=window.__fenixTarget;if(!e.matches('input[type=checkbox],input[type=radio]'))throw Error();return e.checked})()".into()).await?;
                        checked.as_bool() != Some(action == "check")
                    } else {
                        true
                    };
                    if perform {
                        for count in 1..=if action == "double_click" { 2 } else { 1 } {
                            for kind in ["mousePressed", "mouseReleased"] {
                                s.cdp
                                    .call(
                                        Some(session),
                                        "Input.dispatchMouseEvent",
                                        mouse(kind, json!({"button":"left","clickCount":count})),
                                    )
                                    .await?;
                            }
                        }
                    }
                }
                "hover" => {
                    s.cdp
                        .call(
                            Some(session),
                            "Input.dispatchMouseEvent",
                            mouse("mouseMoved", json!({})),
                        )
                        .await?;
                }
                "scroll" => {
                    s.cdp.call(Some(session),"Input.dispatchMouseEvent",mouse("mouseWheel",json!({"deltaX":number(args,"dx",0.).clamp(-10000.,10000.),"deltaY":number(args,"dy",600.).clamp(-10000.,10000.)}))).await?;
                }
                "drag" => {
                    s.cdp
                        .call(
                            Some(session),
                            "Input.dispatchMouseEvent",
                            mouse("mousePressed", json!({"button":"left","clickCount":1})),
                        )
                        .await?;
                    let tx = number(args, "to_x", x);
                    let ty = number(args, "to_y", y);
                    for step in 1..=10 {
                        s.cdp.call(Some(session),"Input.dispatchMouseEvent",json!({"type":"mouseMoved","x":x+(tx-x)*step as f64/10.,"y":y+(ty-y)*step as f64/10.,"buttons":1,"button":"left"})).await?;
                    }
                    s.cdp.call(Some(session),"Input.dispatchMouseEvent",json!({"type":"mouseReleased","x":tx,"y":ty,"button":"left","clickCount":1})).await?;
                }
                "fill" | "type" => {
                    let value = text(args, "text", "");
                    if value.len() > 16000 {
                        bail!("Input text exceeds 16000 bytes");
                    }
                    self.eval(s,session,frame,if action=="fill" {"(()=>{let e=window.__fenixTarget;e.focus();if(e.select)e.select();else {let r=document.createRange();r.selectNodeContents(e);let s=getSelection();s.removeAllRanges();s.addRange(r)}})()"} else {"window.__fenixTarget.focus()"}.into()).await?;
                    if value.is_empty() && action == "fill" {
                        send_key(s, session, "Backspace").await?;
                    } else {
                        s.cdp
                            .call(Some(session), "Input.insertText", json!({"text":value}))
                            .await?;
                    }
                }
                "key" => {
                    send_key(s, session, text(args, "key", "Enter")).await?;
                }
                "select" => {
                    let value = json!(text(args, "value", ""));
                    self.eval(s,session,frame,format!("(()=>{{const e=window.__fenixTarget;if(e.tagName!=='SELECT')throw Error();if(![...e.options].some(o=>o.value==={value}))throw Error();e.value={value};e.dispatchEvent(new Event('input',{{bubbles:true}}));e.dispatchEvent(new Event('change',{{bubbles:true}}));return true}})()")).await?;
                }
                "upload" => {
                    let path = Path::new(text(args, "path", ""));
                    if !path.is_absolute()
                        || !path.is_file()
                        || std::fs::metadata(path)?.len() > 100 * 1024 * 1024
                    {
                        bail!("Upload requires an authorized regular file of at most 100 MiB");
                    }
                    let tree = s
                        .cdp
                        .call(Some(session), "Page.getFrameTree", json!({}))
                        .await?;
                    let frame = frame
                        .unwrap_or_else(|| tree["frameTree"]["frame"]["id"].as_str().unwrap_or(""));
                    let world = s
                        .cdp
                        .call(
                            Some(session),
                            "Page.createIsolatedWorld",
                            json!({"frameId":frame,"worldName":"fenix-browser"}),
                        )
                        .await?;
                    let object=s.cdp.call(Some(session),"Runtime.evaluate",json!({"contextId":world["executionContextId"],"expression":"window.__fenixTarget"})).await?;
                    s.cdp
                        .call(
                            Some(session),
                            "DOM.setFileInputFiles",
                            json!({"objectId":object["result"]["objectId"],"files":[path]}),
                        )
                        .await?;
                }
                _ => bail!("Unknown browser action"),
            }
        }
        Ok(
            json!({"performed":true,"action":action,"next":"Observe or check the resulting page before continuing"}),
        )
    }

    /// Owner-only encrypted credentials; no plaintext credential is returned to callers.
    pub async fn credentials(&self, mut args: Value) -> Result<Value> {
        let password =
            zeroize::Zeroizing::new(args["master"].take().as_str().unwrap_or("").to_owned());
        let secret =
            zeroize::Zeroizing::new(args["password"].take().as_str().unwrap_or("").to_owned());
        let profile = text(&args, "profile", "").to_owned();
        identifier(&profile)?;
        if profile.starts_with("test-") {
            bail!("Saved credentials require a personal profile");
        }
        self.profile(&profile, false).await?;
        let action = text(&args, "action", "");
        let _guard = self.vault_lock.lock().await;
        let path = self.root.join("credentials.enc");
        let load_path = path.clone();
        let load_password = password.clone();
        let mut entries =
            tokio::task::spawn_blocking(move || vault::load(&load_path, &load_password)).await??;
        let mut sessions = self.sessions.lock().await;
        let s = sessions
            .get_mut(&profile)
            .context("Open the personal profile first")?;
        let session = s.tabs.get(&s.active).context("Select a tab first")?.clone();
        let origin = self
            .eval(s, &session, None, "location.origin".into())
            .await?;
        let origin = origin.as_str().context("Page has no origin")?;
        let parsed = url::Url::parse(origin)?;
        if parsed.scheme() != "https"
            && !(parsed.scheme() == "http"
                && matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")))
        {
            bail!("Credentials require HTTPS or local development");
        }
        match action {
            "save" => {
                let username = text(&args, "username", "");
                if username.len() > 256 || secret.is_empty() || secret.len() > 4096 {
                    bail!("Invalid credential size");
                }
                entries.retain(|e| e.profile != profile || e.origin != origin);
                entries.push(vault::Entry {
                    profile,
                    origin: origin.into(),
                    username: username.into(),
                    password: secret.to_string(),
                });
                tokio::task::spawn_blocking(move || vault::save(&path, &password, &entries))
                    .await??;
            }
            "fill" => {
                let e = entries
                    .iter()
                    .find(|e| e.profile == profile && e.origin == origin)
                    .context("No saved credentials for this profile and site")?;
                let payload = json!({"origin":origin,"username":e.username,"password":e.password,"userSelector":text(&args,"username_selector","input[autocomplete=username],input[type=email]"),"passwordSelector":text(&args,"password_selector","input[type=password]")});
                let script = format!(
                    r#"(()=>{{const a={payload};if(location.origin!==a.origin)throw Error();const p=[...document.querySelectorAll(a.passwordSelector)].filter(e=>e.getClientRects().length);const u=[...document.querySelectorAll(a.userSelector)].filter(e=>e.getClientRects().length);if(p.length!==1||u.length>1)throw Error();function fill(e,v){{Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(e,v);e.dispatchEvent(new Event('input',{{bubbles:true}}));e.dispatchEvent(new Event('change',{{bubbles:true}}));}}if(u.length)fill(u[0],a.username);fill(p[0],a.password);return true}})()"#
                );
                self.eval(s, &session, None, script).await?;
            }
            "delete" => {
                entries.retain(|e| e.profile != profile || e.origin != origin);
                tokio::task::spawn_blocking(move || vault::save(&path, &password, &entries))
                    .await??;
            }
            _ => bail!("Unknown credential action"),
        }
        Ok(json!({"completed":true}))
    }
}
async fn send_key(s: &Session, session: &str, value: &str) -> Result<()> {
    let parts = value.split('+').collect::<Vec<_>>();
    let key = *parts.last().unwrap_or(&"Enter");
    let mut modifiers = 0;
    for part in &parts[..parts.len() - 1] {
        modifiers |= match *part {
            "Alt" => 1,
            "Control" | "Ctrl" => 2,
            "Meta" => 4,
            "Shift" => 8,
            _ => bail!("Unsupported key modifier"),
        };
    }
    let code = match key {
        "Enter" => 13,
        "Tab" => 9,
        "Backspace" => 8,
        "Escape" => 27,
        "Delete" => 46,
        "ArrowLeft" => 37,
        "ArrowUp" => 38,
        "ArrowRight" => 39,
        "ArrowDown" => 40,
        "Home" => 36,
        "End" => 35,
        "PageUp" => 33,
        "PageDown" => 34,
        " " => 32,
        k if k.len() == 1 => k.as_bytes()[0].to_ascii_uppercase() as u32,
        _ => bail!("Unsupported key"),
    };
    for kind in ["keyDown", "keyUp"] {
        s.cdp
            .call(
                Some(session),
                "Input.dispatchKeyEvent",
                json!({"type":kind,"key":key,"windowsVirtualKeyCode":code,"modifiers":modifiers}),
            )
            .await?;
    }
    Ok(())
}
