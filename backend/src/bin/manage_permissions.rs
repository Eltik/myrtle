use sqlx::PgPool;
use std::io::{self, Write};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("\n========================================");
    println!("   Myrtle Permission Management Tool");
    println!("========================================\n");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    println!("Connected to database.\n");

    loop {
        println!("What would you like to do?");
        println!("  1. Search users");
        println!("  2. Update user role");
        println!("  3. Manage tier list permissions");
        println!("  4. List tier lists");
        println!("  5. Exit");
        print!("\nChoice: ");
        io::stdout().flush()?;

        let choice = read_line()?;

        match choice.trim() {
            "1" => search_users(&pool).await?,
            "2" => update_user_role(&pool).await?,
            "3" => manage_tier_list_permissions(&pool).await?,
            "4" => list_tier_lists(&pool).await?,
            "5" | "q" | "exit" => {
                println!("\nGoodbye!");
                break;
            }
            _ => println!("\nInvalid choice. Please try again.\n"),
        }
    }

    Ok(())
}

fn read_line() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

async fn search_users(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Search Users ---");
    print!("Enter search term (uid, nickname, or leave blank to list all): ");
    io::stdout().flush()?;

    let search = read_line()?;

    let users: Vec<UserRow> = if search.is_empty() {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, uid, server, role, data->'status'->>'nickName' as nickname, created_at FROM users ORDER BY created_at DESC LIMIT 20"
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, uid, server, role, data->'status'->>'nickName' as nickname, created_at FROM users WHERE uid ILIKE $1 OR data->'status'->>'nickName' ILIKE $1 ORDER BY created_at DESC LIMIT 20"
        )
        .bind(format!("%{search}%"))
        .fetch_all(pool)
        .await?
    };

    if users.is_empty() {
        println!("\nNo users found.\n");
    } else {
        println!(
            "\n{:<38} {:<20} {:<10} {:<20} {:<15}",
            "ID", "UID", "Server", "Nickname", "Role"
        );
        println!("{}", "-".repeat(103));
        for user in &users {
            println!(
                "{:<38} {:<20} {:<10} {:<20} {:<15}",
                user.id,
                truncate(&user.uid, 18),
                user.server,
                truncate(&user.nickname.clone().unwrap_or_default(), 18),
                user.role
            );
        }
        println!();
    }

    Ok(())
}

async fn update_user_role(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Update User Role ---");
    print!("Enter user ID or UID: ");
    io::stdout().flush()?;

    let input = read_line()?;
    if input.is_empty() {
        println!("Cancelled.\n");
        return Ok(());
    }

    // Try to find user by ID or UID
    let user: Option<UserRow> = if let Ok(uuid) = Uuid::parse_str(&input) {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, uid, server, role, data->'status'->>'nickName' as nickname, created_at FROM users WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, uid, server, role, data->'status'->>'nickName' as nickname, created_at FROM users WHERE uid = $1",
        )
        .bind(&input)
        .fetch_optional(pool)
        .await?
    };

    let user = match user {
        Some(u) => u,
        None => {
            println!("User not found.\n");
            return Ok(());
        }
    };

    println!("\nUser found:");
    println!("  ID:       {}", user.id);
    println!("  UID:      {}", user.uid);
    println!("  Nickname: {}", user.nickname.clone().unwrap_or_default());
    println!("  Server:   {}", user.server);
    println!("  Role:     {}", user.role);

    println!("\nAvailable roles:");
    println!("  1. user             - Default, no special permissions");
    println!("  2. tier_list_editor - Can edit tier lists they have permission for");
    println!("  3. tier_list_admin  - Can manage all tier lists");
    println!("  4. super_admin      - Full access to everything");
    print!("\nSelect new role (1-4) or 'c' to cancel: ");
    io::stdout().flush()?;

    let choice = read_line()?;
    let new_role = match choice.trim() {
        "1" => "user",
        "2" => "tier_list_editor",
        "3" => "tier_list_admin",
        "4" => "super_admin",
        "c" | "cancel" => {
            println!("Cancelled.\n");
            return Ok(());
        }
        _ => {
            println!("Invalid choice.\n");
            return Ok(());
        }
    };

    if new_role == user.role {
        println!("User already has this role.\n");
        return Ok(());
    }

    print!(
        "Update role from '{}' to '{}'? (y/n): ",
        user.role, new_role
    );
    io::stdout().flush()?;

    let confirm = read_line()?;
    if confirm.to_lowercase() != "y" {
        println!("Cancelled.\n");
        return Ok(());
    }

    sqlx::query("UPDATE users SET role = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_role)
        .bind(user.id)
        .execute(pool)
        .await?;

    println!("\nRole updated successfully!\n");
    Ok(())
}

