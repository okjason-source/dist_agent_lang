//! Production SQLite backend for `database::*` (requires `sqlite-storage` feature).
//! Connection strings: `sqlite:///absolute/path.db` or `sqlite://relative/path.db` (no `..`).

use crate::runtime::values::Value;
use crate::stdlib::database::{ColumnSchema, QueryResult, TableSchema};
use rusqlite::types::ValueRef;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Parse `sqlite://...` into a resolved path; rejects `..`.
pub fn parse_sqlite_path(connection_string: &str) -> Result<PathBuf, String> {
    let s = connection_string.trim();
    let prefix = "sqlite://";
    if !s.starts_with(prefix) {
        return Err(format!(
            "Unsupported connection string (use {}path); PostgreSQL/MySQL are not wired yet",
            prefix
        ));
    }
    let rest = s[prefix.len()..].trim();
    if rest.is_empty() {
        return Err("Empty path after sqlite://".to_string());
    }
    if rest.contains("..") {
        return Err("sqlite path may not contain ..".to_string());
    }
    let path = if rest.starts_with('/') {
        PathBuf::from(rest)
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(rest)
    };
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    Ok(path)
}

pub fn open_sqlite(path: &Path) -> Result<Arc<Mutex<Connection>>, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
        .map_err(|e| e.to_string())?;
    Ok(Arc::new(Mutex::new(conn)))
}

fn value_to_param(v: &Value) -> rusqlite::types::Value {
    match v {
        Value::Int(i) => rusqlite::types::Value::Integer(*i),
        Value::Float(f) => rusqlite::types::Value::Real(*f),
        Value::String(s) => rusqlite::types::Value::Text(s.clone()),
        Value::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
        Value::Null => rusqlite::types::Value::Null,
        Value::List(a) | Value::Array(a) if a.len() == 1 => value_to_param(&a[0]),
        other => rusqlite::types::Value::Text(format!("{}", other)),
    }
}

fn value_ref_to_value(v: ValueRef<'_>) -> Value {
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::Int(i),
        ValueRef::Real(f) => Value::Float(f),
        ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => Value::String(format!("<blob {} bytes>", b.len())),
    }
}

/// Run SQL with bound parameters. Uses readonly() to choose query vs execute.
pub fn run_sql(
    conn: &Arc<Mutex<Connection>>,
    sql: String,
    params: Vec<Value>,
) -> Result<QueryResult, String> {
    let conn = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rusqlite_params: Vec<rusqlite::types::Value> = params.iter().map(value_to_param).collect();

    if stmt.readonly() {
        let column_names: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let mut rows_out = Vec::new();
        let params_slice: Vec<&dyn rusqlite::ToSql> = rusqlite_params
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();
        let mut rows = stmt
            .query(rusqlite::params_from_iter(params_slice))
            .map_err(|e| e.to_string())?;
        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let mut map = HashMap::new();
            for (i, name) in column_names.iter().enumerate() {
                let v = row.get_ref(i).map_err(|e| e.to_string())?;
                map.insert(name.clone(), value_ref_to_value(v));
            }
            rows_out.push(map);
        }
        let n = rows_out.len() as i64;
        Ok(QueryResult {
            rows: rows_out,
            row_count: n,
            affected_rows: 0,
        })
    } else {
        let params_slice: Vec<&dyn rusqlite::ToSql> = rusqlite_params
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();
        let ar = stmt
            .execute(rusqlite::params_from_iter(params_slice))
            .map_err(|e| e.to_string())?;
        Ok(QueryResult {
            rows: Vec::new(),
            row_count: 0,
            affected_rows: ar as i64,
        })
    }
}

pub fn list_tables_sqlite(conn: &Arc<Mutex<Connection>>) -> Result<Vec<String>, String> {
    let conn = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type IN ('table','view') AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn table_schema_sqlite(
    conn: &Arc<Mutex<Connection>>,
    table_name: &str,
) -> Result<TableSchema, String> {
    if !table_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Invalid table name (use [a-zA-Z0-9_]+)".to_string());
    }
    let conn = conn.lock().map_err(|e| e.to_string())?;
    let pragma = format!("PRAGMA table_info(\"{}\")", table_name);
    let mut stmt = conn.prepare(&pragma).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, i32>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut columns = Vec::new();
    for r in rows {
        let (name, typ, notnull, dflt, pk) = r.map_err(|e| e.to_string())?;
        columns.push(ColumnSchema {
            name,
            data_type: typ,
            is_nullable: notnull == 0,
            is_primary_key: pk != 0,
            default_value: dflt,
        });
    }
    if columns.is_empty() {
        return Err(format!(
            "Table '{}' not found or has no columns",
            table_name
        ));
    }
    Ok(TableSchema {
        name: table_name.to_string(),
        columns,
    })
}

pub fn ping_sqlite(conn: &Arc<Mutex<Connection>>) -> Result<bool, String> {
    let conn = conn.lock().map_err(|e| e.to_string())?;
    conn.query_row("SELECT 1", [], |row| row.get::<_, i32>(0))
        .map_err(|e| e.to_string())?;
    Ok(true)
}

pub fn explain_sqlite(
    conn: &Arc<Mutex<Connection>>,
    sql: String,
) -> Result<HashMap<String, Value>, String> {
    let conn = conn.lock().map_err(|e| e.to_string())?;
    let explain_sql = format!("EXPLAIN QUERY PLAN {}", sql);
    let mut stmt = conn.prepare(&explain_sql).map_err(|e| e.to_string())?;
    let ncols = stmt.column_count();
    let mut rows_out = Vec::new();
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let mut line = String::new();
        for i in 0..ncols {
            if let Ok(v) = row.get_ref(i) {
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(&format!("{:?}", v));
            }
        }
        rows_out.push(line);
    }
    let mut plan = HashMap::new();
    plan.insert("plan_lines".to_string(), Value::String(rows_out.join("\n")));
    plan.insert(
        "detail".to_string(),
        Value::String("SQLite EXPLAIN QUERY PLAN".to_string()),
    );
    Ok(plan)
}

pub fn backup_sqlite_file(src: &Arc<Mutex<Connection>>, dest_path: &Path) -> Result<(), String> {
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let src_conn = src.lock().map_err(|e| e.to_string())?;
    let mut dest = Connection::open(dest_path).map_err(|e| e.to_string())?;
    let backup = rusqlite::backup::Backup::new(&*src_conn, &mut dest)
        .map_err(|e: rusqlite::Error| e.to_string())?;
    backup
        .run_to_completion(100, Duration::from_millis(100), None)
        .map_err(|e: rusqlite::Error| e.to_string())?;
    Ok(())
}

pub fn restore_sqlite_file(db_path: &Path, backup_path: &Path) -> Result<(), String> {
    if !backup_path.exists() {
        return Err(format!("Backup not found: {:?}", backup_path));
    }
    std::fs::copy(backup_path, db_path).map_err(|e| e.to_string())?;
    Ok(())
}
