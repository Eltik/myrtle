//! Discord-shaped building blocks for the operator view: embed blocks that split and pack
//! themselves inside Discord's limits, select options capped at 25, and range grids.

use ::serenity::builder::{CreateEmbed, CreateEmbedFooter};

use crate::api::operator_detail::GridCell;
use crate::utils::ellipsize;

/// Discord's per-embed limits.
const TITLE_MAX: usize = 256;
const DESCRIPTION_MAX: usize = 4096;
const FIELD_NAME_MAX: usize = 256;
pub const FIELD_MAX: usize = 1024;
const FIELDS_PER_EMBED: usize = 25;
/// Discord's per-message limits: the characters of all embeds together, and the embed count.
pub const CHARS_PER_MESSAGE: usize = 6000;
const EMBEDS_PER_MESSAGE: usize = 10;
/// What one message's blocks may use, leaving room for the footer and the part suffixes.
const MESSAGE_BUDGET: usize = 5600;
/// What one block may use before it is split, so it always fits beside a header.
const BLOCK_BUDGET: usize = 4800;
/// Discord's cap on a select's options.
pub const SELECT_MAX: usize = 25;
/// A select's option label and description cap.
const OPTION_TEXT_MAX: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

/// One embed's worth of content, before Discord's limits are applied.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Block {
    pub title: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub fields: Vec<Field>,
    pub thumbnail: Option<String>,
    pub image: Option<String>,
}

impl Block {
    #[must_use]
    pub fn titled(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            ..Self::default()
        }
    }

    /// Add a field; empty values are skipped. A value over 1024 characters becomes several
    /// fields, split between lines, the later ones named "(cont.)".
    pub fn field(&mut self, name: impl AsRef<str>, value: impl AsRef<str>, inline: bool) {
        let (name, value) = (name.as_ref().trim(), value.as_ref().trim());
        if value.is_empty() {
            return;
        }
        let name = if name.is_empty() { "\u{200b}" } else { name };
        for (i, chunk) in split_lines(value, FIELD_MAX).into_iter().enumerate() {
            let name = if i == 0 {
                name.to_string()
            } else {
                format!("{name} (cont.)")
            };
            self.fields.push(Field {
                name: ellipsize(&name, FIELD_NAME_MAX),
                value: chunk,
                inline,
            });
        }
    }

    /// Append a paragraph to the description.
    pub fn paragraph(&mut self, text: impl AsRef<str>) {
        let text = text.as_ref().trim();
        if text.is_empty() {
            return;
        }
        match &mut self.description {
            Some(d) => {
                d.push_str("\n\n");
                d.push_str(text);
            }
            None => self.description = Some(text.to_string()),
        }
    }

    /// The characters Discord counts toward the message cap (footer aside).
    #[must_use]
    pub fn chars(&self) -> usize {
        let len = |s: &Option<String>| s.as_deref().map_or(0, |s| s.chars().count());
        len(&self.title)
            + len(&self.description)
            + self
                .fields
                .iter()
                .map(|f| f.name.chars().count() + f.value.chars().count())
                .sum::<usize>()
    }

    /// This block as one or more blocks that each fit Discord's per-embed limits and
    /// [`BLOCK_BUDGET`]. The title stays on the first; later ones are titled "(cont.)". The
    /// thumbnail stays on the first, the image goes to the last.
    fn split(self) -> Vec<Self> {
        if self.chars() <= BLOCK_BUDGET
            && self.fields.len() <= FIELDS_PER_EMBED
            && self.description.as_deref().map_or(0, |d| d.chars().count()) <= DESCRIPTION_MAX
        {
            return vec![self];
        }
        let title = self.title.clone();
        let title_len = title.as_deref().map_or(0, |t| t.chars().count()) + " (cont.)".len();
        let cont = |first: bool| Self {
            title: title.as_ref().map(|t| {
                if first {
                    t.clone()
                } else {
                    format!("{t} (cont.)")
                }
            }),
            ..Self::default()
        };
        let mut out = vec![Self {
            url: self.url.clone(),
            thumbnail: self.thumbnail.clone(),
            ..cont(true)
        }];
        if let Some(description) = self.description.as_deref() {
            let room = DESCRIPTION_MAX.min(BLOCK_BUDGET - title_len);
            for (i, chunk) in split_lines(description, room).into_iter().enumerate() {
                if i > 0 {
                    out.push(cont(false));
                }
                if let Some(last) = out.last_mut() {
                    last.description = Some(chunk);
                }
            }
        }
        for field in self.fields {
            let size = field.name.chars().count() + field.value.chars().count();
            let full = out.last().is_some_and(|b| {
                b.fields.len() == FIELDS_PER_EMBED || b.chars() + size > BLOCK_BUDGET
            });
            if full {
                out.push(cont(false));
            }
            if let Some(last) = out.last_mut() {
                last.fields.push(field);
            }
        }
        if let Some(last) = out.last_mut() {
            last.image = self.image;
        }
        out
    }

    fn into_embed(self, colour: u32) -> CreateEmbed {
        let mut embed = CreateEmbed::new().colour(colour);
        if let Some(title) = self.title {
            embed = embed.title(ellipsize(&title, TITLE_MAX));
        }
        if let Some(url) = self.url {
            embed = embed.url(url);
        }
        if let Some(description) = self.description {
            embed = embed.description(ellipsize(&description, DESCRIPTION_MAX));
        }
        if let Some(thumbnail) = self.thumbnail {
            embed = embed.thumbnail(thumbnail);
        }
        if let Some(image) = self.image {
            embed = embed.image(image);
        }
        embed.fields(
            self.fields
                .into_iter()
                .map(|f| (f.name, ellipsize(&f.value, FIELD_MAX), f.inline)),
        )
    }
}

