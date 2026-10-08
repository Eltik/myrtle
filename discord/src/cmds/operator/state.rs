//! The operator view's state, carried whole in each control's `custom_id`.
//!
//! Nothing is stored server side: a select's `custom_id` names the owner, the operator, the
//! page, the page's sub-selection and the part, and every interaction re-renders from it. A
//! control never expires, survives restarts, and costs no memory.
//!
//! Format: `op:<owner>:<operator id>:<page>:<sel>:<part>:<control>`, plain ASCII, `:` as the
//! separator (operator ids are `[a-z0-9_]`). An empty `<sel>` means the page's default.

/// Every operator control's `custom_id` starts with this, followed by [`SEPARATOR`].
pub const PREFIX: &str = "op";
const SEPARATOR: char = ':';
/// Discord's cap on a `custom_id`.
pub const CUSTOM_ID_MAX: usize = 100;

/// The operator view's pages, in the order the page select lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    Skills,
    Summons,
    Modules,
    Base,
    Costs,
    Outfits,
    Lore,
    Voice,
    Paradox,
}

impl Page {
    pub const ALL: [Self; 10] = [
        Self::Overview,
        Self::Skills,
        Self::Summons,
        Self::Modules,
        Self::Base,
        Self::Costs,
        Self::Outfits,
        Self::Lore,
        Self::Voice,
        Self::Paradox,
    ];

    /// The short code a `custom_id` carries.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Overview => "ov",
            Self::Skills => "sk",
            Self::Summons => "su",
            Self::Modules => "md",
            Self::Base => "bs",
            Self::Costs => "co",
            Self::Outfits => "of",
            Self::Lore => "lo",
            Self::Voice => "vo",
            Self::Paradox => "px",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.code() == code)
    }

    /// The page's name in the select and the footer.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Skills => "Skills",
            Self::Summons => "Summons",
            Self::Modules => "Modules",
            Self::Base => "Base skills",
            Self::Costs => "Upgrade costs",
            Self::Outfits => "Outfits",
            Self::Lore => "Lore",
            Self::Voice => "Voice lines",
            Self::Paradox => "Paradox Simulation",
        }
    }
}

/// Which of a message's selects a `custom_id` belongs to. Discord needs the ids within one
/// message to differ, and the handler needs to know which part of the state the value sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    /// The page select: its value is a page code.
    Page,
    /// The page's own select (elite, skill level, module stage, outfit...): a number.
    Sel,
    /// The part select, when a page runs over one message: a number.
    Part,
}

