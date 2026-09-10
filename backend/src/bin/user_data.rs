//! Export one user's data to a single JSON file, and import it back.
//!
//! The whole-database pair (`export-database` / `import-database`) moves every
//! table as a directory of JSONL files. This binary moves ONE account as ONE file:
//! the `users` row plus every table that hangs off it (see
//! [`backend::user_export`] for the set and for what is deliberately left out).
//!
//! Usage:
//!   cargo run --release --bin user-data -- export --uid 123456789 [--server EN]
//!   cargo run --release --bin user-data -- export --user-id 8e3b...c1 -o alice.json
//!   cargo run --release --bin user-data -- import --file alice.json --dry-run
//!   cargo run --release --bin user-data -- import --file alice.json --replace
//!
//! Reads `DATABASE_URL` from the environment (or `.env`).
//!
//! Identity across databases:
//!   `users.id` is a per-database uuid, so it is NOT the key an import matches on.
//!   The portable identity is the natural key `(uid, server_id)`, which carries the
//!   UNIQUE constraint. Import resolves the target user by that pair and rewrites
//!   every child row's `user_id` to whatever `users.id` the target database uses.
//!   That is what makes prod -> local restores work.
//!
//! Import strategy:
//!   - One transaction; a failure anywhere leaves the database untouched.
//!   - The anchor `users` row is `UPSERTed` on `(uid, server_id)`, so an existing
//!     account keeps its id (and therefore its references from tier lists, audit
//!     rows, and anything else outside this export).
//!   - Child tables are then deleted in reverse topological order and replayed
//!     forward. This is a REPLACE, not a merge: the user ends up with exactly the
//!     rows in the file.
//!   - Each table's column list is the intersection of the columns in the file and
//!     the columns the live table actually has, so an export taken before or after
//!     a migration still loads - a column the file lacks takes its DEFAULT rather
//!     than being forced to NULL.
//!   - Triggers stay ENABLED. The audit triggers on `users` and `user_scores` then
//!     record the restore (with a NULL actor), and foreign keys are enforced on the
//!     way in, which is what we want for a few thousand rows. `--no-triggers`
//!     suspends both for a silent restore.
//!
//! Safety:
//!   - `--dry-run` performs the entire import and rolls back, printing the row
//!     counts it would have written. Use it first.
//!   - Overwriting an account that already holds data requires `--replace`.

// CLI tool. A bin is its own crate root, so it does not inherit the lint policy in
// lib.rs and repeats the relevant allows here:
//   - `main` dispatches two subcommands and the import pass is one linear
//     transaction, so both run long.
//   - the row counts Postgres reports back are non-negative by construction.
//   - the only float is a KiB figure printed for humans; precision loss above 2^52
//     bytes is not a concern for a file we hold in memory.
//   - `args`/`arg` in the parsers read better than any distinct pair would.
#![allow(
    clippy::too_many_lines,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::similar_names
)]

use anyhow::{Context, Result, bail};
use backend::user_export::{
    CREDENTIALS_TABLE, EXPORT_KIND, FORMAT_VERSION, USER_TABLES, UserTable,
};
use chrono::Utc;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::{Executor, PgPool, Postgres, Row, Transaction, postgres::PgPoolOptions, types::Json};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    time::Duration,
};
use uuid::Uuid;

/// The export envelope: exactly what lands in the single JSON file.
#[derive(Serialize, Deserialize)]
struct Export {
    /// Marker so pointing this at the wrong JSON file fails loudly.
    kind: String,
    format_version: u32,
    exported_at: String,
    /// Where the file came from - informational, and echoed on import so an
    /// operator can see which database and account they are restoring.
    source: SourceInfo,
    /// True when `user_game_credentials` was included.
    includes_credentials: bool,
    /// The anchor `users` row.
    user: Map<String, Value>,
    /// Row counts per table, so a file can be inspected without parsing the body.
    counts: BTreeMap<String, usize>,
    /// table name -> rows. Ordered map keeps the file diffable between exports.
    tables: BTreeMap<String, Vec<Value>>,
}

#[derive(Serialize, Deserialize)]
struct SourceInfo {
    /// `users.id` in the SOURCE database. Not used to match on import.
    id: Uuid,
    uid: String,
    server_id: i16,
    server_code: String,
    nickname: Option<String>,
    database_version: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let mut args = std::env::args().skip(1);
    let sub = args.next().unwrap_or_default();
    let rest: Vec<String> = args.collect();

