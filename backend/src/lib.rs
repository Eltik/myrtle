// ── Clippy: pragmatic lint policy for an application crate ──────────────────
// `backend` is an internal application crate (it backs our own binaries), not a
// published library, so we run `pedantic`/`nursery` for signal but allow the
// categories below that don't earn their keep here. Genuinely actionable findings
// have been fixed; these remaining groups are intentional.
#![allow(
    // Library-API doc/ergonomics lints — not meaningful for an internal app crate
    // with no external consumers.
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_long_first_doc_paragraph,
    clippy::doc_link_with_quotes,
    clippy::must_use_candidate,
    clippy::implicit_hasher,
    clippy::ref_option,
    // Intentional numeric casts pervasive in the DPS/grading math. Replacing every
    // `as` with `try_from` + error handling would add noise without value.
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    // Style/nursery lints whose rewrites don't improve this code (less readable,
    // derive-macro false positives, or intentional explicit forms).
    clippy::option_if_let_else,
    clippy::match_same_arms,
    clippy::assigning_clones,
    clippy::bool_to_int_with_if,
    clippy::trait_duplication_in_bounds,
    clippy::items_after_statements,
    clippy::useless_let_if_seq,
    clippy::needless_pass_by_value,
    clippy::future_not_send,
    clippy::unchecked_time_subtraction,
    clippy::manual_midpoint,
    clippy::similar_names,
    clippy::struct_field_names,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    // Floating-point lints: `mul_add`/`ln_1p` are more accurate but change numerical
    // results, and the DPS/HPS math must match the reference implementation. We keep
    // the explicit `a * b + c` form and silence the lint rather than alter outputs.
    clippy::suboptimal_flops,
    clippy::imprecise_flops
)]

pub mod app;
pub mod core;
pub mod database;
pub mod db_export;
pub mod dps;
pub mod utils;
