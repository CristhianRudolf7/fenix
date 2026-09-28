//! Deterministic BM25 discovery over the task's authorized tool catalog.
use crate::Runtime;
use anyhow::{Result, bail};
use fenix_core::ToolDefinition;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

fn essential(name: &str) -> bool {
    matches!(
        name,
        "skill_list"
            | "skill_search"
            | "skill_load"
            | "tool_search"
            | "files_list"
            | "files_glob"
            | "file_patch"
            | "environment_inspect"
            | "tool_program"
            | "file_read"
            | "file_write"
            | "file_edit"
            | "files_search"
            | "command_run"
            | "command_status"
            | "ask_user"
            | "autonomy_manage"
            | "subagent_finish"
            | "task_finish"
            | "plan_review_read"
            | "completion_audit_read"
    )
}
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}
fn search_text(tool: &ToolDefinition) -> String {
    let hints = match tool.name.as_str() {
        "tool_program" => {
            "programa encadenar herramientas bucles lotes referencias filtrar resultados reducir llamadas"
        }
        "file_history" => "historial versiones restaurar deshacer cambios archivos tarea git local",
        "skill_learning" => {
            "aprender habilidades skills procedimiento candidatos revisar activar archivar mantenimiento"
        }
        "workspace_open" => {
            "abrir mostrar visualizar archivo carpeta pestaña diapositiva slide página panel derecho open preview workspace"
        }
        "library_list" | "library_search" | "library_read" | "library_export"
        | "library_import" => {
            "biblioteca documentos fuentes citas investigación buscar comparar contrato archivos PDF Word Excel embeddings library document retrieval citations"
        }
        "document_extract" => {
            "pdf OCR escaneado escanear extraer texto imagen páginas lectura factura contrato"
        }
        "document_read" | "document_create" | "document_edit" => {
            "documento word excel powerpoint docx xlsx pptx hoja cálculo celdas presentación diapositivas"
        }
        "design_templates" | "design_present" | "design_list" | "design_read"
        | "design_publish" | "design_import" | "diagram_edit" => {
            "diseño design vista previa visualizar diagrama gráfica presentación documento imagen video plantilla"
        }
        "file_patch" | "file_edit" | "file_write" => "archivo editar crear modificar parche",
        "files_glob" | "files_list" => "archivos carpeta buscar rutas patrones",
        "code_query" => {
            "código referencias definición símbolos diagnóstico lsp symbol outline find read name_path class method function"
        }
        "command_input" => "terminal entrada escribir enviar stdin interactiva teclado eof",
        "command_list" | "command_output" | "command_cancel" => {
            "proceso comando salida registro historial cancelar servicios"
        }
        "subagent_spawn" | "subagent_send" | "subagent_status" | "subagent_cancel" => {
            "subagente delegar tarea mensaje corregir continuar estado"
        }
        "worktree_manage" => {
            "git worktree rama integrar cambios paralelo merge conflicto branch workspace"
        }
        "plan_manage" => "planificar planificación hitos plan tareas dependencias",
        "memory_search" | "memory_manage" | "memory_context" | "knowledge_manage" => {
            "memoria preferencias recordar guardar"
        }
        "schedule_manage" | "workflow_manage" => {
            "recordatorio cron programar horario flujo automatización pasos diagrama workflow n8n"
        }
        "connector_call" => {
            "integraciones conexiones google workspace drive docs sheets slides gmail calendar tasks contacts contactos forms formularios chat meet esquemas discovery microsoft outlook gmail correo calendario slack github postgres sqlite sql proveedores plugins"
        }
        "skill_load" => "habilidad cargar skill instrucciones",
        "project_manage" => "proyecto contexto estado",
        "project_spec" => {
            "contrato proyecto requisitos especificación comportamiento cambios propuesta añadir modificar eliminar conflicto specification requirement delta"
        }
        "web_read" | "web_search" => {
            "internet web investigar buscar página global reddit hackernews hacker news github comunidades últimos días trends"
        }
        "browser_connection" => "whatsapp integración conectar vincular sesión QR",
        name if name.starts_with("browser_") => {
            "navegador navegar clic escribir formulario pantalla responsivo sesión pestaña captura scroll"
        }
        "task_inspect" => {
            "auditoría tarea checkpoint registro evidencia buscar resultados guardados salidas herramientas logs errores saved output search"
        }
        _ => "",
    };
    format!("{} {} {} {}", tool.name, tool.name, tool.description, hints)
}
impl Runtime {
    pub(crate) fn exposed_tools(&self, task: &str) -> Result<Vec<ToolDefinition>> {
        let loaded = self
            .store
            .get::<Vec<String>>("loaded_tools", task)?
            .unwrap_or_default();
        let mut tools = self.task_tool_catalog(task)?;
        let catalog = tools
            .iter()
            .filter(|t| !essential(&t.name) && !loaded.contains(&t.name))
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        tools.retain(|t| essential(&t.name) || loaded.contains(&t.name));
        if let Some(search) = tools.iter_mut().find(|t| t.name == "tool_search") {
            search
                .description
                .push_str(&format!(" Search only when the required schema is absent from this response. Call task_finish, command_run and other visible tools directly. Not yet loaded: {catalog}."));
        }
        Ok(tools)
    }
    /// Discovery cannot grant authorization or execute a guessed tool call.
    pub(crate) fn ensure_tool_loaded(&self, task: &str, name: &str) -> Result<Option<Value>> {
        let catalog = self.task_tool_catalog(task)?;
        if !catalog.iter().any(|t| t.name == name) {
            bail!("Tool is unavailable in this task: {name}");
        }
        if self.exposed_tools(task)?.iter().any(|t| t.name == name) {
            return Ok(None);
        }
        let result = self.search_tools(task, &json!({"names":[name]}))?;
        Ok(Some(
            json!({"executed":false,"code":"tool_schema_loaded","tools":result["tools"],"instruction":"No action executed. Read the exact schema and send a new valid call. Existing permissions still apply."}),
        ))
    }
    pub(crate) fn search_tools(&self, task: &str, args: &Value) -> Result<Value> {
        let tools = self.task_tool_catalog(task)?;
        let requested = args["names"]
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>())
            .unwrap_or_default();
        if requested.len() > 8 {
            bail!("Load at most eight tools per search");
        }
        let query = args["query"].as_str().unwrap_or_default();
        if query.len() > 1000 {
            bail!("Tool query exceeds 1000 bytes");
        }
        if query.trim().is_empty() && requested.is_empty() {
            return Ok(
                json!({"catalog":tools.iter().map(|t|json!({"name":t.name,"description":t.description.chars().take(180).collect::<String>()})).collect::<Vec<_>>(),"instruction":"Search a purpose or supply exact names to load schemas."}),
            );
        }
        for name in &requested {
            if !tools.iter().any(|t| t.name == *name) {
                bail!("Tool is unavailable in this task: {name}");
            }
        }
        let docs = tools
            .iter()
            .map(|t| words(&search_text(t)))
            .collect::<Vec<_>>();
        let avg = docs.iter().map(Vec::len).sum::<usize>() as f64 / docs.len().max(1) as f64;
        let mut frequency = HashMap::<&str, usize>::new();
        for doc in &docs {
            for word in doc.iter().map(String::as_str).collect::<HashSet<_>>() {
                *frequency.entry(word).or_default() += 1;
            }
        }
        let tokens = words(query);
        let mut ranked = tools
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let mut score = if requested.contains(&t.name.as_str()) {
                    1e6
                } else {
                    0.0
                };
                if t.name == query {
                    score += 1e5;
                }
                for token in &tokens {
                    let tf = docs[i].iter().filter(|w| *w == token).count() as f64;
                    let df = *frequency.get(token.as_str()).unwrap_or(&0) as f64;
                    let idf = (1.0 + (docs.len() as f64 - df + 0.5) / (df + 0.5)).ln();
                    score += idf * tf * 2.2
                        / (tf + 1.2 * (0.25 + 0.75 * docs[i].len() as f64 / avg.max(1.0)));
                }
                (score, t)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect::<Vec<_>>();
        ranked.sort_by(|(a, ta), (b, tb)| b.total_cmp(a).then_with(|| ta.name.cmp(&tb.name)));
        let limit = args["limit"].as_u64().unwrap_or(5).clamp(1, 8) as usize;
        let selected = ranked
            .into_iter()
            .take(limit.max(requested.len()))
            .map(|(_, t)| t)
            .collect::<Vec<_>>();
        let visible = self
            .exposed_tools(task)?
            .into_iter()
            .map(|t| t.name)
            .collect::<HashSet<_>>();
        let mut loaded = self
            .store
            .get::<Vec<String>>("loaded_tools", task)?
            .unwrap_or_default();
        for tool in &selected {
            if !essential(&tool.name) {
                loaded.retain(|name| name != &tool.name);
                loaded.push(tool.name.clone());
            }
        }
        // Tools remain discoverable even when older schemas leave the prompt.
        if loaded.len() > 24 {
            loaded.drain(..loaded.len() - 24);
        }
        self.store.put("loaded_tools", task, &loaded)?;
        Ok(
            json!({"tools":selected.iter().filter(|t|!visible.contains(&t.name)).map(|t|json!({"name":t.name,"description":t.description,"parameters":t.parameters})).collect::<Vec<_>>(),"already_available":selected.iter().filter(|t|visible.contains(&t.name)).map(|t|t.name.as_str()).collect::<Vec<_>>(),"loaded":true,"instruction":"Call already_available tools directly using the schemas in your current tool list; do not search again. Newly loaded schemas are available on the next response. Authorization remains scoped to this task."}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn guessed_tools_load_schemas_without_executing() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
        let task = rt.store.enqueue("Create an artifact", "web", None)?;
        let receipt = rt.ensure_tool_loaded(&task.id, "code_query")?.unwrap();
        assert_eq!(receipt["executed"], false);
        assert_eq!(
            receipt["tools"][0]["parameters"]["required"],
            json!(["path", "operation"])
        );
        assert!(rt.ensure_tool_loaded(&task.id, "code_query")?.is_none());
        assert!(!dir.path().join("artifact").exists());
        let error = crate::tools::validate(
            "file_edit",
            &json!({"path":"a","old_text":"b","new_text":"c"}),
        )
        .unwrap_err();
        let detail = error
            .downcast_ref::<crate::tools::ArgumentFailure>()
            .unwrap();
        assert!(!detail.executed);
        assert!(
            detail
                .required
                .as_array()
                .unwrap()
                .contains(&json!("expected"))
        );
        assert!(detail.received_fields.contains(&"old_text".to_owned()));
        Ok(())
    }
    #[tokio::test]
    async fn discovery_reduces_prompt_and_cannot_unlock_worker_delegation() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills")))
            .await
            .unwrap();
        let task = runtime
            .store
            .enqueue_bound(
                "Fix code",
                "web",
                None,
                &fenix_storage::TaskBinding::default(),
            )
            .unwrap();
        runtime.store.claim_next().unwrap();
        let full = runtime.task_tool_catalog(&task.id).unwrap();
        let initial = runtime.exposed_tools(&task.id).unwrap();
        assert!(
            serde_json::to_vec(&initial).unwrap().len()
                < serde_json::to_vec(&full).unwrap().len() / 2
        );
        assert!(initial.iter().any(|t| t.name == "file_patch"));
        assert!(!initial.iter().any(|t| t.name == "code_query"));
        for name in ["file_read", "file_write", "file_edit"] {
            assert!(initial.iter().any(|tool| tool.name == name));
        }
        let result = runtime
            .search_tools(&task.id, &json!({"names":["file_patch"]}))
            .unwrap();
        assert_eq!(result["tools"], json!([]));
        assert_eq!(result["already_available"], json!(["file_patch"]));
        // Selecting an older schema must refresh it before the bounded cache evicts entries.
        let old_cache = full
            .iter()
            .filter(|t| !essential(&t.name))
            .map(|t| t.name.clone())
            .take(24)
            .collect::<Vec<_>>();
        runtime
            .store
            .put("loaded_tools", &task.id, &old_cache)
            .unwrap();
        let fresh = full
            .iter()
            .find(|t| !essential(&t.name) && !old_cache.contains(&t.name))
            .unwrap();
        runtime
            .search_tools(&task.id, &json!({"names":[old_cache[0],fresh.name]}))
            .unwrap();
        let refreshed: Vec<String> = runtime
            .store
            .get("loaded_tools", &task.id)
            .unwrap()
            .unwrap();
        assert_eq!(refreshed.len(), 24);
        assert!(refreshed.contains(&old_cache[0]));
        assert!(refreshed.contains(&fresh.name));
        runtime
            .search_tools(&task.id, &json!({"names":["file_patch"]}))
            .unwrap();
        assert!(
            runtime
                .exposed_tools(&task.id)
                .unwrap()
                .iter()
                .any(|t| t.name == "file_patch")
        );
        let result = runtime
            .search_tools(&task.id, &json!({"query":"recordatorio","limit":1}))
            .unwrap();
        assert_eq!(result["tools"][0]["name"], "schedule_manage");
        let result = runtime
            .search_tools(&task.id, &json!({"query":"whatsapp","limit":1}))
            .unwrap();
        assert_eq!(result["tools"][0]["name"], "browser_connection");
        let status = runtime
            .execute_tool(
                &task.id,
                "web",
                &crate::config::Settings::initial(dir.path().to_owned()),
                "browser_connection",
                &json!({"connection":"whatsapp","action":"status"}),
                &tokio_util::sync::CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(status["state"], "disconnected");
        assert!(crate::tools::validate("browser_connection", &json!({"action":"qr"})).is_err());

        let child = runtime
            .store
            .create_subagent(
                &task.id,
                &fenix_storage::SubagentSpec {
                    profile: "general".into(),
                    objective: "Read sources".into(),
                    expected_output: "References".into(),
                    context: String::new(),
                    constraints: String::new(),
                },
            )
            .unwrap();
        assert!(
            runtime
                .search_tools(&child.id, &json!({"names":["subagent_spawn"]}))
                .is_err()
        );
        assert!(
            runtime
                .search_tools(&child.id, &json!({"names":["subagent_send"]}))
                .is_err()
        );
    }
}
