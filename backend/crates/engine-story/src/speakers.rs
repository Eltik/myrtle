//! WHO IS SPEAKING, per line, over a parsed script: the stage's slots, the
//! cut-in plate and the interlude windows, tracked by the same rules the
//! reader's engine plays them by (`frontend/src/lib/story/engine.ts`, and
//! `components/story/reader/speaker.ts` for the lit-slot join), so a name the census pins on a sprite
//! is the name a reader sees beside that sprite lit.
//!
//! Nothing on the wire joins a `[name="Nine"]` line to a sprite key, so the
//! join is the SCENE: the client LIGHTS the speaking slot. The rules mirrored
//! here, each with its engine source:
//!
//! - `[Character(name=, name2=, focus=)]` clears the stage. Two names fill the
//!   left and right slots, one fills the middle. `focus` is an INT, truncated:
//!   absent or 0 lights both, 1 lights slot 1 and dims slot 2, 2 the reverse,
//!   -1 is exactly 1, 3 dims both.
//! - `[charslot(slot=, name=, focus=)]` puts a sprite in one slot (a DIFFERENT
//!   sprite inherits the slot's lit flag, the same one only swaps its art),
//!   and then applies `focus` to every slot: EMPTY or `all`/`a` lights all,
//!   `n`/`none` dims all, a list lights the listed and DIMS the rest. A bare
//!   `[charslot]` empties the stage; a `slot` that is not `l`/`m`/`r` is
//!   ignored whole.
//! - `[charactercutin(name=)]` shows a plate; no name closes it.
//! - `[interlude(channel=, maskid=, type=3, name=, switch=, clear=)]` opens a
//!   radio window on a channel, puts a character in it, flags it `speaking`,
//!   and `clear` closes it.
//!
//! The SPEAKER of a named line, in order:
//!
//! 1. The lit slots. One lit slot is the answer. NOTHING lit and exactly ONE
//!    sprite on stage is still that sprite (`focus="n"` dims a lone Amiya
//!    through `main_15-08_end`).
//! 2. With no speaker on the stage, every PLATE that is shown: the cut-in,
//!    and each interlude window's character.
//! 3. Nothing lit with two or more on stage and no plate, or nothing shown at
//!    all, is genuinely ambiguous and attributes the line to nobody.
//!
//! The plate's place in that order is MEASURED, not assumed. The first cut
//! made a shown plate the whole answer, and the real-data test
//! `plates_over_the_stage_priced_both_ways` prices three readings over the
//! 2,517 EN lines spoken while a plate is up, by the share of their weight
//! that lands on a folder whose primary name (taken from plate-free lines
//! only) IS the line's speaker: plate exclusive 0.4602, plate joins the lit
//! slots 0.4973, plate as fallback 0.5319. Fallback wins on both kinds,
//! cut-ins (1,782 lines: 0.4871 / 0.5015 / 0.5123) and windows (735 lines:
//! 0.3949 / 0.4870 / 0.5793): a call plate stays up through the replies of
//! the people on stage. [`PlateRule`] keeps the other two readings.
//!
//! Where several candidates stand, the line is SPLIT evenly over them (two lit
//! slots count half each). The reader's colour rule breaks that tie by recency
//! instead, because a plate can only take one ink; a census can carry the
//! ambiguity in its weights, so it does not have to guess.

use std::collections::BTreeMap;

use crate::parser::StoryCommand;

/// The three stage slots, in the engine's own frame-state order.
const SLOTS: [&str; 3] = ["l", "m", "r"];

/// The commands that put a sprite up. Every argument of theirs whose key
/// starts with `name` (`name`, `name2`) is a raw sprite name, which is what a
/// caller resolving names ahead of the walk needs to visit.
pub const PLACING_COMMANDS: [&str; 4] = ["character", "charslot", "charactercutin", "interlude"];

/// One sprite on stage.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SlotState {
    /// The raw sprite name as written, trimmed (`avg_npc_043_1#2`).
    name: String,
    /// The folder it resolves to, as the resolver spells it.
    folder: String,
    lit: bool,
}

/// One window of an interlude channel.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Window {
    /// The folder of the character drawn in it.
    character: Option<String>,
}

/// A named line and the folders it is attributed to, with their weights.
/// The weights of one line sum to 1, or the list is empty.
#[derive(Debug, Clone, PartialEq)]
pub struct SpokenLine {
    pub speaker: String,
    pub sprites: Vec<(String, f64)>,
    /// 1-based line number in the script file.
    pub line: u32,
    /// The line's text as written (rich-text tags and `{@nickname}` intact).
    pub text: String,
    /// Which plates were up when the line was spoken: bit 0 a cut-in, bit 1
    /// an interlude window with a character in it. 0 is the stage alone.
    pub plate: u8,
}

/// How the walk reads a shown plate (a cut-in or a window) against the stage.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PlateRule {
    /// The plate is the whole answer while it is up.
    Exclusive,
    /// The plate joins the lit slots as one more candidate.
    Join,
    /// The plate answers only when the stage gives no speaker (the default,
    /// measured best of the three; see the module doc).
    #[default]
    Fallback,
}

