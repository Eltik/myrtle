//! The game's own "from the archive" screen for one story group.
//!
//! What this module OWNS is the join from `story_review_meta_table` (the slots
//! a group's archive lists) and `activity_table.MissionArchives` (the
//! recordings shelf, which is keyed by zone and sits in a different table
//! entirely) to the resolved sections the reader draws. Its art is resolved at
//! BUILD time, off the game data the index cache is keyed by, so a re-extract
//! that moves a file rebuilds the URLs with the index.
//!
//! Scale: the whole archive half is 6 EN groups and about 300 leaf items, and
//! the build is 31 ms of the 7,279 ms debug index build, the `textures/ui` walk
//! included. That is why it is cached with the index rather than assembled per
//! request. Section counts and the on-disk census are in
//! `docs/story-reader.md`, section "1. What is true about the data".

use std::collections::{HashMap, HashSet};

use super::cache::cached_index;
use super::dto::{
    ArchiveClip, ArchiveClipTrack, ArchiveFile, ArchiveLandmark, ArchiveLog, ArchiveLogChapter,
    ArchiveNewsFormat, ArchiveNewsItem, ArchiveNewsLine, ArchivePicture, ArchiveRecordingNode,
    ArchiveTrack, StoryArchive, StoryArchiveSection,
};
use super::opt;
use crate::app::{error::ApiError, state::AppState};
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::story_review_meta::ActArchiveComponent;
use crate::core::gamedata::types::voice::LangType;
use crate::core::hypergryph::constants::Server;
use crate::core::story::StoryAssetIndex;

/// The audio directories a recorded clip can sit in, with the language each
/// one is. There is no `voice_jp` in the tree: JAPANESE is the bare `voice`
/// directory, the same convention the operator voice lines follow.
const CLIP_LANGS: [(&str, LangType); 4] = [
    ("voice_cn", LangType::CnMandarin),
    ("voice", LangType::Jp),
    ("voice_en", LangType::En),
    ("voice_kr", LangType::Kr),
];

/// The library group a `story_review_meta_table` component belongs to.
///
/// Every event is listed TWICE, live (`act17side`) and retro (`act17sre`),
/// with identical slots, and only the live id is a library group on EN. The
/// retro key is therefore mapped back to its live id and only used when the
/// live component is missing, so a tree that ships only the retro half still
/// answers.
fn archive_group_id(gd: &GameData, key: &str) -> Option<String> {
    if gd.story_reviews.contains_key(key) {
        return Some(key.to_owned());
    }
    let live = format!("{}side", key.strip_suffix("sre")?);
    gd.story_reviews.contains_key(&live).then_some(live)
}

/// Component key -> library group id, live component winning over retro.
fn archive_components(gd: &GameData) -> HashMap<String, &ActArchiveComponent> {
    let mut keys: Vec<&String> = gd
        .story_archives
        .act_archive_data
        .components
        .keys()
        .collect();
    // Direct hits first, so a retro twin can only fill a gap.
    keys.sort_by_key(|k| (!gd.story_reviews.contains_key(k.as_str()), (*k).clone()));
    let mut out: HashMap<String, &ActArchiveComponent> = HashMap::new();
    for key in keys {
        if let Some(group_id) = archive_group_id(gd, key) {
            out.entry(group_id)
                .or_insert(&gd.story_archives.act_archive_data.components[key]);
        }
    }
    out
}

/// The art the archive draws, by lowercased stem.
///
/// Neither index the reader already has covers it: the landmark plates are
/// loose files under `textures/ui/[uc]deepsea` and the news mastheads, file
/// plates and inline pictures are in the `act_archive*` spritepacks, while
/// `StoryAssetIndex` walks `textures/avg` and `AssetIndex` classifies by
/// directory name. The walk is therefore its own, and it is NARROW: the
/// wanted stems are collected from the table first, so the 11,161 files under
/// `textures/ui` cost one `read_dir` pass each and nothing is retained but
/// the handful that were asked for.
fn archive_art(assets_dir: &std::path::Path, wanted: &HashSet<String>) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = HashMap::new();
    if wanted.is_empty() {
        return out;
    }
    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(dir) = std::fs::read_dir(assets_dir.join("textures/spritepack")) {
        for entry in dir.flatten() {
            if entry
                .file_name()
                .to_str()
                .is_some_and(|n| n.starts_with("act_archive"))
            {
                roots.push(entry.path());
            }
        }
    }
    roots.sort();
    // The spritepacks are walked first, so a stem that exists in both trees
    // is served from the archive's own pack.
    roots.push(assets_dir.join("textures/ui"));
    for root in roots {
        for entry in walkdir::WalkDir::new(&root)
            .min_depth(1)
            .sort_by_file_name()
        {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path
                .extension()
                .is_none_or(|e| !e.eq_ignore_ascii_case("png"))
            {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let stem = stem.to_ascii_lowercase();
            if !wanted.contains(&stem) || out.contains_key(&stem) {
                continue;
            }
            let Some(rel) = path
                .strip_prefix(assets_dir)
                .ok()
                .and_then(std::path::Path::to_str)
            else {
                continue;
            };
            out.insert(stem, format!("/{rel}"));
        }
    }
    out
}

