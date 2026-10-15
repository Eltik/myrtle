//! `/voiceline`: play an operator's voice line.
//!
//! The line plays in the invoker's voice channel through the TTS session (`crate::tts`), queued
//! behind whatever is being read out. With the bot not in voice, it joins the invoker's channel
//! to play the line, without reading the channel's chat. Wherever the line can't play in voice
//! (the invoker isn't in one, the bot is in another channel, the queue is full), the reply
//! carries the recording as an audio file instead, with the reason.
//!
//! Lines are the operator's own voice set, the one the `/collection` Voice page lists. Outfit
//! and dialect sets (Ling's `nian#12`, `CN_TOPOLECT`) aren't offered.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ::serenity::builder::{CreateAttachment, CreateEmbed, CreateEmbedFooter};
use poise::CreateReply;
use poise::serenity_prelude as serenity;

use crate::api::gamedata::{GameData, Operator};
use crate::api::operator_detail::{VoiceClip, VoiceData, VoiceLine};
use crate::checks::require_guild;
use crate::cmds::collection::{
    autocomplete_operator, no_match, operator_names, operator_priority, resolve,
};
use crate::cmds::operator::pages::voice_set;
use crate::cmds::{begin_lookup, list_unavailable};
use crate::gametext::strip_rich_text;
use crate::search::search;
use crate::tts::{self, JoinRefused, session::Utterance};
use crate::types::{Context, Error};
use crate::ui::{AUTOCOMPLETE_MAX, CHOICE_NAME_MAX, COLOR_INFO, DESCRIPTION_MAX, TITLE_MAX};
use crate::utils::ellipsize;

/// The game's voice-over languages, as `/api/voices` names them, in the frontend's order, with
/// the frontend's English labels. The first one a line has is its default.
const LANGUAGES: [(&str, &str); 11] = [
    ("JP", "Japanese"),
    ("CN_MANDARIN", "Mandarin"),
    ("EN", "English"),
    ("KR", "Korean"),
    ("CN_TOPOLECT", "Cantonese"),
    ("GER", "German"),
    ("ITA", "Italian"),
    ("RUS", "Russian"),
    ("FRE", "French"),
    ("SPA", "Spanish"),
    ("LINKAGE", "Linkage"),
];

/// How long the line and language autocompletes wait on a voice-line fetch they don't have
/// cached. Discord drops an autocomplete answer after 3 s.
const AUTOCOMPLETE_FETCH: Duration = Duration::from_millis(2500);

