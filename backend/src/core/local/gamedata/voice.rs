use std::collections::HashMap;

use crate::core::local::types::voice::{LangType, RawVoice, Voice, VoiceData, VoiceLang, Voices};

/// Get the voice directory for a language type
fn get_voice_dir(lang: &LangType) -> &'static str {
    match lang {
        LangType::Jp => "voice",
        LangType::CnMandarin => "voice_cn",
        LangType::En => "voice_en",
        LangType::Kr => "voice_kr",
        // Custom languages
        LangType::CnTopolect | LangType::Ger | LangType::Ita | LangType::Rus | LangType::Fre => {
            "voice_custom"
        }
        LangType::Linkage => "voice", // fallback
    }
}

/// Check if a language is a custom type
fn is_custom(lang: &LangType) -> bool {
    !matches!(
        lang,
        LangType::En | LangType::Jp | LangType::Kr | LangType::CnMandarin
    )
}

/// Get the appended suffix for custom languages
fn appended_custom(lang: &LangType) -> &'static str {
    match lang {
        LangType::Ger => "_ger",
        LangType::Ita => "_ita",
        LangType::Rus => "_rus",
        LangType::CnTopolect => "_cn_topolect",
        LangType::Fre => "_fre",
        _ => "",
    }
}

/// Build the voice URL for a given voice asset and language
fn build_voice_url(voice_asset: &str, lang: &LangType) -> String {
    let parts: Vec<&str> = voice_asset.split('/').collect();
    if parts.len() < 2 {
        return format!("/audio/sound_beta_2/voice/{}.wav", voice_asset);
    }

    let original_dir = parts[0];
    let file = parts[1];
    let voice_dir = get_voice_dir(lang);

    // Handle directory name transformation based on language
    let dir = if !original_dir.contains("_cn_topolect") {
        let part_count = original_dir.split('_').count();
        if !is_custom(lang) && part_count > 3 {
            // Remove the last segment for non-custom languages
            original_dir
                .split('_')
                .take(part_count - 1)
                .collect::<Vec<_>>()
                .join("_")
        } else if is_custom(lang) && part_count > 3 {
            original_dir.to_string()
        } else if is_custom(lang) {
            format!("{}{}", original_dir, appended_custom(lang))
        } else {
            original_dir.to_string()
        }
    } else {
        let part_count = original_dir.split('_').count();
        if !is_custom(lang) && part_count > 4 {
            original_dir
                .split('_')
                .take(part_count - 2)
                .collect::<Vec<_>>()
                .join("_")
        } else if is_custom(lang) && part_count > 4 {
            original_dir.to_string()
        } else if is_custom(lang) {
            format!("{}{}", original_dir, appended_custom(lang))
        } else {
            original_dir.to_string()
        }
    };

    format!(
        "/audio/sound_beta_2/{}/{}/{}.wav",
        voice_dir,
        dir.to_lowercase(),
        file
    )
}

/// Enrich a single voice with language data
fn enrich_voice(id: &str, raw: &RawVoice, voice_lang_dict: &HashMap<String, VoiceLang>) -> Voice {
    // Find the voice lang entry for this character
    let voice_lang = voice_lang_dict
        .values()
        .find(|vl| vl.char_id == raw.char_id);

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
                    .get(&format!("{:?}", lang).to_uppercase())
                    .or_else(|| {
                        // Try to find by matching voice_lang_type
                        vl.dict.values().find(|e| &e.voice_lang_type == lang)
                    })
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

/// Enrich all voices with language data and URLs
pub fn enrich_all_voices(
    char_words: &HashMap<String, RawVoice>,
    voice_lang_dict: &HashMap<String, VoiceLang>,
) -> HashMap<String, Voice> {
    char_words
        .iter()
        .map(|(id, raw)| {
            let enriched = enrich_voice(id, raw, voice_lang_dict);
            (id.clone(), enriched)
        })
        .collect()
}

/// Get all voices for a specific character
pub fn get_character_voices<'a>(char_id: &str, voices: &'a Voices) -> Vec<&'a Voice> {
    voices
        .char_words
        .values()
        .filter(|v| v.char_id == char_id)
        .collect()
}