/// Every art stem the archives of `components` ask for, lowercased.
fn wanted_archive_art(
    gd: &GameData,
    components: &HashMap<String, &ActArchiveComponent>,
) -> HashSet<String> {
    let res = &gd.story_archives.act_archive_res_data;
    let mut wanted: HashSet<String> = HashSet::new();
    let mut want = |s: &str| {
        let s = s.trim();
        if !s.is_empty() {
            wanted.insert(s.to_ascii_lowercase());
        }
    };
    for component in components.values() {
        if let Some(list) = component.landmark.as_ref() {
            for entry in list {
                if let Some(l) = res.landmarks.get(&entry.value.id) {
                    want(&l.landmark_pic);
                }
            }
        }
        for item in component.story.iter().flat_map(|s| &s.items) {
            if let Some(s) = res.stories.get(&item.value.id) {
                want(&s.pic);
                want(&s.title_pic);
            }
        }
        for item in component.news.iter().flat_map(|s| &s.items) {
            let Some(n) = res.news.get(&item.value.id) else {
                continue;
            };
            if let Some(f) = n.news_format.as_ref() {
                want(&f.type_logo);
                want(&f.type_main_logo);
            }
            for line in &n.news_lines {
                if line.line_type == "ImageContent" {
                    want(&line.content);
                }
            }
        }
    }
    wanted
}

/// A slot's entries in the act's own order, dropping the ids the content half
/// does not carry.
fn slot_ids(
    slot: Option<&crate::core::gamedata::types::story_review_meta::ArchiveSlot>,
) -> Vec<String> {
    let Some(slot) = slot else { return Vec::new() };
    let mut items: Vec<_> = slot.items.iter().collect();
    items.sort_by(|a, b| {
        a.value
            .sort_id
            .cmp(&b.value.sort_id)
            .then_with(|| a.key.cmp(&b.key))
    });
    items.into_iter().map(|i| i.value.id.clone()).collect()
}

