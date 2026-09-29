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
        if owner
            && action == "type"
            && frame.is_none()
            && args.get("ref").is_none()
            && args.get("selector").is_none()
        {
            // Human typing follows Chromium's focused control, including iframe
            // inputs. Do not refocus the parent document or inspect field values.
            let value = text(args, "text", "");
            if value.len() > 16000 {
                bail!("Input text exceeds 16000 bytes");
            }
            s.cdp
                .call(Some(session), "Input.insertText", json!({"text":value}))
                .await?;
            return Ok(json!({"performed":true,"action":"type"}));
        }
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
            let prompt = text(args, "text", "");
            if prompt.len() > 16000 {
                bail!("Prompt text exceeds 16000 bytes");
            }
            s.cdp
                .call(
                    Some(session),
                    "Page.handleJavaScriptDialog",
                    json!({"accept":args["accept"].as_bool().unwrap_or(false),"promptText":prompt}),
                )
                .await?;
        } else {
            let targeted = args.get("ref").is_some() || args.get("selector").is_some();
            if let Some(frame) = frame {
                self.scroll_frame(s, session, frame).await?;
            }
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
                (x, y) = self
                    .frame_point(
                        s,
                        session,
                        frame,
                        (x, y),
                        targeted && !matches!(action, "upload" | "scroll"),
                    )
                    .await?;
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
                "click" | "double_click" | "right_click" | "middle_click" | "check" | "uncheck" => {
                    let perform = if matches!(action, "check" | "uncheck") {
                        let checked=self.eval(s,session,frame,"(()=>{const e=window.__fenixTarget;if(!e.matches('input[type=checkbox],input[type=radio]'))throw Error();return e.checked})()".into()).await?;
                        if action == "uncheck" && target["inputType"] == "radio" && checked == true
                        {
                            bail!(
                                "A radio button cannot be unchecked by clicking it; select another option"
                            );
                        }
                        checked.as_bool() != Some(action == "check")
                    } else {
                        true
                    };
                    if perform {
                        let button = match action {
                            "right_click" => "right",
                            "middle_click" => "middle",
                            _ => "left",
                        };
                        for count in 1..=if action == "double_click" { 2 } else { 1 } {
                            // The owner UI forwards the native click sequence. A
                            // double click must not replay two additional clicks.
                            let count = if action == "click" {
                                number(args, "click_count", 1.).clamp(1., 3.) as u32
                            } else {
                                count
                            };
                            for kind in ["mousePressed", "mouseReleased"] {
                                s.cdp
                                    .call(
                                        Some(session),
                                        "Input.dispatchMouseEvent",
                                        mouse(kind, json!({"button":button,"clickCount":count})),
                                    )
                                    .await?;
                            }
                        }
                    }
                    if matches!(action, "check" | "uncheck") {
                        let checked=self.eval(s,session,frame,"Boolean(window.__fenixTarget?.isConnected && window.__fenixTarget.checked)".into()).await?;
                        if checked.as_bool() != Some(action == "check") {
                            bail!(
                                "Page prevented the requested checked state; inspect before retrying"
                            );
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
                    let prepared = self
                        .eval(
                            s,
                            session,
                            frame,
                            include_str!("prepare_input.js").replace(
                                "INPUT",
                                &json!({"action":action,"text":value}).to_string(),
                            ),
                        )
                        .await?;
                    if let Some(error) = prepared["error"].as_str() {
                        bail!("{error}");
                    }
                    if prepared["native"] == true {
                        if value.is_empty() && action == "fill" {
                            send_key(s, session, "Backspace").await?;
                        } else {
                            s.cdp
                                .call(Some(session), "Input.insertText", json!({"text":value}))
                                .await?;
                        }
                    }
                    if action == "fill" {
                        let expected = json!(value);
                        let accepted=self.eval(s,session,frame,format!("(()=>{{const e=window.__fenixTarget;if(!e?.isConnected)return false;let expected={expected};if(e instanceof HTMLInputElement){{const p=document.createElement('input');p.type=e.type;p.value=expected;expected=p.value;}}else if(e instanceof HTMLTextAreaElement)expected=expected.replace(/\\r\\n?/g,'\\n');return (e.isContentEditable?(expected===''&&e.textContent===''?'':e.innerText):e.value)===expected;}})()")).await?;
                        if accepted != true {
                            bail!(
                                "Page changed or rejected the entered value; inspect before retrying"
                            );
                        }
                    }
                }

                "key" => {
                    if targeted {
                        let focused=self.eval(s,session,frame,"(()=>{const e=window.__fenixTarget;e.focus();return e.isConnected && window.__fenixDOM.contains(e,window.__fenixDOM.active());})()".into()).await?;
                        if focused != true {
                            bail!("Requested key target could not receive focus");
                        }
                    }
                    send_key(s, session, text(args, "key", "Enter")).await?;
                }
                "select" => {
                    let value = json!(text(args, "value", ""));
                    let selected=self.eval(s,session,frame,format!("(()=>{{const e=window.__fenixTarget;if(e.tagName!=='SELECT')throw Error();const option=[...e.options].find(o=>o.value==={value});if(!option||option.matches(':disabled'))return {{error:'Option is missing or disabled'}};e.value={value};e.dispatchEvent(new Event('input',{{bubbles:true}}));e.dispatchEvent(new Event('change',{{bubbles:true}}));return true}})()")).await?;
                    if let Some(error) = selected["error"].as_str() {
                        bail!("{error}");
                    }
                }
                "upload" => {
                    let path = Path::new(text(args, "path", ""));
                    if !path.is_absolute()
                        || !path.is_file()
                        || std::fs::metadata(path)?.len() > 100 * 1024 * 1024
                    {
                        bail!("Upload requires an authorized regular file of at most 100 MiB");
                    }
                    let staged;
                    let path = if s.execution.is_some() {
                        let bytes = fenix_execution::fs::read(path, 100 * 1024 * 1024)?;
                        staged = s
                            .directory
                            .join("uploads")
                            .join(uuid::Uuid::new_v4().to_string())
                            .join(path.file_name().context("Missing upload name")?);
                        fenix_execution::fs::create(&staged, &bytes)?;
                        staged.as_path()
                    } else {
                        path
                    };
                    let world = self.world(s, session, frame).await?;
                    let object = s
                        .cdp
                        .call(
                            Some(&world.session),
                            "Runtime.evaluate",
                            json!({"contextId":world.id,"expression":"window.__fenixTarget"}),
                        )
                        .await?;
                    let result = s
                        .cdp
                        .call(
                            Some(&world.session),
                            "DOM.setFileInputFiles",
                            json!({"objectId":object["result"]["objectId"],"files":[path]}),
                        )
                        .await;
                    let _ = s
                        .cdp
                        .call(
                            Some(&world.session),
                            "Runtime.releaseObject",
                            json!({"objectId":object["result"]["objectId"]}),
                        )
                        .await;
                    result?;
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
        let tab = text(&args, "tab", &s.active).to_owned();
        let session = self.attach(s, &tab).await?;
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
    let mut key = value;
    let mut modifiers = 0;
    loop {
        let found = [
            ("Control+", 2),
            ("Ctrl+", 2),
            ("Alt+", 1),
            ("Meta+", 4),
            ("Shift+", 8),
        ]
        .into_iter()
        .find(|(prefix, _)| key.starts_with(prefix));
        if let Some((prefix, flag)) = found {
            modifiers |= flag;
            key = &key[prefix.len()..];
        } else {
            break;
        }
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
        k if k.chars().count() == 1 => {
            if k.is_ascii() {
                k.as_bytes()[0].to_ascii_uppercase() as u32
            } else {
                0
            }
        }
        _ => bail!("Unsupported key"),
    };
    let text = if modifiers & (1 | 2 | 4) == 0 {
        if key == "Enter" {
            "\r"
        } else if key.chars().count() == 1 {
            key
        } else {
            ""
        }
    } else {
        ""
    };
    for kind in ["keyDown", "keyUp"] {
        let mut params =
            json!({"type":kind,"key":key,"windowsVirtualKeyCode":code,"modifiers":modifiers});
        if kind == "keyDown" && !text.is_empty() {
            params["text"] = json!(text);
            params["unmodifiedText"] = json!(text);
        }
        s.cdp
            .call(Some(session), "Input.dispatchKeyEvent", params)
            .await?;
    }
    Ok(())
}
