//! The personnel file as the Archives read it: every line of an operator's
//! handbook Basic Info, Physical Exam and Clinical Analysis, kept as the game
//! wrote it and, where the wording allows, read into a number or a category.
//!
//! The handbook is free text, so every reading here is a claim with a test
//! behind it, and a value the reader cannot settle stays `None` with its raw
//! text kept. Nothing is filled in: a file that leaves a field blank is
//! counted as blank. Labels and value words are per-language tables
//! ([`Vocab`]: EN, CN, JP, KR), picked by the server the tables come from
//! ([`Lang::for_server`]): every server's handbook is written in one language
//! (2026-10-11 label scripts: EN 416 Latin, CN 438 Han, JP 414 with kana plus
//! 5 kanji-only, KR 419 Hangul). Coverage over the real tables of every
//! server is printed by `tests/archives_real_data_test.rs`.
//!
//! This is a separate reader from `enrich::profile` on purpose: that one maps
//! race and birthplace onto closed enums (an unseen spelling becomes
//! `Unknown`) and reads only the first line after a label, which leaves EN's
//! `[Infection Status]` empty because EN writes the sentence on the next line.
//! The Archives count the spellings the files actually use.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::gamedata::types::handbook::HandbookStoryTextAudio;
use crate::core::hypergryph::constants::Server;

/// What the infection sentence says, read by keyword.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum InfectionState {
    Infected,
    Clear,
    /// The file states something, but no keyword reads it safely.
    Unclear,
    /// The file has no infection line at all.
    #[default]
    Unstated,
}

/// The six columns of Rhodes Island's physical examination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum ExamAxis {
    Strength,
    Mobility,
    Resilience,
    Tactics,
    Combat,
    Arts,
}

impl ExamAxis {
    pub const ALL: [Self; 6] = [
        Self::Strength,
        Self::Mobility,
        Self::Resilience,
        Self::Tactics,
        Self::Combat,
        Self::Arts,
    ];
}

/// One step of the exam's rating scale, lowest first in [`ExamStep::ALL`].
///
/// The steps are the CN table's five words, the game's source language:
/// 缺陷 Flawed, 普通 Normal, 标准 Standard, 优良 Excellent, 卓越 Outstanding.
/// EN prints three more words, and the other servers settle where each one
/// sits (2026-10-11 tables, all 5 instances): Feeble (`char_501_durin`) and
/// Defective (`char_376_therex`) are 缺陷 / 欠落 / 부족 on CN / JP / KR, so
/// Flawed; Exceptional (`char_2013_cerber`, `char_306_leizi`,
/// `char_1043_leizi2`) is 卓越 / 卓越 / 월등, so Outstanding. That is a
/// derivation from the data, and it replaced an invented 8-step scale.
///
/// The ORDER of the five is not in any table (no enum or rank exists); it is
/// the plain reading of the words, lowest to highest, and stays a named
/// constant for that reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum ExamStep {
    Flawed,
    Normal,
    Standard,
    Excellent,
    Outstanding,
}

impl ExamStep {
    /// Lowest first. A rating's `step` is its index here.
    pub const ALL: [Self; 5] = [
        Self::Flawed,
        Self::Normal,
        Self::Standard,
        Self::Excellent,
        Self::Outstanding,
    ];

    #[must_use]
    pub fn rank(self) -> u8 {
        let i = Self::ALL.iter().position(|s| *s == self).unwrap_or(0);
        u8::try_from(i).unwrap_or(0)
    }
}

/// The language a server's handbook is written in; it picks the [`Vocab`]
/// the files are read with.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Lang {
    #[default]
    En,
    Cn,
    Ja,
    Ko,
}

impl Lang {
    pub const ALL: [Self; 4] = [Self::En, Self::Cn, Self::Ja, Self::Ko];

    /// The language of `server`'s tables. `None` for TW: its Traditional
    /// Chinese labels match none of these tables, and no TW tree is
    /// extracted to build one from, so its files are left unread rather
    /// than misread.
    #[must_use]
    pub const fn for_server(server: Server) -> Option<Self> {
        match server {
            Server::EN => Some(Self::En),
            Server::CN | Server::Bilibili => Some(Self::Cn),
            Server::JP => Some(Self::Ja),
            Server::KR => Some(Self::Ko),
            Server::TW => None,
        }
    }

    #[must_use]
    pub const fn vocab(self) -> &'static Vocab {
        match self {
            Self::En => &EN,
            Self::Cn => &CN,
            Self::Ja => &JA,
            Self::Ko => &KO,
        }
    }
}

/// One language's labels and value words. Every entry is a spelling counted
/// in that server's `handbook_info_table` (EN 2026-09-05, CN/JP/KR
/// 2026-10-06 extracts); the coverage it buys is printed by
/// `tests/archives_real_data_test.rs`.
#[derive(Debug)]
pub struct Vocab {
    pub code_name: &'static [&'static str],
    pub gender: &'static [&'static str],
    pub combat: &'static [&'static str],
    pub birthplace: &'static [&'static str],
    pub birthday: &'static [&'static str],
    pub race: &'static [&'static str],
    pub height: &'static [&'static str],
    pub infection: &'static [&'static str],
    pub manufacturer: &'static [&'static str],
    /// Exam column labels, in [`ExamAxis::ALL`] order.
    pub axes: [&'static [&'static str]; 6],
    /// Rating word and the step it names.
    pub ratings: &'static [(&'static str, ExamStep)],
    /// Gender words that read as [`ArchiveGender::Female`] / [`ArchiveGender::Male`].
    pub female: &'static [&'static str],
    pub male: &'static [&'static str],
    /// Infection keywords, lowercase. A sentence holding any `clear` one is
    /// clear; otherwise it is infected when every word of one `infected`
    /// group is in it. Only the part after the last `contrast` word is read.
    pub clear: &'static [&'static str],
    pub infected: &'static [&'static [&'static str]],
    pub contrast: &'static [&'static str],
    /// Exact clinical labels; EN reads its many spellings by keyword instead
    /// (see `read_clinical`).
    pub blood: &'static [&'static str],
    pub assimilation: &'static [&'static str],
    /// Units, lowercase, compared after the number.
    pub cm: &'static [&'static str],
    pub density: &'static [&'static str],
    pub percent: &'static [&'static str],
    /// A number right after one of these is a bound, not a reading.
    pub bounds: &'static [char],
    /// Combat experience: prefixes that read 0 and 0.5 years, suffixes after
    /// the number for years and months, number words, and whether the number
    /// may be written in kanji (`二十五年`).
    pub none: &'static [&'static str],
    pub half: &'static [&'static str],
    pub years: &'static [&'static str],
    pub months: &'static [&'static str],
    pub number_words: &'static [(&'static str, f64)],
    pub kanji_numbers: bool,
    /// Date of birth: month and day markers (`12月23日`); `None` reads EN
    /// month names.
    pub date_marks: Option<(char, char)>,
}

