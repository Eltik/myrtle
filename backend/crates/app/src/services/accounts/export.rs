use std::collections::BTreeMap;

use chrono::Utc;
use myrtle_db::tools::{self, Column, Table};
use myrtle_db::{Conn, Db, DbError};
use myrtle_types::{Server, UserId};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::AppError;
use crate::privacy::{CREDENTIALS_TABLE, EXPORT_KIND, FORMAT_VERSION, USER_TABLES, UserTable};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserExport {
    pub kind: String,
    pub format_version: u32,
    pub exported_at: String,
    pub source: ExportSource,
    pub includes_credentials: bool,
    pub user: Map<String, Value>,
    pub counts: BTreeMap<String, usize>,
    pub tables: BTreeMap<String, Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSource {
    pub id: Uuid,
    pub uid: String,
    pub server_id: i16,
    pub server_code: String,
    pub nickname: Option<String>,
    pub database_version: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ImportOptions {
    pub replace: bool,
    pub dry_run: bool,
    pub suspend_triggers: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableLoad {
    Absent,
    Loaded { inserted: u64, skipped: u64 },
}

#[derive(Debug, Clone)]
pub struct ImportReport {
    pub existing: Option<(UserId, i64)>,
    pub target: UserId,
    pub cleared: u64,
    pub tables: Vec<(&'static str, TableLoad)>,
    pub committed: bool,
}

impl ImportReport {
    #[must_use]
    pub fn inserted(&self) -> u64 {
        self.loads().map(|(inserted, _)| inserted).sum()
    }

    #[must_use]
    pub fn skipped(&self) -> u64 {
        self.loads().map(|(_, skipped)| skipped).sum()
    }

    fn loads(&self) -> impl Iterator<Item = (u64, u64)> + '_ {
        self.tables.iter().filter_map(|(_, load)| match load {
            TableLoad::Loaded { inserted, skipped } => Some((*inserted, *skipped)),
            TableLoad::Absent => None,
        })
    }
}

#[must_use]
pub fn user_tables(include_credentials: bool) -> Vec<&'static UserTable> {
    let mut tables: Vec<&'static UserTable> = USER_TABLES.iter().collect();
    if include_credentials {
        tables.push(&CREDENTIALS_TABLE);
    }
    tables
}

#[must_use]
pub fn usable_columns(live: Vec<Column>, rows: &[Value], regenerate: &[&str]) -> Vec<Column> {
    live.into_iter()
        .filter(|c| !regenerate.contains(&c.name()))
        .filter(|c| {
            rows.iter()
                .any(|r| r.as_object().is_some_and(|m| m.contains_key(c.name())))
        })
        .collect()
}

pub fn parse_export(bytes: &[u8]) -> Result<UserExport, AppError> {
    let export: UserExport = serde_json::from_slice(bytes)
        .map_err(|e| AppError::bad_request(format!("not a user export: {e}")))?;
    if export.kind != EXPORT_KIND {
        return Err(AppError::bad_request(format!(
            "not a user export (kind {:?}, expected {EXPORT_KIND:?})",
            export.kind
        )));
    }
    if export.format_version != FORMAT_VERSION {
        return Err(AppError::bad_request(format!(
            "format_version {} is not supported (expected {FORMAT_VERSION})",
            export.format_version
        )));
    }
    Ok(export)
}

pub async fn export_user(
    db: &Db,
    user: UserId,
    include_credentials: bool,
) -> Result<UserExport, AppError> {
    let mut conn = db.conn().await?;
    let mut tx = tools::begin_snapshot(&mut conn).await?;
    let database_version = tools::server_version(&mut tx).await?;
    let account = tools::find_account(&mut tx, user)
        .await?
        .ok_or_else(|| AppError::NotFound(Some(format!("no user {}", user.0))))?;
    let Ok(Value::Object(user_row)) = serde_json::from_str(&tools::user_json(&mut tx, user).await?)
    else {
        return Err(AppError::internal("the users row is not a JSON object"));
    };
    let (mut counts, mut tables) = (BTreeMap::new(), BTreeMap::new());
    for table in user_tables(include_credentials) {
        let live = live_table(&mut tx, table.name).await?;
        let json = tools::scoped_json(&mut tx, &live, table.scope, user)
            .await
            .map_err(failed("read", table.name))?;
        let rows: Vec<Value> = serde_json::from_str(&json).map_err(AppError::internal)?;
        counts.insert(table.name.to_owned(), rows.len());
        tables.insert(table.name.to_owned(), rows);
    }
    drop(tx);
    Ok(UserExport {
        kind: EXPORT_KIND.to_owned(),
        format_version: FORMAT_VERSION,
        exported_at: Utc::now().to_rfc3339(),
        source: ExportSource {
            id: account.id,
            uid: account.uid,
            server_id: account.server_id,
            server_code: account.server_code,
            nickname: account.nickname,
            database_version,
        },
        includes_credentials: include_credentials,
        user: user_row,
        counts,
        tables,
    })
}

pub async fn import_user(
    db: &Db,
    export: &UserExport,
    options: ImportOptions,
) -> Result<ImportReport, AppError> {
    let uid = export
        .user
        .get("uid")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::bad_request("the users row has no string uid"))?;
    let server_id = export
        .user
        .get("server_id")
        .and_then(Value::as_i64)
        .ok_or_else(|| AppError::bad_request("the users row has no integer server_id"))?;
    let server = i16::try_from(server_id)
        .ok()
        .and_then(Server::from_index)
        .ok_or_else(|| AppError::bad_request(format!("unknown server_id {server_id}")))?;
    let selected = user_tables(export.includes_credentials);

    let mut tx = db.begin().await?;
    if options.suspend_triggers {
        tools::suspend_triggers(&mut tx).await?;
    }

    let existing = match tools::find_user_id(&mut tx, uid, server).await? {
        Some(id) => {
            let mut rows = 0;
            for table in &selected {
                let live = live_table(&mut tx, table.name).await?;
                rows += tools::count_scoped(&mut tx, &live, table.scope, id).await?;
            }
            if rows > 0 && !options.replace && !options.dry_run {
                return Err(AppError::Conflict(format!(
                    "user uid={uid} server={} already exists (id={}) with {rows} rows: pass --replace to overwrite, or --dry-run to preview",
                    server.as_str(),
                    id.0
                )));
            }
            Some((id, rows))
        }
        None => None,
    };

    let mut row = export.user.clone();
    match existing {
        Some((id, _)) => {
            row.insert("id".into(), json!(id.0));
        }
        None => {
            if tools::user_id_taken(&mut tx, UserId(export.source.id)).await? {
                row.remove("id");
            }
        }
    }
    let row = Value::Object(row);
    let users = live_table(&mut tx, "users").await?;
    let columns = usable_columns(
        tools::columns(&mut tx, &users).await?,
        std::slice::from_ref(&row),
        &[],
    );
    let target = tools::upsert_user_json(&mut tx, &columns, &row)
        .await
        .map_err(failed("upsert", "users"))?;

    let mut cleared = 0;
    for table in selected.iter().rev() {
        let live = live_table(&mut tx, table.name).await?;
        cleared += tools::delete_scoped(&mut tx, &live, table.scope, target)
            .await
            .map_err(failed("clear", table.name))?;
    }

    let mut tables = Vec::new();
    for table in &selected {
        let Some(rows) = export.tables.get(table.name) else {
            tables.push((table.name, TableLoad::Absent));
            continue;
        };
        if rows.is_empty() {
            continue;
        }
        let owned: Vec<Value> = rows
            .iter()
            .map(|row| match (row, table.owner_column()) {
                (Value::Object(m), Some(column)) => {
                    let mut m = m.clone();
                    m.insert(column.to_owned(), json!(target.0));
                    Value::Object(m)
                }
                _ => row.clone(),
            })
            .collect();
        let live = live_table(&mut tx, table.name).await?;
        let columns = usable_columns(
            tools::columns(&mut tx, &live).await?,
            &owned,
            table.regenerate,
        );
        if columns.is_empty() {
            return Err(AppError::bad_request(format!(
                "no column of {} in the file matches the live table",
                table.name
            )));
        }
        let submitted = owned.len() as u64;
        let inserted =
            tools::insert_rows(&mut tx, &live, &columns, table.guard, &Value::Array(owned))
                .await
                .map_err(failed("insert into", table.name))?;
        tables.push((
            table.name,
            TableLoad::Loaded {
                inserted,
                skipped: submitted.saturating_sub(inserted),
            },
        ));
    }

    if options.dry_run {
        tx.rollback().await?;
    } else {
        tx.commit().await?;
    }
    Ok(ImportReport {
        existing,
        target,
        cleared,
        tables,
        committed: !options.dry_run,
    })
}

async fn live_table(conn: &mut Conn, name: &str) -> Result<Table, AppError> {
    tools::table(conn, name)
        .await?
        .ok_or_else(|| AppError::internal(format!("table {name} does not exist")))
}

fn failed(verb: &'static str, table: &'static str) -> impl FnOnce(DbError) -> AppError {
    move |e| AppError::Internal(format!("{verb} {table}: {e}"))
}
