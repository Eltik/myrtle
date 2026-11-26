use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::Serialize;

use crate::compare::{Comparison, Diff};
use crate::corpus::CorpusEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Equal,
    Different(Comparison),
    Error(String),
    RateLimited {
        a: u16,
        b: u16,
    },
    Skipped(String),
}

impl Outcome {
    const fn label(&self) -> &'static str {
        match self {
            Self::Equal => "equal",
            Self::Different(_) => "different",
            Self::Error(_) => "error",
            Self::RateLimited { .. } => "rate_limited",
            Self::Skipped(_) => "skipped",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RouteStats {
    pub total: usize,
    pub equal: usize,
    pub different: usize,
    pub errors: usize,
    pub rate_limited: usize,
    pub skipped: usize,
}

impl RouteStats {
    const fn problems(&self) -> usize {
        self.different + self.errors + self.rate_limited
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Problem {
    pub id: String,
    pub route: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub outcome: &'static str,
    pub total_diffs: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diffs: Vec<Diff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Summary {
    pub a: String,
    pub b: String,
    pub rel_tol: f64,
    pub total: usize,
    pub equal: usize,
    pub different: usize,
    pub errors: usize,
    pub rate_limited: usize,
    pub skipped: usize,
    pub routes: BTreeMap<String, RouteStats>,
    pub problems: Vec<Problem>,
    pub problems_not_listed: usize,
    pub skip_reasons: BTreeMap<String, usize>,
}

impl Summary {
    pub fn new(a: &str, b: &str, rel_tol: f64) -> Self {
        Self {
            a: a.to_owned(),
            b: b.to_owned(),
            rel_tol,
            total: 0,
            equal: 0,
            different: 0,
            errors: 0,
            rate_limited: 0,
            skipped: 0,
            routes: BTreeMap::new(),
            problems: Vec::new(),
            problems_not_listed: 0,
            skip_reasons: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, entry: &CorpusEntry, outcome: Outcome, max_problems: usize) {
        self.total += 1;
        let stats = self.routes.entry(entry.route.clone()).or_default();
        stats.total += 1;
        let (total_diffs, diffs, message) = match &outcome {
            Outcome::Equal => {
                self.equal += 1;
                stats.equal += 1;
                return;
            }
            Outcome::Skipped(reason) => {
                self.skipped += 1;
                stats.skipped += 1;
                *self.skip_reasons.entry(reason.clone()).or_default() += 1;
                return;
            }
            Outcome::Different(c) => {
                self.different += 1;
                stats.different += 1;
                (c.total, c.diffs.clone(), None)
            }
            Outcome::Error(m) => {
                self.errors += 1;
                stats.errors += 1;
                (0, Vec::new(), Some(m.clone()))
            }
            Outcome::RateLimited { a, b } => {
                self.rate_limited += 1;
                stats.rate_limited += 1;
                (0, Vec::new(), Some(format!("HTTP 429 (A={a}, B={b})")))
            }
        };
        if self.problems.len() >= max_problems {
            self.problems_not_listed += 1;
            return;
        }
        self.problems.push(Problem {
            id: entry.id.clone(),
            route: entry.route.clone(),
            path: entry.path.clone(),
            role: entry.role.clone(),
            outcome: outcome.label(),
            total_diffs,
            diffs,
            message,
        });
    }

    pub const fn passed(&self) -> bool {
        self.different == 0 && self.errors == 0 && self.rate_limited == 0
    }

    pub fn render_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(
            s,
            "parity  A={}  B={}  (floats at {:e} relative)",
            self.a, self.b, self.rel_tol
        );
        let _ = writeln!(
            s,
            "requests {}  equal {}  different {}  errors {}  rate-limited {}  skipped {}",
            self.total, self.equal, self.different, self.errors, self.rate_limited, self.skipped
        );
        let mut bad: Vec<(&String, &RouteStats)> = self
            .routes
            .iter()
            .filter(|(_, r)| r.problems() > 0)
            .collect();
        bad.sort_by(|(na, a), (nb, b)| b.problems().cmp(&a.problems()).then(na.cmp(nb)));
        if !bad.is_empty() {
            let width = bad.iter().map(|(n, _)| n.len()).max().unwrap_or(5).max(5);
            let _ = writeln!(
                s,
                "\nroutes with problems ({} of {}):",
                bad.len(),
                self.routes.len()
            );
            let _ = writeln!(
                s,
                "  {:<width$}  {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
                "route", "total", "equal", "diff", "error", "429", "skip"
            );
            for (name, r) in &bad {
                let _ = writeln!(
                    s,
                    "  {:<width$}  {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
                    name, r.total, r.equal, r.different, r.errors, r.rate_limited, r.skipped
                );
            }
        }
        let clean = self.routes.values().filter(|r| r.problems() == 0).count();
        let _ = writeln!(s, "\n{clean} routes with no problems");
        if !self.skip_reasons.is_empty() {
            let _ = writeln!(s, "\nskipped:");
            for (reason, n) in &self.skip_reasons {
                let _ = writeln!(s, "  {n:>6}  {reason}");
            }
        }
        if !self.problems.is_empty() {
            let _ = writeln!(s, "\nfirst {} problems:", self.problems.len());
            for (i, p) in self.problems.iter().enumerate() {
                let role = p
                    .role
                    .as_deref()
                    .map(|r| format!(" as {r}"))
                    .unwrap_or_default();
                let _ = write!(s, "[{}] GET {}{role}  ({})", i + 1, p.path, p.route);
                match &p.message {
                    Some(m) => {
                        let _ = writeln!(s, "  {}: {m}", p.outcome);
                    }
                    None => {
                        let _ = writeln!(s, "  {} diffs", p.total_diffs);
                    }
                }
                for d in &p.diffs {
                    let _ = writeln!(s, "    {d}");
                }
                if p.total_diffs > p.diffs.len() {
                    let _ = writeln!(s, "    ... {} more", p.total_diffs - p.diffs.len());
                }
            }
            if self.problems_not_listed > 0 {
                let _ = writeln!(
                    s,
                    "... {} more problems not listed (see --max-problems)",
                    self.problems_not_listed
                );
            }
        }
        let _ = writeln!(
            s,
            "\nresult: {}",
            if self.passed() { "PASS" } else { "FAIL" }
        );
        s
    }
}