    match sub.as_str() {
        "export" => run_export(parse_export_args(&rest)?).await,
        "import" => run_import(parse_import_args(&rest)?).await,
        "-h" | "--help" | "" => {
            print_help();
            Ok(())
        }
        other => {
            print_help();
            bail!("unknown subcommand: {other}");
        }
    }
}

async fn connect() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .context("failed to connect to database")
}

// ── export ──────────────────────────────────────────────────────────────────

struct ExportArgs {
    uid: Option<String>,
    user_id: Option<Uuid>,
    server: Option<String>,
    output: Option<PathBuf>,
    include_credentials: bool,
}

async fn run_export(args: ExportArgs) -> Result<()> {
    let pool = connect().await?;
    let mut conn = pool.acquire().await?;

    // One snapshot for the whole export, so the tables cannot disagree with each
    // other if the user syncs midway through.
    conn.execute("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .await
        .context("failed to begin snapshot transaction")?;

    let source = resolve_user(&mut conn, &args).await?;
    println!(
        "exporting {} (uid={} server={} id={})",
        source.nickname.as_deref().unwrap_or("<no nickname>"),
        source.uid,
        source.server_code,
        source.id
    );

    let user = fetch_user_row(&mut conn, source.id).await?;

    let mut tables = BTreeMap::new();
    let mut counts = BTreeMap::new();
    let mut total = 0usize;

    let selected = selected_tables(args.include_credentials);
    for table in selected {
        let rows = fetch_table(&mut conn, table, source.id).await?;
        println!("  {:<24} {:>7} rows", table.name, rows.len());
        total += rows.len();
        counts.insert(table.name.to_string(), rows.len());
        tables.insert(table.name.to_string(), rows);
    }

    conn.execute("COMMIT").await.ok();

    let output = args.output.unwrap_or_else(|| {
        PathBuf::from(format!(
            "user_{}_{}_{}.json",
            source.uid,
            source.server_code.to_lowercase(),
            Utc::now().timestamp()
        ))
    });

    let export = Export {
        kind: EXPORT_KIND.to_string(),
        format_version: FORMAT_VERSION,
        exported_at: Utc::now().to_rfc3339(),
        includes_credentials: args.include_credentials,
        source,
        user,
        counts,
        tables,
    };

    let file =
        File::create(&output).with_context(|| format!("failed to create {}", output.display()))?;
    serde_json::to_writer_pretty(BufWriter::with_capacity(1 << 20, file), &export)
        .with_context(|| format!("failed to write {}", output.display()))?;

    let bytes = std::fs::metadata(&output).map_or(0, |m| m.len());
    println!(
        "\nWrote {total} rows across {} tables ({:.1} KiB) -> {}",
        export.tables.len(),
        bytes as f64 / 1024.0,
        output.display()
    );
    if args.include_credentials {
        println!(
            "WARNING: this file contains the user's encrypted game credentials. They \
             decrypt only under the same GAME_CREDENTIAL_KEY and the same users.id."
        );
    }
    Ok(())
}

/// Tables to move, in topological order. Credentials go last: nothing references
/// them, and keeping them at the tail makes the opt-in obvious in the output.
fn selected_tables(include_credentials: bool) -> Vec<&'static UserTable> {
    let mut v: Vec<&'static UserTable> = USER_TABLES.iter().collect();
    if include_credentials {
        v.push(&CREDENTIALS_TABLE);
    }
    v
}

