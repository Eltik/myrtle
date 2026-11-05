use std::collections::HashMap;

use myrtle_gamedata_raw::tables::voice::{
    LangType, RawVoice, Voice, VoiceData, VoiceLang, VoiceLangDictEntry,
};

struct LangInfo {
    dir: &'static str,
    suffix: &'static str,
}

const fn lang_info(lang: &LangType) -> Option<LangInfo> {
    Some(match lang {
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
        LangType::Unknown => return None,
    })
}

#[must_use]
pub fn enrich_all_voices(
    char_words: &HashMap<String, RawVoice>,
    voice_lang_dict: &HashMap<String, VoiceLang>,
) -> HashMap<String, Voice> {
    let mut entries: Vec<(&String, &VoiceLang)> = voice_lang_dict.iter().collect();
    entries.sort_unstable_by_key(|(key, _)| key.as_str());
    let mut by_word_key: HashMap<&str, &VoiceLang> = HashMap::new();
    for (_, vl) in entries {
        for word_key in &vl.wordkeys {
            by_word_key.entry(word_key.as_str()).or_insert(vl);
        }
    }

    char_words
        .iter()
        .map(|(id, raw)| {
            let voice_lang = by_word_key
                .get(raw.word_key.as_str())
                .copied()
                .or_else(|| voice_lang_dict.get(&raw.char_id));
            (id.clone(), enrich_voice(id, raw, voice_lang))
        })
        .collect()
}

fn enrich_voice(id: &str, raw: &RawVoice, voice_lang: Option<&VoiceLang>) -> Voice {
    let (data, languages) = if let Some(vl) = voice_lang {
        let mut langs: Vec<(&String, &VoiceLangDictEntry)> = vl.dict.iter().collect();
        langs.sort_unstable_by_key(|(key, _)| key.as_str());

        let languages: Vec<LangType> = langs
            .iter()
            .map(|(_, e)| e.voice_lang_type.clone())
            .collect();

        let data: Vec<VoiceData> = langs
            .iter()
            .map(|(_, e)| VoiceData {
                voice_url: build_voice_url(&raw.voice_asset, &e.voice_lang_type),
                language: Some(e.voice_lang_type.clone()),
                cv_name: Some(e.cv_name.clone()),
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

fn build_voice_url(voice_asset: &str, lang: &LangType) -> Option<String> {
    let info = lang_info(lang)?;
    let Some((original_dir, file)) = voice_asset.split_once('/') else {
        return Some(format!("/audio/sound_beta_2/voice/{voice_asset}.ogg").replace('#', "%23"));
    };

    let base_dir = original_dir
        .to_lowercase()
        .replace("_cn_topolect", "")
        .replace("_ger", "")
        .replace("_ita", "")
        .replace("_rus", "")
        .replace("_fre", "")
        .replace("_spa", "");

    let dir = format!("{base_dir}{}", info.suffix);
    Some(format!("/audio/sound_beta_2/{}/{dir}/{file}.ogg", info.dir).replace('#', "%23"))
}
