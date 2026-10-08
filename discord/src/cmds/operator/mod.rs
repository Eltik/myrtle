//! `/collection operator`: one operator, page by page.
//!
//! The reply is a set of embeds plus up to three selects: the page, the page's own choice
//! (promotion, skill level, module stage, cost type, summon, outfit, archive file, voice line
//! group), and the part, when a page runs past one message. Each select's `custom_id` carries
//! the whole view ([`state`]), so an interaction re-fetches and re-renders from scratch and a
//! control never expires.
//!
//! Controls belong to whoever ran the command. Anyone else who uses one gets the view they
//! picked as an ephemeral copy whose controls are theirs; the public message is left alone.

pub mod images;
pub mod layout;
pub mod pages;
pub mod state;

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::hash::BuildHasher;
use std::sync::Arc;

use ::serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, CreateActionRow, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, EditInteractionResponse,
};
use ::serenity::builder::CreateEmbed;
use poise::CreateReply;
use poise::serenity_prelude as serenity;

use crate::api::gamedata::{GameData, Operator};
use crate::api::operator_detail::{ParadoxEntry, ParadoxStage, SkinData, TableSkill};
use crate::types::{Data, Error};
use images::{HttpProber, ImageCheck};
use layout::{Choice, Parts, capped_choices};
use pages::{Sources, applicable, build, needs};
use state::{Control, Page, ViewState};

/// A rendered view, ready to send or to edit in.
pub struct Rendered {
    /// A note above the embeds, such as why a requested page isn't shown.
    pub content: Option<String>,
    pub embeds: Vec<CreateEmbed>,
    pub components: Vec<CreateActionRow>,
}

impl Rendered {
    #[must_use]
    pub fn into_reply(self) -> CreateReply {
        let mut reply = CreateReply::default().components(self.components);
        reply.embeds = self.embeds;
        if let Some(content) = self.content {
            reply = reply.content(content);
        }
        reply
    }

    fn into_edit(self) -> EditInteractionResponse {
        EditInteractionResponse::new()
            .content(self.content.unwrap_or_default())
            .embeds(self.embeds)
            .components(self.components)
    }
}

/// The in-game rarity colours, roughly: grey, green, blue, purple, gold, orange.
#[must_use]
pub const fn rarity_colour(rarity: u8) -> u32 {
    match rarity {
        6 => 0x00FF_7F27,
        5 => 0x00FF_C90E,
        4 => 0x00D8_B2FF,
        3 => 0x0000_B2FF,
        2 => 0x00DC_E537,
        _ => 0x00A0_A0A0,
    }
}

/// Fetch what `state`'s page needs and render it. A page this operator doesn't have falls
/// back to the overview, with a note saying so.
pub async fn render(
    gamedata: &GameData,
    images: &ImageCheck,
    client: &reqwest::Client,
    frontend: &str,
    op: &Operator,
    state: &ViewState,
) -> Result<Rendered, Error> {
    // The Voice page is listed only for operators with lines, so the lines are fetched for
    // the Voice page itself and the first time an operator is shown; after that the answer is
    // remembered.
    let known_voices = gamedata.known_voice_presence(&op.id).await;
    let fetch_voices = state.page == Page::Voice || known_voices.is_none();
    let (detail, ranges, paradox, voices) = tokio::join!(
        gamedata.operator_detail(&op.id),
        gamedata.ranges(),
        gamedata.paradox(),
        async {
            if fetch_voices {
                Some(gamedata.voices(&op.id).await)
            } else {
                None
            }
        }
    );
    let detail = detail?;
    // Without the range table the pages still render, just with no range grids.
    let ranges = ranges
        .inspect_err(|e| tracing::warn!("operator view: ranges: {e}"))
        .unwrap_or_default();
    let paradox = match paradox {
        Ok(p) => Some(p),
        Err(e) => {
            tracing::warn!("operator view: Paradox Simulation index: {e}");
            None
        }
    };
    let entry = paradox.as_deref().and_then(|p| p.get(&op.id));
    let voices = match voices {
        Some(Ok(v)) => Some(v),
        Some(Err(e)) if state.page == Page::Voice => return Err(e),
        Some(Err(e)) => {
            tracing::warn!("operator view: voice lines of {}: {e}", op.id);
            None
        }
        None => None,
    };
    let has_voices = match &voices {
        Some(v) => !v.char_words.is_empty(),
        None => known_voices.unwrap_or(false),
    };
    let pages = applicable(&detail, entry.is_some(), has_voices);

    let mut state = state.clone();
    let note = fall_back(&mut state, &pages, op, paradox.is_some());

    let extras = Extras::fetch(gamedata, op, state.page, entry).await?;
    let sources = Sources {
        op,
        detail: &detail,
        ranges: &ranges,
        paradox: entry,
        items: extras.items.as_deref(),
        skill_table: extras.skill_table.as_deref(),
        skins: extras.skins.as_ref(),
        voices: voices.as_ref(),
        paradox_stage: extras.paradox_stage.as_ref().map(|(s, _)| s),
        paradox_preview: extras
            .paradox_stage
            .as_ref()
            .and_then(|(_, p)| p.as_deref()),
        urls: gamedata,
        frontend,
    };
    let view = view(&sources, &pages, &state)?;
    // Always after the defer: the probes run inside the 15 minutes a deferred reply has, not
    // the 3 s an undeferred one does.
    let missing = images
        .missing(&HttpProber(client), &view.image_urls())
        .await;
    Ok(Rendered {
        content: note,
        embeds: view.embeds(&missing),
        components: view.components,
    })
}