async fn resolve_user(conn: &mut sqlx::PgConnection, args: &ExportArgs) -> Result<SourceInfo> {
    let database_version: String = sqlx::query_scalar("SHOW server_version")
        .fetch_one(&mut *conn)
        .await?;

    let base = "SELECT u.id, u.uid, u.server_id, s.code AS server_code, u.nickname \
                FROM users u JOIN servers s ON s.id = u.server_id ";

    let mut rows: Vec<(Uuid, String, i16, String, Option<String>)> = if let Some(id) = args.user_id
    {
        sqlx::query_as(&format!("{base} WHERE u.id = $1"))
            .bind(id)
            .fetch_all(&mut *conn)
            .await
            .context("user lookup by id failed")?
    } else {
        let uid = args
            .uid
            .as_ref()
            .expect("parse_export_args guarantees one of the two");
        sqlx::query_as(&format!("{base} WHERE u.uid = $1 ORDER BY u.created_at"))
            .bind(uid)
            .fetch_all(&mut *conn)
            .await
            .context("user lookup by uid failed")?
    };

    if let Some(server) = &args.server {
        rows.retain(|r| r.3.eq_ignore_ascii_case(server));
    }

    match rows.len() {
        0 => bail!(
            "no user found for {}",
            args.user_id.map_or_else(
                || format!(
                    "uid={}{}",
                    args.uid.as_deref().unwrap_or("?"),
                    args.server
                        .as_deref()
                        .map_or(String::new(), |s| format!(" server={s}"))
                ),
                |id| format!("id={id}")
            )
        ),
        1 => {}
        _ => {
            // Ambiguity here would silently export the wrong account, so refuse
            // rather than guessing at one.
            let servers: Vec<&str> = rows.iter().map(|r| r.3.as_str()).collect();
            bail!(
                "uid={} exists on multiple servers ({}); pass --server to choose",
                args.uid.as_deref().unwrap_or("?"),
                servers.join(", ")
            );
        }
    }

    let (id, uid, server_id, server_code, nickname) = rows.remove(0);
    Ok(SourceInfo {
        id,
        uid,
        server_id,
        server_code,
        nickname,
        database_version,
    })
}

async fn fetch_user_row(conn: &mut sqlx::PgConnection, id: Uuid) -> Result<Map<String, Value>> {
    let raw: String =
        sqlx::query_scalar("SELECT row_to_json(t)::text FROM users t WHERE t.id = $1")
            .bind(id)
            .fetch_one(&mut *conn)
            .await
            .context("failed to read users row")?;
    match serde_json::from_str(&raw).context("failed to parse users row")? {
        Value::Object(m) => Ok(m),
        _ => bail!("users row did not decode to a JSON object"),
    }
}

async fn fetch_table(
    conn: &mut sqlx::PgConnection,
    table: &UserTable,
    user_id: Uuid,
) -> Result<Vec<Value>> {
    // Ordering by the row's own JSON text makes repeated exports of an unchanged
    // account byte-identical, so two files can be diffed directly.
    let sql = format!(
        "SELECT COALESCE(json_agg(j ORDER BY j::text), '[]'::json)::text \
         FROM (SELECT row_to_json(t) AS j FROM {} t WHERE {}) q",
        table.name, table.filter
    );
    let raw: String = sqlx::query_scalar(&sql)
        .bind(user_id)
        .fetch_one(&mut *conn)
        .await
        .with_context(|| format!("failed to read {}", table.name))?;
    let rows: Vec<Value> = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse rows of {}", table.name))?;
    Ok(rows)
}

// ── import ──────────────────────────────────────────────────────────────────

struct ImportArgs {
    file: PathBuf,
    replace: bool,
    dry_run: bool,
    no_triggers: bool,
}

