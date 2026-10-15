// Internal app crate, not a published library: `pedantic`/`nursery` run for
// signal, the groups below are allowed on purpose.
#![allow(
    // Library-API doc/ergonomics lints: nothing outside this crate calls it.
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_long_first_doc_paragraph,
    clippy::doc_link_with_quotes,
    clippy::must_use_candidate,
    clippy::implicit_hasher,
    clippy::ref_option,
    // The DPS/grading math casts on purpose everywhere; `try_from` on each
    // would be noise.
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    // Style/nursery lints whose rewrites read worse here, misfire on derive
    // macros, or undo a deliberate explicit form.
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
    // `mul_add`/`ln_1p` are more accurate but change numerical results, and the
    // DPS/HPS math must match the reference implementation.
    clippy::suboptimal_flops,
    clippy::imprecise_flops
)]
// All `unsafe` is the CSPRNG FFI in `utils::random`; each block needs a `// SAFETY:`.
#![warn(clippy::undocumented_unsafe_blocks)]

pub mod app;
pub mod core;
pub mod database;
pub mod db_export;
pub mod dps;
pub mod user_export;
pub mod utils;
