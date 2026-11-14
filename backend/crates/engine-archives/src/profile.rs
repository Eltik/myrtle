use serde::{Deserialize, Serialize};

use myrtle_gamedata_raw::tables::handbook::HandbookStoryTextAudio;
use myrtle_types::Server;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InfectionState {
    Infected,
    Clear,
    /// File states something but is unparsable
    Unclear,
    #[default]
    Unstated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExamStep {
    Flawed,
    Normal,
    Standard,
    Excellent,
    Outstanding,
}

impl ExamStep {
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
    pub axes: [&'static [&'static str]; 6],
    pub ratings: &'static [(&'static str, ExamStep)],
    pub female: &'static [&'static str],
    pub male: &'static [&'static str],
    pub clear: &'static [&'static str],
    pub infected: &'static [&'static [&'static str]],
    pub contrast: &'static [&'static str],
    pub blood: &'static [&'static str],
    pub assimilation: &'static [&'static str],
    pub cm: &'static [&'static str],
    pub density: &'static [&'static str],
    pub percent: &'static [&'static str],
    pub bounds: &'static [char],
    pub none: &'static [&'static str],
    pub half: &'static [&'static str],
    pub years: &'static [&'static str],
    pub months: &'static [&'static str],
    pub number_words: &'static [(&'static str, f64)],
    pub kanji_numbers: bool,
    pub date_marks: Option<(char, char)>,
}

pub const ASCII_BOUNDS: [char; 3] = ['<', '>', '~'];

pub static EN: Vocab = Vocab {
    code_name: &["Code Name", "Codename"],
    gender: &["Gender"],
    combat: &["Combat Experience"],
    birthplace: &["Place of Birth"],
    birthday: &["Date of Birth"],
    race: &["Race"],
    height: &["Height"],
    infection: &["Infection Status", "Originium Infection Condition"],
    manufacturer: &["Manufacturer"],
    axes: [
        &["Physical Strength"],
        &["Mobility"],
        &["Physical Resilience"],
        &["Tactical Acumen"],
        &["Combat Skill", "Combat Skills"],
        &[
            "Originium Arts Assimilation",
            "Originium Arts Assilimation",
            "Originium ArtsAssimilation",
        ],
    ],
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
    bounds: &ASCII_BOUNDS,
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
    infected: &[&["感染者"]],
    contrast: &[],
    blood: &["血液中源石密度"],
    assimilation: &["源石融合率"],
    cm: &["cm"],
    density: &["u/l", "/l"],
    percent: &["%", "％"],
    bounds: &['<', '>', '~', '＜', '＞', '～'],
    none: &["なし", "無し"],
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
    height: &["신장", "높이"],
    infection: &["광석병 감염 상황"],
    manufacturer: &["제조 업체"],
    axes: [
        &["물리적 강도"],
        &["전장 기동력"],
        &["생체 인내도"],
        &["전술 계획력"],
        &["전투 기술력", "전술 기술력"],
        &["오리지늄 아츠 적응성"],
    ],
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
    blood: &["혈중 오리지늄 결정 밀도", "혈중 오리지늄 밀도"],
    assimilation: &["체세포와 오리지늄 융합률"],
    cm: &["cm"],
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArchiveGender {
    Female,
    Male,
}

impl ArchiveGender {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamRating {
    pub axis: ExamAxis,
    pub raw: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<u8>,
    /// Has `■■` where rating would be
    pub redacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileField {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub race: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthplace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthday: Option<String>,
    /// 1..=12.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_month: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_day: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_cm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combat_experience: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combat_years: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infection: Option<String>,
    pub infection_state: InfectionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blood_density: Option<String>,
    /// Blood Originium-Crystal Density in u/L
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blood_density_ul: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assimilation: Option<String>,
    /// Cell-Originium Assimilation in percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assimilation_pct: Option<f64>,
    pub exam: Vec<ExamRating>,
    /// `[Manufacturer]` for robots and such
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    pub extra: Vec<ProfileField>,
}

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

#[must_use]
pub fn exam_step(w: &Vocab, value: &str) -> Option<u8> {
    let v = value.trim();
    w.ratings
        .iter()
        .find(|(word, _)| *word == v)
        .map(|(_, step)| step.rank())
}

fn read_clinical(w: &Vocab, p: &mut ArchiveProfile, text: &str) {
    let english = std::ptr::eq(w, &raw const EN);
    for (label, value) in bracketed(text) {
        let l = label.to_lowercase();
        let blood = if english {
            l.starts_with("blood") && l.contains("density")
        } else {
            w.blood.contains(&label.as_str())
        };
        if p.blood_density.is_none() && blood {
            p.blood_density_ul = number_before(&value, w.density, w.bounds);
            p.blood_density = trimmed(&value);
        }
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

fn kanji_number(s: &str) -> Option<(f64, usize)> {
    const DIGITS: [char; 10] = ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
    let (mut total, mut cur, mut len) = (0u32, 0u32, 0usize);
    for c in s.chars() {
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

const MAX_HEIGHT_M: f64 = 3.0;

#[must_use]
pub fn height_cm(w: &Vocab, value: &str) -> Option<f64> {
    if let Some(cm) = number_before(value, w.cm, w.bounds) {
        return Some(cm);
    }
    number_before(value, &["m"], w.bounds)
        .filter(|m| *m < MAX_HEIGHT_M)
        .map(|m| (m * 100.0).round())
}

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
        _ => {
            if let Some((n, len)) = kanji_number(&v).filter(|_| w.kanji_numbers) {
                (n, v[len..].trim_start())
            } else {
                let (word, n) = w.number_words.iter().find(|(k, _)| v.starts_with(k))?;
                (*n, v[word.len()..].trim_start())
            }
        }
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