/// A page laid out as messages ("parts"): each a list of blocks that together fit one message.
#[derive(Debug, Clone)]
pub struct Parts(pub Vec<Vec<Block>>);

impl Parts {
    /// Lay `blocks` out. The first block is the page's header; a part after the first opens
    /// with `slim`, a short copy of it, so every message still says whose page it is.
    #[must_use]
    pub fn pack(blocks: Vec<Block>, slim: &Block) -> Self {
        let mut blocks = blocks.into_iter().flat_map(Block::split);
        let mut parts: Vec<Vec<Block>> = Vec::new();
        let mut current: Vec<Block> = blocks.next().into_iter().collect();
        let mut used: usize = current.iter().map(Block::chars).sum();
        for block in blocks {
            let size = block.chars();
            if current.len() == EMBEDS_PER_MESSAGE || used + size > MESSAGE_BUDGET {
                parts.push(std::mem::take(&mut current));
                current.push(slim.clone());
                used = slim.chars();
            }
            used += size;
            current.push(block);
        }
        if !current.is_empty() {
            parts.push(current);
        }
        Self(parts)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn part(&self, index: usize) -> Option<&Vec<Block>> {
        self.0.get(index.min(self.0.len().saturating_sub(1)))
    }

    /// Every thumbnail and image link part `index` (clamped) sets, each once.
    #[must_use]
    pub fn image_urls(&self, index: usize) -> Vec<String> {
        let mut urls: Vec<String> = Vec::new();
        for block in self.part(index).into_iter().flatten() {
            for url in [&block.thumbnail, &block.image].into_iter().flatten() {
                if !urls.contains(url) {
                    urls.push(url.clone());
                }
            }
        }
        urls
    }

    /// Part `index` (clamped) as embeds, with `footer` on the last one and every thumbnail or
    /// image in `missing` left out.
    #[must_use]
    pub fn embeds<S: std::hash::BuildHasher>(
        &self,
        index: usize,
        colour: u32,
        footer: &str,
        missing: &std::collections::HashSet<String, S>,
    ) -> Vec<CreateEmbed> {
        let Some(part) = self.part(index) else {
            return Vec::new();
        };
        let count = part.len();
        part.iter()
            .cloned()
            .enumerate()
            .map(|(i, mut block)| {
                block.thumbnail = block.thumbnail.filter(|u| !missing.contains(u));
                block.image = block.image.filter(|u| !missing.contains(u));
                let embed = block.into_embed(colour);
                if i + 1 == count {
                    embed.footer(CreateEmbedFooter::new(footer))
                } else {
                    embed
                }
            })
            .collect()
    }

    /// The characters part `index` sends, footer included: what Discord holds to 6000.
    #[must_use]
    pub fn chars(&self, index: usize, footer: &str) -> usize {
        self.0.get(index).map_or(0, |p| {
            p.iter().map(Block::chars).sum::<usize>() + footer.chars().count()
        })
    }
}

/// Split `text` into chunks of at most `max` characters, breaking between lines. A single
/// line longer than `max` is shortened with an ellipsis.
#[must_use]
pub fn split_lines(text: &str, max: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut len = 0;
    for line in text.lines() {
        let line = if line.chars().count() > max {
            ellipsize(line, max)
        } else {
            line.to_string()
        };
        let line_len = line.chars().count();
        let extra = usize::from(!current.is_empty());
        if len + extra + line_len > max {
            chunks.push(std::mem::take(&mut current));
            len = 0;
        }
        if !current.is_empty() {
            current.push('\n');
            len += 1;
        }
        current.push_str(&line);
        len += line_len;
    }
    if !current.trim().is_empty() {
        chunks.push(current);
    }
    chunks
}

/// One option of a select, before it is built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    pub default: bool,
}