/// The walk's one open choice, see [`PlateRule`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SpeakerRules {
    pub plates: PlateRule,
}

/// What one script's walk yields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpeakerWalk {
    /// Every line with a non-empty `[name=...]`, attributed or not.
    pub lines: Vec<SpokenLine>,
    /// Every time a sprite is PUT up (a slot, the cut-in, a window), as
    /// (folder, raw name). This is what "most used expression" counts.
    pub placements: Vec<(String, String)>,
}

impl SpeakerWalk {
    /// Resolve `name` and record it as put up; `None`, and nothing recorded,
    /// when it has no folder, which is when the engine draws nothing.
    fn put_up(
        &mut self,
        name: &str,
        folder_of: &impl Fn(&str) -> Option<String>,
    ) -> Option<String> {
        let folder = folder_of(name)?;
        self.placements.push((folder.clone(), name.to_owned()));
        Some(folder)
    }
}

/// The stage as the walk sees it.
#[derive(Debug, Clone, Default)]
struct Tracker {
    slots: BTreeMap<&'static str, SlotState>,
    /// The cut-in plate's folder.
    cutin: Option<String>,
    /// Interlude channel -> its window.
    windows: BTreeMap<String, Window>,
}

fn slot_of(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "l" | "left" => Some("l"),
        "m" | "middle" => Some("m"),
        "r" | "right" => Some("r"),
        _ => None,
    }
}

/// An integer argument as the engine reads it, `Math.trunc(num(v, 0))`: an
/// unparseable or absent value is 0.
fn engine_int(v: Option<&String>) -> i64 {
    v.and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|f| f.is_finite())
        .map_or(0, |f| f.trunc() as i64)
}

/// `bool(v, false)`: only a written `true` is true.
fn truthy(v: &str) -> bool {
    v.trim().eq_ignore_ascii_case("true")
}

impl Tracker {
    fn put(
        &mut self,
        walk: &mut SpeakerWalk,
        slot: &'static str,
        name: &str,
        lit: bool,
        folder_of: &impl Fn(&str) -> Option<String>,
    ) {
        let name = name.trim();
        if let Some(folder) = walk.put_up(name, folder_of) {
            self.slots.insert(
                slot,
                SlotState {
                    name: name.to_owned(),
                    folder,
                    lit,
                },
            );
        }
    }

    fn character(
        &mut self,
        c: &StoryCommand,
        walk: &mut SpeakerWalk,
        folder_of: &impl Fn(&str) -> Option<String>,
    ) {
        self.slots.clear();
        let name1 = c.args.get("name");
        let name2 = c.args.get("name2");
        if name1.is_none() && name2.is_none() {
            return;
        }
        let focus = engine_int(c.args.get("focus"));
        let lit1 = focus != 2 && focus != 3;
        let lit2 = focus == 0 || focus == 2;
        match (name1, name2) {
            (Some(a), Some(b)) => {
                self.put(walk, "l", a, lit1, folder_of);
                self.put(walk, "r", b, lit2, folder_of);
            }
            (Some(one), None) | (None, Some(one)) => self.put(walk, "m", one, lit1, folder_of),
            (None, None) => {}
        }
    }

