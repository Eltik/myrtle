use std::collections::HashMap;

use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::serde_helpers::FbKeyValue;
use myrtle_gamedata_raw::tables::activity::MissionArchiveClip;
use myrtle_gamedata_raw::tables::story::{ActArchiveComponent, ArchiveSlot, ArchiveSlotItem};
use myrtle_gamedata_raw::tables::voice::LangType;

use crate::assets::StoryAssetIndex;
use crate::dto::{
    ArchiveClip, ArchiveClipTrack, ArchiveFile, ArchiveLandmark, ArchiveLog, ArchiveLogChapter,
    ArchiveNewsFormat, ArchiveNewsItem, ArchiveNewsLine, ArchivePicture, ArchiveRecordingNode,
    ArchiveTrack, StoryArchive, StoryArchiveSection,
};
use crate::opt;

const CLIP_LANGS: [(&str, LangType); 4] = [
    ("voice_cn", LangType::CnMandarin),
    ("voice", LangType::Jp),
    ("voice_en", LangType::En),
    ("voice_kr", LangType::Kr),
];

fn len<T>(items: &[T]) -> u32 {
    u32::try_from(items.len()).unwrap_or(u32::MAX)
}

fn archive_components(m: &GameModel) -> HashMap<String, &ActArchiveComponent> {
    let mut components: Vec<_> = m
        .story_archives
        .act_archive_data
        .components
        .iter()
        .collect();
    components.sort_by_key(|(k, _)| (!m.story_reviews.contains_key(k.as_str()), k.as_str()));
    let mut out = HashMap::new();
    for (key, component) in components {
        let group = if m.story_reviews.contains_key(key) {
            Some(key.clone())
        } else {
            key.strip_suffix("sre")
                .map(|act| format!("{act}side"))
                .filter(|live| m.story_reviews.contains_key(live))
        };
        if let Some(group) = group {
            out.entry(group).or_insert(component);
        }
    }
    out
}

fn sorted(
    items: &[FbKeyValue<String, ArchiveSlotItem>],
) -> Vec<&FbKeyValue<String, ArchiveSlotItem>> {
    let mut items: Vec<_> = items.iter().collect();
    items.sort_by(|a, b| {
        a.value
            .sort_id
            .cmp(&b.value.sort_id)
            .then_with(|| a.key.cmp(&b.key))
    });
    items
}

#[must_use]
pub fn slot_ids(slot: Option<&ArchiveSlot>) -> Vec<&str> {
    slot.map_or_else(Vec::new, |s| {
        sorted(&s.items)
            .into_iter()
            .map(|i| i.value.id.as_str())
            .collect()
    })
}

