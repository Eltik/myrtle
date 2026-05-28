use crate::db;
use crate::types::{Context, Error};
use ::serenity::model::user::User;
use poise::CreateReply;
use poise::serenity_prelude as serenity;

/// Manage rich embeds sent by the bot.
///
/// Subcommands: `create`, `edit`, `source`. Requires the Manage Messages permission.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "MANAGE_MESSAGES",
    subcommands("create", "edit", "source"),
    subcommand_required
)]
pub async fn embed(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Create and send a new embed.
///
/// Provide individual fields, or a raw `json` payload (e.g. from `/embed source`) for full
/// control including multiple fields. Fields are layered on top of `json` when both are given.
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, guild_only, required_permissions = "MANAGE_MESSAGES")]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Channel to send the embed to (defaults to the current channel)"]
    channel: Option<serenity::ChannelId>,
    #[description = "Embed title"] title: Option<String>,
    #[description = "Embed description / body"] description: Option<String>,
    #[description = "Hex color, e.g. #5865F2"] color: Option<String>,
    #[description = "URL the title links to"] url: Option<String>,
    #[description = "Large image URL"] image: Option<String>,
    #[description = "Thumbnail image URL"] thumbnail: Option<String>,
    #[description = "Author name"] author: Option<String>,
    #[description = "Author icon URL"] author_icon: Option<String>,
    #[description = "Footer text"] footer: Option<String>,
    #[description = "Footer icon URL"] footer_icon: Option<String>,
    #[description = "Add the current timestamp"] timestamp: Option<bool>,
    #[description = "Raw embed JSON (overridden by any fields above)"] json: Option<String>,
) -> Result<(), Error> {
    let has_content = json.is_some()
        || title.is_some()
        || description.is_some()
        || image.is_some()
        || thumbnail.is_some()
        || author.is_some()
        || footer.is_some();
    if !has_content {
        return Err("Give the embed some content: a title, description, image, author, footer, or raw json.".into());
    }

    let base = match json.as_deref() {
        Some(j) => parse_embed_json(j)?,
        None => serenity::CreateEmbed::new(),
    };
    let embed = apply_fields(
        base,
        EmbedFields {
            title,
            description,
            color,
            url,
            image,
            thumbnail,
            author,
            author_icon,
            footer,
            footer_icon,
            timestamp,
        },
    )?;

    let target = channel.unwrap_or_else(|| ctx.channel_id());
    let sent = target
        .send_message(ctx.http(), serenity::CreateMessage::new().embed(embed))
        .await
        .map_err(|e| format!("Couldn't send the embed to <#{target}>: {e}"))?;

    let link = message_link(ctx.guild_id(), target, sent.id);
    ctx.send(
        CreateReply::default()
            .content(format!("Embed sent to <#{target}>. {link}"))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Edit an embed the bot already sent.
///
/// Identify the message by link, `channel-message` id pair, or raw message id. Provided fields are
/// layered on top of the existing embed; pass `json` to replace the embed wholesale.
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, guild_only, required_permissions = "MANAGE_MESSAGES")]
pub async fn edit(
    ctx: Context<'_>,
    #[description = "Message link, channel-message id, or message id"] message: String,
    #[description = "Channel the message is in (if not derivable from the link)"] channel: Option<
        serenity::ChannelId,
    >,
    #[description = "Embed title"] title: Option<String>,
    #[description = "Embed description / body"] description: Option<String>,
    #[description = "Hex color, e.g. #5865F2"] color: Option<String>,
    #[description = "URL the title links to"] url: Option<String>,
    #[description = "Large image URL"] image: Option<String>,
    #[description = "Thumbnail image URL"] thumbnail: Option<String>,
    #[description = "Author name"] author: Option<String>,
    #[description = "Author icon URL"] author_icon: Option<String>,
    #[description = "Footer text"] footer: Option<String>,
    #[description = "Footer icon URL"] footer_icon: Option<String>,
    #[description = "Add the current timestamp"] timestamp: Option<bool>,
    #[description = "Raw embed JSON (replaces the existing embed)"] json: Option<String>,
) -> Result<(), Error> {
    let (link_channel, message_id) = parse_message_ref(&message)
        .ok_or("Couldn't parse that message reference. Use a message link, `channel-message` id, or message id.")?;
    let target = channel.or(link_channel).unwrap_or_else(|| ctx.channel_id());

    let msg = target
        .message(ctx.http(), message_id)
        .await
        .map_err(|_| format!("Couldn't find message {message_id} in <#{target}>."))?;

    if msg.author.id != ctx.framework().bot_id {
        return Err("I can only edit embeds on messages I sent myself.".into());
    }

    let base = if let Some(j) = json.as_deref() {
        parse_embed_json(j)?
    } else if let Some(existing) = msg.embeds.into_iter().next() {
        serenity::CreateEmbed::from(existing)
    } else {
        serenity::CreateEmbed::new()
    };
    let embed = apply_fields(
        base,
        EmbedFields {
            title,
            description,
            color,
            url,
            image,
            thumbnail,
            author,
            author_icon,
            footer,
            footer_icon,
            timestamp,
        },
    )?;

    target
        .edit_message(
            ctx.http(),
            message_id,
            serenity::EditMessage::new().embed(embed),
        )
        .await
        .map_err(|e| format!("Couldn't edit that message: {e}"))?;

    let link = message_link(ctx.guild_id(), target, message_id);
    ctx.send(
        CreateReply::default()
            .content(format!("Embed updated. {link}"))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Fetch the JSON source of a message's embed(s).
///
/// Identify the message by link, `channel-message` id pair, or raw message id. The output can be
/// pasted straight back into `/embed create` or `/embed edit` via their `json` field.
#[poise::command(slash_command, guild_only, required_permissions = "MANAGE_MESSAGES")]
pub async fn source(
    ctx: Context<'_>,
    #[description = "Message link, channel-message id, or message id"] message: String,
    #[description = "Channel the message is in (if not derivable from the link)"] channel: Option<
        serenity::ChannelId,
    >,
) -> Result<(), Error> {
    let (link_channel, message_id) = parse_message_ref(&message)
        .ok_or("Couldn't parse that message reference. Use a message link, `channel-message` id, or message id.")?;
    let target = channel.or(link_channel).unwrap_or_else(|| ctx.channel_id());

    let msg = target
        .message(ctx.http(), message_id)
        .await
        .map_err(|_| format!("Couldn't find message {message_id} in <#{target}>."))?;

    if msg.embeds.is_empty() {
        return Err("That message has no embeds.".into());
    }

    // A lone embed is serialized as a single object so it round-trips into the `json` field;
    // multiple embeds are serialized as an array.
    let json = if msg.embeds.len() == 1 {
        serde_json::to_string_pretty(&msg.embeds[0])?
    } else {
        serde_json::to_string_pretty(&msg.embeds)?
    };

    let block = format!("```json\n{json}\n```");
    let reply = if block.len() <= 2000 {
        CreateReply::default().content(block)
    } else {
        // Too large for a message body; ship it as a file.
        CreateReply::default().attachment(serenity::CreateAttachment::bytes(
            json.into_bytes(),
            "embed.json",
        ))
    };
    ctx.send(reply.ephemeral(true)).await?;
    Ok(())
}

/// Ban an user from the guild.
///
/// Optionally provide a reason (recorded in the audit log) and the number of days of recent
/// messages to delete (0-7). Requires the Ban Members permission.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "BAN_MEMBERS",
    required_permissions = "BAN_MEMBERS"
)]
pub async fn ban_user(
    ctx: Context<'_>,
    #[description = "User to ban"] user: User,
    #[description = "Reason for the ban"] reason: Option<String>,
    #[description = "Days of messages to delete (0-7)"] delete_message_days: Option<u8>,
) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;

    if user.id == ctx.author().id {
        return Err("You can't ban yourself.".into());
    }
    if user.id == ctx.framework().bot_id {
        return Err("I can't ban myself.".into());
    }

    let dmd = delete_message_days.unwrap_or(0);
    if dmd > 7 {
        return Err("`delete_message_days` must be between 0 and 7.".into());
    }

    match reason.as_deref() {
        Some(r) => guild
            .ban_with_reason(ctx.http(), user.id, dmd, r)
            .await
            .map_err(|e| format!("Couldn't ban {}: {e}", user.tag()))?,
        None => guild
            .ban(ctx.http(), user.id, dmd)
            .await
            .map_err(|e| format!("Couldn't ban {}: {e}", user.tag()))?,
    }

    let suffix = reason
        .as_deref()
        .map(|r| format!(" Reason: {r}"))
        .unwrap_or_default();
    ctx.send(
        CreateReply::default()
            .content(format!("Banned {} ({}).{suffix}", user.tag(), user.id))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Autocomplete the `user` argument of `/unban_user` with the guild's banned users.
///
/// Fetches up to 1000 bans per keystroke and filters them by username or id substring. Discord
/// caps autocomplete responses at 25 entries. Errors and non-guild invocations yield an empty
/// list — the user can always paste a raw id.
async fn autocomplete_banned_user(
    ctx: Context<'_>,
    partial: &str,
) -> Vec<serenity::AutocompleteChoice> {
    let Some(guild) = ctx.guild_id() else {
        return Vec::new();
    };
    let Ok(bans) = guild.bans(ctx.http(), None, None).await else {
        return Vec::new();
    };
    let needle = partial.to_lowercase();
    bans.into_iter()
        .filter(|b| {
            needle.is_empty()
                || b.user.name.to_lowercase().contains(&needle)
                || b.user.id.to_string().contains(&needle)
        })
        .take(25)
        .map(|b| {
            serenity::AutocompleteChoice::new(
                format!("{} ({})", b.user.tag(), b.user.id),
                b.user.id.to_string(),
            )
        })
        .collect()
}

/// Unban a user from the guild.
///
/// The `user` field autocompletes against currently banned users; you can also paste a raw user
/// ID. Optionally provide a reason (recorded in the audit log). Requires the Ban Members
/// permission.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "BAN_MEMBERS",
    required_permissions = "BAN_MEMBERS"
)]
pub async fn unban_user(
    ctx: Context<'_>,
    #[description = "Banned user (autocompletes) or a raw user ID"]
    #[autocomplete = "autocomplete_banned_user"]
    user: String,
    #[description = "Reason for the unban (shown in the audit log)"] reason: Option<String>,
) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;

    let user_id = user
        .trim()
        .parse::<u64>()
        .map(serenity::UserId::new)
        .map_err(|_| {
            format!("'{user}' isn't a valid user ID. Pick from autocomplete or paste a numeric ID.")
        })?;

    ctx.http()
        .remove_ban(guild, user_id, reason.as_deref())
        .await
        .map_err(|e| format!("Couldn't unban <@{user_id}>: {e}"))?;

    let suffix = reason
        .as_deref()
        .map(|r| format!(" Reason: {r}"))
        .unwrap_or_default();
    ctx.send(
        CreateReply::default()
            .content(format!("Unbanned <@{user_id}> ({user_id}).{suffix}"))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Kick a user from the guild.
///
/// Kicks an user from the guild. Requires the `KICK_MEMBERS`
/// permission.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "KICK_MEMBERS",
    required_permissions = "KICK_MEMBERS"
)]
pub async fn kick_user(
    ctx: Context<'_>,
    #[description = "User to kick"] user: User,
    #[description = "Reason for the kick"] reason: Option<String>,
) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;

    if user.id == ctx.author().id {
        return Err("You can't kick yourself.".into());
    }
    if user.id == ctx.framework().bot_id {
        return Err("I can't kick myself.".into());
    }

    match reason.as_deref() {
        Some(r) => guild
            .kick_with_reason(ctx.http(), user.id, r)
            .await
            .map_err(|e| format!("Couldn't kick {}: {e}", user.tag()))?,
        None => guild
            .kick(ctx.http(), user.id)
            .await
            .map_err(|e| format!("Couldn't kick {}: {e}", user.tag()))?,
    }

    let suffix = reason
        .as_deref()
        .map(|r| format!(" Reason: {r}"))
        .unwrap_or_default();
    ctx.send(
        CreateReply::default()
            .content(format!("Kicked {} ({}).{suffix}", user.tag(), user.id))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Manage the role automatically granted to new members of this guild.
///
/// Subcommands: `set`, `clear`, `show`.
/// Requires the Manage Roles permission.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "MANAGE_ROLES",
    subcommands("autorole_set", "autorole_clear", "autorole_show"),
    subcommand_required
)]
pub async fn autorole(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set the auto-role for this guild.
#[poise::command(
    slash_command,
    guild_only,
    rename = "set",
    required_permissions = "MANAGE_ROLES"
)]
pub async fn autorole_set(
    ctx: Context<'_>,
    #[description = "Role to grant new members on join"] role: serenity::Role,
) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    db::set_auto_role(&ctx.data().pool, guild, role.id)
        .await
        .map_err(|e| format!("Couldn't save auto-role: {e}"))?;
    ctx.send(
        CreateReply::default()
            .content(format!("Auto-role set to <@&{}>.", role.id))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Clear the auto-role for this guild.
#[poise::command(
    slash_command,
    guild_only,
    rename = "clear",
    required_permissions = "MANAGE_ROLES"
)]
pub async fn autorole_clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    db::clear_auto_role(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't clear auto-role: {e}"))?;
    ctx.send(
        CreateReply::default()
            .content("Auto-role cleared.")
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Show the current auto-role for this guild.
#[poise::command(
    slash_command,
    guild_only,
    rename = "show",
    required_permissions = "MANAGE_ROLES"
)]
pub async fn autorole_show(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    let content = match db::get_auto_role(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't read auto-role: {e}"))?
    {
        Some(role_id) => format!("Auto-role: <@&{role_id}>"),
        None => "No auto-role configured.".to_string(),
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

/// Optional embed fields collected from a slash command invocation.
///
/// Packaging these as one nominal type keeps `apply_fields` from monomorphizing
/// across many generic positions — rustc treats this as one struct instead of
/// 12 separate `Option<T>` parameter slots.
#[derive(Default)]
struct EmbedFields {
    title: Option<String>,
    description: Option<String>,
    color: Option<String>,
    url: Option<String>,
    image: Option<String>,
    thumbnail: Option<String>,
    author: Option<String>,
    author_icon: Option<String>,
    footer: Option<String>,
    footer_icon: Option<String>,
    timestamp: Option<bool>,
}

/// Layer the provided optional fields onto an existing embed builder.
fn apply_fields(
    mut embed: serenity::CreateEmbed,
    fields: EmbedFields,
) -> Result<serenity::CreateEmbed, Error> {
    if let Some(title) = fields.title {
        embed = embed.title(title);
    }
    if let Some(description) = fields.description {
        embed = embed.description(description);
    }
    if let Some(color) = fields.color {
        embed = embed.colour(parse_color(&color)?);
    }
    if let Some(url) = fields.url {
        embed = embed.url(url);
    }
    if let Some(image) = fields.image {
        embed = embed.image(image);
    }
    if let Some(thumbnail) = fields.thumbnail {
        embed = embed.thumbnail(thumbnail);
    }
    if let Some(name) = fields.author {
        let mut a = serenity::CreateEmbedAuthor::new(name);
        if let Some(icon) = fields.author_icon {
            a = a.icon_url(icon);
        }
        embed = embed.author(a);
    }
    if let Some(text) = fields.footer {
        let mut f = serenity::CreateEmbedFooter::new(text);
        if let Some(icon) = fields.footer_icon {
            f = f.icon_url(icon);
        }
        embed = embed.footer(f);
    }
    if fields.timestamp == Some(true) {
        embed = embed.timestamp(serenity::Timestamp::now());
    }
    Ok(embed)
}

/// Parse a raw embed JSON object into a `CreateEmbed`.
fn parse_embed_json(input: &str) -> Result<serenity::CreateEmbed, Error> {
    let embed: serenity::Embed =
        serde_json::from_str(input).map_err(|e| format!("Invalid embed JSON: {e}"))?;
    Ok(serenity::CreateEmbed::from(embed))
}

/// Parse a color string: `#RRGGBB`, `RRGGBB`, `0xRRGGBB`, or a decimal value.
fn parse_color(input: &str) -> Result<serenity::Colour, Error> {
    let trimmed = input.trim();
    let hex = trimmed
        .strip_prefix('#')
        .or_else(|| trimmed.strip_prefix("0x"))
        .or_else(|| trimmed.strip_prefix("0X"));

    let value = match hex {
        Some(h) => u32::from_str_radix(h, 16),
        // No explicit hex marker: try decimal first, then bare hex (e.g. "ff0000").
        None => trimmed
            .parse::<u32>()
            .or_else(|_| u32::from_str_radix(trimmed, 16)),
    }
    .map_err(|_| format!("Invalid color '{input}'. Use a hex code like #5865F2."))?;

    Ok(serenity::Colour::new(value))
}

/// Resolve a message reference into an optional channel and a message id.
///
/// Accepts a full message URL (`.../channels/<guild>/<channel>/<message>`), a `channel-message`
/// id pair, or a bare message id (channel inferred by the caller).
fn parse_message_ref(input: &str) -> Option<(Option<serenity::ChannelId>, serenity::MessageId)> {
    let input = input.trim();

    if let Some(idx) = input.find("/channels/") {
        let segments: Vec<&str> = input[idx + "/channels/".len()..].split('/').collect();
        // [guild, channel, message]
        if segments.len() >= 3 {
            let channel = segments[1].parse::<u64>().ok()?;
            let message = segments[2].parse::<u64>().ok()?;
            return Some((
                Some(serenity::ChannelId::new(channel)),
                serenity::MessageId::new(message),
            ));
        }
        return None;
    }

    if let Some((channel, message)) = input.split_once('-')
        && let (Ok(channel), Ok(message)) =
            (channel.trim().parse::<u64>(), message.trim().parse::<u64>())
    {
        return Some((
            Some(serenity::ChannelId::new(channel)),
            serenity::MessageId::new(message),
        ));
    }

    input
        .parse::<u64>()
        .ok()
        .map(|id| (None, serenity::MessageId::new(id)))
}

/// Build a clickable jump link to a message.
fn message_link(
    guild: Option<serenity::GuildId>,
    channel: serenity::ChannelId,
    message: serenity::MessageId,
) -> String {
    match guild {
        Some(g) => format!("https://discord.com/channels/{g}/{channel}/{message}"),
        None => format!("https://discord.com/channels/@me/{channel}/{message}"),
    }
}
