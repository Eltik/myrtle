//! Rungs (contract section 2): where an answer comes from, the profile a
//! rung runs, admission per rung, rung selection and step-down at claim
//! time, and the Cloudflare Neuron budget.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use super::{Mode, TrevorConfig};

/// Where an answer comes from. A job's rung is never `cache`; a response's is
/// `cache` when the answer was already stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorRung")]
#[schema(as = TrevorRung)]
pub enum Rung {
    Cache,
    Cf,
    Cpu,
    Passages,
}

impl Rung {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cache => "cache",
            Self::Cf => "cf",
            Self::Cpu => "cpu",
            Self::Passages => "passages",
        }
    }

    /// A job rung as stored; anything unknown reads as the model-free rung.
    pub fn parse_job(s: &str) -> Self {
        match s {
            "cf" => Self::Cf,
            "cpu" => Self::Cpu,
            _ => Self::Passages,
        }
    }

    /// The pipeline profile of a job on this rung under contract v1: `cf`
    /// runs the full pipeline, `cpu` the slim one; `passages` ignores it
    /// (stored `full`).
    pub const fn profile(self) -> Profile {
        match self {
            Self::Cpu => Profile::Slim,
            _ => Profile::Full,
        }
    }

    /// The profile under the configured CPU profile (contract v1.1 section
    /// 4: `cpu` runs `mid` by default; `TREVOR_CPU_PROFILE=slim` restores
    /// [`Rung::profile`]).
    pub const fn profile_with(self, cpu: Profile) -> Profile {
        match self {
            Self::Cpu => cpu,
            _ => Profile::Full,
        }
    }

    /// The ETA while fewer than [`ETA_MIN_SAMPLES`] jobs finished on the rung.
    pub const fn fallback_eta_secs(self) -> i64 {
        match self {
            Self::Cf => 15,
            Self::Cpu => 120,
            Self::Cache | Self::Passages => 1,
        }
    }
}

/// The worker pipeline profile for a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorProfile")]
#[schema(as = TrevorProfile)]
pub enum Profile {
    Full,
    Slim,
    /// The E2B eval winner (contract v1.1 section 4): keyword router, k 8, a
    /// 3,600-token passage budget, 250 output tokens, one model call.
    Mid,
}

impl Profile {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Slim => "slim",
            Self::Mid => "mid",
        }
    }
}

/// The model rungs a worker reports live in its heartbeat.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(default)]
#[ts(export, rename = "TrevorRungFlags")]
#[schema(as = TrevorRungFlags)]
pub struct RungFlags {
    pub cf: bool,
    pub cpu: bool,
}

/// What the rung choice reads (contract section 2).
#[derive(Debug, Clone, Copy, Default)]
pub struct RungInputs {
    /// The rungs of the most recent heartbeat, online or not.
    pub rungs: RungFlags,
    /// Neurons spent today (UTC) by `cf` answers.
    pub neurons_today: i64,
    /// Jobs queued or running on the `cpu` rung (normal jobs only once the
    /// deferred queue exists).
    pub cpu_depth: i64,
}

/// What admission control reads besides [`RungInputs`] (contract v1.1
/// section 1b).
#[derive(Debug, Clone, Copy, Default)]
pub struct AdmissionInputs {
    pub rung: RungInputs,
    /// Normal jobs queued or running on the `cf` rung.
    pub cf_depth: i64,
    /// Online workers whose heartbeat offers `cpu`.
    pub cpu_workers: i64,
    /// Jobs open (queued or running, normal or deferred) on every server.
    pub open_total: i64,
}

/// Admission control's verdict for a new question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    /// A model rung (or, with no model rung offered at all, the passages
    /// rung) takes it now.
    Now(Rung),
    /// Every offered model rung is full: a quick passages job now, and the
    /// full answer as a deferred job on `deferred` when the global cap
    /// allows (`None` when it does not).
    Overflow { deferred: Option<Rung> },
}

/// The CPU rung's cap: `TREVOR_CPU_QUEUE_MAX` per online cpu worker. A
/// heartbeat that offers `cpu` while no cpu worker is online (a restart)
/// counts as one, so a single worker keeps the v1 cap exactly.
pub fn cpu_cap(cfg: &TrevorConfig, cpu_workers: i64) -> i64 {
    cfg.cpu_queue_max.saturating_mul(cpu_workers.max(1))
}