impl Choice {
    #[must_use]
    /// An option. Discord rejects an empty label, so a blank one reads "Option N" (N from a
    /// numeric value, counted from 1); [`capped_choices`] passes a better fallback of its own.
    pub fn new(label: &str, value: impl Into<String>, description: Option<&str>) -> Self {
        let value = value.into();
        let label = if label.trim().is_empty() {
            match value.parse::<usize>() {
                Ok(n) => format!("Option {}", n + 1),
                Err(_) => format!("Option {value}"),
            }
        } else {
            label.trim().to_string()
        };
        Self {
            label: ellipsize(&label, OPTION_TEXT_MAX),
            value,
            description: description
                .map(str::trim)
                .filter(|d| !d.is_empty())
                .map(|d| ellipsize(d, OPTION_TEXT_MAX)),
            default: false,
        }
    }
}

/// How many items one window of a capped select shows: 25 less the two paging options.
const WINDOW: usize = SELECT_MAX - 2;

/// The options for choosing one of `count` items, `selected` among them.
///
/// `item(i)` is the label and description of item `i`; the option's value is `i`. An item
/// whose label is blank reads "`unit` N" ("File 3").
///
/// Up to 25 items fit as they are. Past that, the select shows the window of 23 holding
/// `selected`, plus an "Earlier" option (the first item of the window before) and a "More"
/// option (the first item of the window after). Those values are never in the current window,
/// so every value stays unique.
#[must_use]
pub fn capped_choices(
    count: usize,
    selected: usize,
    noun: &str,
    unit: &str,
    item: impl Fn(usize) -> (String, Option<String>),
) -> Vec<Choice> {
    let one = |i: usize| {
        let (label, description) = item(i);
        let label = if label.trim().is_empty() {
            format!("{unit} {}", i + 1)
        } else {
            label
        };
        let mut choice = Choice::new(&label, i.to_string(), description.as_deref());
        choice.default = i == selected;
        choice
    };
    if count <= SELECT_MAX {
        return (0..count).map(one).collect();
    }
    let window = selected.min(count - 1) / WINDOW;
    let start = window * WINDOW;
    let end = (start + WINDOW).min(count);
    let mut choices = Vec::with_capacity(SELECT_MAX);
    if window > 0 {
        let prev = start - WINDOW;
        choices.push(Choice::new(
            &format!("Earlier {noun} ({} to {})", prev + 1, start),
            prev.to_string(),
            None,
        ));
    }
    choices.extend((start..end).map(one));
    if end < count {
        choices.push(Choice::new(
            &format!("More {noun} ({} to {})", end + 1, (end + WINDOW).min(count)),
            end.to_string(),
            None,
        ));
    }
    choices
}

