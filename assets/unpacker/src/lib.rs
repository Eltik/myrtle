// Clippy: pedantic/nursery run for signal, but this is an internal asset-pipeline
// tool (Unity-bundle / FSB5 / texture / FlatBuffers parsing + codegen). The groups
// below are intentional noise here: pervasive byte/bit casts, long parser/codegen
// functions, single-char math vars, indicatif template strings, and internal-only
// docs. Generated FB modules are exempted separately at their `mod` declarations.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_long_first_doc_paragraph,
    clippy::implicit_hasher,
    clippy::option_if_let_else,
    clippy::manual_let_else,
    clippy::match_same_arms,
    clippy::items_after_statements,
    clippy::needless_pass_by_value,
    clippy::branches_sharing_code,
    clippy::or_fun_call,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::struct_excessive_bools,
    clippy::fn_params_excessive_bools,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::format_push_string,
    clippy::literal_string_with_formatting_args
)]

pub mod export;
pub mod unity;

#[macro_use]
pub mod fb_json_macros;
// Machine-generated FlatBuffers code - exempt from clippy (regenerated, not hand-edited).
#[allow(clippy::all, clippy::pedantic, clippy::nursery)]
pub mod fb_json_auto;
#[allow(clippy::all, clippy::pedantic, clippy::nursery)]
pub mod fb_json_auto_yostar;
pub mod flatbuffers_decode;
#[allow(clippy::all, clippy::pedantic, clippy::nursery)]
pub mod generated_fbs;
#[allow(clippy::all, clippy::pedantic, clippy::nursery)]
pub mod generated_fbs_yostar;