/// Admission control (contract v1.1 section 1b): `cf` while the heartbeat
/// offers it, the day's ledger is under budget and fewer than
/// `TREVOR_CF_QUEUE_MAX` normal jobs are on it (`0`: no cap); else `cpu`
/// while offered and under [`cpu_cap`]; else, when a model rung is offered
/// but full, an overflow (deferred on the rung that will serve it, unless
/// `TREVOR_GLOBAL_QUEUE_MAX` jobs are already open); else `passages`.
pub fn admit_rung(inputs: &AdmissionInputs, cfg: &TrevorConfig) -> Admission {
    let r = &inputs.rung;
    let cf_usable = r.rungs.cf && r.neurons_today < cf_serving_budget(cfg);
    let cf_room = cfg.cf_queue_max <= 0 || inputs.cf_depth < cfg.cf_queue_max;
    if cf_usable && cf_room {
        return Admission::Now(Rung::Cf);
    }
    if r.rungs.cpu && r.cpu_depth < cpu_cap(cfg, inputs.cpu_workers) {
        return Admission::Now(Rung::Cpu);
    }
    let full = if r.rungs.cpu {
        Some(Rung::Cpu)
    } else if cf_usable {
        Some(Rung::Cf)
    } else {
        None
    };
    match full {
        None => Admission::Now(Rung::Passages),
        Some(rung) => Admission::Overflow {
            deferred: (inputs.open_total < cfg.global_queue_max).then_some(rung),
        },
    }
}

/// Neurons `cf` answers may spend today before the summary reserve.
pub fn cf_serving_budget(cfg: &TrevorConfig) -> i64 {
    (cfg.cf_daily_neurons - cfg.cf_summary_reserve).max(0)
}

/// The rung a new job gets: `cf` while the heartbeat reports it and the
/// day's ledger is under the serving budget, else `cpu` while the heartbeat
/// reports it and its queue is short, else `passages`.
pub fn select_rung(inputs: &RungInputs, cfg: &TrevorConfig) -> Rung {
    if inputs.rungs.cf && inputs.neurons_today < cf_serving_budget(cfg) {
        Rung::Cf
    } else if inputs.rungs.cpu && inputs.cpu_depth < cfg.cpu_queue_max {
        Rung::Cpu
    } else {
        Rung::Passages
    }
}

/// The rung a claimed job runs on: the one it was queued with, stepped down
/// when the claiming worker no longer offers it or the Neuron budget ran out
/// while it waited. Never stepped up: a job queued as `passages` because the
/// CPU queue was full stays the quick answer its asker was promised.
pub fn settle_rung(
    assigned: Rung,
    worker: RungFlags,
    neurons_today: i64,
    cfg: &TrevorConfig,
) -> Rung {
    // TREVOR_MODE=passages (or off): no model runs, whatever was queued.
    if cfg.mode != Mode::On {
        return Rung::Passages;
    }
    match assigned {
        Rung::Cf if worker.cf && neurons_today < cf_serving_budget(cfg) => Rung::Cf,
        Rung::Cf | Rung::Cpu if worker.cpu => Rung::Cpu,
        _ => Rung::Passages,
    }
}

/// Cloudflare Neurons of one `cf` answer:
/// `ceil(tokensIn * 9091 / 1e6 + tokensOut * 27273 / 1e6)`.
pub fn neurons(tokens_in: u64, tokens_out: u64) -> i64 {
    let micro = u128::from(tokens_in) * 9091 + u128::from(tokens_out) * 27273;
    i64::try_from(micro.div_ceil(1_000_000)).unwrap_or(i64::MAX)
}

/// The day's `cf` budget as a bucket, never a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorCfBudget")]
#[schema(as = TrevorCfBudget)]
pub enum CfBudget {
    Ok,
    Low,
    Spent,
}