const ASCII_BOUNDS: [char; 3] = ['<', '>', '~'];

pub static EN: Vocab = Vocab {
    code_name: &["Code Name", "Codename"],
    gender: &["Gender"],
    combat: &["Combat Experience"],
    birthplace: &["Place of Birth"],
    birthday: &["Date of Birth"],
    race: &["Race"],
    height: &["Height"],
    // Breeze, Ethan and Durnar (`char_275_breeze`, `char_355_ethan`,
    // `char_260_durnar`) write `[Originium Infection Condition]`.
    infection: &["Infection Status", "Originium Infection Condition"],
    manufacturer: &["Manufacturer"],
    axes: [
        &["Physical Strength"],
        &["Mobility"],
        &["Physical Resilience"],
        &["Tactical Acumen"],
        &["Combat Skill", "Combat Skills"],
        // Both misspellings are EN's own.
        &[
            "Originium Arts Assimilation",
            "Originium Arts Assilimation",
            "Originium ArtsAssimilation",
        ],
    ],
    // Feeble, Defective and Exceptional: see [`ExamStep`] for where the
    // other servers put them.
    ratings: &[
        ("Flawed", ExamStep::Flawed),
        ("Feeble", ExamStep::Flawed),
        ("Defective", ExamStep::Flawed),
        ("Normal", ExamStep::Normal),
        ("Standard", ExamStep::Standard),
        ("Excellent", ExamStep::Excellent),
        ("Outstanding", ExamStep::Outstanding),
        ("Exceptional", ExamStep::Outstanding),
    ],
    female: &["Female"],
    male: &["Male"],
    clear: &[
        "non-infected",
        "non infected",
        "uninfected",
        "not infected",
        "no infection",
        "no originium infection",
        "confirming there is no",
    ],
    infected: &[&["infect", "confirm"]],
    contrast: &[" but "],
    blood: &[],
    assimilation: &[],
    cm: &["cm"],
    density: &["u/l", "/l"],
    percent: &["%"],
    bounds: &ASCII_BOUNDS,
    none: &["none", "no combat"],
    half: &["half"],
    years: &["year"],
    months: &["month"],
    number_words: &[
        ("one", 1.0),
        ("two", 2.0),
        ("three", 3.0),
        ("four", 4.0),
        ("five", 5.0),
        ("six", 6.0),
        ("seven", 7.0),
        ("eight", 8.0),
        ("nine", 9.0),
        ("ten", 10.0),
        ("eleven", 11.0),
        ("twelve", 12.0),
    ],
    kanji_numbers: false,
    date_marks: None,
};

pub static CN: Vocab = Vocab {
    code_name: &["代号"],
    gender: &["性别"],
    combat: &["战斗经验"],
    birthplace: &["出身地"],
    birthday: &["生日"],
    race: &["种族"],
    height: &["身高"],
    infection: &["矿石病感染情况"],
    manufacturer: &["制造商"],
    axes: [
        &["物理强度"],
        &["战场机动"],
        &["生理耐受"],
        &["战术规划"],
        &["战斗技巧"],
        &["源石技艺适应性"],
    ],
    ratings: &[
        ("缺陷", ExamStep::Flawed),
        ("普通", ExamStep::Normal),
        ("标准", ExamStep::Standard),
        ("优良", ExamStep::Excellent),
        ("优秀", ExamStep::Excellent),
        ("卓越", ExamStep::Outstanding),
    ],
    female: &["女"],
    male: &["男"],
    clear: &["非感染者", "未感染"],
    infected: &[&["感染者"]],
    contrast: &[],
    blood: &["血液源石结晶密度"],
    assimilation: &["体细胞与源石融合率"],
    cm: &["cm", "厘米"],
    density: &["u/l", "/l"],
    percent: &["%"],
    // CN's one `＜1%` (full-width) still reads as 1: widening this would
    // move the CN numbers, which this table keeps as they were.
    bounds: &ASCII_BOUNDS,
    // CN writes years in hanzi (`五年`, `两年`) and leads 132 files with
    // `没有战斗经验`; `半年非实战训练` is EN's `Half-year mock combat training`.
    none: &["无", "没有"],
    half: &["半年"],
    years: &["年"],
    months: &["个月"],
    number_words: &[],
    kanji_numbers: true,
    date_marks: Some(('月', '日')),
};