/// Play an operator's voice line in your voice channel.
///
/// The line plays in the voice channel you're in, after anything being read out there. Out of
/// voice, the reply carries the line as an audio file.
#[poise::command(slash_command, guild_only)]
pub async fn voiceline(
    ctx: Context<'_>,
    #[description = "Operator name"]
    #[autocomplete = "autocomplete_operator"]
    operator: String,
    #[description = "Which line (default a random one)"]
    #[autocomplete = "autocomplete_line"]
    line: Option<String>,
    #[description = "Voice-over language (default Japanese, else the first the line has)"]
    #[autocomplete = "autocomplete_language"]
    language: Option<String>,
) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let gamedata = begin_lookup(ctx).await?;
    let operators = gamedata
        .operators()
        .await
        .map_err(|e| list_unavailable("voiceline", "operator", &e))?;
    let op = resolve(
        &operators,
        &operator,
        |o| o.id.as_str(),
        operator_names,
        operator_priority,
    )
    .ok_or_else(|| no_match("operator", &operator))?;

    // The voice lines, the recording and joining voice can each outlast the 3 s an interaction
    // allows. Errors from here on answer publicly, in place of the "thinking" placeholder.
    ctx.defer().await?;
    let data = gamedata.voice_lines(&op.id).await.map_err(|e| -> Error {
        tracing::warn!("voiceline: voice lines of {}: {e}", op.id);
        format!(
            "Couldn't load {}'s voice lines right now. Try again in a minute.",
            op.name
        )
        .into()
    })?;
    let lines = voice_set(&data, &op.id);
    if lines.is_empty() {
        return Err(format!("{} has no voice lines.", op.name).into());
    }
    let picked = match line.as_deref() {
        Some(query) => find_line(&lines, query).ok_or_else(|| {
            format!(
                "{} has no line matching `{}`. Pick one from the suggestions.",
                op.name,
                ellipsize(query.trim(), 100)
            )
        })?,
        None => lines[random_index(lines.len())],
    };
    let clip = pick_clip(picked, language.as_deref())?;

    let audio = match gamedata.voice_clip(&clip.voice_url).await {
        Ok(audio) => audio,
        Err(e) => {
            tracing::warn!("voiceline: {}: {e}", clip.voice_url);
            ctx.send(CreateReply::default().content(format!(
                "Couldn't fetch {}'s \"{}\" right now. Try again in a minute.",
                op.name, picked.voice_title
            )))
            .await?;
            return Ok(());
        }
    };

    let embed = line_embed(&gamedata, op, picked, clip);
    let reply = match play_in_voice(ctx, guild, audio.clone()).await {
        Ok(status) => CreateReply::default().content(status).embed(embed),
        Err(why) => {
            let file = CreateAttachment::bytes(audio, clip_filename(op, picked, clip));
            CreateReply::default()
                .content(why.unwrap_or_default())
                .embed(embed)
                .attachment(file)
        }
    };
    ctx.send(reply).await?;
    Ok(())
}

/// Queue `audio` in the invoker's voice channel, joining it when the bot isn't in voice.
///
/// `Ok` carries the status line for the reply. `Err` means the line goes out as a file, with
/// the reason to show, or `None` when the invoker simply isn't in voice.
async fn play_in_voice(
    ctx: Context<'_>,
    guild: serenity::GuildId,
    audio: Vec<u8>,
) -> Result<String, Option<String>> {
    let tts = &ctx.data().tts;
    let (channel, stage) =
        tts::voice_channel_of(ctx.serenity_context(), guild, ctx.author().id).ok_or(None)?;
    if stage {
        return Err(Some(
            "I don't play in stage channels, so here it is as a file.".into(),
        ));
    }
    let session = match tts.start_session(guild, channel, false) {
        Ok((session, joined)) => {
            if !joined.await.unwrap_or(false) {
                return Err(Some(format!(
                    "I couldn't join <#{channel}>, so here it is as a file. Check that I can \
                     connect and speak there, and that it isn't full."
                )));
            }
            session
        }
        Err(JoinRefused::AlreadyHere) => match tts.session(guild) {
            Some(session) if !session.is_stopping() => session,
            _ => {
                return Err(Some(format!(
                    "I'm leaving <#{channel}>, so here it is as a file."
                )));
            }
        },
        Err(JoinRefused::Elsewhere(other)) => {
            return Err(Some(format!(
                "I'm in <#{other}> in this server, so here it is as a file."
            )));
        }
        Err(JoinRefused::Unavailable) => {
            return Err(Some(
                "Voice playback is switched off on this bot, so here it is as a file.".into(),
            ));
        }
        Err(JoinRefused::TooSoon) => {
            return Err(Some(
                "I joined or left voice here a moment ago, so here it is as a file.".into(),
            ));
        }
    };
    let waiting = session.busy() || session.queued() > 0;
    if !session.push(ctx.author().id, Utterance::Clip(audio)) {
        return Err(Some(format!(
            "The queue in <#{channel}> is full, so here it is as a file."
        )));
    }
    Ok(if waiting {
        format!("Queued in <#{channel}>.")
    } else {
        format!("Playing in <#{channel}>.")
    })
}

/// The line whose id is `query` (an autocomplete pick), else the best match on title and text.
fn find_line<'a>(lines: &[&'a VoiceLine], query: &str) -> Option<&'a VoiceLine> {
    let query = query.trim();
    lines
        .iter()
        .find(|l| l.char_word_id == query)
        .or_else(|| {
            search(lines, query, 1, line_names, |_| 0)
                .into_iter()
                .next()
        })
        .copied()
}