/// The operator's tile.
const OWN_TILE: &str = "🟧";
/// A tile in range.
const RANGE_TILE: &str = "⬜";
/// A tile outside the range, inside the bounding box.
const EMPTY_TILE: &str = "⬛";

/// A range as a grid of emoji, the operator facing right, cropped to the smallest box that
/// holds the operator and every tile in range.
#[must_use]
pub fn range_grid(cells: &[GridCell]) -> String {
    let own = GridCell { row: 0, col: 0 };
    let tiles = || cells.iter().chain(std::iter::once(&own));
    let (Some(top), Some(bottom), Some(left), Some(right)) = (
        tiles().map(|c| c.row).max(),
        tiles().map(|c| c.row).min(),
        tiles().map(|c| c.col).min(),
        tiles().map(|c| c.col).max(),
    ) else {
        return OWN_TILE.to_string();
    };
    let mut rows = Vec::new();
    for row in (bottom..=top).rev() {
        let mut line = String::new();
        for col in left..=right {
            let cell = GridCell { row, col };
            line.push_str(if cell == own {
                OWN_TILE
            } else if cells.contains(&cell) {
                RANGE_TILE
            } else {
                EMPTY_TILE
            });
        }
        rows.push(line);
    }
    rows.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(list: &[(i32, i32)]) -> Vec<GridCell> {
        list.iter()
            .map(|&(row, col)| GridCell { row, col })
            .collect()
    }

    #[test]
    fn grid_of_a_three_by_three() {
        // `3-6` (Ling at E0): rows -1..1, cols 0..2, the operator's own tile among them.
        let range: Vec<(i32, i32)> = (-1..=1)
            .flat_map(|row| (0..=2).map(move |col| (row, col)))
            .collect();
        assert_eq!(range_grid(&cells(&range)), "⬜⬜⬜\n🟧⬜⬜\n⬜⬜⬜");
    }

    #[test]
    fn grid_of_a_line() {
        // A lancer-style line: three tiles ahead, none behind.
        let grid = range_grid(&cells(&[(0, 1), (0, 2), (0, 3)]));
        assert_eq!(grid, "🟧⬜⬜⬜");
    }

    #[test]
    fn grid_of_an_asymmetric_range() {
        // `y-1`: one column behind, two ahead on the operator's row only.
        let grid = range_grid(&cells(&[
            (1, -1),
            (1, 0),
            (1, 1),
            (0, -1),
            (0, 0),
            (0, 1),
            (0, 2),
            (-1, -1),
            (-1, 0),
            (-1, 1),
        ]));
        assert_eq!(grid, "⬜⬜⬜⬛\n⬜🟧⬜⬜\n⬜⬜⬜⬛");
        // Nothing in range: just the operator.
        assert_eq!(range_grid(&[]), "🟧");
        // A range entirely above the operator still keeps the operator's row.
        assert_eq!(range_grid(&cells(&[(2, 0)])), "⬜\n⬛\n🟧");
    }

    #[test]
    fn selects_cap_at_25_with_paging_options() {
        let label = |i: usize| (format!("Outfit {}", i + 1), None);
        // 25 or fewer: all of them, no paging.
        let all = capped_choices(25, 3, "outfits", "Outfit", label);
        assert_eq!(all.len(), 25);
        assert!(all[3].default && all.iter().filter(|c| c.default).count() == 1);

        // 60 items, the 31st selected: window 23..46 with both paging options.
        let mid = capped_choices(60, 30, "outfits", "Outfit", label);
        assert_eq!(mid.len(), 25);
        assert_eq!(mid[0].value, "0");
        assert!(mid[0].label.starts_with("Earlier outfits (1 to 23)"));
        assert_eq!(mid[1].value, "23");
        assert_eq!(mid[24].value, "46");
        assert!(mid[24].label.starts_with("More outfits (47 to 60)"));
        assert!(mid.iter().any(|c| c.value == "30" && c.default));

        // First and last windows carry only the one paging option they need.
        let first = capped_choices(60, 0, "outfits", "Outfit", label);
        assert_eq!(first.len(), 24);
        assert_eq!(first.last().unwrap().value, "23");
        let last = capped_choices(60, 59, "outfits", "Outfit", label);
        assert_eq!(last.len(), 15);
        assert_eq!(last[0].value, "23");

        // Values are unique in every window.
        for selected in 0..60 {
            let choices = capped_choices(60, selected, "outfits", "Outfit", label);
            let mut values: Vec<&str> = choices.iter().map(|c| c.value.as_str()).collect();
            values.sort_unstable();
            values.dedup();
            assert_eq!(values.len(), choices.len());
            assert!(choices.len() <= SELECT_MAX);
        }
    }

    #[test]
    fn blank_labels_get_a_fallback() {
        assert_eq!(Choice::new("  ", "2", None).label, "Option 3");
        assert_eq!(Choice::new("", "md", None).label, "Option md");
        assert_eq!(Choice::new(" Basic Info ", "0", None).label, "Basic Info");
        let choices = capped_choices(3, 0, "files", "File", |i| {
            (
                if i == 1 {
                    " ".to_string()
                } else {
                    format!("F{i}")
                },
                None,
            )
        });
        let labels: Vec<&str> = choices.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, vec!["F0", "File 2", "F2"]);
        assert!(choices.iter().all(|c| !c.label.trim().is_empty()));
    }

    #[test]
    fn long_blocks_split_and_pack_within_limits() {
        let mut lore = Block::titled("Archive File 1");
        lore.description = Some("line of archive text\n".repeat(400)); // ~8,400 chars
        let header = Block::titled("Ling");
        let parts = Parts::pack(vec![header.clone(), lore], &header);
        assert!(parts.len() >= 2);
        for (i, part) in parts.0.iter().enumerate() {
            assert!(part.len() <= EMBEDS_PER_MESSAGE);
            assert!(parts.chars(i, "myrtle.moe · Lore") <= CHARS_PER_MESSAGE);
            for block in part {
                assert!(
                    block
                        .description
                        .as_deref()
                        .map_or(0, |d| d.chars().count())
                        <= 4096
                );
            }
        }

        let mut many = Block::titled("Voice");
        for i in 0..40 {
            many.field(format!("Line {i}"), "x".repeat(300), false);
        }
        let parts = Parts::pack(vec![header.clone(), many], &header);
        let blocks: Vec<&Block> = parts.0.iter().flatten().collect();
        assert!(blocks.iter().all(|b| b.fields.len() <= FIELDS_PER_EMBED));
        assert_eq!(blocks.iter().map(|b| b.fields.len()).sum::<usize>(), 40);

        let mut long = Block::default();
        long.field("Big", "y".repeat(1500), false);
        assert!(
            long.fields
                .iter()
                .all(|f| f.value.chars().count() <= FIELD_MAX)
        );
    }

    #[test]
    fn splits_long_fields_between_lines() {
        let line = "x".repeat(600);
        let text = format!("{line}\n{line}\n{line}");
        let chunks = split_lines(&text, FIELD_MAX);
        assert_eq!(chunks.len(), 3);
        assert!(chunks.iter().all(|c| c.chars().count() <= FIELD_MAX));
        let long = "y".repeat(1500);
        let chunks = split_lines(&long, FIELD_MAX);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].ends_with('…') && chunks[0].chars().count() == FIELD_MAX);
    }
}