pub static JA: Vocab = Vocab {
    code_name: &["コードネーム"],
    gender: &["性別"],
    combat: &["戦闘経験"],
    birthplace: &["出身地"],
    birthday: &["誕生日"],
    race: &["種族"],
    // Robots write 高さ where EN keeps `[Height]` for them too.
    height: &["身長", "高さ"],
    infection: &["鉱石病感染状況"],
    manufacturer: &["製造元"],
    axes: [
        &["物理強度"],
        &["戦場機動"],
        &["生理的耐性"],
        &["戦術立案"],
        &["戦闘技術"],
        &["アーツ適性"],
    ],
    ratings: &[
        ("欠落", ExamStep::Flawed),
        ("普通", ExamStep::Normal),
        ("標準", ExamStep::Standard),
        ("優秀", ExamStep::Excellent),
        ("卓越", ExamStep::Outstanding),
    ],
    female: &["女", "女性"],
    male: &["男", "男性"],
    clear: &["非感染者"],
    // `感染者と推定される` (Nearl) is EN's `Infection confirmed`.
    infected: &[&["感染者"]],
    contrast: &[],
    // 潤滑油中 / 循環冷却液中源石密度 and 機体の / 外骨格の源石融合率 are machines.
    blood: &["血液中源石密度"],
    assimilation: &["源石融合率"],
    cm: &["cm"],
    density: &["u/l", "/l"],
    percent: &["%", "％"],
    bounds: &['<', '>', '~', '＜', '＞', '～'],
    none: &["なし", "無し"],
    // `模擬訓練半年` is EN's `Half-year mock combat training`.
    half: &["半年", "模擬訓練半年"],
    years: &["年"],
    months: &["ヶ月", "か月", "カ月"],
    number_words: &[],
    kanji_numbers: true,
    date_marks: Some(('月', '日')),
};

pub static KO: Vocab = Vocab {
    code_name: &["코드네임", "코드 네임"],
    gender: &["성별"],
    combat: &["전투 경험"],
    birthplace: &["출신지"],
    birthday: &["생일"],
    race: &["종족"],
    // Robots write 높이 where EN keeps `[Height]` for them too.
    height: &["신장", "높이"],
    infection: &["광석병 감염 상황"],
    manufacturer: &["제조 업체"],
    axes: [
        &["물리적 강도"],
        &["전장 기동력"],
        &["생체 인내도"],
        &["전술 계획력"],
        // Kroos's file prints `전술 기술력` in the Combat Skill row.
        &["전투 기술력", "전술 기술력"],
        &["오리지늄 아츠 적응성"],
    ],
    // 탁월 and 월등 both stand where EN prints Outstanding (3 and 77 ratings).
    ratings: &[
        ("부족", ExamStep::Flawed),
        ("보통", ExamStep::Normal),
        ("표준", ExamStep::Standard),
        ("우수", ExamStep::Excellent),
        ("월등", ExamStep::Outstanding),
        ("탁월", ExamStep::Outstanding),
    ],
    female: &["여"],
    male: &["남"],
    clear: &["비감염자"],
    infected: &[&["감염자"]],
    contrast: &[],
    // 윤활유 중 / 순환 냉각액중 밀도 and 외골격과 / 신체와 오리지늄 융합률 are
    // machines (`신체와` is Ju's, EN's `Machine-Originium Assimilation`).
    blood: &["혈중 오리지늄 결정 밀도", "혈중 오리지늄 밀도"],
    assimilation: &["체세포와 오리지늄 융합률"],
    cm: &["cm"],
    // `u/` reads `char_144_red`'s `0.13u/` and `char_1038_whitw2`'s `0.33u/n`,
    // the game's own typos for u/L (EN prints both as u/L).
    density: &["u/l", "/l", "u/"],
    percent: &["%"],
    bounds: &ASCII_BOUNDS,
    none: &["없음"],
    half: &["반년", "반 년"],
    years: &["년"],
    months: &["개월"],
    number_words: &[],
    kanji_numbers: false,
    date_marks: Some(('월', '일')),
};

/// A gender word read the same in every language, so By the Numbers counts
/// one bar per gender on every server. Sent as a key; the word shown is the
/// server's own spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum ArchiveGender {
    Female,
    Male,
}

impl ArchiveGender {
    /// The gender a file's word names in `w`'s language. A word with no
    /// reading (`Unknown`, 断罪, `Lady`) gives `None` and is counted as
    /// written.
    #[must_use]
    pub fn read(w: &Vocab, raw: &str) -> Option<Self> {
        let v = raw.trim();
        if w.female.contains(&v) {
            Some(Self::Female)
        } else if w.male.contains(&v) {
            Some(Self::Male)
        } else {
            None
        }
    }
}

/// One exam column as the file states it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ExamRating {
    pub axis: ExamAxis,
    pub raw: String,
    /// Index into [`ExamStep::ALL`], lowest first; `None` for a redacted,
    /// split (`Standard/Excellent`) or free-text rating.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub step: Option<u8>,
    /// The file prints a row of blocks (`■■`) where the rating would be.
    pub redacted: bool,
}

/// A bracketed Basic Info line the reader has no column for, kept verbatim
/// (`[Weight] 58kg`, `[Dungeoneering Experience] 7 years`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ProfileField {
    pub label: String,
    pub value: String,
}

/// One operator's file. Every `*_raw`-style string is the file's own text;
/// each parsed sibling is `None` when the text does not read as one.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveProfile {
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_name: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub race: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthplace: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthday: Option<String>,
    /// 1..=12.
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_month: Option<u8>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_day: Option<u8>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_cm: Option<f64>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combat_experience: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combat_years: Option<f64>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infection: Option<String>,
    pub infection_state: InfectionState,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blood_density: Option<String>,
    /// Blood Originium-Crystal Density in u/L.
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blood_density_ul: Option<f64>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assimilation: Option<String>,
    /// Cell-Originium Assimilation in percent.
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assimilation_pct: Option<f64>,
    /// The six exam columns the file has, in [`ExamAxis::ALL`] order.
    pub exam: Vec<ExamRating>,
    /// `[Manufacturer]`: present only on a unit that was built.
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    pub extra: Vec<ProfileField>,
}

