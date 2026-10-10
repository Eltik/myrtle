use std::collections::HashMap;

use crate::core::gamedata::types::voice::{
    LangType, RawVoice, Voice, VoiceData, VoiceLang, VoiceLangDictEntry,
};

struct LangInfo {
    dir: &'static str,
    suffix: &'static str,
}

/// Audio directory and filename suffix for a language.
///
/// `None` for `LangType::Unknown`, the `#[serde(other)]` fallback for a voice
/// language this build doesn't name: there's no directory to guess, so the caller
/// omits the URL instead of emitting a 404. The character's other languages still
/// resolve.
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

/// Attach each line's voice set: languages, audio URLs and CV names.
///
/// A line resolves through its own `wordKey`. Every `VoiceLangDict` entry lists
/// the word keys it covers (`Wordkeys`), and a character can own several
/// entries: its base set and a skin's (`char_472_pasngr` and
/// `char_472_pasngr_epoque#17`), where the base set may carry an extra language
/// whose lines sit under their own word key (`char_2023_ling_CN_TOPOLECT`).
/// Keying by `charId` instead let whichever entry the `HashMap` iterated last
/// win for every line of the character, so a skin's set replaced the base set
/// (or the reverse) and the choice changed from one process to the next.
///
/// No word key is listed twice in today's tables; if one ever is, the entry
/// with the smallest dict key wins, so the result still does not depend on
/// iteration order. A line whose word key no entry lists falls back to the
/// entry keyed by its `charId`, the base set.
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

/// Languages are listed in dict-key order (`CN_MANDARIN`, `CN_TOPOLECT`, `EN`,
/// ...), the order `voice_actors` in the operators service uses: the dict is a
/// `HashMap`, so its own order changed per process.
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
    // Files are Vorbis-in-Ogg. Percent-encode `#` (skin folders like
    // `char_245_cello_sale#12`) or the browser reads it as a fragment; same as
    // `audio_url` in `gamedata::assets`.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn lang(word_key: &str, lang: LangType, cv: &str) -> VoiceLangDictEntry {
        VoiceLangDictEntry {
            wordkey: word_key.to_owned(),
            voice_lang_type: lang,
            cv_name: vec![cv.to_owned()],
            voice_path: None,
        }
    }

    fn set(char_id: &str, word_keys: &[&str], langs: Vec<(&str, VoiceLangDictEntry)>) -> VoiceLang {
        VoiceLang {
            wordkeys: word_keys.iter().map(|w| (*w).to_owned()).collect(),
            char_id: char_id.to_owned(),
            dict: langs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
        }
    }

    fn line(char_id: &str, word_key: &str, asset_dir: &str) -> RawVoice {
        RawVoice {
            char_word_id: format!("{word_key}_CN_001"),
            word_key: word_key.to_owned(),
            char_id: char_id.to_owned(),
            voice_id: "CN_001".to_owned(),
            voice_asset: format!("{asset_dir}/CN_001"),
            ..RawVoice::default()
        }
    }

    /// The shape of `char_2024_chyue` in every server's table: a base set with a
    /// topolect language whose lines have their own word key, and a skin set
    /// without it.
    fn chongyue() -> (HashMap<String, RawVoice>, HashMap<String, VoiceLang>) {
        let c = "char_2024_chyue";
        let dict = HashMap::from([
            (
                c.to_owned(),
                set(
                    c,
                    &[c, "char_2024_chyue_CN_TOPOLECT"],
                    vec![
                        ("JP", lang(c, LangType::Jp, "Toshihiko Seki")),
                        (
                            "CN_TOPOLECT",
                            lang("char_2024_chyue_CN_TOPOLECT", LangType::CnTopolect, "Bill"),
                        ),
                        ("CN_MANDARIN", lang(c, LangType::CnMandarin, "Ling Zhao")),
                    ],
                ),
            ),
            (
                "char_2024_chyue_cfa#1".to_owned(),
                set(
                    c,
                    &["char_2024_chyue_cfa#1"],
                    vec![
                        (
                            "JP",
                            lang("char_2024_chyue_cfa#1", LangType::Jp, "Toshihiko Seki"),
                        ),
                        (
                            "CN_MANDARIN",
                            lang("char_2024_chyue_cfa#1", LangType::CnMandarin, "Ling Zhao"),
                        ),
                    ],
                ),
            ),
        ]);
        let words = HashMap::from([
            ("base".to_owned(), line(c, c, c)),
            (
                "topolect".to_owned(),
                line(
                    c,
                    "char_2024_chyue_CN_TOPOLECT",
                    "char_2024_chyue_CN_TOPOLECT",
                ),
            ),
            (
                "skin".to_owned(),
                line(c, "char_2024_chyue_cfa#1", "char_2024_chyue_cfa#1"),
            ),
        ]);
        (words, dict)
    }

    #[test]
    fn each_line_takes_the_voice_set_that_lists_its_word_key() {
        let (words, dict) = chongyue();
        let out = enrich_all_voices(&words, &dict);
        let langs = |id: &str| out[id].languages.clone().expect("a voice set");

        let base = vec![LangType::CnMandarin, LangType::CnTopolect, LangType::Jp];
        assert_eq!(langs("base"), base);
        assert_eq!(langs("topolect"), base);
        assert_eq!(langs("skin"), vec![LangType::CnMandarin, LangType::Jp]);

        let topolect = &out["base"].data.as_ref().expect("data")[1];
        assert_eq!(topolect.cv_name.as_deref(), Some(&["Bill".to_owned()][..]));
        assert_eq!(
            topolect.voice_url.as_deref(),
            Some("/audio/sound_beta_2/voice_custom/char_2024_chyue_cn_topolect/CN_001.ogg")
        );
        assert_eq!(
            out["skin"].data.as_ref().expect("data")[0]
                .voice_url
                .as_deref(),
            Some("/audio/sound_beta_2/voice_cn/char_2024_chyue_cfa%231/CN_001.ogg")
        );
    }

    /// Fresh maps get fresh hash seeds, so a merge that depends on iteration
    /// order shows up as a second answer within a few rounds.
    #[test]
    fn the_result_does_not_depend_on_hash_map_order() {
        let (words, dict) = chongyue();
        let first = serde_json::to_string(&enrich_all_voices(&words, &dict)["skin"]).unwrap();
        for _ in 0..64 {
            let (words, dict) = chongyue();
            let out = enrich_all_voices(&words, &dict);
            assert_eq!(serde_json::to_string(&out["skin"]).unwrap(), first);
            assert_eq!(
                out["base"].languages.as_ref().map(Vec::len),
                Some(3),
                "the base line kept its topolect language"
            );
        }
    }

    #[test]
    fn a_word_key_no_set_lists_falls_back_to_the_base_set() {
        let (mut words, dict) = chongyue();
        words.insert(
            "stray".to_owned(),
            line(
                "char_2024_chyue",
                "char_2024_chyue_unlisted",
                "char_2024_chyue",
            ),
        );
        let out = enrich_all_voices(&words, &dict);
        assert_eq!(out["stray"].languages.as_ref().map(Vec::len), Some(3));
    }
}
