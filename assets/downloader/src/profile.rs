#[must_use]
pub fn keep_for_operators(name: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "anon/",
        "chararts/",
        "skinpack/",
        "spritepack/char_portrait_",
        "arts/charportraits/",
        "spritepack/ui_char_avatar_",
        "spritepack/skill_icons_",
        "spritepack/ui_equip_",
    ];
    name.ends_with(".idx") || PREFIXES.iter().any(|p| name.starts_with(p))
}
