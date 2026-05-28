//! Interactive CLI for managing user roles and per-tier-list permissions.
//!
//! Usage:
//!   cargo run --release --bin manage-permissions
//!
//! Reads `DATABASE_URL` from the environment (or `.env`). If the optional
//! `ADMIN_UID` env var is set to an Arknights UID, that user is recorded as
//! `granted_by` on new permission grants; otherwise the grant records NULL.

use anyhow::{Context, Result};
use backend::{
    core::auth::permissions::{GlobalRole, Permission},
    database::queries,
};
use dotenv::dotenv;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{
    io::{self, Write},
    time::Duration,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("\n========================================");
    println!("   Myrtle Permission Management Tool");
    println!("========================================\n");

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;
    println!("Connected to database.\n");

    // Optional operator identity - stamped onto grants so audit_log shows who
    // ran the tool. Missing/unknown UID just becomes NULL.
    let operator = if let Some(uid) = std::env::var("ADMIN_UID").ok().filter(|s| !s.is_empty()) { if let Some(u) = find_user_by_id_or_uid(&pool, &uid).await? {
        println!("Operating as: {} ({})\n", u.uid, display_name(&u));
        Some(u.id)
    } else {
        println!("ADMIN_UID '{uid}' not found — grants will record NULL.\n");
        None
    } } else {
        println!("ADMIN_UID not set — grants will record NULL.\n");
        None
    };

    loop {
        println!("What would you like to do?");
        println!("  1. Search users");
        println!("  2. Update user role");
        println!("  3. Manage tier list permissions");
        println!("  4. List tier lists");
        println!("  5. Toggle tier list visibility (is_listed)");
        println!("  6. Exit");
        print!("\nChoice: ");
        io::stdout().flush()?;

        match read_line()?.as_str() {
            "1" => search_users(&pool).await?,
            "2" => update_user_role(&pool).await?,
            "3" => manage_tier_list_permissions(&pool, operator).await?,
            "4" => list_tier_lists(&pool).await?,
            "5" => toggle_visibility(&pool).await?,
            "6" | "q" | "exit" => {
                println!("\nGoodbye!");
                break;
            }
            _ => println!("\nInvalid choice. Please try again.\n"),
        }
    }

    Ok(())
}

// ─── User browsing / role updates ───────────────────────────────────────────

async fn search_users(pool: &PgPool) -> Result<()> {
    println!("\n--- Search Users ---");
    print!("Enter search term (uid, nickname, or blank for recent 20): ");
    io::stdout().flush()?;
    let search = read_line()?;

    let users = if search.is_empty() {
        sqlx::query_as::<_, UserRow>(USER_SELECT_RECENT)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as::<_, UserRow>(USER_SELECT_SEARCH)
            .bind(format!("%{search}%"))
            .fetch_all(pool)
            .await?
    };

    print_user_table(&users);
    Ok(())
}

async fn update_user_role(pool: &PgPool) -> Result<()> {
    println!("\n--- Update User Role ---");
    print!("Enter user ID (UUID) or Arknights UID: ");
    io::stdout().flush()?;
    let input = read_line()?;
    if input.is_empty() {
        println!("Cancelled.\n");
        return Ok(());
    }

    let Some(user) = find_user_by_id_or_uid(pool, &input).await? else {
        println!("User not found.\n");
        return Ok(());
    };

    println!("\nUser found:");
    println!("  ID:       {}", user.id);
    println!("  UID:      {}", user.uid);
    println!("  Nickname: {}", display_name(&user));
    println!("  Server:   {}", user.server);
    println!("  Role:     {}", user.role);

    let Some(new_role) = prompt_choice("\nAvailable roles:", GlobalRole::all(), describe_role)?
    else {
        println!("Cancelled.\n");
        return Ok(());
    };

    if new_role.to_string() == user.role {
        println!("User already has this role.\n");
        return Ok(());
    }

    print!("Update role from '{}' to '{new_role}'? (y/n): ", user.role);
    io::stdout().flush()?;
    if !confirm()? {
        println!("Cancelled.\n");
        return Ok(());
    }

    queries::users::update_role(pool, user.id, &new_role.to_string())
        .await
        .context("update_role")?;
    println!("\nRole updated successfully!\n");
    Ok(())
}

// ─── Tier list permissions ──────────────────────────────────────────────────

