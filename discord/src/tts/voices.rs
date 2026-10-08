//! The voices a member can pick with `/tts voice`, as Google Translate language codes.
//!
//! Every entry was requested live (2026-10-08, "hello, doctor" in its language) and answered
//! HTTP 200 with `audio/mpeg`. Accents come from the region in `tl` (`en-GB`, `fr-CA`, `pt-PT`),
//! not from the host's TLD: `tl=en` on translate.google.co.uk, .com.au and .co.in returned the same
//! two encodings `.com` alternates between, while `tl=en-GB` returned its own bytes on both `.com`
//! and `.co.uk`. So every request goes to translate.google.com. `es-US` was dropped as a duplicate
//! of `es-MX` (the same bytes).

/// One selectable voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voice {
    /// What `/tts voice set` takes and the database stores.
    pub id: &'static str,
    pub label: &'static str,
    /// Google Translate's `tl` parameter.
    pub tl: &'static str,
}

const fn voice(id: &'static str, label: &'static str, tl: &'static str) -> Voice {
    Voice { id, label, tl }
}

/// At most 25, the size of a Discord autocomplete list.
pub const VOICES: &[Voice] = &[
    voice("en-us", "English (US)", "en-US"),
    voice("en-gb", "English (UK)", "en-GB"),
    voice("en-au", "English (Australia)", "en-AU"),
    voice("en-in", "English (India)", "en-IN"),
    voice("ja", "Japanese", "ja"),
    voice("ko", "Korean", "ko"),
    voice("zh-cn", "Chinese (Mandarin, Simplified)", "zh-CN"),
    voice("zh-tw", "Chinese (Taiwan)", "zh-TW"),
    voice("yue", "Cantonese", "yue"),
    voice("es-es", "Spanish (Spain)", "es-ES"),
    voice("es-mx", "Spanish (Mexico)", "es-MX"),
    voice("fr-fr", "French (France)", "fr-FR"),
    voice("fr-ca", "French (Canada)", "fr-CA"),
    voice("pt-br", "Portuguese (Brazil)", "pt-BR"),
    voice("pt-pt", "Portuguese (Portugal)", "pt-PT"),
    voice("de", "German", "de"),
    voice("it", "Italian", "it"),
    voice("ru", "Russian", "ru"),
    voice("id", "Indonesian", "id"),
    voice("vi", "Vietnamese", "vi"),
    voice("th", "Thai", "th"),
    voice("fil", "Filipino", "fil"),
    voice("nl", "Dutch", "nl"),
    voice("pl", "Polish", "pl"),
    voice("tr", "Turkish", "tr"),
];

/// The voice used when neither the member nor the config picks one.
pub const FALLBACK: Voice = VOICES[0];

/// The voice with `id`, ignoring case and surrounding space.
#[must_use]
pub fn find(id: &str) -> Option<Voice> {
    let id = id.trim();
    VOICES
        .iter()
        .copied()
        .find(|v| v.id.eq_ignore_ascii_case(id))
}

/// The voice a member speaks in: their stored choice when it is still a known voice, else
/// `default`. A voice later removed from [`VOICES`] falls back rather than failing.
#[must_use]
pub fn resolve(stored: Option<&str>, default: Voice) -> Voice {
    stored.and_then(find).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fits_an_autocomplete_list_and_ids_are_unique() {
        assert!(VOICES.len() <= 25);
        for (i, v) in VOICES.iter().enumerate() {
            assert!(
                VOICES[i + 1..].iter().all(|w| w.id != v.id && w.tl != v.tl),
                "{} is listed twice",
                v.id
            );
        }
    }

    #[test]
    fn parse_and_fallback() {
        assert_eq!(find("en-gb").map(|v| v.tl), Some("en-GB"));
        assert_eq!(find(" JA ").map(|v| v.tl), Some("ja"));
        assert_eq!(find("klingon"), None);
        let ja = find("ja").unwrap();
        assert_eq!(resolve(Some("fr-ca"), ja).tl, "fr-CA");
        assert_eq!(
            resolve(Some("retired-voice"), ja),
            ja,
            "unknown stored voice: default"
        );
        assert_eq!(resolve(None, ja), ja);
        assert_eq!(FALLBACK.id, "en-us");
    }
}
