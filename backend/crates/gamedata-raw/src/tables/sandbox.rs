use serde_json::Value;

#[derive(Debug, Default)]
pub struct SandboxTables {
    /// `sandbox_perm_table`, untyped
    pub perm: Value,
}