/// Read one operator's handbook sections in `lang`. `None` when the operator
/// has no handbook text at all.
#[must_use]
pub fn parse_archive_profile(
    lang: Lang,
    sections: &[HandbookStoryTextAudio],
) -> Option<ArchiveProfile> {
    let texts: Vec<&str> = sections
        .iter()
        .filter_map(|s| s.stories.first().map(|st| st.story_text.as_str()))
        .collect();
    if texts.is_empty() {
        return None;
    }
    Some(parse_texts_in(lang.vocab(), &texts))
}

/// The reader over plain section texts, in handbook order. Basic Info is the
/// first section, the exam the second; the clinical lines are looked for in
/// the first five sections, because a few files put Profile ahead of them.
#[must_use]
pub fn parse_texts_in(w: &Vocab, texts: &[&str]) -> ArchiveProfile {
    let mut p = ArchiveProfile::default();
    if let Some(basic) = texts.first() {
        read_basic(w, &mut p, basic);
    }
    if let Some(exam) = texts.get(1) {
        read_exam(w, &mut p, exam);
    }
    for text in texts.iter().take(5) {
        read_clinical(w, &mut p, text);
    }
    p
}

/// Every `[Label] value` pair in a section. The value is the rest of the
/// label's line; when that is empty it is the next lines up to the next label
/// or a blank line (EN writes `[Infection Status]` and its sentence on two
/// lines). A label with a line break inside it is the game's own corrupted
/// text and is skipped.
#[must_use]
pub fn bracketed(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut open: Option<usize> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some((label, value)) = split_label(line) {
            out.push((label.to_owned(), value.to_owned()));
            open = value.is_empty().then_some(out.len() - 1);
            continue;
        }
        match open {
            Some(i) if !line.is_empty() => {
                let v = &mut out[i].1;
                if !v.is_empty() {
                    v.push(' ');
                }
                v.push_str(line);
            }
            _ => open = None,
        }
    }
    out
}

fn split_label(line: &str) -> Option<(&str, &str)> {
    let (open, close) = if line.starts_with('[') {
        ('[', ']')
    } else if line.starts_with('【') {
        ('【', '】')
    } else {
        return None;
    };
    let end = line.find(close)?;
    let label = line[open.len_utf8()..end].trim();
    (!label.is_empty()).then(|| (label, line[end + close.len_utf8()..].trim()))
}

/// `value` trimmed, `None` when nothing is left.
pub(super) fn trimmed(value: &str) -> Option<String> {
    let v = value.trim();
    (!v.is_empty()).then(|| v.to_owned())
}

fn read_basic(w: &Vocab, p: &mut ArchiveProfile, text: &str) {
    for (label, value) in bracketed(text) {
        let is = |labels: &[&str]| labels.contains(&label.as_str());
        if is(w.code_name) {
            p.code_name = trimmed(&value);
        } else if is(w.gender) {
            // Arene's file reads `Male]`: the game's own stray bracket.
            p.gender = trimmed(value.trim_end_matches(']'));
        } else if is(w.combat) {
            p.combat_years = combat_years(w, &value);
            p.combat_experience = trimmed(&value);
        } else if is(w.birthplace) {
            p.birthplace = trimmed(&value);
        } else if is(w.birthday) {
            if let Some((m, d)) = birthday(w, &value) {
                p.birth_month = Some(m);
                p.birth_day = Some(d);
            }
            p.birthday = trimmed(&value);
        } else if is(w.race) {
            p.race = trimmed(&value);
        } else if is(w.height) {
            p.height_cm = height_cm(w, &value);
            p.height = trimmed(&value);
        } else if is(w.infection) {
            // Wind Chimes repeats the label; the first one stands.
            if p.infection.is_none() {
                p.infection_state = infection_state(w, &value);
                p.infection = trimmed(&value);
            }
        } else if is(w.manufacturer) {
            p.manufacturer = trimmed(&value);
        } else if label.chars().count() <= MAX_EXTRA_LABEL && !value.is_empty() {
            p.extra.push(ProfileField { label, value });
        }
    }
}

/// A Basic Info "label" longer than this is a sentence the game bracketed by
/// mistake, not a field, and is not kept as an extra. A presentation
/// threshold, not a game fact.
const MAX_EXTRA_LABEL: usize = 40;

fn exam_axis(w: &Vocab, label: &str) -> Option<ExamAxis> {
    w.axes
        .iter()
        .position(|labels| labels.contains(&label))
        .map(|i| ExamAxis::ALL[i])
}

fn read_exam(w: &Vocab, p: &mut ArchiveProfile, text: &str) {
    let mut found: Vec<ExamRating> = Vec::new();
    for (label, value) in bracketed(text) {
        let Some(axis) = exam_axis(w, &label) else {
            continue;
        };
        if found.iter().any(|r| r.axis == axis) {
            continue;
        }
        found.push(ExamRating {
            axis,
            step: exam_step(w, &value),
            redacted: value.contains('■'),
            raw: value,
        });
    }
    found.sort_by_key(|r| ExamAxis::ALL.iter().position(|a| *a == r.axis));
    p.exam = found;
}

/// The scale step a rating word names. A split (`Standard/Excellent`) or a
/// rating with anything else on it is not read: the file does not give one
/// step.
#[must_use]
pub fn exam_step(w: &Vocab, value: &str) -> Option<u8> {
    let v = value.trim();
    w.ratings
        .iter()
        .find(|(word, _)| *word == v)
        .map(|(_, step)| step.rank())
}

