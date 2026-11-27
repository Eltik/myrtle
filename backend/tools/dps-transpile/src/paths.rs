use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static ROOT: LazyLock<PathBuf> = LazyLock::new(|| {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    root.canonicalize().unwrap_or(root)
});

pub fn workspace_root() -> &'static Path {
    &ROOT
}

pub fn default_repo() -> PathBuf {
    workspace_root().join("external/ArknightsDpsCompare")
}

pub fn hand_dir() -> PathBuf {
    workspace_root().join("crates/dps-formulas/src/custom/hand")
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Damage,
    Healing,
}

impl Mode {
    pub const fn py_source_rel(&self) -> &'static str {
        match self {
            Self::Damage => "damagecalc/damage_formulas.py",
            Self::Healing => "damagecalc/healing_formulas.py",
        }
    }

    /// Healers may extend `Operator` or `Healer`
    pub const fn class_base_alt(&self) -> &'static str {
        match self {
            Self::Damage => "Operator",
            Self::Healing => "(?:Operator|Healer)",
        }
    }

    /// Methods to try
    pub const fn method_names(&self) -> &'static [&'static str] {
        match self {
            Self::Damage => &["skill_dps"],
            Self::Healing => &["skill_hps", "skill_dps"],
        }
    }

    pub fn formulas_json_path(&self) -> PathBuf {
        workspace_root().join(match self {
            Self::Damage => "crates/dps-formulas/data/operator_formulas.json",
            Self::Healing => "crates/dps-formulas/data/heal_formulas.json",
        })
    }

    pub fn generated_rs_path(&self) -> PathBuf {
        workspace_root().join(match self {
            Self::Damage => "crates/dps-formulas/src/custom/generated.rs",
            Self::Healing => "crates/dps-formulas/src/custom/generated_hps.rs",
        })
    }

    pub fn dispatch_rs_path(&self) -> PathBuf {
        workspace_root().join(match self {
            Self::Damage => "crates/dps-formulas/src/custom/dispatch.rs",
            Self::Healing => "crates/dps-formulas/src/custom/dispatch_hps.rs",
        })
    }

    pub const fn generated_mod_name(&self) -> &'static str {
        match self {
            Self::Damage => "generated",
            Self::Healing => "generated_hps",
        }
    }

    pub fn expected_json_path(&self) -> PathBuf {
        workspace_root().join(match self {
            Self::Damage => "tools/parity/tests/fixtures/expected_dps.json",
            Self::Healing => "tools/parity/tests/fixtures/expected_hps.json",
        })
    }

    pub const fn output_shape(&self) -> OutputShape {
        match self {
            Self::Damage => OutputShape::F64Dps,
            Self::Healing => OutputShape::HpsTriple,
        }
    }

    pub const fn human_label(&self) -> &'static str {
        match self {
            Self::Damage => "DPS",
            Self::Healing => "HPS",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OutputShape {
    /// `Option<f64>`
    F64Dps,
    /// `Option<HpsResult>` with `skill_hps`, `base_hps`, `avg_hps`
    HpsTriple,
}

pub const HEAL_FALLBACKS: &[&str] = &["AmiyaMedic", "ReedAlter"];