async fn run_import(args: ImportArgs) -> Result<()> {
    let export = load_export(&args.file)?;

    let pool = connect().await?;
    let mut tx = pool.begin().await.context("failed to begin transaction")?;

    if args.no_triggers {
        // Same lever the whole-database importer pulls: suspends user triggers AND
        // foreign-key enforcement for this transaction.
        tx.execute("SET LOCAL session_replication_role = replica")
            .await
            .context("failed to set session_replication_role")?;
    }

    println!(
        "file: {} (exported {} from {} uid={} server={})",
        args.file.display(),
        export.exported_at,
        export.source.id,
        export.source.uid,
        export.source.server_code
    );

    // The natural key is what travels; the source uuid is only a hint.
    let uid = string_field(&export.user, "uid")?;
    let server_id = server_id_field(&export.user, "server_id")?;

    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE uid = $1 AND server_id = $2")
            .bind(&uid)
            .bind(server_id)
            .fetch_optional(&mut *tx)
            .await
            .context("failed to look up target user")?;

    if let Some(id) = existing {
        let rows = count_existing(&mut tx, id, export.includes_credentials).await?;
        // A dry run rolls back unconditionally, so it needs no permission to
        // overwrite - gating it here would make the very command this error
        // recommends impossible to run.
        if rows > 0 && !args.replace && !args.dry_run {
            bail!(
                "user uid={uid} server_id={server_id} already exists (id={id}) with {rows} rows. \
                 Re-run with --replace to overwrite, or --dry-run to preview."
            );
        }
        println!("target: existing user id={id} ({rows} rows will be replaced)");
    } else {
        println!("target: no such user yet - it will be created");
    }

    let target_id = upsert_user(&mut tx, &export, existing).await?;
    println!("anchor: users.id = {target_id}");

    let selected = selected_tables(export.includes_credentials);

    // Delete children in reverse topological order so no FK is ever left dangling
    // mid-transaction, even with triggers and FK checks enabled.
    let mut deleted_total = 0u64;
    for table in selected.iter().rev() {
        let sql = format!("DELETE FROM {} t WHERE {}", table.name, table.filter);
        let n = sqlx::query(&sql)
            .bind(target_id)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("failed to clear {}", table.name))?
            .rows_affected();
        deleted_total += n;
    }
    println!("cleared {deleted_total} existing rows");

    let mut inserted_total = 0u64;
    let mut skipped_total = 0u64;
    for table in &selected {
        let Some(rows) = export.tables.get(table.name) else {
            // A file written by an older build simply lacks the table. Treat it as
            // empty rather than aborting a restore that is otherwise complete.
            println!("  {:<24} {:>7}  (absent from file)", table.name, "-");
            continue;
        };
        if rows.is_empty() {
            continue;
        }
        let (inserted, skipped) = insert_rows(&mut tx, table, rows, target_id).await?;
        inserted_total += inserted;
        skipped_total += skipped;
        let note = if skipped > 0 {
            format!("  ({skipped} skipped: referenced row absent)")
        } else {
            String::new()
        };
        println!("  {:<24} {inserted:>7} rows{note}", table.name);
    }

    if args.dry_run {
        tx.rollback().await.context("failed to roll back dry run")?;
        println!(
            "\nDRY RUN: rolled back. Would have written {inserted_total} rows \
             (and skipped {skipped_total}) for uid={uid} server={server_id}."
        );
        return Ok(());
    }

    tx.commit().await.context("failed to commit import")?;
    println!(
        "\nImported {inserted_total} rows for uid={uid} server={server_id} (users.id={target_id})."
    );
    if skipped_total > 0 {
        println!(
            "{skipped_total} row(s) were skipped because the tier list they reference does not \
             exist in this database."
        );
    }
    if export.includes_credentials && target_id != export.source.id {
        println!(
            "NOTE: users.id changed ({} -> {target_id}), so the imported game credentials \
             cannot be decrypted - they are sealed against the original id.",
            export.source.id
        );
    }
    Ok(())
}

fn load_export(path: &Path) -> Result<Export> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let export: Export = serde_json::from_reader(BufReader::with_capacity(1 << 20, file))
        .with_context(|| format!("failed to parse {}", path.display()))?;

    if export.kind != EXPORT_KIND {
        bail!(
            "{} is not a user export (kind={:?}, expected {EXPORT_KIND:?})",
            path.display(),
            export.kind
        );
    }
    if export.format_version != FORMAT_VERSION {
        bail!(
            "format_version {} is not supported (expected {FORMAT_VERSION})",
            export.format_version
        );
    }
    Ok(export)
}

async fn count_existing(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    include_credentials: bool,
) -> Result<i64> {
    let mut total = 0i64;
    for table in selected_tables(include_credentials) {
        let sql = format!(
            "SELECT COUNT(*) FROM {} t WHERE {}",
            table.name, table.filter
        );
        let n: i64 = sqlx::query_scalar(&sql)
            .bind(user_id)
            .fetch_one(&mut **tx)
            .await
            .with_context(|| format!("failed to count {}", table.name))?;
        total += n;
    }
    Ok(total)
}