fn read_clinical(w: &Vocab, p: &mut ArchiveProfile, text: &str) {
    // EN spells the clinical labels many ways and is read by keyword; the
    // other languages list their exact labels.
    let english = std::ptr::eq(w, &EN);
    for (label, value) in bracketed(text) {
        let l = label.to_lowercase();
        // EN: `Blood Originium-Crystal Density`, `Blood Crystal Density`.
        // The robots' oil and coolant densities are not blood and are not read.
        let blood = if english {
            l.starts_with("blood") && l.contains("density")
        } else {
            w.blood.contains(&label.as_str())
        };
        if p.blood_density.is_none() && blood {
            // `char_1030_noirc2` drops the unit's `u` (`0.29/L`).
            p.blood_density_ul = number_before(&value, w.density, w.bounds);
            p.blood_density = trimmed(&value);
        }
        // EN: `Cell-Originium Assimilation`, `Cell-Originium Fusion Rate` and
        // the `Somatic` forms. Machine and exoskeleton rates are not cells.
        let cells = if english {
            l.contains("cell-originium") && (l.contains("assimilation") || l.contains("fusion"))
        } else {
            w.assimilation.contains(&label.as_str())
        };
        if p.assimilation.is_none() && cells {
            p.assimilation_pct = number_before(&value, w.percent, w.bounds);
            p.assimilation = trimmed(&value);
        }
    }
}

/// The first number in `value` directly followed (spaces allowed) by one of
/// `units`, compared case-insensitively. A value with a comparator from
/// `bounds` in front of the number (`<1%`) is a bound, not a reading, and
/// gives `None`.
#[must_use]
pub fn number_before(value: &str, units: &[&str], bounds: &[char]) -> Option<f64> {
    let lower = value.to_lowercase();
    let (start, end, n) = first_number(&lower)?;
    let before = lower[..start].trim_end();
    if before.ends_with(bounds) {
        return None;
    }
    let rest = lower[end..].trim_start();
    units.iter().any(|u| rest.starts_with(u)).then_some(n)
}

/// Byte span and value of the first decimal number in `s`.
fn first_number(s: &str) -> Option<(usize, usize, f64)> {
    let bytes = s.as_bytes();
    let start = bytes.iter().position(u8::is_ascii_digit)?;
    let mut end = start;
    let mut dot = false;
    while end < bytes.len() {
        let b = bytes[end];
        if b.is_ascii_digit() {
            end += 1;
        } else if b == b'.' && !dot && bytes.get(end + 1).is_some_and(u8::is_ascii_digit) {
            dot = true;
            end += 1;
        } else {
            break;
        }
    }
    s[start..end].parse().ok().map(|n| (start, end, n))
}

/// A leading kanji or hanzi numeral up to the hundreds (`三`, `十二`,
/// `二十五`, `七十`, `两`) and its byte length.
fn kanji_number(s: &str) -> Option<(f64, usize)> {
    const DIGITS: [char; 10] = ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
    let (mut total, mut cur, mut len) = (0u32, 0u32, 0usize);
    for c in s.chars() {
        // 两 is CN's other two (`两年`); JP never writes it.
        let digit = if c == '两' {
            Some(2)
        } else {
            DIGITS.iter().position(|k| *k == c)
        };
        if let Some(d) = digit {
            cur = u32::try_from(d).ok()?;
        } else if c == '十' || c == '百' {
            let unit = if c == '十' { 10 } else { 100 };
            total += cur.max(1) * unit;
            cur = 0;
        } else {
            break;
        }
        len += c.len_utf8();
    }
    (len > 0).then(|| (f64::from(total + cur), len))
}

/// A height in metres at or above this is not read as metres: nobody on
/// file is 3 m tall, and a bare `m` after a larger number is some other
/// figure. A sanity threshold, not a game fact.
const MAX_HEIGHT_M: f64 = 3.0;

/// Height in centimetres: `163cm`, `163 cm`, `1.69m`. `35cm (sitting) 75cm
/// (stretching...)` reads as its first figure, 35, the one the file leads
/// with.
#[must_use]
pub fn height_cm(w: &Vocab, value: &str) -> Option<f64> {
    if let Some(cm) = number_before(value, w.cm, w.bounds) {
        return Some(cm);
    }
    number_before(value, &["m"], w.bounds)
        .filter(|m| *m < MAX_HEIGHT_M)
        .map(|m| (m * 100.0).round())
}

/// Combat experience in years. `None`, `No combat experience`, 无, 没有, なし
/// and 없음 read as 0; `Half a year`, 半年, 반년 and `6 Months` as 0.5; `1 Month`
/// as 1/12. Undisclosed, unknown, classified and dashes are not a duration
/// and give `None`.
#[must_use]
pub fn combat_years(w: &Vocab, value: &str) -> Option<f64> {
    let v = value.trim().to_lowercase();
    if v.is_empty() {
        return None;
    }
    if w.none.iter().any(|k| v.starts_with(k)) {
        return Some(0.0);
    }
    if w.half.iter().any(|k| v.starts_with(k)) {
        return Some(0.5);
    }
    let (amount, rest) = match first_number(&v) {
        Some((start, end, n)) if v[..start].trim().is_empty() => (n, v[end..].trim_start()),
        _ => match kanji_number(&v).filter(|_| w.kanji_numbers) {
            Some((n, len)) => (n, v[len..].trim_start()),
            None => {
                let (word, n) = w.number_words.iter().find(|(k, _)| v.starts_with(k))?;
                (*n, v[word.len()..].trim_start())
            }
        },
    };
    if w.years.iter().any(|k| rest.starts_with(k)) {
        Some(amount)
    } else if w.months.iter().any(|k| rest.starts_with(k)) {
        Some((amount / 12.0 * 100.0).round() / 100.0)
    } else {
        None
    }
}

const MONTHS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];