async fn manage_tier_list_permissions(pool: &PgPool, operator: Option<Uuid>) -> Result<()> {
    println!("\n--- Manage Tier List Permissions ---");

    let lists = sqlx::query_as::<_, TierListRow>(
        "SELECT id, name, slug FROM tier_lists WHERE is_active = true ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    if lists.is_empty() {
        println!("No tier lists found. Create one first.\n");
        return Ok(());
    }

    println!("\nAvailable tier lists:");
    for (i, tl) in lists.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, tl.name, tl.slug);
    }
    let Some(tier_list) = prompt_index("\nSelect tier list (number) or 'c' to cancel: ", &lists)?
    else {
        println!("Cancelled.\n");
        return Ok(());
    };
    println!("\nSelected: {} ({})", tier_list.name, tier_list.slug);

    let permissions = fetch_permissions(pool, tier_list.id).await?;
    print_permission_table(&permissions);

    println!("\nActions:");
    println!("  1. Grant permission");
    println!("  2. Revoke permission");
    println!("  3. Back");
    print!("\nChoice: ");
    io::stdout().flush()?;
    match read_line()?.as_str() {
        "1" => grant_permission(pool, tier_list.id, operator).await?,
        "2" => revoke_permission(pool, tier_list.id, &permissions).await?,
        _ => println!(),
    }
    Ok(())
}

async fn grant_permission(pool: &PgPool, tier_list_id: Uuid, operator: Option<Uuid>) -> Result<()> {
    print!("\nEnter user ID (UUID) or Arknights UID: ");
    io::stdout().flush()?;
    let input = read_line()?;
    if input.is_empty() {
        println!("Cancelled.\n");
        return Ok(());
    }

    let Some(user) = find_user_by_id_or_uid(pool, &input).await? else {
        println!("User not found.\n");
        return Ok(());
    };
    println!("\nUser: {} ({})", user.uid, display_name(&user));

    let Some(perm) = prompt_choice(
        "\nPermission levels:",
        Permission::all(),
        describe_permission,
    )?
    else {
        println!("Cancelled.\n");
        return Ok(());
    };

    // Inline SQL (not the helper) so granted_by can be NULL when no operator
    // is configured. The ON CONFLICT keeps re-running the tool idempotent.
    sqlx::query(
        "INSERT INTO tier_list_permissions (tier_list_id, user_id, permission, granted_by)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (tier_list_id, user_id, permission) DO UPDATE SET granted_at = NOW()",
    )
    .bind(tier_list_id)
    .bind(user.id)
    .bind(perm.to_string())
    .bind(operator)
    .execute(pool)
    .await
    .context("grant_permission insert")?;

    println!("\nGranted '{perm}' to {}.\n", user.uid);
    Ok(())
}

async fn revoke_permission(
    pool: &PgPool,
    tier_list_id: Uuid,
    permissions: &[PermissionRow],
) -> Result<()> {
    if permissions.is_empty() {
        println!("No permissions to revoke.\n");
        return Ok(());
    }

    print!("\nEnter user ID (UUID) or Arknights UID to revoke from: ");
    io::stdout().flush()?;
    let input = read_line()?;
    if input.is_empty() {
        println!("Cancelled.\n");
        return Ok(());
    }

    let matching: Vec<&PermissionRow> = permissions
        .iter()
        .filter(|p| p.user_id.to_string() == input || p.uid == input)
        .collect();

    if matching.is_empty() {
        println!("No permissions found for this user on this tier list.\n");
        return Ok(());
    }

    println!("\nPermissions for this user:");
    for (i, p) in matching.iter().enumerate() {
        println!("  {}. {}", i + 1, p.permission);
    }
    print!("\nSelect permission to revoke (number) or 'all': ");
    io::stdout().flush()?;
    let choice = read_line()?;

    if choice.eq_ignore_ascii_case("all") {
        for p in &matching {
            queries::tier_lists::revoke_permission(pool, tier_list_id, p.user_id, &p.permission)
                .await
                .context("revoke_permission")?;
        }
        println!("\nAll permissions revoked.\n");
    } else if let Ok(n) = choice.parse::<usize>() {
        if let Some(p) = matching.get(n.wrapping_sub(1)) {
            queries::tier_lists::revoke_permission(pool, tier_list_id, p.user_id, &p.permission)
                .await
                .context("revoke_permission")?;
            println!("\nPermission '{}' revoked.\n", p.permission);
        } else {
            println!("Invalid choice.\n");
        }
    } else {
        println!("Invalid choice.\n");
    }
    Ok(())
}

// ─── Tier list listing / visibility ─────────────────────────────────────────

