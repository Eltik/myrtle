//! Embed colours and Discord's message and component limits, shared by every command and
//! announcer.

// Standard Discord brand hex colors; keep them in conventional `0xRRGGBB` form.
#![allow(clippy::unreadable_literal)]

/// Discord green: success, joins, completed updates.
pub const COLOR_OK: u32 = 0x57F287;
/// Discord yellow: warnings, edits, leaves.
pub const COLOR_WARN: u32 = 0xFEE75C;
/// Discord red: failures, deletes, bans.
pub const COLOR_BAD: u32 = 0xED4245;
/// Discord blurple: neutral information.
pub const COLOR_INFO: u32 = 0x5865F2;
/// Discord fuchsia: moderator actions and birthdays.
pub const COLOR_PINK: u32 = 0xEB459E;

/// Characters in a message's plain content.
pub const CONTENT_MAX: usize = 2000;
/// Characters in an embed title.
pub const TITLE_MAX: usize = 256;
/// Characters in an embed description.
pub const DESCRIPTION_MAX: usize = 4096;
/// Characters in an embed field name.
pub const FIELD_NAME_MAX: usize = 256;
/// Characters in an embed field value.
pub const FIELD_MAX: usize = 1024;
/// Fields in one embed.
pub const FIELDS_PER_EMBED: usize = 25;
/// Embeds in one message.
pub const EMBEDS_PER_MESSAGE: usize = 10;
/// Characters across all of a message's embeds together.
pub const CHARS_PER_MESSAGE: usize = 6000;
/// Options in one select menu.
pub const SELECT_MAX: usize = 25;
/// Characters in a select option's label or description.
pub const OPTION_TEXT_MAX: usize = 100;
/// Choices in one autocomplete response.
pub const AUTOCOMPLETE_MAX: usize = 25;
/// Characters in an autocomplete choice's name.
pub const CHOICE_NAME_MAX: usize = 100;
