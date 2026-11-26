use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::compare::{CompareOptions, DEFAULT_REL_TOL};
use crate::glob::PointerGlob;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteRulesFile {
    #[serde(default)]
    pub unordered: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub skip: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulesFile {
    pub rel_tol: Option<f64>,
    pub abs_tol: Option<f64>,
    #[serde(default)]
    pub unordered: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub routes: BTreeMap<String, RouteRulesFile>,
}

impl RulesFile {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading rules {}", path.display()))?;
        serde_json::from_str(&text).with_context(|| format!("parsing rules {}", path.display()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuleOverrides {
    pub rel_tol: Option<f64>,
    pub abs_tol: Option<f64>,
    pub unordered: Vec<String>,
    pub ignore: Vec<String>,
    pub max_diffs: usize,
}

#[derive(Debug, Clone)]
pub struct Rules {
    base: CompareOptions,
    routes: HashMap<String, CompareOptions>,
    skip: HashMap<String, bool>,
}

fn globs(list: &[String]) -> Result<Vec<PointerGlob>> {
    list.iter()
        .map(|g| PointerGlob::parse(g).map_err(anyhow::Error::msg))
        .collect()
}

impl Rules {
    pub fn build(file: Option<RulesFile>, cli: &RuleOverrides) -> Result<Self> {
        let file = file.unwrap_or_default();
        let rel_tol = cli.rel_tol.or(file.rel_tol).unwrap_or(DEFAULT_REL_TOL);
        let abs_tol = cli.abs_tol.or(file.abs_tol).unwrap_or(0.0);
        if !(rel_tol >= 0.0 && rel_tol.is_finite() && abs_tol >= 0.0 && abs_tol.is_finite()) {
            bail!("tolerances must be finite and >= 0 (rel {rel_tol}, abs {abs_tol})");
        }
        let mut unordered = globs(&file.unordered)?;
        unordered.extend(globs(&cli.unordered)?);
        let mut ignore = globs(&file.ignore)?;
        ignore.extend(globs(&cli.ignore)?);
        let base = CompareOptions {
            rel_tol,
            abs_tol,
            unordered,
            ignore,
            max_diffs: cli.max_diffs.max(1),
        };
        let mut routes = HashMap::new();
        let mut skip = HashMap::new();
        for (route, r) in file.routes {
            let mut o = base.clone();
            o.unordered.extend(globs(&r.unordered)?);
            o.ignore.extend(globs(&r.ignore)?);
            routes.insert(route.clone(), o);
            skip.insert(route, r.skip);
        }
        Ok(Self { base, routes, skip })
    }

    pub fn for_route(&self, route: &str) -> &CompareOptions {
        self.routes.get(route).unwrap_or(&self.base)
    }

    pub fn skipped(&self, route: &str) -> bool {
        self.skip.get(route).copied().unwrap_or(false)
    }

    pub const fn rel_tol(&self) -> f64 {
        self.base.rel_tol
    }
}