fn line_names<'a>(line: &'a &VoiceLine) -> Vec<&'a str> {
    let mut names = vec![line.voice_title.as_str()];
    names.extend(line.voice_text.as_deref());
    names
}

/// The recording in `language` (a code or a label, any case), else Japanese, else the first
/// language the line has.
fn pick_clip<'a>(line: &'a VoiceLine, language: Option<&str>) -> Result<&'a VoiceClip, Error> {
    let clip = match language {
        Some(wanted) => {
            let code = language_code(wanted);
            line.data
                .iter()
                .find(|c| c.language.eq_ignore_ascii_case(code))
                .ok_or_else(|| {
                    format!(
                        "\"{}\" isn't recorded in {}. It has: {}.",
                        line.voice_title,
                        ellipsize(wanted.trim(), 40),
                        languages_of(line).join(", ")
                    )
                })?
        }
        None => LANGUAGES
            .iter()
            .find_map(|(code, _)| line.data.iter().find(|c| c.language == *code))
            .or_else(|| line.data.first())
            .ok_or_else(|| format!("\"{}\" has no recording.", line.voice_title))?,
    };
    Ok(clip)
}

/// The code for a language given as a code or a label (`en`, `English`, `CN_MANDARIN`).
/// Anything unknown is returned as typed, to be compared with the codes as they come.
fn language_code(typed: &str) -> &str {
    let typed = typed.trim();
    LANGUAGES
        .iter()
        .find(|(code, label)| code.eq_ignore_ascii_case(typed) || label.eq_ignore_ascii_case(typed))
        .map_or(typed, |(code, _)| code)
}

fn language_label(code: &str) -> &str {
    LANGUAGES
        .iter()
        .find(|(c, _)| *c == code)
        .map_or(code, |(_, label)| label)
}

/// The line's languages as labels, in [`LANGUAGES`] order, then any the table doesn't know.
fn languages_of(line: &VoiceLine) -> Vec<&str> {
    let mut known: Vec<&str> = LANGUAGES
        .iter()
        .filter(|(code, _)| line.data.iter().any(|c| c.language == *code))
        .map(|(_, label)| *label)
        .collect();
    known.extend(
        line.data
            .iter()
            .map(|c| c.language.as_str())
            .filter(|l| LANGUAGES.iter().all(|(code, _)| code != l)),
    );
    known
}

/// An index under `len`, from the clock: a random line needs no better randomness.
fn random_index(len: usize) -> usize {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    usize::try_from(nanos).unwrap_or(0) % len.max(1)
}

fn line_embed(
    gamedata: &GameData,
    op: &Operator,
    line: &VoiceLine,
    clip: &VoiceClip,
) -> CreateEmbed {
    let title = format!("{}: {}", op.name, line.voice_title);
    let text = line
        .voice_text
        .as_deref()
        .map(strip_rich_text)
        .unwrap_or_default();
    let mut footer = language_label(&clip.language).to_string();
    if !clip.cv_name.is_empty() {
        footer.push_str(" · CV ");
        footer.push_str(&clip.cv_name.join(", "));
    }
    CreateEmbed::new()
        .title(ellipsize(&title, TITLE_MAX))
        .description(ellipsize(&text, DESCRIPTION_MAX))
        .color(COLOR_INFO)
        .thumbnail(gamedata.api_url(&format!("/avatar/{}", op.id)))
        .footer(CreateEmbedFooter::new(footer))
}

/// `Amiya_Idle_JP.ogg`: letters, digits, `-` and `_` only.
fn clip_filename(op: &Operator, line: &VoiceLine, clip: &VoiceClip) -> String {
    let clean = |s: &str| -> String {
        let s: String = s
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        s.trim_matches('_').to_string()
    };
    let parts: Vec<String> = [
        op.name.as_str(),
        line.voice_title.as_str(),
        clip.language.as_str(),
    ]
    .into_iter()
    .map(clean)
    .filter(|p| !p.is_empty())
    .collect();
    if parts.is_empty() {
        "voiceline.ogg".into()
    } else {
        format!("{}.ogg", parts.join("_"))
    }
}