/// Month and day: `Dec. 23`, `February 13`, `July 22`, 12月23日, 4월 23일. A
/// word answer (`Unknown`, `Undisclosed`) gives `None`.
#[must_use]
pub fn birthday(w: &Vocab, value: &str) -> Option<(u8, u8)> {
    let v = value.trim().to_lowercase();
    if let Some((month_mark, day_mark)) = w.date_marks {
        let (m, rest) = v.split_once(month_mark)?;
        let month: u8 = m.trim().parse().ok()?;
        let day: u8 = rest.trim_end_matches(day_mark).trim().parse().ok()?;
        return valid_date(month, day);
    }
    let month = MONTHS.iter().position(|m| v.starts_with(m))?;
    let rest = v.trim_start_matches(|c: char| c.is_ascii_alphabetic() || c == '.');
    let (_, _, day) = first_number(rest)?;
    if !rest.trim_start().starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    valid_date(u8::try_from(month + 1).ok()?, day as u8)
}

fn valid_date(month: u8, day: u8) -> Option<(u8, u8)> {
    ((1..=12).contains(&month) && (1..=31).contains(&day)).then_some((month, day))
}

/// Read the infection sentence. Negations are checked first, because
/// "infected" is a substring of "uninfected" (and 感染者 of 非感染者, 감염자
/// of 비감염자); what is left reads as infected only when it holds every word
/// of one of the language's infected groups (EN: it both names infection and
/// says it was confirmed).
/// The one EN sentence that names symptoms it does NOT show and then confirms
/// infection ("Showed no outward symptoms ..., but Infection was confirmed")
/// is caught by reading what follows a `but`.
#[must_use]
pub fn infection_state(w: &Vocab, value: &str) -> InfectionState {
    let v = value.trim().to_lowercase();
    if v.is_empty() {
        return InfectionState::Unstated;
    }
    let tail = w
        .contrast
        .iter()
        .find_map(|c| v.rsplit_once(c).map(|(_, t)| t))
        .unwrap_or(v.as_str());
    if w.clear.iter().any(|k| tail.contains(k)) {
        return InfectionState::Clear;
    }
    if w.infected
        .iter()
        .any(|group| group.iter().all(|k| tail.contains(k)))
    {
        return InfectionState::Infected;
    }
    InfectionState::Unclear
}

#[cfg(test)]
mod tests {
    use super::*;

    // `char_500_noirc`'s three sections on EN, 2026-10-11.
    const BASIC: &str = "[Code Name] Noir Corne\n[Gender] Male\n[Combat Experience] 8 years\n[Place of Birth] Higashi\n[Date of Birth] Aug. 30\n[Race] Oni\n[Height] 180cm\n[Infection Status]\nInfection confirmed by medical examination.";
    const EXAM: &str = "[Physical Strength] Standard\n[Mobility] Normal\n[Physical Resilience] Excellent\n[Tactical Acumen] Normal\n[Combat Skill] Normal\n[Originium Arts Assilimation] Standard";
    const CLINICAL: &str = "Radiological scan showed blurred outlines. The subject is confirmed to be infected. \n\n[Cell-Originium Assimilation] 11%\nNo major infection symptoms have yet appeared. \n\n[Blood Originium-Crystal Density] 0.22u/L\nSeverity of infection remains low.";

    #[test]
    fn reads_a_whole_file() {
        let p = parse_texts_in(&EN, &[BASIC, EXAM, "Profile text.", CLINICAL]);
        assert_eq!(p.code_name.as_deref(), Some("Noir Corne"));
        assert_eq!(p.gender.as_deref(), Some("Male"));
        assert_eq!(p.combat_years, Some(8.0));
        assert_eq!(p.birthplace.as_deref(), Some("Higashi"));
        assert_eq!((p.birth_month, p.birth_day), (Some(8), Some(30)));
        assert_eq!(p.race.as_deref(), Some("Oni"));
        assert_eq!(p.height_cm, Some(180.0));
        assert_eq!(
            p.infection.as_deref(),
            Some("Infection confirmed by medical examination.")
        );
        assert_eq!(p.infection_state, InfectionState::Infected);
        assert_eq!(p.assimilation_pct, Some(11.0));
        assert_eq!(p.blood_density_ul, Some(0.22));
        assert_eq!(p.exam.len(), 6);
        assert_eq!(p.exam[5].axis, ExamAxis::Arts);
        assert_eq!(p.exam[5].step, Some(ExamStep::Standard.rank()));
        assert!(p.extra.is_empty());
    }

    #[test]
    fn heights() {
        assert_eq!(height_cm(&EN, "163 cm"), Some(163.0));
        assert_eq!(height_cm(&EN, "1.69m"), Some(169.0));
        assert_eq!(
            height_cm(
                &EN,
                "35cm (sitting) 75cm (stretching, including tail, approximate)"
            ),
            Some(35.0)
        );
        assert_eq!(height_cm(&EN, "Undisclosed"), None);
        assert_eq!(height_cm(&EN, "1G #qI3?JAYXFV ZHanG"), None);
    }

    #[test]
    fn combat_experience() {
        assert_eq!(combat_years(&EN, "None"), Some(0.0));
        assert_eq!(combat_years(&EN, "None (self-declared)"), Some(0.0));
        assert_eq!(combat_years(&EN, "No combat experience"), Some(0.0));
        assert_eq!(combat_years(&EN, "Half a year"), Some(0.5));
        assert_eq!(
            combat_years(&EN, "Half-year mock combat training"),
            Some(0.5)
        );
        assert_eq!(combat_years(&EN, "6 Months"), Some(0.5));
        assert_eq!(combat_years(&EN, "Six months"), Some(0.5));
        assert_eq!(combat_years(&EN, "1 Month"), Some(0.08));
        assert_eq!(combat_years(&EN, "Seven Years"), Some(7.0));
        assert_eq!(combat_years(&EN, "15 years (Self-reported)"), Some(15.0));
        assert_eq!(combat_years(&EN, "70 years"), Some(70.0));
        assert_eq!(combat_years(&EN, "Undisclosed"), None);
        // The game's own two dashes, written as escapes.
        assert_eq!(combat_years(&EN, "\u{2014}\u{2014}"), None);
        assert_eq!(combat_years(&EN, "N/A"), None);
        assert_eq!(combat_years(&EN, "Classified"), None);
    }