async fn manage_tier_list_permissions(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Manage Tier List Permissions ---");

    // First, list available tier lists
    let tier_lists: Vec<TierListRow> =
        sqlx::query_as::<_, TierListRow>("SELECT id, name, slug FROM tier_lists ORDER BY name")
            .fetch_all(pool)
            .await?;

    if tier_lists.is_empty() {
        println!("No tier lists found. Create one first.\n");
        return Ok(());
    }

    println!("\nAvailable tier lists:");
    for (i, tl) in tier_lists.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, tl.name, tl.slug);
    }

    print!("\nSelect tier list (number) or 'c' to cancel: ");
    io::stdout().flush()?;

    let choice = read_line()?;
    if choice == "c" || choice == "cancel" {
        println!("Cancelled.\n");
        return Ok(());
    }

    let index: usize = match choice.parse::<usize>() {
        Ok(n) if n > 0 && n <= tier_lists.len() => n - 1,
        _ => {
            println!("Invalid choice.\n");
            return Ok(());
        }
    };

    let tier_list = &tier_lists[index];
    println!("\nSelected: {} ({})", tier_list.name, tier_list.slug);

    // Show current permissions
    let permissions: Vec<PermissionRow> = sqlx::query_as::<_, PermissionRow>(
        r#"
        SELECT p.id, p.user_id, p.permission, u.uid, u.data->'status'->>'nickName' as nickname
        FROM tier_list_permissions p
        JOIN users u ON p.user_id = u.id
        WHERE p.tier_list_id = $1
        ORDER BY p.permission, u.uid
        "#,
    )
    .bind(tier_list.id)
    .fetch_all(pool)
    .await?;

    if !permissions.is_empty() {
        println!("\nCurrent permissions:");
        println!(
            "{:<38} {:<20} {:<20} {:<10}",
            "User ID", "UID", "Nickname", "Permission"
        );
        println!("{}", "-".repeat(88));
        for p in &permissions {
            println!(
                "{:<38} {:<20} {:<20} {:<10}",
                p.user_id,
                truncate(&p.uid, 18),
                truncate(&p.nickname.clone().unwrap_or_default(), 18),
                p.permission
            );
        }
    } else {
        println!("\nNo permissions set for this tier list.");
    }

    println!("\nActions:");
    println!("  1. Grant permission");
    println!("  2. Revoke permission");
    println!("  3. Back");
    print!("\nChoice: ");
    io::stdout().flush()?;

    let action = read_line()?;
    match action.trim() {
        "1" => grant_permission(pool, tier_list.id).await?,
        "2" => revoke_permission(pool, tier_list.id, &permissions).await?,
        _ => println!(),
    }

    Ok(())
}

async fn grant_permission(
    pool: &PgPool,
    tier_list_id: Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    print!("\nEnter user ID or UID: ");
    io::stdout().flush()?;

    let input = read_line()?;
    if input.is_empty() {
        println!("Cancelled.\n");
        return Ok(());
    }

    // Find user
    let user: Option<UserRow> = if let Ok(uuid) = Uuid::parse_str(&input) {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, uid, server, role, data->'status'->>'nickName' as nickname, created_at FROM users WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, uid, server, role, data->'status'->>'nickName' as nickname, created_at FROM users WHERE uid = $1",
        )
        .bind(&input)
        .fetch_optional(pool)
        .await?
    };

    let user = match user {
        Some(u) => u,
        None => {
            println!("User not found.\n");
            return Ok(());
        }
    };

    println!(
        "\nUser: {} ({})",
        user.uid,
        user.nickname.clone().unwrap_or_default()
    );

    println!("\nPermission levels:");
    println!("  1. view    - Can view the tier list");
    println!("  2. edit    - Can modify placements");
    println!("  3. publish - Can create new versions");
    println!("  4. admin   - Can manage permissions + delete");
    print!("\nSelect permission (1-4): ");
    io::stdout().flush()?;

    let choice = read_line()?;
    let permission = match choice.trim() {
        "1" => "view",
        "2" => "edit",
        "3" => "publish",
        "4" => "admin",
        _ => {
            println!("Invalid choice.\n");
            return Ok(());
        }
    };

    sqlx::query(
        r#"
        INSERT INTO tier_list_permissions (tier_list_id, user_id, permission)
        VALUES ($1, $2, $3)
        ON CONFLICT (tier_list_id, user_id, permission) DO UPDATE SET granted_at = NOW()
        "#,
    )
    .bind(tier_list_id)
    .bind(user.id)
    .bind(permission)
    .execute(pool)
    .await?;

    println!("\nGranted '{}' permission to {}.\n", permission, user.uid);
    Ok(())
}

