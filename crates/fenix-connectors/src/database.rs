use super::*;
use std::path::Component;

pub(crate) fn workspace_path(workspace: &Path, value: &str, create: bool) -> Result<PathBuf> {
    let relative = Path::new(value);
    ensure!(
        !relative.is_absolute()
            && relative
                .components()
                .all(|p| matches!(p, Component::Normal(_))),
        "Use a relative path inside this user's workspace"
    );
    ensure!(
        !relative.components().any(|p| {
            let name = p.as_os_str().to_string_lossy();
            name == ".git"
                || name.starts_with(".env")
                || matches!(name.as_ref(), "secrets" | "credentials")
        }),
        "Credential paths are not available"
    );
    let root = std::fs::canonicalize(workspace)?;
    let path = root.join(relative);
    let mut parent = path.as_path();
    while !parent.exists() && create {
        parent = parent.parent().context("Invalid destination")?;
    }
    ensure!(
        std::fs::canonicalize(parent)?.starts_with(&root),
        "Path escapes this user's workspace"
    );
    // Reject symlink ancestors even when their target currently remains inside the workspace.
    let mut current = root;
    for part in relative.components() {
        current.push(part);
        if let Ok(metadata) = std::fs::symlink_metadata(&current) {
            ensure!(
                !metadata.file_type().is_symlink(),
                "Symbolic links are not allowed for connection files"
            );
        }
    }
    if path.exists() {
        let _ = fenix_execution::fs::open(&path)?;
    }
    Ok(path)
}

pub(crate) async fn execute(
    c: Connection,
    operation: &str,
    args: Value,
    workspace: PathBuf,
) -> Result<Value> {
    ensure!(
        matches!(operation, "test" | "query"),
        "Unsupported database operation"
    );
    let sql = if operation == "test" {
        "SELECT 1 AS connected".into()
    } else {
        args["sql"].as_str().context("SQL is required")?.to_owned()
    };
    ensure!(sql.len() <= 100_000, "SQL is too large");
    tokio::task::spawn_blocking(move || -> Result<Value> {
        ensure!(c.provider == "sqlite", "Unsupported database provider");
        sqlite(&c, &workspace, &sql, &args)
    })
    .await?
}

fn sqlite(c: &Connection, workspace: &Path, sql: &str, args: &Value) -> Result<Value> {
    let path = workspace_path(
        workspace,
        c.settings["path"]
            .as_str()
            .context("SQLite workspace path required")?,
        false,
    )?;
    let flags = if c.allow_write {
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
    } else {
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
    };
    // SQLite NOFOLLOW rejects symlinks in the database path. /proc/self/fd is
    // intentionally unsuitable here because SQLite rejects that symlink too.
    let db = rusqlite::Connection::open_with_flags(
        &path,
        flags
            | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
            | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    db.busy_timeout(std::time::Duration::from_secs(5))?;
    use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
    db.authorizer(Some(|context: AuthContext<'_>| match context.action {
        AuthAction::Attach { .. } | AuthAction::Detach { .. } | AuthAction::Pragma { .. } => {
            Authorization::Deny
        }
        _ => Authorization::Allow,
    }));
    let start = std::time::Instant::now();
    db.progress_handler(
        1000,
        Some(move || start.elapsed() > std::time::Duration::from_secs(15)),
    );
    let mut statement = db.prepare(sql)?;
    ensure!(
        c.allow_write || statement.readonly(),
        "This database connection is read-only"
    );
    let parameters: Vec<rusqlite::types::Value> = args["parameters"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|v| match v {
            Value::Null => rusqlite::types::Value::Null,
            Value::Bool(v) => rusqlite::types::Value::Integer(i64::from(*v)),
            Value::Number(v) => v
                .as_i64()
                .map(rusqlite::types::Value::Integer)
                .unwrap_or_else(|| rusqlite::types::Value::Real(v.as_f64().unwrap_or_default())),
            Value::String(v) => rusqlite::types::Value::Text(v.clone()),
            _ => rusqlite::types::Value::Text(v.to_string()),
        })
        .collect();
    let columns = statement
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    if columns.is_empty() {
        return Ok(
            json!({"affected_rows":statement.execute(rusqlite::params_from_iter(parameters))?}),
        );
    }
    let mut query = statement.query(rusqlite::params_from_iter(parameters))?;
    let mut rows = vec![];
    while let Some(row) = query.next()? {
        let mut object = serde_json::Map::new();
        for (index, name) in columns.iter().enumerate() {
            use rusqlite::types::ValueRef;
            object.insert(
                name.clone(),
                match row.get_ref(index)? {
                    ValueRef::Null => Value::Null,
                    ValueRef::Integer(v) => json!(v),
                    ValueRef::Real(v) => json!(v),
                    ValueRef::Text(v) => json!(String::from_utf8_lossy(v)),
                    ValueRef::Blob(v) => json!({"bytes":v.len()}),
                },
            );
        }
        rows.push(Value::Object(object));
        if rows.len() > 200 {
            break;
        }
    }
    let truncated = rows.len() > 200;
    rows.truncate(200);
    Ok(json!({"columns":columns,"rows":rows,"truncated":truncated}))
}
