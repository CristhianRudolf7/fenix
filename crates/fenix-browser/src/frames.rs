//! Frame ownership, including out-of-process iframes. Never disable site isolation.
use super::*;
pub(super) struct FrameSet {
    pub tree: Value,
    contexts: BTreeMap<String, String>,
    roots: BTreeMap<String, String>,
}
pub(super) struct World {
    pub session: String,
    pub id: Value,
}
fn collect(tree: &Value, session: &str, map: &mut BTreeMap<String, String>) {
    if let Some(id) = tree["frame"]["id"].as_str() {
        map.insert(id.into(), session.into());
    }
    if let Some(children) = tree["childFrames"].as_array() {
        for child in children {
            collect(child, session, map);
        }
    }
}
fn node<'a>(tree: &'a Value, id: &str) -> Option<&'a Value> {
    if tree["frame"]["id"] == id {
        return Some(tree);
    }
    tree["childFrames"]
        .as_array()?
        .iter()
        .find_map(|child| node(child, id))
}
fn insert(tree: &mut Value, parent: &str, child: Value) -> bool {
    if tree["frame"]["id"] == parent {
        if !tree["childFrames"].is_array() {
            tree["childFrames"] = json!([]);
        }
        let children = tree["childFrames"].as_array_mut().unwrap();
        if let Some(existing) = children
            .iter_mut()
            .find(|v| v["frame"]["id"] == child["frame"]["id"])
        {
            *existing = child;
        } else {
            children.push(child);
        }
        return true;
    }
    if let Some(children) = tree["childFrames"].as_array_mut() {
        for branch in children {
            if insert(branch, parent, child.clone()) {
                return true;
            }
        }
    }
    false
}
impl Browser {
    async fn remote_session(&self, s: &Session, id: &str) -> Result<String> {
        let mut sessions = s.frame_sessions.lock().await;
        if let Some(session) = sessions.get(id) {
            return Ok(session.clone());
        }
        let r = s
            .cdp
            .call(
                None,
                "Target.attachToTarget",
                json!({"targetId":id,"flatten":true}),
            )
            .await?;
        let session = r["sessionId"]
            .as_str()
            .context("Missing iframe session")?
            .to_owned();
        for method in ["Page.enable", "Runtime.enable", "Network.enable"] {
            s.cdp.call(Some(&session), method, json!({})).await?;
        }
        let port = self.control_port.load(Ordering::Relaxed);
        s.cdp.call(Some(&session),"Network.setBlockedURLs",json!({"urls":[format!("http://127.0.0.1:{port}/*"),format!("http://localhost:{port}/*"),format!("http://[::1]:{port}/*")]})).await?;
        sessions.insert(id.into(), session.clone());
        Ok(session)
    }
    pub(super) async fn frame_set(&self, s: &Session, session: &str) -> Result<FrameSet> {
        let result = s
            .cdp
            .call(Some(session), "Page.getFrameTree", json!({}))
            .await?;
        let mut tree = result["frameTree"].clone();
        let mut contexts = BTreeMap::new();
        let mut roots = BTreeMap::new();
        collect(&tree, session, &mut contexts);
        roots.insert(
            session.into(),
            tree["frame"]["id"]
                .as_str()
                .context("Missing main frame")?
                .into(),
        );
        let targets = s.cdp.call(None, "Target.getTargets", json!({})).await?;
        let mut pending = targets["targetInfos"]
            .as_array()
            .context("Missing targets")?
            .iter()
            .filter(|t| t["type"] == "iframe")
            .take(64)
            .collect::<Vec<_>>();
        let live = pending
            .iter()
            .filter_map(|v| v["targetId"].as_str())
            .collect::<Vec<_>>();
        s.frame_sessions
            .lock()
            .await
            .retain(|id, _| live.contains(&id.as_str()));
        // Only attach descendants of this tab. A supplied frame cannot select another tab.
        for _ in 0..32 {
            let mut changed = false;
            let mut unresolved = vec![];
            for target in pending {
                let parent = target["parentFrameId"]
                    .as_str()
                    .or_else(|| target["parentId"].as_str())
                    .unwrap_or("");
                if !contexts.contains_key(parent) {
                    unresolved.push(target);
                    continue;
                }
                let id = target["targetId"].as_str().context("Missing iframe ID")?;
                let mut remote = self.remote_session(s, id).await?;
                let result = match s
                    .cdp
                    .call(Some(&remote), "Page.getFrameTree", json!({}))
                    .await
                {
                    Ok(v) => v,
                    Err(_) => {
                        s.frame_sessions.lock().await.remove(id);
                        remote = self.remote_session(s, id).await?;
                        s.cdp
                            .call(Some(&remote), "Page.getFrameTree", json!({}))
                            .await?
                    }
                };
                let mut child = result["frameTree"].clone();
                child["frame"]["parentId"] = json!(parent);
                if child["frame"]["name"].as_str().is_none_or(str::is_empty) {
                    let parent_session = &contexts[parent];
                    let owner = s
                        .cdp
                        .call(
                            Some(parent_session),
                            "DOM.getFrameOwner",
                            json!({"frameId":id}),
                        )
                        .await?;
                    let description = s
                        .cdp
                        .call(
                            Some(parent_session),
                            "DOM.describeNode",
                            json!({"backendNodeId":owner["backendNodeId"]}),
                        )
                        .await?;
                    if let Some(attrs) = description["node"]["attributes"].as_array() {
                        for pair in attrs.as_chunks::<2>().0 {
                            if pair[0] == "name" {
                                child["frame"]["name"] = pair[1].clone();
                            }
                        }
                    }
                }
                collect(&child, &remote, &mut contexts);
                roots.insert(remote, id.into());
                insert(&mut tree, parent, child);
                changed = true;
            }
            pending = unresolved;
            if !changed || pending.is_empty() {
                break;
            }
        }
        Ok(FrameSet {
            tree,
            contexts,
            roots,
        })
    }
    pub(super) async fn world(
        &self,
        s: &Session,
        session: &str,
        frame: Option<&str>,
    ) -> Result<World> {
        let (session, frame) = if let Some(frame) = frame {
            let set = self.frame_set(s, session).await?;
            (
                set.contexts
                    .get(frame)
                    .context("Frame no longer belongs to this tab; inspect frames again")?
                    .clone(),
                frame.to_owned(),
            )
        } else {
            let result = s
                .cdp
                .call(Some(session), "Page.getFrameTree", json!({}))
                .await?;
            (
                session.to_owned(),
                result["frameTree"]["frame"]["id"]
                    .as_str()
                    .context("Missing main frame")?
                    .to_owned(),
            )
        };
        let world = s
            .cdp
            .call(
                Some(&session),
                "Page.createIsolatedWorld",
                json!({"frameId":frame,"worldName":"fenix-browser","grantUniveralAccess":false}),
            )
            .await?;
        Ok(World {
            session,
            id: world["executionContextId"].clone(),
        })
    }
    pub(super) async fn scroll_frame(&self, s: &Session, session: &str, frame: &str) -> Result<()> {
        let set = self.frame_set(s, session).await?;
        let mut current = frame.to_owned();
        let mut owners = vec![];
        for _ in 0..32 {
            let n = node(&set.tree, &current).context("Unknown frame")?;
            let Some(parent) = n["frame"]["parentId"].as_str() else {
                break;
            };
            let ctx = set.contexts.get(parent).context("Missing parent frame")?;
            let owner = s
                .cdp
                .call(Some(ctx), "DOM.getFrameOwner", json!({"frameId":current}))
                .await?;
            owners.push((ctx.clone(), owner["backendNodeId"].clone()));
            if ctx == session {
                break;
            }
            current = set.roots[ctx].clone();
        }
        for (ctx, id) in owners.into_iter().rev() {
            s.cdp
                .call(
                    Some(&ctx),
                    "DOM.scrollIntoViewIfNeeded",
                    json!({"backendNodeId":id}),
                )
                .await?;
        }
        Ok(())
    }
    pub(super) async fn frame_point(
        &self,
        s: &Session,
        session: &str,
        frame: &str,
        point: (f64, f64),
        require_hit: bool,
    ) -> Result<(f64, f64)> {
        let (mut x, mut y) = point;
        let set = self.frame_set(s, session).await?;
        let mut current = frame.to_owned();
        for _ in 0..32 {
            let n = node(&set.tree, &current).context("Unknown frame")?;
            let Some(parent) = n["frame"]["parentId"].as_str() else {
                return Ok((x, y));
            };
            let ctx = &set.contexts[parent];
            let owner = s
                .cdp
                .call(Some(ctx), "DOM.getFrameOwner", json!({"frameId":current}))
                .await?;
            let model = s
                .cdp
                .call(
                    Some(ctx),
                    "DOM.getBoxModel",
                    json!({"backendNodeId":owner["backendNodeId"]}),
                )
                .await?;
            let q = model["model"]["content"]
                .as_array()
                .context("Frame has no visible content box")?;
            let dimensions = self
                .eval(
                    s,
                    session,
                    Some(&current),
                    "({width:innerWidth,height:innerHeight})".into(),
                )
                .await?;
            let u = x / number(&dimensions, "width", 1.);
            let v = y / number(&dimensions, "height", 1.);
            let n = |i: usize| q.get(i).and_then(Value::as_f64).unwrap_or(0.);
            (x, y) = (
                n(0) + u * (n(2) - n(0)) + v * (n(6) - n(0)),
                n(1) + u * (n(3) - n(1)) + v * (n(7) - n(1)),
            );
            if require_hit {
                // This DOM command uses document coordinates and stops at process
                // boundaries. Check every boundary, including the embedding element.
                let offset = self
                    .eval(s, ctx, None, "({x:scrollX,y:scrollY})".into())
                    .await?;
                let hit=s.cdp.call(Some(ctx),"DOM.getNodeForLocation",json!({"x":(x+number(&offset,"x",0.)).round() as i64,"y":(y+number(&offset,"y",0.)).round() as i64,"includeUserAgentShadowDOM":true})).await?;
                if hit["frameId"] != current && hit["backendNodeId"] != owner["backendNodeId"] {
                    bail!("Frame target is covered or clipped by its parent page");
                }
            }
            if ctx == session {
                return Ok((x, y));
            }
            current = set.roots[ctx].clone();
        }
        bail!("Frame nesting exceeds the interaction limit")
    }
}