async fn list_tier_lists(pool: &PgPool) -> Result<()> {
    println!("\n--- Tier Lists ---");
    let lists = sqlx::query_as::<_, TierListDetailRow>(
        r"SELECT
              t.id, t.name, t.slug, t.is_active, t.is_listed, t.list_type,
              (SELECT COUNT(*) FROM tiers WHERE tier_list_id = t.id) AS tier_count,
              (SELECT COUNT(*) FROM tier_placements tp
                 JOIN tiers ti ON tp.tier_id = ti.id
                WHERE ti.tier_list_id = t.id) AS placement_count
            FROM tier_lists t
            ORDER BY t.created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    if lists.is_empty() {
        println!("No tier lists found.\n");
        return Ok(());
    }

    println!(
        "\n{:<38} {:<25} {:<18} {:<8} {:<8} {:<10} {:<6} {:<6}",
        "ID", "Name", "Slug", "Active", "Listed", "Type", "Tiers", "Ops"
    );
    println!("{}", "-".repeat(122));
    for tl in &lists {
        println!(
            "{:<38} {:<25} {:<18} {:<8} {:<8} {:<10} {:<6} {:<6}",
            tl.id,
            truncate(&tl.name, 23),
            truncate(&tl.slug, 16),
            yesno(tl.is_active),
            yesno(tl.is_listed),
            truncate(&tl.list_type, 10),
            tl.tier_count,
            tl.placement_count,
        );
    }
    println!();
    Ok(())
}

async fn toggle_visibility(pool: &PgPool) -> Result<()> {
    println!("\n--- Toggle Tier List Visibility ---");
    print!("Enter tier list slug: ");
    io::stdout().flush()?;
    let slug = read_line()?;
    if slug.is_empty() {
        println!("Cancelled.\n");
        return Ok(());
    }

    let row: Option<(Uuid, String, bool)> =
        sqlx::query_as("SELECT id, name, is_listed FROM tier_lists WHERE slug = $1")
            .bind(&slug)
            .fetch_optional(pool)
            .await?;
    let Some((id, name, is_listed)) = row else {
        println!("Tier list not found.\n");
        return Ok(());
    };

    let next = !is_listed;
    print!(
        "Flip '{}' listed = {} → {}? (y/n): ",
        name,
        yesno(is_listed),
        yesno(next)
    );
    io::stdout().flush()?;
    if !confirm()? {
        println!("Cancelled.\n");
        return Ok(());
    }

    queries::tier_lists::set_visibility(pool, id, next)
        .await
        .context("set_visibility")?;
    println!("\nVisibility updated to {}.\n", yesno(next));
    Ok(())
}

// ─── Lookup + presentation helpers ──────────────────────────────────────────

/// Resolves either a UUID or an Arknights UID to a single `UserRow`. When the
/// same Arknights UID exists on multiple servers we return the first match by
/// `created_at` — collisions are rare and the operator can re-search by UUID
/// if they need a specific one.
async fn find_user_by_id_or_uid(pool: &PgPool, input: &str) -> Result<Option<UserRow>> {
    if let Ok(uuid) = Uuid::parse_str(input) {
        Ok(sqlx::query_as::<_, UserRow>(USER_SELECT_BY_ID)
            .bind(uuid)
            .fetch_optional(pool)
            .await?)
    } else {
        Ok(sqlx::query_as::<_, UserRow>(USER_SELECT_BY_UID)
            .bind(input)
            .fetch_optional(pool)
            .await?)
    }
}

async fn fetch_permissions(pool: &PgPool, tier_list_id: Uuid) -> Result<Vec<PermissionRow>> {
    Ok(sqlx::query_as::<_, PermissionRow>(
        r"SELECT p.user_id, p.permission, u.uid, u.nickname
             FROM tier_list_permissions p
             JOIN users u ON p.user_id = u.id
            WHERE p.tier_list_id = $1
            ORDER BY p.permission, u.uid",
    )
    .bind(tier_list_id)
    .fetch_all(pool)
    .await?)
}

fn print_user_table(users: &[UserRow]) {
    if users.is_empty() {
        println!("\nNo users found.\n");
        return;
    }
    println!(
        "\n{:<38} {:<20} {:<8} {:<20} {:<18}",
        "ID", "UID", "Server", "Nickname", "Role"
    );
    println!("{}", "-".repeat(106));
    for u in users {
        println!(
            "{:<38} {:<20} {:<8} {:<20} {:<18}",
            u.id,
            truncate(&u.uid, 18),
            u.server,
            truncate(&display_name(u), 18),
            u.role,
        );
    }
    println!();
}

fn print_permission_table(perms: &[PermissionRow]) {
    if perms.is_empty() {
        println!("\nNo permissions set for this tier list.");
        return;
    }
    println!("\nCurrent permissions:");
    println!(
        "{:<38} {:<20} {:<20} {:<10}",
        "User ID", "UID", "Nickname", "Permission"
    );
    println!("{}", "-".repeat(90));
    for p in perms {
        println!(
            "{:<38} {:<20} {:<20} {:<10}",
            p.user_id,
            truncate(&p.uid, 18),
            truncate(p.nickname.as_deref().unwrap_or(""), 18),
            p.permission,
        );
    }
}

// ─── Generic prompt helpers ────────────────────────────────────────────────

fn read_line() -> Result<String> {
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .context("failed to read stdin")?;
    Ok(buf.trim().to_string())
}

fn confirm() -> Result<bool> {
    Ok(read_line()?.eq_ignore_ascii_case("y"))
}

/// Render a numbered menu of `T: Display` and return the user's selection. The
/// caller supplies a `describe` function so each option can carry a short help
/// blurb. Returns `None` on `c`/`cancel`/blank input.
fn prompt_choice<T: Copy + std::fmt::Display>(
    header: &str,
    options: &[T],
    describe: impl Fn(T) -> &'static str,
) -> Result<Option<T>> {
    println!("{header}");
    for (i, opt) in options.iter().enumerate() {
        println!("  {}. {:<20} - {}", i + 1, opt.to_string(), describe(*opt));
    }
    print!("\nChoice (or 'c' to cancel): ");
    io::stdout().flush()?;
    let raw = read_line()?;
    if raw.is_empty() || raw.eq_ignore_ascii_case("c") || raw.eq_ignore_ascii_case("cancel") {
        return Ok(None);
    }
    // Accept either the 1-based index or the variant name itself.
    if let Ok(n) = raw.parse::<usize>()
        && let Some(opt) = options.get(n.wrapping_sub(1))
    {
        return Ok(Some(*opt));
    }
    if let Some(opt) = options
        .iter()
        .find(|o| o.to_string().eq_ignore_ascii_case(&raw))
    {
        return Ok(Some(*opt));
    }
    println!("Invalid choice.\n");
    Ok(None)
}

fn prompt_index<'a, T>(prompt: &str, options: &'a [T]) -> Result<Option<&'a T>> {
    print!("{prompt}");
    io::stdout().flush()?;
    let raw = read_line()?;
    if raw.is_empty() || raw.eq_ignore_ascii_case("c") || raw.eq_ignore_ascii_case("cancel") {
        return Ok(None);
    }
    let Ok(n) = raw.parse::<usize>() else {
        println!("Invalid choice.\n");
        return Ok(None);
    };
    Ok(options.get(n.wrapping_sub(1)))
}