    #[test]
    fn birthdays() {
        assert_eq!(birthday(&EN, "Dec. 23"), Some((12, 23)));
        assert_eq!(birthday(&EN, "February 13"), Some((2, 13)));
        assert_eq!(birthday(&EN, "Jun. 1"), Some((6, 1)));
        assert_eq!(birthday(&CN, "12月23日"), Some((12, 23)));
        assert_eq!(birthday(&EN, "Unknown"), None);
        assert_eq!(birthday(&EN, "Reported as Unknown"), None);
    }

    #[test]
    fn originium_numbers() {
        assert_eq!(number_before("0.2 u/L", &["u/l"], &ASCII_BOUNDS), Some(0.2));
        assert_eq!(number_before("Undisclosed", &["u/l"], &ASCII_BOUNDS), None);
        assert_eq!(number_before("<1%", &["%"], &ASCII_BOUNDS), None);
        assert_eq!(number_before("0%.", &["%"], &ASCII_BOUNDS), Some(0.0));
        assert_eq!(
            number_before(
                "0% Operator Degenbrecher shows no signs of Originium infection.",
                &["%"],
                &ASCII_BOUNDS
            ),
            Some(0.0)
        );
        assert_eq!(number_before("7.4%", &["%"], &ASCII_BOUNDS), Some(7.4));
        assert_eq!(number_before("?", &["%"], &ASCII_BOUNDS), None);
    }

    #[test]
    fn en_infection_condition_label() {
        let p = parse_texts_in(
            &EN,
            &[
                "[Code Name] Breeze\n[Originium Infection Condition] \nMedical tests have confirmed that no infection is present.",
            ],
        );
        assert_eq!(p.infection_state, InfectionState::Clear);
    }

    #[test]
    fn infection_sentences() {
        use InfectionState::{Clear, Infected, Unclear, Unstated};
        let cases = [
            (
                "Medical tests have confirmed that no infection is present.",
                Clear,
            ),
            ("Confirmed Infected by medical examination.", Infected),
            ("Non-infected as confirmed by medical report.", Clear),
            ("Confirmed uninfected by medical examination.", Clear),
            (
                "Originium crystals have not been found on the surface of the body, confirming there is no infection.",
                Clear,
            ),
            (
                "Showed no outward symptoms of Originium Infection, but Infection was confirmed by medical examination.",
                Infected,
            ),
            (
                "Traces of Originium Crystals have been detected in her body, confirming she is infected.",
                Infected,
            ),
            ("Medical tests confirm she is infected.", Infected),
            ("Medical tests confirm uninfected status.", Clear),
            ("Infection confirmed through clinical diagnosis.", Infected),
            ("Undisclosed", Unclear),
            ("", Unstated),
        ];
        for (text, want) in cases {
            assert_eq!(infection_state(&EN, text), want, "{text}");
        }
    }

    #[test]
    fn exam_steps_refuse_splits_and_blocks() {
        assert_eq!(exam_step(&EN, "Standard"), Some(2));
        assert_eq!(exam_step(&CN, "缺陷"), Some(0));
        // EN's extra words sit where CN, JP and KR put them.
        assert_eq!(exam_step(&EN, "Feeble"), exam_step(&EN, "Flawed"));
        assert_eq!(exam_step(&EN, "Defective"), exam_step(&CN, "缺陷"));
        assert_eq!(exam_step(&EN, "Exceptional"), exam_step(&CN, "卓越"));
        assert_eq!(exam_step(&EN, "Standard/Excellent"), None);
        assert_eq!(exam_step(&EN, "■■"), None);
        let p = parse_texts_in(
            &EN,
            &["[Code Name] X", "[Physical Strength] ■■\n[Mobility] 24km/h"],
        );
        assert!(p.exam[0].redacted);
        assert_eq!(p.exam[1].step, None);
    }

    #[test]
    fn unknown_basic_labels_are_kept_as_extras() {
        let p = parse_texts_in(
            &EN,
            &[
                "[Code Name] Lancet-2\n[Model] Medical robot\n[Manufacturer] Rhodes Island\n[Gender] Male]",
            ],
        );
        assert_eq!(p.manufacturer.as_deref(), Some("Rhodes Island"));
        assert_eq!(p.gender.as_deref(), Some("Male"));
        assert_eq!(
            p.extra,
            vec![ProfileField {
                label: "Model".into(),
                value: "Medical robot".into()
            }]
        );
    }

    // `char_500_noirc` on JP and KR, 2026-10-06 extracts.
    const JA_BASIC: &str = "【コードネーム】ノイルホーン\n【性別】男性\n【戦闘経験】八年\n【出身地】極東\n【誕生日】8月30日\n【種族】鬼\n【身長】180cm\n【鉱石病感染状況】\nメディカルチェックの結果、感染者に認定。";
    const JA_EXAM: &str = "【物理強度】標準\n【戦場機動】普通\n【生理的耐性】優秀\n【戦術立案】普通\n【戦闘技術】普通\n【アーツ適性】標準";
    const JA_CLINICAL: &str = "以上の結果から、鉱石病感染者と判定。\n\n【源石融合率】11%\n現状体表に鉱石病の兆候は見られない。\n\n【血液中源石密度】0.22u/L\n感染状況は浅い。";
    const KO_BASIC: &str = "[코드네임] 노이르 코른\n[성별] 남\n[전투 경험] 8년\n[출신지] 극동\n[생일] 8월 30일\n[종족] 오니\n[신장] 180cm\n[광석병 감염 상황]\n의학 테스트 보고서 참고 결과 감염자로 확인.";
    const KO_EXAM: &str = "[물리적 강도] 표준\n[전장 기동력] 보통\n[생체 인내도] 우수\n[전술 계획력] 보통\n[전투 기술력] 보통\n[오리지늄 아츠 적응성] 표준";
    const KO_CLINICAL: &str = "[체세포와 오리지늄 융합률] 11%\n\n[혈중 오리지늄 결정 밀도] 0.22u/L";