/// UPSERT the anchor row on `(uid, server_id)` and return the id the rest of the
/// import must hang off.
///
/// When the account already exists we keep its id rather than the file's: outside
/// references (tier lists, audit rows) point at it, and rewriting it would orphan
/// them. When it does not, the file's id is reused so a round-trip into an empty
/// database is exact - unless some unrelated row already holds that uuid, in which
/// case we let the database mint a fresh one.
async fn upsert_user(
    tx: &mut Transaction<'_, Postgres>,
    export: &Export,
    existing: Option<Uuid>,
) -> Result<Uuid> {
    let mut row = export.user.clone();

    if let Some(id) = existing {
        row.insert("id".into(), json!(id));
    } else {
        let source_id = export.source.id;
        let taken: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE id = $1")
            .bind(source_id)
            .fetch_optional(&mut **tx)
            .await
            .context("failed to probe users.id")?;
        if taken.is_some() {
            // Another account owns this uuid here; drop the key so the column
            // DEFAULT (gen_random_uuid) supplies a free one.
            row.remove("id");
        }
    }

    let live = live_columns(tx, "users").await?;
    let as_rows = [Value::Object(row.clone())];
    let cols = usable_columns(&live, &as_rows, &[]);
    if !cols.iter().any(|c| c == "uid") || !cols.iter().any(|c| c == "server_id") {
        bail!("the export's users row is missing uid/server_id; it cannot be matched");
    }

    // Everything except the identity and the creation stamp is refreshed from the
    // file; `id` must not move, and `created_at` records when the account first
    // appeared here, not when the file was written.
    let updates: Vec<String> = cols
        .iter()
        .filter(|c| !matches!(c.as_str(), "id" | "uid" | "server_id" | "created_at"))
        .map(|c| format!("{c} = EXCLUDED.{c}"))
        .collect();
    let conflict = if updates.is_empty() {
        "DO NOTHING".to_string()
    } else {
        format!("DO UPDATE SET {}", updates.join(", "))
    };

    let list = cols.join(", ");
    let sql = format!(
        "INSERT INTO users ({list}) \
         SELECT {list} FROM jsonb_populate_recordset(NULL::users, $1::jsonb) \
         ON CONFLICT (uid, server_id) {conflict} \
         RETURNING id"
    );

    let payload = Value::Array(vec![Value::Object(row)]);
    let id: Uuid = sqlx::query_scalar(&sql)
        .bind(Json(payload))
        .fetch_one(&mut **tx)
        .await
        .context("failed to upsert users row")?;
    Ok(id)
}

/// Insert one table's rows, returning `(inserted, skipped)`.
async fn insert_rows(
    tx: &mut Transaction<'_, Postgres>,
    table: &UserTable,
    rows: &[Value],
    target_id: Uuid,
) -> Result<(u64, u64)> {
    // Point every row at the user this database knows, not the one the file came
    // from. Without this a cross-database restore inserts orphans.
    let owned: Vec<Value> = rows
        .iter()
        .map(|row| match (row, table.owner_column) {
            (Value::Object(m), Some(col)) => {
                let mut m = m.clone();
                m.insert(col.to_string(), json!(target_id));
                Value::Object(m)
            }
            _ => row.clone(),
        })
        .collect();

    let live = live_columns(tx, table.name).await?;
    let cols = usable_columns(&live, &owned, table.regenerate);
    if cols.is_empty() {
        bail!(
            "no column of {} in the file matches the live table; refusing to insert",
            table.name
        );
    }

    let list = cols.join(", ");
    let guard = table
        .import_guard
        .map_or(String::new(), |g| format!(" WHERE {g}"));
    let sql = format!(
        "INSERT INTO {} ({list}) \
         SELECT {list} FROM jsonb_populate_recordset(NULL::{}, $1::jsonb){guard}",
        table.name, table.name
    );

    let submitted = owned.len() as u64;
    let inserted = sqlx::query(&sql)
        .bind(Json(Value::Array(owned)))
        .execute(&mut **tx)
        .await
        .with_context(|| format!("failed to insert into {}", table.name))?
        .rows_affected();

    Ok((inserted, submitted.saturating_sub(inserted)))
}

/// Columns the live table actually has, in physical order.
async fn live_columns(tx: &mut Transaction<'_, Postgres>, table: &str) -> Result<Vec<String>> {
    let rows = sqlx::query(
        "SELECT a.attname::text AS name FROM pg_attribute a \
         WHERE a.attrelid = $1::regclass AND a.attnum > 0 AND NOT a.attisdropped \
         ORDER BY a.attnum",
    )
    .bind(table)
    .fetch_all(&mut **tx)
    .await
    .with_context(|| format!("failed to read columns of {table}"))?;

    rows.into_iter()
        .map(|r| r.try_get::<String, _>("name").map_err(Into::into))
        .collect()
}

/// The columns to write: present in the live table AND in the file, minus the ones
/// the database should reassign.
///
/// Intersecting both ways is what makes the format survive a migration in either
/// direction. A column the file does not carry is left out of the INSERT so it
/// takes its DEFAULT - listing it would force a NULL and trip a NOT NULL that the
/// default was there to satisfy. A column the file carries but the table has lost
/// is simply dropped.
fn usable_columns(live: &[String], rows: &[Value], regenerate: &[&str]) -> Vec<String> {
    live.iter()
        .filter(|c| !regenerate.contains(&c.as_str()))
        .filter(|c| {
            rows.iter()
                .any(|r| r.as_object().is_some_and(|m| m.contains_key(c.as_str())))
        })
        .cloned()
        .collect()
}