/// The responses only some pages read (see [`needs`]), fetched for the page being shown.
struct Extras {
    items: Option<Arc<HashMap<String, String>>>,
    skill_table: Option<Arc<HashMap<String, TableSkill>>>,
    skins: Option<SkinData>,
    /// The Paradox stage's detail and preview.
    paradox_stage: Option<(ParadoxStage, Option<String>)>,
}

impl Extras {
    /// At most two extras per page, and each is one request: no need to run them together.
    async fn fetch(
        gamedata: &GameData,
        op: &Operator,
        page: Page,
        paradox: Option<&ParadoxEntry>,
    ) -> Result<Self, Error> {
        let want = needs(page);
        let items = if want.items {
            // Costs and rewards still render without names, by item id.
            gamedata
                .materials()
                .await
                .inspect_err(|e| tracing::warn!("operator view: item names: {e}"))
                .ok()
        } else {
            None
        };
        let skill_table = if want.skill_table {
            Some(gamedata.skill_table().await?)
        } else {
            None
        };
        let skins = if want.skins {
            Some(gamedata.skins(&op.id).await?)
        } else {
            None
        };
        let paradox_stage = match (want.paradox_stage, paradox) {
            (true, Some(entry)) => Some(paradox_stage(gamedata, &entry.code).await?),
            _ => None,
        };
        Ok(Self {
            items,
            skill_table,
            skins,
            paradox_stage,
        })
    }
}

/// Point `state` at the overview when its page isn't one this operator has, and say why.
fn fall_back(
    state: &mut ViewState,
    pages: &[Page],
    op: &Operator,
    paradox_listed: bool,
) -> Option<String> {
    if pages.contains(&state.page) {
        return None;
    }
    let note = if state.page == Page::Paradox && !paradox_listed {
        "The Paradox Simulation list is unavailable right now; here is the overview.".to_string()
    } else {
        format!(
            "{} has no {} page; here is the overview.",
            op.name,
            state.page.label().to_lowercase()
        )
    };
    state.page = Page::Overview;
    state.sel = None;
    state.part = 0;
    Some(note)
}

/// The Paradox stage's detail and preview. The handbook names the stage by its code
/// (`mem_ling_1`); the stage index maps that to the id the detail route takes
/// (`level_memory_ling_1`) and carries the preview.
async fn paradox_stage(
    gamedata: &GameData,
    code: &str,
) -> Result<(ParadoxStage, Option<String>), Error> {
    let stages = gamedata.stages().await?;
    let indexed = stages.iter().find(|s| s.code == code);
    let stage_id = indexed.map_or_else(
        || format!("level_memory_{}", code.trim_start_matches("mem_")),
        |s| s.stage_id.clone(),
    );
    let detail = gamedata.paradox_stage(&stage_id).await?;
    Ok((detail, indexed.and_then(|s| s.preview.clone())))
}

/// A built view: every part, which one is shown, and the controls.
pub struct View {
    pub components: Vec<CreateActionRow>,
    pub parts: Parts,
    pub part: usize,
    pub colour: u32,
    pub footer: String,
}

impl View {
    /// The thumbnail and image links the shown part sets.
    #[must_use]
    pub fn image_urls(&self) -> Vec<String> {
        self.parts.image_urls(self.part)
    }

    /// The shown part as embeds, leaving out the images in `missing`.
    #[must_use]
    pub fn embeds<S: BuildHasher>(&self, missing: &HashSet<String, S>) -> Vec<CreateEmbed> {
        self.parts
            .embeds(self.part, self.colour, &self.footer, missing)
    }
}

