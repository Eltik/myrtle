use std::collections::HashMap;

use myrtle_gamedata::enrich::voice::*;
use myrtle_gamedata_raw::tables::voice::{LangType, RawVoice, VoiceLang, VoiceLangDictEntry};

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
