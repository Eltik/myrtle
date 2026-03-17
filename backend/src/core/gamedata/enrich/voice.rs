use std::collections::HashMap;

use crate::core::gamedata::types::voice::{LangType, RawVoice, Voice, VoiceData, VoiceLang};

struct LangInfo {
    dir: &'static str,
    suffix: &'static str,
}

const fn lang_info(lang: &LangType) -> LangInfo {
    match lang {
        LangType::Jp => LangInfo {
            dir: "voice",
            suffix: "",
        },
        LangType::CnMandarin => LangInfo {
            dir: "voice_cn",
            suffix: "",
        },
        LangType::En => LangInfo {
            dir: "voice_en",
            suffix: "",
        },
        LangType::Kr => LangInfo {
            dir: "voice_kr",
            suffix: "",
        },
        LangType::Ger => LangInfo {
            dir: "voice_custom",
            suffix: "_ger",
        },
        LangType::Ita => LangInfo {
            dir: "voice_custom",
            suffix: "_ita",
        },
        LangType::Rus => LangInfo {
            dir: "voice_custom",
            suffix: "_rus",
        },
        LangType::CnTopolect => LangInfo {
            dir: "voice_custom",
            suffix: "_cn_topolect",
        },
        LangType::Fre => LangInfo {
            dir: "voice_custom",
            suffix: "_fre",
        },
        LangType::Spa => LangInfo {
            dir: "voice_custom",
            suffix: "_spa",
        },
        LangType::Linkage => LangInfo {
            dir: "voice",
            suffix: "",
        },
    }
}

pub fn enrich_all_voices(
    char_words: &HashMap<String, RawVoice>,
    voice_lang_dict: &HashMap<String, VoiceLang>,
) -> HashMap<String, Voice> {
    let lang_by_char: HashMap<&str, &VoiceLang> = voice_lang_dict
        .values()
        .map(|vl| (vl.char_id.as_str(), vl))
        .collect();

    char_words
        .iter()
        .map(|(id, raw)| {
            let enriched = enrich_voice(id, raw, lang_by_char.get(raw.char_id.as_str()).copied());
            (id.clone(), enriched)
        })
        .collect()
}

fn enrich_voice(id: &str, raw: &RawVoice, voice_lang: Option<&VoiceLang>) -> Voice {
    let (data, languages) = if let Some(vl) = voice_lang {
        let languages: Vec<LangType> = vl
            .dict
            .values()
            .map(|e| e.voice_lang_type.clone())
            .collect();

        let data: Vec<VoiceData> = languages
            .iter()
            .map(|lang| {
                let cv_name = vl
                    .dict
                    .values()
                    .find(|e| &e.voice_lang_type == lang)
                    .map(|e| e.cv_name.clone());

                VoiceData {
                    voice_url: Some(build_voice_url(&raw.voice_asset, lang)),
                    language: Some(lang.clone()),
                    cv_name,
                }
            })
            .collect();

        (Some(data), Some(languages))
    } else {
        (None, None)
    };

    Voice {
        char_word_id: raw.char_word_id.clone(),
        word_key: raw.word_key.clone(),
        char_id: raw.char_id.clone(),
        voice_id: raw.voice_id.clone(),
        voice_text: raw.voice_text.clone(),
        voice_title: raw.voice_title.clone(),
        voice_index: raw.voice_index,
        voice_type: raw.voice_type.clone(),
        unlock_type: raw.unlock_type.clone(),
        unlock_param: raw.unlock_param.clone(),
        lock_description: raw.lock_description.clone(),
        place_type: raw.place_type.clone(),
        voice_asset: raw.voice_asset.clone(),
        id: Some(id.to_string()),
        data,
        languages,
    }
}

fn build_voice_url(voice_asset: &str, lang: &LangType) -> String {
    let info = lang_info(lang);
    let Some((original_dir, file)) = voice_asset.split_once('/') else {
        return format!("/audio/sound_beta_2/voice/{voice_asset}.wav");
    };

    // Strip any existing language suffix from dir, then append the target suffix
    let base_dir = original_dir
        .to_lowercase()
        .replace("_cn_topolect", "")
        .replace("_ger", "")
        .replace("_ita", "")
        .replace("_rus", "")
        .replace("_fre", "")
        .replace("_spa", "");

    let dir = format!("{base_dir}{}", info.suffix);
    format!("/audio/sound_beta_2/{}/{dir}/{file}.wav", info.dir)
}