/// One group's archive, art and audio already resolved.
fn build_archive(
    gd: &GameData,
    component: &ActArchiveComponent,
    group_id: &str,
    story_assets: &StoryAssetIndex,
    art: &HashMap<String, String>,
) -> StoryArchive {
    let res = &gd.story_archives.act_archive_res_data;
    let url = |name: &str| art.get(&name.trim().to_ascii_lowercase()).cloned();
    let mut sections: Vec<StoryArchiveSection> = Vec::new();

    if let Some(chapters_raw) = component.log.as_ref() {
        let mut count = 0_u32;
        let chapters: Vec<ArchiveLogChapter> = chapters_raw
            .iter()
            .map(|c| {
                let logs: Vec<ArchiveLog> = c
                    .value
                    .logs
                    .iter()
                    .filter_map(|id| {
                        res.logs.get(id).map(|l| ArchiveLog {
                            id: l.log_id.clone(),
                            text: l.log_desc.clone(),
                        })
                    })
                    .collect();
                count += u32::try_from(logs.len()).unwrap_or(0);
                ArchiveLogChapter {
                    id: c.key.clone(),
                    name: c.value.chapter_name.clone(),
                    display_id: opt(&c.value.display_id),
                    chapter_icon: opt(&c.value.chapter_icon),
                    unlock_desc: opt(&c.value.unlock_des),
                    logs,
                }
            })
            .collect();
        if count > 0 {
            sections.push(StoryArchiveSection::Logs { count, chapters });
        }
    }

    if let Some(entries) = component.landmark.as_ref() {
        let mut entries: Vec<_> = entries.iter().collect();
        entries.sort_by(|a, b| {
            a.value
                .sort_id
                .cmp(&b.value.sort_id)
                .then_with(|| a.key.cmp(&b.key))
        });
        let landmarks: Vec<ArchiveLandmark> = entries
            .into_iter()
            .filter_map(|e| res.landmarks.get(&e.value.id))
            .map(|l| ArchiveLandmark {
                id: l.landmark_id.clone(),
                name: l.landmark_name.clone(),
                eng_name: opt(&l.landmark_eng_name).filter(|n| *n != l.landmark_name),
                description: l.landmark_desc.clone(),
                picture_url: url(&l.landmark_pic),
            })
            .collect();
        if !landmarks.is_empty() {
            sections.push(StoryArchiveSection::Landmarks {
                count: u32::try_from(landmarks.len()).unwrap_or(0),
                landmarks,
            });
        }
    }

    let news: Vec<ArchiveNewsItem> = slot_ids(component.news.as_ref())
        .iter()
        .filter_map(|id| res.news.get(id))
        .map(|n| ArchiveNewsItem {
            id: n.id.clone(),
            title: n.desc.clone(),
            author: opt(&n.news_author),
            format: n.news_format.as_ref().map(|f| ArchiveNewsFormat {
                type_id: f.type_id.clone(),
                type_name: f.type_name.clone(),
                logo_url: url(&f.type_logo),
                main_logo_url: url(&f.type_main_logo),
            }),
            lines: n
                .news_lines
                .iter()
                .map(|l| {
                    if l.line_type == "ImageContent" {
                        ArchiveNewsLine::Image {
                            name: l.content.clone(),
                            url: url(&l.content),
                        }
                    } else {
                        ArchiveNewsLine::Text {
                            text: l.content.clone(),
                        }
                    }
                })
                .collect(),
        })
        .collect();
    if !news.is_empty() {
        sections.push(StoryArchiveSection::News {
            count: u32::try_from(news.len()).unwrap_or(0),
            news,
        });
    }

    let files: Vec<ArchiveFile> = slot_ids(component.story.as_ref())
        .iter()
        .filter_map(|id| res.stories.get(id))
        .map(|s| ArchiveFile {
            id: s.id.clone(),
            title: s.desc.clone(),
            date: opt(&s.date),
            text: s.text.clone(),
            picture_url: url(&s.pic),
            title_picture_url: url(&s.title_pic),
        })
        .collect();
    if !files.is_empty() {
        sections.push(StoryArchiveSection::Files {
            count: u32::try_from(files.len()).unwrap_or(0),
            files,
        });
    }

    let pictures: Vec<ArchivePicture> = slot_ids(component.pic.as_ref())
        .iter()
        .filter_map(|id| res.pics.get(id))
        .map(|p| ArchivePicture {
            id: p.id.clone(),
            title: p.desc.clone(),
            description: opt(&p.pic_description),
            picture_type: p.type_.clone(),
            url: story_assets.resolve_image(&p.asset_path).map(|(u, _)| u),
        })
        .collect();
    if !pictures.is_empty() {
        sections.push(StoryArchiveSection::Gallery {
            count: u32::try_from(pictures.len()).unwrap_or(0),
            pictures,
        });
    }

    let tracks: Vec<ArchiveTrack> = slot_ids(component.music.as_ref())
        .iter()
        .filter_map(|id| {
            let audio = gd.story_archives.act_archive_res_data.audios.get(id);
            let bank = gd
                .music
                .music(id)
                .and_then(|m| gd.music.bank(&m.bank))
                .or_else(|| gd.music.bank(id))?;
            let loop_url = story_assets.resolve_audio(bank.loop_.as_deref()?)?;
            Some(ArchiveTrack {
                id: id.clone(),
                name: audio.and_then(|a| opt(&a.name)),
                // The description repeats the id on all 82 EN rows, so it
                // is dropped when it does and nothing is ever sent.
                description: audio.and_then(|a| opt(&a.desc)).filter(|d| d != id),
                intro_url: bank
                    .intro
                    .as_deref()
                    .and_then(|i| story_assets.resolve_audio(i)),
                loop_url,
            })
        })
        .collect();
    if !tracks.is_empty() {
        sections.push(StoryArchiveSection::Music {
            count: u32::try_from(tracks.len()).unwrap_or(0),
            tracks,
        });
    }

    StoryArchive {
        group_id: group_id.to_owned(),
        sections,
    }
}

/// The clips of one recording, in every language whose file is on disk. The
/// check is a `stat` per language because the clip ids REPEAT across
/// languages (`EX_CN_101` is four files), so the reader's basename index
/// would answer with whichever one it walked first.
fn clip_tracks(
    assets_dir: &std::path::Path,
    char_id: &str,
    voice_id: &str,
) -> Vec<ArchiveClipTrack> {
    CLIP_LANGS
        .iter()
        .filter_map(|(dir, lang)| {
            let rel = format!("audio/audio/sound_beta_2/{dir}/{char_id}/{voice_id}.ogg");
            assets_dir.join(&rel).is_file().then(|| ArchiveClipTrack {
                language: lang.clone(),
                url: format!("/{rel}"),
            })
        })
        .collect()
}