fn string_field(row: &Map<String, Value>, key: &str) -> Result<String> {
    row.get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .with_context(|| format!("export's users row is missing a string `{key}`"))
}

/// Read `server_id` as the `smallint` the column actually is.
///
/// Checked rather than cast: a truncating `as i16` would turn a malformed 65538
/// into 2, quietly matching a real server and restoring the account onto it.
fn server_id_field(row: &Map<String, Value>, key: &str) -> Result<i16> {
    let raw = row
        .get(key)
        .and_then(Value::as_i64)
        .with_context(|| format!("export's users row is missing an integer `{key}`"))?;
    i16::try_from(raw).with_context(|| format!("`{key}` is out of range for smallint: {raw}"))
}

// ── argument parsing ────────────────────────────────────────────────────────

fn parse_export_args(argv: &[String]) -> Result<ExportArgs> {
    let mut args = ExportArgs {
        uid: None,
        user_id: None,
        server: None,
        output: None,
        include_credentials: false,
    };
    let mut it = argv.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--uid" => args.uid = Some(it.next().context("--uid requires a value")?.clone()),
            "--user-id" => {
                let raw = it.next().context("--user-id requires a value")?;
                args.user_id = Some(Uuid::parse_str(raw).context("--user-id must be a UUID")?);
            }
            "--server" => {
                args.server = Some(it.next().context("--server requires a value")?.clone());
            }
            "--output" | "-o" => {
                args.output = Some(PathBuf::from(
                    it.next().context("--output requires a value")?,
                ));
            }
            "--include-credentials" => args.include_credentials = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => bail!("unknown argument: {other}"),
        }
    }
    if args.uid.is_none() && args.user_id.is_none() {
        print_help();
        bail!("export requires --uid or --user-id");
    }
    if args.uid.is_some() && args.user_id.is_some() {
        bail!("--uid and --user-id are mutually exclusive");
    }
    Ok(args)
}

fn parse_import_args(argv: &[String]) -> Result<ImportArgs> {
    let mut file: Option<PathBuf> = None;
    let mut replace = false;
    let mut dry_run = false;
    let mut no_triggers = false;
    let mut it = argv.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--file" | "-f" => {
                file = Some(PathBuf::from(it.next().context("--file requires a path")?));
            }
            "--replace" => replace = true,
            "--dry-run" => dry_run = true,
            "--no-triggers" => no_triggers = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => bail!("unknown argument: {other}"),
        }
    }
    Ok(ImportArgs {
        file: file.context("import requires --file <path>")?,
        replace,
        dry_run,
        no_triggers,
    })
}

fn print_help() {
    eprintln!(
        "Usage: user-data export [--uid <uid> | --user-id <uuid>] [--server <code>] [-o <path>]\n\
         \x20      user-data import --file <path> [--replace] [--dry-run] [--no-triggers]\n\
         \n\
         Moves ONE account as ONE JSON file: the users row plus every table that\n\
         hangs off it. Import matches the target on (uid, server_id) and rewrites\n\
         each row's user_id, so a file exported from one database restores into\n\
         another.\n\
         \n\
         export options:\n\
         \x20 --uid <uid>              Arknights UID\n\
         \x20 --user-id <uuid>         Internal database UUID\n\
         \x20 --server <code>          Required when the UID exists on several servers\n\
         \x20 -o, --output <path>      Output file (default: user_<uid>_<server>_<unix>.json)\n\
         \x20 --include-credentials    Also export the encrypted game credentials.\n\
         \x20                          Sensitive, and only usable in a database that\n\
         \x20                          shares GAME_CREDENTIAL_KEY and the same users.id.\n\
         \n\
         import options:\n\
         \x20 -f, --file <path>        Export file to load\n\
         \x20 --replace                Required to overwrite an account that has data\n\
         \x20 --dry-run                Run the whole import, report it, then roll back\n\
         \x20 --no-triggers            Suspend audit triggers and FK checks for the load\n\
         \n\
         Reads DATABASE_URL from the environment (or .env)."
    );
}