// ---------------------------------------------------------------------------
// Autocomplete
// ---------------------------------------------------------------------------

/// The value typed or picked so far for another option of the command.
fn option_value(ctx: Context<'_>, name: &str) -> Option<String> {
    let poise::Context::Application(app) = ctx else {
        return None;
    };
    app.interaction
        .data
        .options
        .iter()
        .find(|o| o.name == name)
        .and_then(|o| match &o.value {
            serenity::CommandDataOptionValue::String(s) => Some(s.clone()),
            serenity::CommandDataOptionValue::Autocomplete { value, .. } => Some(value.clone()),
            _ => None,
        })
}

/// The voice lines of the operator in the `operator` option, from the cache or a fetch that
/// fits inside the autocomplete deadline.
async fn chosen_voice_lines(ctx: Context<'_>) -> Option<(String, Arc<VoiceData>)> {
    let query = option_value(ctx, "operator")?;
    let gamedata = &ctx.data().gamedata;
    let operators = gamedata.cached_operators().await?;
    let id = resolve(
        &operators,
        &query,
        |o| o.id.as_str(),
        operator_names,
        operator_priority,
    )?
    .id
    .clone();
    let data = tokio::time::timeout(AUTOCOMPLETE_FETCH, gamedata.voice_lines(&id))
        .await
        .ok()?
        .ok()?;
    Some((id, data))
}

async fn autocomplete_line(ctx: Context<'_>, partial: &str) -> Vec<serenity::AutocompleteChoice> {
    let Some((id, data)) = chosen_voice_lines(ctx).await else {
        return Vec::new();
    };
    let lines = voice_set(&data, &id);
    let hits: Vec<&&VoiceLine> = if partial.trim().is_empty() {
        lines.iter().take(AUTOCOMPLETE_MAX).collect()
    } else {
        search(&lines, partial, AUTOCOMPLETE_MAX, line_names, |_| 0)
    };
    hits.into_iter()
        .map(|line| {
            let text = line
                .voice_text
                .as_deref()
                .map(strip_rich_text)
                .unwrap_or_default();
            let label = if text.is_empty() {
                line.voice_title.clone()
            } else {
                format!("{}: {text}", line.voice_title)
            };
            serenity::AutocompleteChoice::new(
                ellipsize(&label, CHOICE_NAME_MAX),
                line.char_word_id.clone(),
            )
        })
        .collect()
}

