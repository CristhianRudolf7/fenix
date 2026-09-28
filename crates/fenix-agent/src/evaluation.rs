//! Native, isolated evaluation lifecycle. The external benchmark owns grading.
use crate::{Runtime, config::Settings};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

mod interaction;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationConfig {
    pub instruction: String,
    pub settings: Settings,
    /// Read once; never included in reports. Use an external proxy for untrusted tasks.
    pub api_key_file: PathBuf,
    pub timeout_seconds: u64,
    #[serde(default = "waiting_seconds")]
    pub waiting_seconds: u64,
    pub benchmark: String,
    pub dataset_revision: String,
    pub task_name: String,
    /// Optional container-to-host recording proxy tunnel. Provider HTTP remains loopback-only.
    #[serde(default)]
    pub proxy_upstream: Option<std::net::SocketAddr>,
    /// Optional typed decisions through the same host recording proxy, with no real key in the container.
    #[serde(default)]
    pub decisions: Option<crate::decisions::DecisionSettings>,
    /// Opt-in local benchmark transport for multi-turn, externally driven scenarios.
    /// The benchmark owns grading and sends only user-visible notifications here.
    #[serde(default)]
    pub interaction_socket: Option<PathBuf>,
}
fn waiting_seconds() -> u64 {
    15
}

fn evaluated_binary_path() -> Result<PathBuf> {
    let image = std::env::current_exe()?;
    if image
        .file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with("ld-linux"))
    {
        // Explicit glibc-loader launches expose the loader through /proc/self/exe.
        // argv[0] still identifies the program, as in the read-only helper path.
        std::fs::canonicalize(
            std::env::args_os()
                .next()
                .context("Evaluated executable path unavailable")?,
        )
        .context("Cannot resolve the evaluated executable behind its loader")
    } else {
        Ok(image)
    }
}

#[derive(Default, Serialize)]
pub struct EvaluationMetrics {
    pub recorded_model_calls: u64,
    pub decision_calls: u64,
    pub decision_cache_hits: u64,
    pub decision_fallbacks: u64,
    pub decision_input_tokens: u64,
    pub decision_output_tokens: u64,
    pub decision_usage_missing: u64,
    pub decision_estimated_cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub usage_missing: u64,
    pub tool_calls: u64,
    pub tool_errors: u64,
    pub recoveries: u64,
}
impl EvaluationMetrics {
    pub fn observe(&mut self, event: &Value) {
        match event["kind"].as_str().unwrap_or("") {
            "model_response" | "context_summary_response" => {
                self.recorded_model_calls += 1;
                let usage = &event["data"]["usage"];
                self.input_tokens += usage["input_tokens"].as_u64().unwrap_or(0);
                self.output_tokens += usage["output_tokens"].as_u64().unwrap_or(0);
                if usage["input_tokens"].as_u64().is_none()
                    || usage["output_tokens"].as_u64().is_none()
                {
                    self.usage_missing += 1;
                }
            }
            "decision_evaluation" => {
                let data = &event["data"];
                if data["available"] != true {
                    self.decision_fallbacks += 1;
                } else if data["cached"] == true {
                    self.decision_cache_hits += 1;
                } else {
                    self.decision_calls += 1;
                    self.decision_input_tokens +=
                        data["usage"]["input_tokens"].as_u64().unwrap_or(0);
                    self.decision_output_tokens +=
                        data["usage"]["output_tokens"].as_u64().unwrap_or(0);
                    self.decision_usage_missing +=
                        u64::from(data["usage"]["input_tokens"].as_u64().is_none());
                    self.decision_estimated_cost_usd +=
                        data["estimated_cost_usd"].as_f64().unwrap_or(0.0);
                }
            }
            "tool_result" => {
                self.tool_calls += 1;
                self.tool_errors += u64::from(event["data"]["is_error"] == true);
            }
            "task_recovery" => self.recoveries += 1,
            _ => {}
        }
    }
}