/// Build `state`'s page from `src`, with no network: everything it reads is in `src`.
pub fn view(src: &Sources<'_>, pages: &[Page], state: &ViewState) -> Result<View, Error> {
    let built = build(src, state.page, state.sel.map(usize::from))?;
    let parts = Parts::pack(built.blocks, &built.slim);
    let part = usize::from(state.part).min(parts.len().saturating_sub(1));
    let mut footer = format!("myrtle.moe · {}", state.page.label());
    if let Some(detail) = built.detail.as_deref() {
        let _ = write!(footer, " · {detail}");
    }
    if parts.len() > 1 {
        let _ = write!(footer, " · Part {} of {}", part + 1, parts.len());
    }

    let state = ViewState {
        #[allow(clippy::cast_possible_truncation)]
        part: part as u16,
        ..state.clone()
    };
    let mut components = Vec::new();
    let page_choices: Vec<Choice> = pages
        .iter()
        .map(|p| Choice::new(p.label(), p.code(), None).selected(*p == state.page))
        .collect();
    components.push(select(state.custom_id(Control::Page), "Page", page_choices));
    if built.choices.len() > 1 {
        components.push(select(
            state.custom_id(Control::Sel),
            built.placeholder,
            built.choices,
        ));
    }
    if parts.len() > 1 {
        let n = parts.len();
        let choices = capped_choices(n, part, "parts", "Part", |i| {
            (format!("Part {} of {n}", i + 1), None)
        });
        components.push(select(state.custom_id(Control::Part), "Part", choices));
    }
    Ok(View {
        components,
        parts,
        part,
        colour: rarity_colour(src.op.rarity),
        footer,
    })
}

fn select(custom_id: String, placeholder: &str, choices: Vec<Choice>) -> CreateActionRow {
    let options = choices
        .into_iter()
        .map(|c| {
            let mut option =
                CreateSelectMenuOption::new(c.label, c.value).default_selection(c.default);
            if let Some(d) = c.description {
                option = option.description(d);
            }
            option
        })
        .collect();
    let mut menu = CreateSelectMenu::new(custom_id, CreateSelectMenuKind::String { options });
    if !placeholder.is_empty() {
        menu = menu.placeholder(placeholder);
    }
    CreateActionRow::SelectMenu(menu)
}

/// Whether a component interaction is one of this view's controls.
#[must_use]
pub fn is_ours(custom_id: &str) -> bool {
    custom_id
        .split_once(':')
        .is_some_and(|(prefix, _)| prefix == state::PREFIX)
}

/// Answer a select on an operator view. Every failure ends in an ephemeral message to the
/// user who chose; nothing fails silently.
pub async fn handle_component(
    ctx: &serenity::Context,
    data: &Data,
    interaction: &ComponentInteraction,
) {
    if let Err(e) = handle(ctx, data, interaction).await {
        tracing::warn!(
            "operator view: interaction {}: {e}",
            interaction.data.custom_id
        );
    }
}

async fn handle(
    ctx: &serenity::Context,
    data: &Data,
    interaction: &ComponentInteraction,
) -> Result<(), Error> {
    let value = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => values.first().cloned(),
        _ => None,
    };
    let next = ViewState::parse(&interaction.data.custom_id).and_then(|(state, control)| {
        value
            .as_deref()
            .and_then(|v| state.apply(control, v))
            .map(|next| (state.owner, next))
    });
    let Some((owner, mut next)) = next else {
        let message = CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .content("This control isn't one I can read. Run `/collection operator` again.");
        interaction
            .create_response(&ctx.http, CreateInteractionResponse::Message(message))
            .await?;
        return Ok(());
    };

    let user = interaction.user.id.get();
    let own = user == owner;
    next.owner = user;
    // Deferring answers inside Discord's 3 s window; the fetches can take longer.
    if own {
        interaction.defer(&ctx.http).await?;
    } else {
        interaction.defer_ephemeral(&ctx.http).await?;
    }

    let rendered = async {
        let operators = data.gamedata.operators().await?;
        let op = operators
            .iter()
            .find(|o| o.id == next.op)
            .ok_or("That operator is no longer in the index.")?;
        render(
            &data.gamedata,
            &data.operator_images,
            &data.http_client,
            data.frontend(),
            op,
            &next,
        )
        .await
    }
    .await;

    // The reason goes to the log; the user gets a plain sentence with no URLs or internals.
    match rendered {
        Ok(view) => match interaction.edit_response(&ctx.http, view.into_edit()).await {
            Ok(_) => return Ok(()),
            Err(e) => tracing::error!("operator view: editing in {:?}: {e}", next),
        },
        Err(e) => tracing::warn!("operator view: rendering {:?}: {e}", next),
    }
    let failure = "Couldn't show that view right now. Try again in a minute.";
    if own {
        interaction
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .ephemeral(true)
                    .content(failure),
            )
            .await?;
    } else {
        interaction
            .edit_response(&ctx.http, EditInteractionResponse::new().content(failure))
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::is_ours;

    #[test]
    fn claims_only_its_own_prefix() {
        assert!(is_ours("op:anything"));
        assert!(is_ours("op:"));
        assert!(!is_ours("op"));
        assert!(!is_ours("opx:1"));
        assert!(!is_ours("other:op:1"));
        assert!(!is_ours(""));
    }
}