    fn charslot(
        &mut self,
        c: &StoryCommand,
        walk: &mut SpeakerWalk,
        folder_of: &impl Fn(&str) -> Option<String>,
    ) {
        let raw_slot = c.args.get("slot");
        let slot = raw_slot.and_then(|s| slot_of(s));
        if slot.is_none() && raw_slot.is_some() {
            return;
        }
        let name = c.args.get("name");
        let focus = c.args.get("focus");
        if slot.is_none() && name.is_none() && focus.is_none() {
            self.slots.clear();
            return;
        }
        if let (Some(slot), Some(name)) = (slot, name) {
            let name = name.trim();
            if let Some(folder) = walk.put_up(name, folder_of) {
                let prev = self.slots.get(slot);
                if prev.is_none_or(|p| p.name != name) {
                    let lit = prev.is_none_or(|p| p.lit);
                    self.slots.insert(
                        slot,
                        SlotState {
                            name: name.to_owned(),
                            folder,
                            lit,
                        },
                    );
                }
            }
        }
        let f = focus
            .map(|s| s.trim().to_ascii_lowercase())
            .unwrap_or_default();
        let all = f.is_empty() || f == "all" || f == "a";
        let listed: Vec<&'static str> = if all || f == "n" || f == "none" {
            Vec::new()
        } else {
            f.split(',').filter_map(slot_of).collect()
        };
        for (key, st) in &mut self.slots {
            st.lit = all || listed.contains(key);
        }
    }

    fn cutin(
        &mut self,
        c: &StoryCommand,
        walk: &mut SpeakerWalk,
        folder_of: &impl Fn(&str) -> Option<String>,
    ) {
        let name = c.args.get("name").map(|s| s.trim()).unwrap_or_default();
        if name.is_empty() {
            self.cutin = None;
            return;
        }
        // An unresolved plate leaves the previous one up, as the engine does.
        if let Some(folder) = walk.put_up(name, folder_of) {
            self.cutin = Some(folder);
        }
    }

    fn interlude(
        &mut self,
        c: &StoryCommand,
        walk: &mut SpeakerWalk,
        folder_of: &impl Fn(&str) -> Option<String>,
    ) {
        let channel = c
            .args
            .get("channel")
            .map_or_else(|| "1".to_owned(), |s| s.trim().to_owned());
        if c.args.get("clear").is_some_and(|v| truthy(v)) {
            self.windows.remove(&channel);
            return;
        }
        let opens = c.args.get("maskid").is_some_and(|m| !m.trim().is_empty());
        if opens {
            self.windows.insert(channel.clone(), Window::default());
        }
        let Some(window) = self.windows.get_mut(&channel) else {
            // No window to draw into: the engine logs it and draws nothing.
            return;
        };
        let kind = engine_int(c.args.get("type"));
        let name = c.args.get("name").map(|s| s.trim()).unwrap_or_default();
        if kind == 3
            && !name.is_empty()
            && let Some(folder) = walk.put_up(name, folder_of)
        {
            window.character = Some(folder);
        }
    }

    fn plate_shown(&self) -> u8 {
        u8::from(self.cutin.is_some())
            | (u8::from(self.windows.values().any(|w| w.character.is_some())) << 1)
    }

    /// The folders a named line belongs to, by the rule in the module doc.
    fn speakers(&self, rules: SpeakerRules) -> Vec<String> {
        let mut plates: Vec<String> = self
            .cutin
            .iter()
            .chain(self.windows.values().filter_map(|w| w.character.as_ref()))
            .cloned()
            .collect();
        if !plates.is_empty() && rules.plates == PlateRule::Exclusive {
            return plates;
        }
        let on_stage: Vec<&SlotState> = SLOTS.iter().filter_map(|k| self.slots.get(k)).collect();
        let mut lit: Vec<String> = on_stage
            .iter()
            .filter(|s| s.lit)
            .map(|s| s.folder.clone())
            .collect();
        if lit.is_empty() && on_stage.len() == 1 {
            lit.push(on_stage[0].folder.clone());
        }
        if rules.plates == PlateRule::Fallback && !lit.is_empty() {
            return lit;
        }
        plates.extend(lit);
        plates
    }
}

/// A display name the census keeps: not empty, and not a mask of `?`, `.`
/// or ellipsis characters (`???`, `？？？`, `...`), which every unrevealed
/// speaker shares and which would name nobody.
#[must_use]
pub fn is_meaningful_name(name: &str) -> bool {
    let t = name.trim();
    !t.is_empty()
        && !t
            .chars()
            .all(|ch| matches!(ch, '?' | '？' | '.' | '…' | '・' | ' ' | '　'))
}

/// Walk one parsed script and attribute every named line. `folder_of` maps a
/// raw sprite name to its folder, or `None` when the extract has no such
/// folder, which is exactly when the engine draws nothing.
#[must_use]
pub fn attribute_speakers(
    commands: &[StoryCommand],
    folder_of: impl Fn(&str) -> Option<String>,
) -> SpeakerWalk {
    attribute_speakers_with(commands, SpeakerRules::default(), folder_of)
}

/// [`attribute_speakers`] under explicit [`SpeakerRules`].
#[must_use]
pub fn attribute_speakers_with(
    commands: &[StoryCommand],
    rules: SpeakerRules,
    folder_of: impl Fn(&str) -> Option<String>,
) -> SpeakerWalk {
    let mut walk = SpeakerWalk::default();
    let mut stage = Tracker::default();
    for c in commands {
        match c.kind.as_str() {
            "character" => stage.character(c, &mut walk, &folder_of),
            "charslot" => stage.charslot(c, &mut walk, &folder_of),
            "charactercutin" => stage.cutin(c, &mut walk, &folder_of),
            "interlude" => stage.interlude(c, &mut walk, &folder_of),
            "name" | "multiline" => {
                let speaker = c.args.get("name").map(|s| s.trim()).unwrap_or_default();
                if speaker.is_empty() {
                    continue;
                }
                let folders = stage.speakers(rules);
                #[allow(clippy::cast_precision_loss)]
                let w = if folders.is_empty() {
                    0.0
                } else {
                    1.0 / folders.len() as f64
                };
                let mut sprites: Vec<(String, f64)> = Vec::new();
                for f in folders {
                    if let Some(seen) = sprites.iter_mut().find(|(s, _)| *s == f) {
                        seen.1 += w;
                    } else {
                        sprites.push((f, w));
                    }
                }
                walk.lines.push(SpokenLine {
                    speaker: speaker.to_owned(),
                    sprites,
                    line: c.line,
                    text: c.text.clone().unwrap_or_default(),
                    plate: stage.plate_shown(),
                });
            }
            _ => {}
        }
    }
    walk
}