fn build_clip(
    assets_dir: &std::path::Path,
    clip: &crate::core::gamedata::types::mission_archive::MissionArchiveClip,
) -> ArchiveClip {
    ArchiveClip {
        char_id: clip.char_id.clone(),
        voice_id: clip.voice_id.clone(),
        index: clip.index,
        tracks: clip_tracks(assets_dir, &clip.char_id, &clip.voice_id),
    }
}

/// Every group's archive, built once per game-data load.
///
/// The recordings shelf is joined LAST and by its own key: it is not in
/// `story_review_meta_table` at all but in `activity_table.MissionArchives`,
/// and it names the group it belongs to by zone (`main_14`), so a group can
/// end up with a recordings section and nothing else.
///
/// Six EN groups get an archive: `act13side` (news, files, a gallery and 4
/// tracks), `act17side` (logs, landmarks, a gallery and 5 tracks),
/// `act25side` (files and a gallery), `act29side` and `act42side` (both
/// EMPTY, because their only slot is a `ChallengeBook` no section carries)
/// and `main_14` (recordings). Everything it puts on the wire is on disk:
/// landmark plates 26/26, file plates 16/16, drawn file titles 4/4, news
/// mastheads 3/3, news pictures 45/45, gallery pictures 57/57, recorded
/// clips 43/43, and over the whole content half, not just the reachable
/// groups, 82/82 archived tracks join to a loop clip. The build is 31 ms of
/// the 7,279 ms debug index build, the `textures/ui` walk included, so it is
/// done here and cached rather than per request.
#[must_use]
pub fn build_archives(
    gd: &GameData,
    story_assets: &StoryAssetIndex,
    assets_dir: &std::path::Path,
) -> HashMap<String, StoryArchive> {
    let components = archive_components(gd);
    let art = archive_art(assets_dir, &wanted_archive_art(gd, &components));
    let mut out: HashMap<String, StoryArchive> = components
        .iter()
        .map(|(group_id, component)| {
            (
                group_id.clone(),
                build_archive(gd, component, group_id, story_assets, &art),
            )
        })
        .collect();

    let mut shelves: Vec<&crate::core::gamedata::types::mission_archive::MissionArchive> =
        gd.mission_archives.values().collect();
    shelves.sort_by(|a, b| a.topic_id.cmp(&b.topic_id));
    for shelf in shelves {
        let Some(group_id) = shelf
            .zones
            .iter()
            .find(|z| gd.story_reviews.contains_key(z.as_str()))
        else {
            continue;
        };
        let nodes: Vec<ArchiveRecordingNode> = shelf
            .nodes
            .iter()
            .map(|n| ArchiveRecordingNode {
                id: n.node_id.clone(),
                title: n.title.clone(),
                unlock_desc: opt(&n.unlock_desc),
                clips: n.clips.iter().map(|c| build_clip(assets_dir, c)).collect(),
            })
            .collect();
        let hidden: Vec<ArchiveClip> = shelf
            .hidden_clips
            .iter()
            .map(|c| build_clip(assets_dir, c))
            .collect();
        let count = nodes.iter().map(|n| n.clips.len()).sum::<usize>() + hidden.len();
        if count == 0 {
            continue;
        }
        out.entry(group_id.clone())
            .or_insert_with(|| StoryArchive {
                group_id: group_id.clone(),
                sections: Vec::new(),
            })
            .sections
            .push(StoryArchiveSection::Recordings {
                count: u32::try_from(count).unwrap_or(0),
                nodes,
                hidden,
            });
    }
    out
}

/// One group's archive, off the cached build: no table is re-read and no
/// file is re-walked. A group the library lists but the archive table does
/// not is 200 with an empty `sections`; an id the library does not list at
/// all is the same 404 the illustrations route answers with.
pub async fn get_group_archive(
    state: &AppState,
    server: Server,
    group_id: &str,
) -> Result<StoryArchive, ApiError> {
    let cache = cached_index(state, server).await?;
    if !cache.illustrations.contains_key(group_id) {
        return Err(ApiError::NotFoundMessage(format!(
            "story group `{group_id}` is not in the library"
        )));
    }
    Ok(cache
        .archives
        .get(group_id)
        .cloned()
        .unwrap_or_else(|| StoryArchive {
            group_id: group_id.to_owned(),
            sections: Vec::new(),
        }))
}