fn build_archive(
    m: &GameModel,
    component: &ActArchiveComponent,
    group_id: &str,
    assets: &StoryAssetIndex,
) -> StoryArchive {
    let res = &m.story_archives.act_archive_res_data;
    let url = |name: &str| assets.archive_art(name).map(str::to_owned);
    let mut sections = Vec::new();

    if let Some(raw) = &component.log {
        let chapters: Vec<ArchiveLogChapter> = raw
            .iter()
            .map(|c| ArchiveLogChapter {
                id: c.key.clone(),
                name: c.value.chapter_name.clone(),
                display_id: opt(&c.value.display_id),
                chapter_icon: opt(&c.value.chapter_icon),
                unlock_desc: opt(&c.value.unlock_des),
                logs: c
                    .value
                    .logs
                    .iter()
                    .filter_map(|id| res.logs.get(id))
                    .map(|l| ArchiveLog {
                        id: l.log_id.clone(),
                        text: l.log_desc.clone(),
                    })
                    .collect(),
            })
            .collect();
        let count = chapters.iter().map(|c| len(&c.logs)).sum::<u32>();
        if count > 0 {
            sections.push(StoryArchiveSection::Logs { count, chapters });
        }
    }

    let landmarks: Vec<ArchiveLandmark> = component
        .landmark
        .as_deref()
        .map(sorted)
        .unwrap_or_default()
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
            count: len(&landmarks),
            landmarks,
        });
    }

    let news: Vec<ArchiveNewsItem> = slot_ids(component.news.as_ref())
        .into_iter()
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
            count: len(&news),
            news,
        });
    }

    let files: Vec<ArchiveFile> = slot_ids(component.story.as_ref())
        .into_iter()
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
            count: len(&files),
            files,
        });
    }

    let pictures: Vec<ArchivePicture> = slot_ids(component.pic.as_ref())
        .into_iter()
        .filter_map(|id| res.pics.get(id))
        .map(|p| ArchivePicture {
            id: p.id.clone(),
            title: p.desc.clone(),
            description: opt(&p.pic_description),
            picture_type: p.type_.clone(),
            url: assets.resolve_image(&p.asset_path).map(|(u, _)| u),
        })
        .collect();
    if !pictures.is_empty() {
        sections.push(StoryArchiveSection::Gallery {
            count: len(&pictures),
            pictures,
        });
    }

    let tracks: Vec<ArchiveTrack> = slot_ids(component.music.as_ref())
        .into_iter()
        .filter_map(|id| {
            let audio = res.audios.get(id);
            let bank = m
                .music
                .music(id)
                .and_then(|music| m.music.bank(&music.bank))
                .or_else(|| m.music.bank(id))?;
            Some(ArchiveTrack {
                id: id.to_owned(),
                name: audio.and_then(|a| opt(&a.name)),
                // The description repeats the id on every EN row
                description: audio.and_then(|a| opt(&a.desc)).filter(|d| d != id),
                intro_url: bank.intro.as_deref().and_then(|i| assets.resolve_audio(i)),
                loop_url: assets.resolve_audio(bank.loop_.as_deref()?)?,
            })
        })
        .collect();
    if !tracks.is_empty() {
        sections.push(StoryArchiveSection::Music {
            count: len(&tracks),
            tracks,
        });
    }

    StoryArchive {
        group_id: group_id.to_owned(),
        sections,
    }
}

fn build_clip(assets: &StoryAssetIndex, clip: &MissionArchiveClip) -> ArchiveClip {
    ArchiveClip {
        char_id: clip.char_id.clone(),
        voice_id: clip.voice_id.clone(),
        index: clip.index,
        tracks: CLIP_LANGS
            .iter()
            .filter(|(dir, _)| assets.has_voice(dir, &clip.char_id, &clip.voice_id))
            .map(|(dir, lang)| ArchiveClipTrack {
                language: lang.clone(),
                url: format!(
                    "/audio/audio/sound_beta_2/{dir}/{}/{}.ogg",
                    clip.char_id, clip.voice_id
                ),
            })
            .collect(),
    }
}

#[must_use]
pub fn build_archives(m: &GameModel, assets: &StoryAssetIndex) -> HashMap<String, StoryArchive> {
    let mut out: HashMap<String, StoryArchive> = archive_components(m)
        .into_iter()
        .map(|(group_id, component)| {
            let archive = build_archive(m, component, &group_id, assets);
            (group_id, archive)
        })
        .collect();

    let mut shelves: Vec<_> = m.mission_archives.values().collect();
    shelves.sort_by(|a, b| a.topic_id.cmp(&b.topic_id));
    for shelf in shelves {
        let Some(group_id) = shelf
            .zones
            .iter()
            .find(|z| m.story_reviews.contains_key(z.as_str()))
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
                clips: n.clips.iter().map(|c| build_clip(assets, c)).collect(),
            })
            .collect();
        let hidden: Vec<ArchiveClip> = shelf
            .hidden_clips
            .iter()
            .map(|c| build_clip(assets, c))
            .collect();
        let count = nodes.iter().map(|n| len(&n.clips)).sum::<u32>() + len(&hidden);
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
                count,
                nodes,
                hidden,
            });
    }
    out
}