    #[test]
    fn jp_and_kr_files_read_like_en() {
        let en = parse_texts_in(&EN, &[BASIC, EXAM, "Profile text.", CLINICAL]);
        for (basic, exam, clinical, lang) in [
            (JA_BASIC, JA_EXAM, JA_CLINICAL, Lang::Ja),
            (KO_BASIC, KO_EXAM, KO_CLINICAL, Lang::Ko),
        ] {
            let w = lang.vocab();
            let p = parse_texts_in(w, &[basic, exam, "Profile.", clinical]);
            assert!(p.code_name.is_some(), "{lang:?}");
            assert_eq!(
                p.gender.as_deref().and_then(|g| ArchiveGender::read(w, g)),
                Some(ArchiveGender::Male)
            );
            assert_eq!(p.combat_years, en.combat_years, "{lang:?}");
            assert_eq!((p.birth_month, p.birth_day), (Some(8), Some(30)));
            assert_eq!(p.height_cm, en.height_cm);
            assert_eq!(p.infection_state, InfectionState::Infected);
            assert_eq!(p.assimilation_pct, en.assimilation_pct);
            assert_eq!(p.blood_density_ul, en.blood_density_ul);
            let steps = |p: &ArchiveProfile| p.exam.iter().map(|r| r.step).collect::<Vec<_>>();
            assert_eq!(steps(&p), steps(&en), "{lang:?}");
            assert!(p.extra.is_empty(), "{lang:?} {:?}", p.extra);
        }
        assert_eq!(Lang::for_server(Server::EN), Some(Lang::En));
        assert_eq!(Lang::for_server(Server::Bilibili), Some(Lang::Cn));
        assert_eq!(Lang::for_server(Server::TW), None);
    }

    #[test]
    fn jp_and_kr_vocabulary() {
        assert_eq!(combat_years(&JA, "二十五年"), Some(25.0));
        assert_eq!(combat_years(&JA, "十二年"), Some(12.0));
        assert_eq!(combat_years(&JA, "七十年"), Some(70.0));
        assert_eq!(combat_years(&JA, "十五年（自称）"), Some(15.0));
        assert_eq!(combat_years(&JA, "一ヶ月"), Some(0.08));
        assert_eq!(combat_years(&JA, "なし（自称）"), Some(0.0));
        assert_eq!(combat_years(&JA, "模擬訓練半年"), Some(0.5));
        assert_eq!(combat_years(&JA, "非公開"), None);
        assert_eq!(combat_years(&KO, "15년(자칭)"), Some(15.0));
        assert_eq!(combat_years(&KO, "6개월"), Some(0.5));
        assert_eq!(combat_years(&KO, "반 년의 비실전 훈련"), Some(0.5));
        assert_eq!(combat_years(&KO, "없음 (본인 주장)"), Some(0.0));
        assert_eq!(combat_years(&KO, "기밀"), None);
        assert_eq!(combat_years(&CN, "五年"), Some(5.0));
        assert_eq!(combat_years(&CN, "两年"), Some(2.0));
        assert_eq!(combat_years(&CN, "四十四年"), Some(44.0));
        assert_eq!(combat_years(&CN, "没有战斗经验（本人声称）"), Some(0.0));
        assert_eq!(combat_years(&CN, "半年非实战训练"), Some(0.5));
        assert_eq!(combat_years(&CN, "未公开"), None);
        assert_eq!(birthday(&KO, "8월22일"), Some((8, 22)));
        assert_eq!(birthday(&JA, "本人は忘れたと主張"), None);
        assert_eq!(height_cm(&JA, "185cm前後"), Some(185.0));
        assert_eq!(number_before("＜1%", JA.percent, JA.bounds), None);
        assert_eq!(number_before("14.5％", JA.percent, JA.bounds), Some(14.5));
        assert_eq!(number_before("0.33u/n", KO.density, KO.bounds), Some(0.33));
        assert_eq!(
            infection_state(&JA, "体表に源石結晶の分布なし。非感染者に認定。"),
            InfectionState::Clear
        );
        assert_eq!(
            infection_state(
                &KO,
                "체표면에 오리지늄 결정이 없으나, 의학 테스트 보고서 참고 결과 감염자로 확인."
            ),
            InfectionState::Infected
        );
        assert_eq!(infection_state(&KO, "미공개"), InfectionState::Unclear);
        assert_eq!(exam_step(&JA, "欠落"), Some(0));
        assert_eq!(exam_step(&KO, "탁월"), Some(4));
        assert_eq!(exam_step(&KO, "표준/우수"), None);
        for (w, word, want) in [
            (&JA, "女性", ArchiveGender::Female),
            (&KO, "여", ArchiveGender::Female),
            (&CN, "男", ArchiveGender::Male),
            (&EN, "Male", ArchiveGender::Male),
        ] {
            assert_eq!(ArchiveGender::read(w, word), Some(want), "{word}");
        }
        assert_eq!(ArchiveGender::read(&CN, "断罪"), None);
        // Each language reads only its own words.
        assert_eq!(ArchiveGender::read(&EN, "女"), None);
    }
}