/// Refuse reused state: evaluations must not inherit personal memory or credentials.
pub async fn run(mut config: EvaluationConfig, output: &Path, keep_services: bool) -> Result<()> {
    if (config.timeout_seconds == 0 && !config.settings.execution.externally_supervised)
        || config.waiting_seconds == 0
    {
        bail!("Evaluation timeouts must be positive");
    }
    if config.decisions.is_some() && config.proxy_upstream.is_none() {
        bail!("Evaluation decisions require the host recording proxy");
    }
    std::fs::create_dir(output).context("Evaluation output must be a new directory")?;
    let tunnel = if let Some(upstream) = config.proxy_upstream {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let mut endpoint = reqwest::Url::parse(
            config
                .settings
                .base_url
                .as_deref()
                .context("A proxy base_url is required")?,
        )?;
        if endpoint.scheme() != "http" || endpoint.host_str() != Some("127.0.0.1") {
            bail!("The evaluation tunnel requires an http://127.0.0.1 base_url");
        }
        endpoint
            .set_port(Some(listener.local_addr()?.port()))
            .map_err(|_| anyhow::anyhow!("Invalid proxy port"))?;
        config.settings.base_url = Some(endpoint.to_string());
        Some(EvaluationTunnel(tokio::spawn(async move {
            let mut connections = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let Ok((mut incoming,_)) = accepted else { break; };
                        connections.spawn(async move {
                            if let Ok(mut outgoing) = tokio::net::TcpStream::connect(upstream).await {
                                let _ = tokio::io::copy_bidirectional(&mut incoming,&mut outgoing).await;
                            }
                        });
                    }
                    _ = connections.join_next(), if !connections.is_empty() => {},
                }
            }
        })))
    } else {
        None
    };
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let binary_path = evaluated_binary_path()?;
    let mut binary = std::fs::File::open(&binary_path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let read = binary.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    crate::write_private(&output.join("manifest.json"), &json!({"schema_version":1,"binary_sha256":format!("{:x}",digest.finalize()),"binary_path":binary_path,"process_image_path":std::env::current_exe()?,"version":env!("CARGO_PKG_VERSION"),"benchmark":config.benchmark,"dataset_revision":config.dataset_revision}).to_string())?;
    let rt = Runtime::open(&output.join("runtime"), Some(&output.join("skills"))).await?;
    let key = std::fs::read_to_string(&config.api_key_file)
        .context("Cannot read evaluation credential")?;
    rt.configure(config.settings.clone(), Some(key.trim().into()))
        .await?;
    if let Some(decisions) = config.decisions.clone() {
        let mut relay = reqwest::Url::parse(
            config
                .settings
                .base_url
                .as_deref()
                .context("Missing evaluation relay")?,
        )?;
        relay.set_path(&format!("{}/jev", relay.path().trim_end_matches('/')));
        *rt.decisions
            .evaluation_relay
            .lock()
            .expect("decision relay") = Some(relay.to_string());
        rt.configure_decisions(decisions, Some(key.trim().into()))?;
    }
    let started = Instant::now();
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    // Freeze auxiliary learning before starting any worker. Evaluation measures
    // this task, not an asynchronously learned procedure or cross-trial adaptation.
    rt.store
        .put("evaluation", "background_learning_disabled", &true)?;
    rt.store.put("skill_learning_settings", "enabled", &false)?;
    let workers = rt.start();
    let execution: Result<String> = async {
        if config.instruction.trim().is_empty() || config.instruction.len() > 32_000 { bail!("Evaluation instruction must contain 1–32000 bytes"); }
        // Benchmark instructions are new tasks, not textual chat-control commands
        // such as "reply" or "cancel". All subsequent execution uses the normal worker.
        let mut interaction = match &config.interaction_socket {
            Some(path) => Some(interaction::Session::connect(path, &config.instruction).await?),
            None => None,
        };
        rt.store.put("evaluation", "noninteractive", &interaction.is_none())?;
        if interaction.is_none() {
            let task = rt.store.enqueue(&rt.redactor.text(&config.instruction), "evaluation", None)?;
            rt.pin_destination(&task.id, "evaluation")?;
            rt.wake.notify_one();
        }
        let mut settled = None;
        let mut waiting = None;
        loop {
            if let Some(session) = &mut interaction {
                if session.tick(&rt).await? {
                    break Ok("benchmark_complete".into());
                }
            }
            let tasks = evaluation_tasks(&rt)?;
            let plans = rt.store.plans()?;
            let running = tasks
                .iter()
                .any(|t| matches!(t.status.as_str(), "running" | "queued"))
                || plans
                    .iter()
                    .any(|p| matches!(p.status.as_str(), "running" | "revising"));
            let blocked = tasks.iter().any(|t| t.status == "waiting")
                || plans
                    .iter()
                    .any(|p| matches!(p.status.as_str(), "paused" | "blocked"));
            // Keep live reporting cheap. Full evidence is already durable in SQLite;
            // export it once after inference stops instead of rewriting all logs per poll.
            crate::write_private(&output.join("progress.partial"), &json!({"elapsed_seconds":started.elapsed().as_secs_f64(),"tasks":tasks,"plans":plans}).to_string())?;
            std::fs::rename(output.join("progress.partial"),output.join("progress.json"))?;
            if config.timeout_seconds != 0 && started.elapsed() >= Duration::from_secs(config.timeout_seconds) {
                break Ok("timeout".into());
            }
            // A running plan with every worker waiting cannot make progress alone.
            // Preserve the genuine question instead of burning the entire deadline.
            let active_tasks = tasks.iter().any(|t| matches!(t.status.as_str(), "running" | "queued"));
            if interaction.is_some() {
                // An idle turn is not the end of an asynchronous scenario. Only
                // the external controller, deadline or cancellation ends it.
            } else if running && (!blocked || active_tasks) {
                settled = None;
                waiting = None;
            } else if blocked {
                settled = None;
                if waiting.get_or_insert_with(Instant::now).elapsed()
                    >= Duration::from_secs(config.waiting_seconds)
                {
                    break Ok("requires_user_input".into());
                }
            } else {
                waiting = None;
                if settled.get_or_insert_with(Instant::now).elapsed() >= Duration::from_secs(2) {
                    break Ok("all_tasks_terminal".into());
                }
            }
            tokio::select! {
                _ = terminate.recv() => break Ok("cancelled".into()),
                _ = tokio::signal::ctrl_c() => break Ok("cancelled".into()),
                _ = tokio::time::sleep(Duration::from_millis(500)) => {},
            }
        }
    }
    .await;
    // Abort worker owners and their JoinSets before the verifier runs. Managed
    // persistent processes live separately and remain until explicit shutdown.
    for worker in &workers {
        worker.abort();
    }
    for worker in workers {
        let _ = worker.await;
    }
    if let Some(tunnel) = tunnel {
        tunnel.close().await;
    }
    let reason = execution
        .as_ref()
        .map(String::as_str)
        .unwrap_or("controller_error");
    let preserve = keep_services && reason == "all_tasks_terminal";
    if !preserve {
        rt.shutdown().await?;
    }
    export(
        &rt,
        &config,
        output,
        started.elapsed(),
        reason,
        execution.as_ref().err().map(|e| e.to_string()),
    )
    .await?;
    crate::write_private(&output.join("ready.json"), &json!({"stop_reason":reason,"services_preserved":preserve,"report":"report.json","reward":null}).to_string())?;
    if preserve {
        tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
        rt.shutdown().await?;
    }
    execution.map(|_| ())
}