async fn autocomplete_language(
    ctx: Context<'_>,
    partial: &str,
) -> Vec<serenity::AutocompleteChoice> {
    let needle = partial.trim().to_lowercase();
    let lines = chosen_voice_lines(ctx).await;
    let has = |code: &str| match &lines {
        Some((id, data)) => voice_set(data, id)
            .iter()
            .any(|l| l.data.iter().any(|c| c.language == code)),
        None => true,
    };
    LANGUAGES
        .iter()
        .filter(|(code, label)| {
            has(code)
                && (needle.is_empty()
                    || code.to_lowercase().contains(&needle)
                    || label.to_lowercase().contains(&needle))
        })
        .map(|(code, label)| serenity::AutocompleteChoice::new(*label, *code))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(language: &str) -> VoiceClip {
        VoiceClip {
            voice_url: format!("/audio/{language}.ogg"),
            language: language.into(),
            cv_name: vec!["Someone".into()],
        }
    }

    fn line(id: &str, title: &str, text: &str, languages: &[&str]) -> VoiceLine {
        VoiceLine {
            char_word_id: id.into(),
            word_key: "char_002_amiya".into(),
            voice_title: title.into(),
            voice_text: Some(text.into()),
            voice_index: 0,
            place_type: "HOME_WAIT".into(),
            data: languages.iter().map(|l| clip(l)).collect(),
        }
    }

    #[test]
    fn language_defaults_to_japanese_then_the_table_order() {
        let all = line("a", "Idle", "...", &["KR", "EN", "JP", "CN_MANDARIN"]);
        assert_eq!(pick_clip(&all, None).unwrap().language, "JP");
        let no_jp = line("a", "Idle", "...", &["KR", "EN"]);
        assert_eq!(pick_clip(&no_jp, None).unwrap().language, "EN");
        let odd = line("a", "Idle", "...", &["XX"]);
        assert_eq!(pick_clip(&odd, None).unwrap().language, "XX");
    }

    #[test]
    fn language_by_code_or_label_in_any_case() {
        let l = line("a", "Idle", "...", &["KR", "EN", "JP", "CN_MANDARIN"]);
        for typed in ["EN", "en", "English", " english "] {
            assert_eq!(
                pick_clip(&l, Some(typed)).unwrap().language,
                "EN",
                "{typed}"
            );
        }
        assert_eq!(
            pick_clip(&l, Some("mandarin")).unwrap().language,
            "CN_MANDARIN"
        );
        let missing = pick_clip(&l, Some("Italian")).unwrap_err().to_string();
        assert!(
            missing.contains("Japanese, Mandarin, English, Korean"),
            "lists what the line has, in order: {missing}"
        );
    }

    #[test]
    fn line_by_pick_or_by_text() {
        let a = line(
            "char_002_amiya_CN_010",
            "Idle",
            "Lots of work to do.",
            &["JP"],
        );
        let b = line(
            "char_002_amiya_CN_001",
            "Appointed as Assistant",
            "Doctor, I'm here.",
            &["JP"],
        );
        let lines = vec![&a, &b];
        assert_eq!(
            find_line(&lines, "char_002_amiya_CN_001")
                .unwrap()
                .char_word_id,
            b.char_word_id
        );
        assert_eq!(
            find_line(&lines, "idle").unwrap().char_word_id,
            a.char_word_id
        );
        assert_eq!(
            find_line(&lines, "assistant").unwrap().char_word_id,
            b.char_word_id
        );
        assert!(find_line(&lines, "zzzz").is_none());
    }

    /// The real path against the public backend: Amiya's and Ling's own sets resolve, every
    /// language of a line fetches as Ogg, and the clip decodes through songbird. Needs the
    /// network, so it is ignored by default:
    /// `cargo test -- --ignored --nocapture voice_lines_fetch_live`.
    #[tokio::test]
    #[ignore = "calls api.myrtle.moe"]
    async fn voice_lines_fetch_live() {
        use songbird::input::Input;
        use songbird::input::codecs::{get_codec_registry, get_probe};

        let gamedata = GameData::new(reqwest::Client::new(), "https://api.myrtle.moe");
        for id in ["char_002_amiya", "char_2023_ling"] {
            let data = gamedata.voice_lines(id).await.expect("voice lines load");
            let lines = voice_set(&data, id);
            assert!(lines.iter().all(|l| l.word_key == id), "{id}: own set only");
            let idle = find_line(&lines, "idle").expect("an idle line");
            for clip in &idle.data {
                let audio = gamedata.voice_clip(&clip.voice_url).await.expect("clip");
                let mut input = Input::from(audio.clone())
                    .make_playable_async(get_codec_registry(), get_probe())
                    .await
                    .expect("songbird probes the clip");
                assert!(input.parsed_mut().is_some());
                println!(
                    "{id} {} {}: {} B",
                    idle.char_word_id,
                    clip.language,
                    audio.len()
                );
            }
            println!(
                "{id}: {} lines, default {}",
                lines.len(),
                pick_clip(idle, None).unwrap().language
            );
        }
    }

    #[test]
    fn random_index_stays_in_range() {
        for len in [1, 2, 7, 38] {
            assert!(random_index(len) < len);
        }
    }
}