const fn describe_role(r: GlobalRole) -> &'static str {
    match r {
        GlobalRole::User => "Default, no special permissions",
        GlobalRole::TierListEditor => "Can edit tier lists they have permission for",
        GlobalRole::TierListAdmin => "Can manage all tier lists",
        GlobalRole::SuperAdmin => "Full access to everything",
    }
}

const fn describe_permission(p: Permission) -> &'static str {
    match p {
        Permission::View => "Can view the tier list",
        Permission::Edit => "Can modify placements",
        Permission::Publish => "Can create new versions",
        Permission::Admin => "Can manage permissions + delete",
    }
}

fn display_name(u: &UserRow) -> String {
    u.nickname.clone().unwrap_or_else(|| "(no nickname)".into())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let cutoff = max_len.saturating_sub(3);
        let truncated: String = s.chars().take(cutoff).collect();
        format!("{truncated}...")
    }
}

const fn yesno(b: bool) -> &'static str {
    if b { "Yes" } else { "No" }
}

// ─── Row types + canonical user SELECTs ─────────────────────────────────────

// All four user-lookup statements parse into `UserRow`, so they share the
// same projection. Each is a single literal so sqlx can statically validate.
const USER_SELECT_BY_ID: &str = "SELECT u.id, u.uid, s.code AS server, u.nickname, u.role \
                                 FROM users u JOIN servers s ON s.id = u.server_id \
                                 WHERE u.id = $1";
const USER_SELECT_BY_UID: &str = "SELECT u.id, u.uid, s.code AS server, u.nickname, u.role \
                                  FROM users u JOIN servers s ON s.id = u.server_id \
                                  WHERE u.uid = $1 ORDER BY u.created_at LIMIT 1";
const USER_SELECT_RECENT: &str = "SELECT u.id, u.uid, s.code AS server, u.nickname, u.role \
                                  FROM users u JOIN servers s ON s.id = u.server_id \
                                  ORDER BY u.created_at DESC LIMIT 20";
const USER_SELECT_SEARCH: &str = "SELECT u.id, u.uid, s.code AS server, u.nickname, u.role \
                                  FROM users u JOIN servers s ON s.id = u.server_id \
                                  WHERE u.uid ILIKE $1 OR u.nickname ILIKE $1 \
                                  ORDER BY u.created_at DESC LIMIT 20";

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    uid: String,
    server: String,
    nickname: Option<String>,
    role: String,
}

#[derive(sqlx::FromRow)]
struct TierListRow {
    id: Uuid,
    name: String,
    slug: String,
}

#[derive(sqlx::FromRow)]
struct TierListDetailRow {
    id: Uuid,
    name: String,
    slug: String,
    is_active: bool,
    is_listed: bool,
    list_type: String,
    tier_count: i64,
    placement_count: i64,
}

#[derive(sqlx::FromRow)]
struct PermissionRow {
    user_id: Uuid,
    permission: String,
    uid: String,
    nickname: Option<String>,
}