struct EvaluationTunnel(tokio::task::JoinHandle<()>);
impl EvaluationTunnel {
    async fn close(mut self) {
        self.0.abort();
        let _ = (&mut self.0).await;
    }
}
impl Drop for EvaluationTunnel {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn export(
    rt: &Runtime,
    config: &EvaluationConfig,
    output: &Path,
    elapsed: Duration,
    reason: &str,
    error: Option<String>,
) -> Result<()> {
    let mut metrics = EvaluationMetrics::default();
    let mut traces = Vec::new();
    for task in evaluation_tasks(rt)? {
        let mut events = Vec::new();
        let mut cursor = 0;
        loop {
            let page = rt.store.events(&task.id, cursor, 1000)?;
            if page.is_empty() {
                break;
            }
            cursor = page
                .last()
                .and_then(|v| v["seq"].as_i64())
                .context("Invalid event cursor")?;
            for event in &page {
                metrics.observe(event);
            }
            events.extend(page);
        }
        traces.push(json!({"task":task,"events":events}));
    }
    let mut report = json!({"schema_version":1,"fenix_version":env!("CARGO_PKG_VERSION"),"benchmark":config.benchmark,
        "dataset_revision":config.dataset_revision,"task_name":config.task_name,"settings":config.settings,
        "elapsed_seconds":elapsed.as_secs_f64(),"stop_reason":reason,"error":error,"metrics":metrics,
        "usage_scope":"Recorded foreground and compaction responses only; provider-side accounting is required for total costs, caching, auxiliary calls and transport retries.",
        "background_learning":{"enabled":false,"policy":"disabled_before_worker_start","reason":"isolated_reproducible_evaluation"},
        "cost_usd":null,"reward":null,"grading":"External official verifier only; completed is not a passing grade.",
        "traces":traces,"processes":rt.processes.list().await});
    rt.redactor.value(&mut report);
    let temporary = output.join("report.partial");
    crate::write_private(&temporary, &serde_json::to_string_pretty(&report)?)?;
    std::fs::rename(temporary, output.join("report.json"))?;
    Ok(())
}

fn evaluation_tasks(rt: &Runtime) -> Result<Vec<fenix_storage::Task>> {
    let mut cursor = 0;
    let mut tasks = Vec::new();
    loop {
        let page = rt.store.tasks_page(cursor, 1000)?;
        if page.is_empty() {
            break;
        }
        for (seq, task) in page {
            cursor = seq;
            tasks.push(task);
        }
    }
    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn program_provenance_resolves_direct_and_loader_launches() {
        let expected = std::fs::canonicalize(std::env::args_os().next().unwrap()).unwrap();
        assert_eq!(evaluated_binary_path().unwrap(), expected);
        assert!(expected.is_file());
    }
    #[tokio::test]
    async fn native_evaluation_exports_an_isolated_ungraded_trace() {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path},
        };
        let dir = tempfile::tempdir().unwrap();
        let server = MockServer::start().await;
        Mock::given(method("POST")).and(path("/responses")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":"fixture","status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Done"}]}],"usage":{"input_tokens":20,"output_tokens":5}}))).mount(&server).await;
        let key = dir.path().join("key");
        std::fs::write(&key, "private-evaluation-fixture").unwrap();
        let mut settings = Settings::initial(dir.path().into());
        settings.base_url = Some(server.uri());
        let output = dir.path().join("trial");
        let config = || EvaluationConfig {
            instruction: "Reply Done".into(),
            settings: settings.clone(),
            api_key_file: key.clone(),
            timeout_seconds: 15,
            waiting_seconds: 1,
            benchmark: "fixture".into(),
            dataset_revision: "pinned-test".into(),
            task_name: "reply".into(),
            proxy_upstream: Some(*server.address()),
            decisions: None,
            interaction_socket: None,
        };
        run(config(), &output, false).await.unwrap();
        let report = std::fs::read_to_string(output.join("report.json")).unwrap();
        assert!(!report.contains("private-evaluation-fixture"));
        let report: Value = serde_json::from_str(&report).unwrap();
        assert_eq!(report["stop_reason"], "all_tasks_terminal");
        assert_eq!(report["reward"], Value::Null);
        assert_eq!(report["background_learning"]["enabled"], false);
        assert_eq!(report["cost_usd"], Value::Null);
        assert!(report["metrics"]["recorded_model_calls"].as_u64().unwrap() >= 1);
        assert!(output.join("ready.json").exists());
        assert!(run(config(), &output, false).await.is_err());
    }
    #[test]
    fn metrics_do_not_confuse_completion_with_reward_or_missing_usage_with_zero_cost() {
        let mut metrics = EvaluationMetrics::default();
        metrics.observe(&json!({"kind":"model_response","data":{"usage":{"input_tokens":10,"output_tokens":20}}}));
        metrics.observe(&json!({"kind":"context_summary_response","data":{"usage":{}}}));
        metrics.observe(&json!({"kind":"task_finished","data":{"status":"completed"}}));
        assert_eq!(metrics.recorded_model_calls, 2);
        assert_eq!(metrics.usage_missing, 1);
        assert_eq!(metrics.input_tokens, 10);
    }
}