async fn revoke_permission(
    pool: &PgPool,
    _tier_list_id: Uuid,
    permissions: &[PermissionRow],
) -> Result<(), Box<dyn std::error::Error>> {
    if permissions.is_empty() {
        println!("No permissions to revoke.\n");
        return Ok(());
    }

    print!("\nEnter user ID or UID to revoke from: ");
    io::stdout().flush()?;

    let input = read_line()?;
    if input.is_empty() {
        println!("Cancelled.\n");
        return Ok(());
    }

    // Find matching permission
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

    print!("\nSelect permission to revoke (number) or 'all' for all: ");
    io::stdout().flush()?;

    let choice = read_line()?;

    if choice == "all" {
        for p in &matching {
            sqlx::query("DELETE FROM tier_list_permissions WHERE id = $1")
                .bind(p.id)
                .execute(pool)
                .await?;
        }
        println!("\nAll permissions revoked.\n");
    } else if let Ok(n) = choice.parse::<usize>() {
        if n > 0 && n <= matching.len() {
            sqlx::query("DELETE FROM tier_list_permissions WHERE id = $1")
                .bind(matching[n - 1].id)
                .execute(pool)
                .await?;
            println!("\nPermission '{}' revoked.\n", matching[n - 1].permission);
        } else {
            println!("Invalid choice.\n");
        }
    } else {
        println!("Invalid choice.\n");
    }

    Ok(())
}

async fn list_tier_lists(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Tier Lists ---");

    let tier_lists: Vec<TierListDetailRow> = sqlx::query_as::<_, TierListDetailRow>(
        r#"
        SELECT
            t.id, t.name, t.slug, t.is_active, t.created_at,
            (SELECT COUNT(*) FROM tiers WHERE tier_list_id = t.id) as tier_count,
            (SELECT COUNT(*) FROM tier_placements tp JOIN tiers ti ON tp.tier_id = ti.id WHERE ti.tier_list_id = t.id) as placement_count
        FROM tier_lists t
        ORDER BY t.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    if tier_lists.is_empty() {
        println!("No tier lists found.\n");
    } else {
        println!(
            "\n{:<38} {:<25} {:<15} {:<8} {:<8} {:<8}",
            "ID", "Name", "Slug", "Active", "Tiers", "Ops"
        );
        println!("{}", "-".repeat(102));
        for tl in &tier_lists {
            println!(
                "{:<38} {:<25} {:<15} {:<8} {:<8} {:<8}",
                tl.id,
                truncate(&tl.name, 23),
                truncate(&tl.slug, 13),
                if tl.is_active { "Yes" } else { "No" },
                tl.tier_count,
                tl.placement_count
            );
        }
        println!();
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// Database row types
#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    uid: String,
    server: String,
    role: String,
    nickname: Option<String>,
    #[allow(dead_code)]
    created_at: chrono::DateTime<chrono::Utc>,
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
    #[allow(dead_code)]
    created_at: chrono::DateTime<chrono::Utc>,
    tier_count: i64,
    placement_count: i64,
}

#[derive(sqlx::FromRow)]
struct PermissionRow {
    id: Uuid,
    user_id: Uuid,
    permission: String,
    uid: String,
    nickname: Option<String>,
}