/// `low` under 20% of the serving budget left, `spent` at none.
pub fn cf_budget(neurons_today: i64, cfg: &TrevorConfig) -> CfBudget {
    let cap = cf_serving_budget(cfg);
    let left = cap - neurons_today;
    if left <= 0 {
        CfBudget::Spent
    } else if left * 5 < cap {
        CfBudget::Low
    } else {
        CfBudget::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> TrevorConfig {
        TrevorConfig::default()
    }

    #[test]
    fn admission_caps_per_rung_and_overflows_to_deferred() {
        let c = cfg();
        let both = RungFlags {
            cf: true,
            cpu: true,
        };
        let cpu_only = RungFlags {
            cf: false,
            cpu: true,
        };
        let adm = |rungs, neurons_today, cpu_depth, cf_depth, cpu_workers, open_total| {
            admit_rung(
                &AdmissionInputs {
                    rung: RungInputs {
                        rungs,
                        neurons_today,
                        cpu_depth,
                    },
                    cf_depth,
                    cpu_workers,
                    open_total,
                },
                &c,
            )
        };
        assert_eq!(adm(both, 0, 0, 0, 1, 0), Admission::Now(Rung::Cf));
        // TREVOR_CF_QUEUE_MAX=4: the fifth goes to cpu.
        assert_eq!(adm(both, 0, 0, 3, 1, 0), Admission::Now(Rung::Cf));
        assert_eq!(adm(both, 0, 0, 4, 1, 0), Admission::Now(Rung::Cpu));
        // 6 per online cpu worker.
        assert_eq!(adm(cpu_only, 0, 5, 0, 1, 0), Admission::Now(Rung::Cpu));
        assert_eq!(
            adm(cpu_only, 0, 6, 0, 1, 10),
            Admission::Overflow {
                deferred: Some(Rung::Cpu)
            }
        );
        assert_eq!(adm(cpu_only, 0, 6, 0, 2, 10), Admission::Now(Rung::Cpu));
        assert_eq!(adm(cpu_only, 0, 11, 0, 2, 10), Admission::Now(Rung::Cpu));
        // A restart (no cpu worker online) keeps the one-worker cap.
        assert_eq!(adm(cpu_only, 0, 5, 0, 0, 0), Admission::Now(Rung::Cpu));
        // The global cap drops the deferred job, never the passages answer.
        assert_eq!(
            adm(cpu_only, 0, 6, 0, 1, 200),
            Admission::Overflow { deferred: None }
        );
        // cf only and full: deferred on cf.
        let cf_only = RungFlags {
            cf: true,
            cpu: false,
        };
        assert_eq!(
            adm(cf_only, 0, 0, 4, 0, 0),
            Admission::Overflow {
                deferred: Some(Rung::Cf)
            }
        );
        // cf spent and no cpu: plain passages, nothing to defer to.
        assert_eq!(
            adm(cf_only, 8000, 0, 0, 0, 0),
            Admission::Now(Rung::Passages)
        );
        assert_eq!(
            adm(RungFlags::default(), 0, 0, 0, 0, 0),
            Admission::Now(Rung::Passages)
        );
        // TREVOR_CF_QUEUE_MAX=0 lifts the cf cap.
        let nocap = TrevorConfig {
            cf_queue_max: 0,
            ..cfg()
        };
        let inputs = AdmissionInputs {
            rung: RungInputs {
                rungs: both,
                neurons_today: 0,
                cpu_depth: 0,
            },
            cf_depth: 1000,
            cpu_workers: 1,
            open_total: 0,
        };
        assert_eq!(admit_rung(&inputs, &nocap), Admission::Now(Rung::Cf));
    }

    #[test]
    fn mode_parses_and_steps_every_claim_down_unless_on() {
        assert_eq!(Mode::parse("on"), Some(Mode::On));
        assert_eq!(Mode::parse(" Passages "), Some(Mode::Passages));
        assert_eq!(Mode::parse("off"), Some(Mode::Off));
        assert_eq!(Mode::parse("maybe"), None);
        let both = RungFlags {
            cf: true,
            cpu: true,
        };
        let p = TrevorConfig {
            mode: Mode::Passages,
            ..cfg()
        };
        assert_eq!(settle_rung(Rung::Cf, both, 0, &p), Rung::Passages);
        assert_eq!(settle_rung(Rung::Cpu, both, 0, &p), Rung::Passages);
        assert_eq!(settle_rung(Rung::Cf, both, 0, &cfg()), Rung::Cf);
    }

    #[test]
    fn the_cpu_profile_is_mid_unless_switched_to_slim() {
        assert_eq!(Rung::Cpu.profile_with(Profile::Mid), Profile::Mid);
        assert_eq!(Rung::Cpu.profile_with(Profile::Slim), Rung::Cpu.profile());
        assert_eq!(Rung::Cf.profile_with(Profile::Mid), Profile::Full);
        assert_eq!(cfg().cpu_profile, Profile::Mid);
    }

    #[test]
    fn rung_selection_follows_heartbeat_budget_and_cpu_depth() {
        let c = cfg();
        let both = RungFlags {
            cf: true,
            cpu: true,
        };
        let pick = |rungs, neurons_today, cpu_depth| {
            select_rung(
                &RungInputs {
                    rungs,
                    neurons_today,
                    cpu_depth,
                },
                &c,
            )
        };
        assert_eq!(pick(both, 0, 0), Rung::Cf);
        // 9000 - 1000 = 8000 serving Neurons: at 7999 spent cf still runs.
        assert_eq!(pick(both, 7999, 0), Rung::Cf);
        assert_eq!(pick(both, 8000, 0), Rung::Cpu);
        assert_eq!(pick(both, 8000, 5), Rung::Cpu);
        assert_eq!(pick(both, 8000, 6), Rung::Passages);
        assert_eq!(
            pick(
                RungFlags {
                    cf: false,
                    cpu: true
                },
                0,
                0
            ),
            Rung::Cpu
        );
        assert_eq!(pick(RungFlags::default(), 0, 0), Rung::Passages);
        // TREVOR_CF_DAILY_NEURONS=0 turns the cf rung off.
        let off = TrevorConfig {
            cf_daily_neurons: 0,
            ..cfg()
        };
        let inputs = RungInputs {
            rungs: both,
            neurons_today: 0,
            cpu_depth: 0,
        };
        assert_eq!(select_rung(&inputs, &off), Rung::Cpu);
        assert_eq!(cf_budget(0, &off), CfBudget::Spent);
    }

    #[test]
    fn a_claimed_job_steps_down_never_up() {
        let c = cfg();
        let both = RungFlags {
            cf: true,
            cpu: true,
        };
        let cpu_only = RungFlags {
            cf: false,
            cpu: true,
        };
        let cf_only = RungFlags {
            cf: true,
            cpu: false,
        };
        assert_eq!(settle_rung(Rung::Cf, both, 0, &c), Rung::Cf);
        assert_eq!(settle_rung(Rung::Cf, both, 8000, &c), Rung::Cpu);
        assert_eq!(settle_rung(Rung::Cf, cpu_only, 0, &c), Rung::Cpu);
        assert_eq!(
            settle_rung(Rung::Cf, RungFlags::default(), 0, &c),
            Rung::Passages
        );
        assert_eq!(settle_rung(Rung::Cpu, both, 0, &c), Rung::Cpu);
        assert_eq!(settle_rung(Rung::Cpu, cf_only, 0, &c), Rung::Passages);
        assert_eq!(settle_rung(Rung::Passages, both, 0, &c), Rung::Passages);
        assert_eq!(Rung::Cf.profile(), Profile::Full);
        assert_eq!(Rung::Cpu.profile(), Profile::Slim);
    }

    #[test]
    fn neurons_round_up_per_the_contract_formula() {
        assert_eq!(neurons(0, 0), 0);
        assert_eq!(neurons(1, 0), 1);
        // 2000 in, 250 out: 18.182 + 6.81825 = 25.00025 -> 26.
        assert_eq!(neurons(2000, 250), 26);
        // 9000 in, 400 out: 81.819 + 10.9092 = 92.7282 -> 93.
        assert_eq!(neurons(9000, 400), 93);
    }

    #[test]
    fn cf_budget_is_a_bucket() {
        let c = cfg();
        assert_eq!(cf_budget(0, &c), CfBudget::Ok);
        // 20% of 8000 is 1600: 1600 left is still ok, 1599 is low.
        assert_eq!(cf_budget(6400, &c), CfBudget::Ok);
        assert_eq!(cf_budget(6401, &c), CfBudget::Low);
        assert_eq!(cf_budget(8000, &c), CfBudget::Spent);
        assert_eq!(cf_budget(9500, &c), CfBudget::Spent);
    }
}
