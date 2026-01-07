//! Utility functions for DPS calculator routes

use unicode_normalization::UnicodeNormalization;

/// Normalizes an operator name from game data format to calculator format.
///
/// This function handles:
/// - Unicode characters with diacritics (ë → e, š → s, í → i)
/// - Special Polish characters (ł → l)
/// - Apostrophes and special characters (removed)
/// - Spaces and hyphens (converted to PascalCase)
/// - Alter/title suffixes (stripped)
/// - Special name mappings (e.g., Wiš'adel → Walter)
///
/// # Examples
/// - "Blue Poison" → "BluePoison"
/// - "Ch'en" → "Chen"
/// - "Młynar" → "Mlynar"
/// - "Wiš'adel" → "Walter"
/// - "Pozëmka" → "Pozemka"
/// - "Exusiai the Sankta" → "Exusiai"
/// - "SilverAsh" → "SilverAsh"
/// - "12F" → "12F" (numbers preserved)
pub fn normalize_operator_name(name: &str) -> String {
    // Special mappings where game name differs significantly from calculator name
    // Wiš'adel uses "Walter" as the internal class name in ArknightsDpsCompare
    if name.contains("Wiš'adel") || name.contains("Wis'adel") || name.contains("Wisadel") {
        return "Walter".to_string();
    }

    // Special mappings for alter operators with non-standard naming
    if name.contains("Skadi the Corrupting Heart") {
        return "Skalter".to_string();
    }
    if name.contains("Lava the Purgatory") {
        return "Lavaalt".to_string();
    }
    if name.contains("Swire the Elegant Wit") {
        return "SwireAlt".to_string();
    }

    // Handle "Operator the Title" format for alter operators
    // e.g., "Ch'en the Holungday" → "ChenAlter", "Eyjafjalla the Hvít Aska" → "EyjafjallaAlter"
    let (base_name, is_alter) = if name.contains(" the ") {
        (name.split(" the ").next().unwrap_or(name), true)
    } else {
        (name, false)
    };

    // Remove special characters and convert to PascalCase
    // Use NFD normalization to separate base characters from combining diacritics
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in base_name.nfd() {
        // Skip combining diacritical marks (Unicode category Mn)
        // These are the accent marks separated by NFD normalization
        if is_combining_mark(c) {
            continue;
        }

        // Handle special characters that NFD doesn't decompose
        let c = match c {
            'ł' | 'Ł' => 'l', // Polish L with stroke
            _ => c,
        };

        if c.is_ascii_alphanumeric() {
            if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        } else if c == ' ' || c == '-' {
            // Space and hyphen: capitalize next character
            capitalize_next = true;
        }
        // Apostrophes and other special characters are just removed (don't capitalize next)
    }

    // Add "Alter" suffix for "Operator the Title" format operators
    if is_alter {
        result.push_str("Alter");
    }

    result
}

/// Checks if a character is a combining diacritical mark
fn is_combining_mark(c: char) -> bool {
    // Unicode combining diacritical marks range: U+0300 to U+036F
    // Also includes some extended ranges
    matches!(c,
        '\u{0300}'..='\u{036F}' |  // Combining Diacritical Marks
        '\u{1AB0}'..='\u{1AFF}' |  // Combining Diacritical Marks Extended
        '\u{1DC0}'..='\u{1DFF}' |  // Combining Diacritical Marks Supplement
        '\u{20D0}'..='\u{20FF}' |  // Combining Diacritical Marks for Symbols
        '\u{FE20}'..='\u{FE2F}'    // Combining Half Marks
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_basic_names() {
        assert_eq!(normalize_operator_name("Blue Poison"), "BluePoison");
        assert_eq!(normalize_operator_name("Ch'en"), "Chen");
        assert_eq!(normalize_operator_name("SilverAsh"), "SilverAsh");
        assert_eq!(normalize_operator_name("12F"), "12F");
        assert_eq!(normalize_operator_name("Projekt Red"), "ProjektRed");
    }

    #[test]
    fn test_normalize_special_characters() {
        // Polish ł
        assert_eq!(normalize_operator_name("Młynar"), "Mlynar");
        // Czech š
        assert_eq!(normalize_operator_name("Wiš'adel"), "Walter"); // Special mapping
        // French ë
        assert_eq!(normalize_operator_name("Pozëmka"), "Pozemka");
        // Icelandic í in alter name - strips " the " part and adds "Alter"
        assert_eq!(
            normalize_operator_name("Eyjafjalla the Hvít Aska"),
            "EyjafjallaAlter"
        );
    }

    #[test]
    fn test_normalize_wisadel_variants() {
        // All variants should map to Walter
        assert_eq!(normalize_operator_name("Wiš'adel"), "Walter");
        assert_eq!(normalize_operator_name("Wis'adel"), "Walter");
        assert_eq!(normalize_operator_name("Wisadel"), "Walter");
    }

    #[test]
    fn test_normalize_alter_names() {
        // Alter names should stay as-is (Alter suffix preserved)
        assert_eq!(normalize_operator_name("Chen Alter"), "ChenAlter");
        // "Operator the Title" format gets converted to "OperatorAlter"
        assert_eq!(normalize_operator_name("Ch'en the Holungday"), "ChenAlter");
        assert_eq!(
            normalize_operator_name("Nearl the Radiant Knight"),
            "NearlAlter"
        );
        assert_eq!(normalize_operator_name("Texas the Omertosa"), "TexasAlter");
        // Special alter mappings with non-standard naming
        assert_eq!(
            normalize_operator_name("Skadi the Corrupting Heart"),
            "Skalter"
        );
        assert_eq!(normalize_operator_name("Lava the Purgatory"), "Lavaalt");
        assert_eq!(normalize_operator_name("Swire the Elegant Wit"), "SwireAlt");
    }
}