impl Control {
    const fn code(self) -> &'static str {
        match self {
            Self::Page => "p",
            Self::Sel => "s",
            Self::Part => "t",
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        match code {
            "p" => Some(Self::Page),
            "s" => Some(Self::Sel),
            "t" => Some(Self::Part),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewState {
    /// The user whose controls these are. Anyone else gets their own ephemeral copy.
    pub owner: u64,
    /// The operator id, `char_2023_ling`.
    pub op: String,
    pub page: Page,
    /// The page's sub-selection; `None` is the page's default.
    pub sel: Option<u16>,
    /// Which part of a page too long for one message.
    pub part: u16,
}

impl ViewState {
    #[must_use]
    pub fn new(owner: u64, op: &str, page: Page) -> Self {
        Self {
            owner,
            op: op.to_string(),
            page,
            sel: None,
            part: 0,
        }
    }

    /// This state as the `custom_id` of `control`.
    #[must_use]
    pub fn custom_id(&self, control: Control) -> String {
        let sel = self.sel.map(|s| s.to_string()).unwrap_or_default();
        let id = format!(
            "{PREFIX}{SEPARATOR}{}{SEPARATOR}{}{SEPARATOR}{}{SEPARATOR}{sel}{SEPARATOR}{}{SEPARATOR}{}",
            self.owner,
            self.op,
            self.page.code(),
            self.part,
            control.code()
        );
        debug_assert!(id.len() <= CUSTOM_ID_MAX, "custom_id too long: {id}");
        id
    }

    /// Read a `custom_id` back. `None` for anything this view didn't write.
    #[must_use]
    pub fn parse(id: &str) -> Option<(Self, Control)> {
        if id.len() > CUSTOM_ID_MAX {
            return None;
        }
        let mut parts = id.split(SEPARATOR);
        if parts.next()? != PREFIX {
            return None;
        }
        let owner = parts.next()?.parse().ok()?;
        let op = parts.next()?;
        if op.is_empty() || !op.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return None;
        }
        let page = Page::from_code(parts.next()?)?;
        let sel = match parts.next()? {
            "" => None,
            s => Some(s.parse().ok()?),
        };
        let part = parts.next()?.parse().ok()?;
        let control = Control::from_code(parts.next()?)?;
        if parts.next().is_some() {
            return None;
        }
        Some((
            Self {
                owner,
                op: op.to_string(),
                page,
                sel,
                part,
            },
            control,
        ))
    }

    /// The state after `control` was set to `value`. Changing page resets the selection and the
    /// part; changing the selection resets the part. `None` for a value no control offers.
    #[must_use]
    pub fn apply(&self, control: Control, value: &str) -> Option<Self> {
        let mut next = self.clone();
        match control {
            Control::Page => {
                next.page = Page::from_code(value)?;
                next.sel = None;
                next.part = 0;
            }
            Control::Sel => {
                next.sel = Some(value.parse().ok()?);
                next.part = 0;
            }
            Control::Part => next.part = value.parse().ok()?,
        }
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_id_round_trips() {
        let mut state = ViewState::new(123_456_789_012_345_678, "char_2023_ling", Page::Skills);
        for control in [Control::Page, Control::Sel, Control::Part] {
            let id = state.custom_id(control);
            assert_eq!(ViewState::parse(&id), Some((state.clone(), control)));
        }
        state.sel = Some(9);
        state.part = 2;
        let id = state.custom_id(Control::Sel);
        assert_eq!(id, "op:123456789012345678:char_2023_ling:sk:9:2:s");
        assert_eq!(ViewState::parse(&id), Some((state, Control::Sel)));
        for page in Page::ALL {
            assert_eq!(Page::from_code(page.code()), Some(page));
        }
    }

    #[test]
    fn custom_id_stays_under_discords_cap() {
        // The largest snowflake and an operator id twice as long as any of the 441 (16 chars).
        let state = ViewState {
            owner: u64::MAX,
            op: "char_9999_".to_string() + &"x".repeat(22),
            page: Page::Paradox,
            sel: Some(u16::MAX),
            part: u16::MAX,
        };
        let id = state.custom_id(Control::Part);
        assert!(id.len() <= CUSTOM_ID_MAX, "{} chars", id.len());
        assert!(id.is_ascii());
        assert_eq!(ViewState::parse(&id).map(|(s, _)| s), Some(state));
    }

    #[test]
    fn rejects_foreign_and_malformed_ids() {
        for id in [
            "",
            "warn:1:2",
            "op:1:char_x:ov::0",
            "op:1:char_x:zz::0:p",
            "op:x:char_x:ov::0:p",
            "op:1:char x:ov::0:p",
            "op:1:char_x:ov:-1:0:p",
            "op:1:char_x:ov::0:q",
            "op:1:char_x:ov::0:p:extra",
        ] {
            assert_eq!(ViewState::parse(id), None, "{id}");
        }
        assert!(ViewState::parse(&"op:1:".repeat(30)).is_none());
    }

    #[test]
    fn applying_a_value_resets_what_depends_on_it() {
        let state = ViewState {
            owner: 1,
            op: "char_x".to_string(),
            page: Page::Skills,
            sel: Some(3),
            part: 1,
        };
        let page = state.apply(Control::Page, "md").unwrap();
        assert_eq!((page.page, page.sel, page.part), (Page::Modules, None, 0));
        let sel = state.apply(Control::Sel, "7").unwrap();
        assert_eq!((sel.sel, sel.part), (Some(7), 0));
        assert_eq!(state.apply(Control::Part, "2").unwrap().part, 2);
        assert!(state.apply(Control::Page, "nope").is_none());
        assert!(state.apply(Control::Sel, "x").is_none());
    }
}
