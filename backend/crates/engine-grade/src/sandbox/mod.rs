//! RA (sandbox) grade, from main's `core::grade::sandbox`.

pub mod score;
pub mod types;

pub use score::{
    MAX_TEMPLATE_SHAPE_LOGS, admit_new_shapes, grade_progress, grade_sandbox, grade_sandbox_detail,
    unread_template_shapes,
};
pub use types::{SandboxGrade, SandboxGradeDetail};
