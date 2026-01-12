//! Search filter application and validation
//!
//! Applies search filters from query parameters to the query builder.

use crate::app::error::ApiError;

use super::query_builder::SearchQueryBuilder;
use super::types::{RangeFilter, SearchQuery};

/// JSONB path constants for searchable user fields
pub mod paths {
    // User status fields
    pub const NICKNAME: &str = "data->'status'->>'nickName'";
    pub const LEVEL: &str = "(data->'status'->>'level')";
    pub const RESUME: &str = "data->'status'->>'resume'";
    pub const SECRETARY: &str = "data->'status'->>'secretary'";

    // Score fields
    pub const TOTAL_SCORE: &str = "(score->>'totalScore')";
    pub const OPERATOR_SCORE: &str = "(score->>'operatorScore')";
    pub const STAGE_SCORE: &str = "(score->>'stageScore')";
    pub const ROGUELIKE_SCORE: &str = "(score->>'roguelikeScore')";
    pub const SANDBOX_SCORE: &str = "(score->>'sandboxScore')";
    pub const MEDAL_SCORE: &str = "(score->>'medalScore')";
    pub const BASE_SCORE: &str = "(score->>'baseScore')";
    pub const COMPOSITE_SCORE: &str = "(score->'grade'->>'compositeScore')";
    pub const GRADE: &str = "score->'grade'->>'grade'";
}

/// Valid server values
const VALID_SERVERS: &[&str] = &["en", "jp", "cn", "kr", "tw"];

/// Valid grade values
const VALID_GRADES: &[&str] = &["S", "A", "B", "C", "D", "F"];

/// Apply all filters from SearchQuery to the query builder
///
/// Returns a list of applied filters for the response metadata.
pub fn apply_filters(
    builder: &mut SearchQueryBuilder,
    params: &SearchQuery,
) -> Result<Vec<String>, ApiError> {
    let mut filters_applied = Vec::new();

    // Text searches (fuzzy by default using ILIKE)
    if let Some(ref nickname) = params.nickname {
        validate_search_term(nickname, "nickname")?;
        builder.add_text_search(paths::NICKNAME, nickname);
        filters_applied.push(format!("nickname:{}", nickname));
    }

    if let Some(ref resume) = params.resume {
        validate_search_term(resume, "resume")?;
        builder.add_text_search(paths::RESUME, resume);
        filters_applied.push(format!("resume:{}", resume));
    }

    // Exact matches
    if let Some(ref uid) = params.uid {
        validate_uid(uid)?;
        builder.add_exact_match_column("uid", uid);
        filters_applied.push(format!("uid:{}", uid));
    }

    if let Some(ref server) = params.server {
        validate_server(server)?;
        builder.add_exact_match_column("server", server);
        filters_applied.push(format!("server:{}", server));
    }

    if let Some(ref grade) = params.grade {
        validate_grade(grade)?;
        builder.add_exact_match_jsonb(paths::GRADE, grade);
        filters_applied.push(format!("grade:{}", grade));
    }

    if let Some(ref secretary) = params.secretary {
        validate_search_term(secretary, "secretary")?;
        builder.add_exact_match_jsonb(paths::SECRETARY, secretary);
        filters_applied.push(format!("secretary:{}", secretary));
    }

    // Range filters
    if let Some(ref level) = params.level
        && let Some(range) = RangeFilter::parse(level)
    {
        builder.add_int_range(paths::LEVEL, &range);
        filters_applied.push(format!("level:{}", level));
    }

    if let Some(ref total_score) = params.total_score
        && let Some(range) = RangeFilter::parse(total_score)
    {
        builder.add_range(paths::TOTAL_SCORE, &range);
        filters_applied.push(format!("totalScore:{}", total_score));
    }

    if let Some(ref composite_score) = params.composite_score
        && let Some(range) = RangeFilter::parse(composite_score)
    {
        builder.add_range(paths::COMPOSITE_SCORE, &range);
        filters_applied.push(format!("compositeScore:{}", composite_score));
    }

    if let Some(ref operator_score) = params.operator_score
        && let Some(range) = RangeFilter::parse(operator_score)
    {
        builder.add_range(paths::OPERATOR_SCORE, &range);
        filters_applied.push(format!("operatorScore:{}", operator_score));
    }

    if let Some(ref stage_score) = params.stage_score
        && let Some(range) = RangeFilter::parse(stage_score)
    {
        builder.add_range(paths::STAGE_SCORE, &range);
        filters_applied.push(format!("stageScore:{}", stage_score));
    }

    if let Some(ref roguelike_score) = params.roguelike_score
        && let Some(range) = RangeFilter::parse(roguelike_score)
    {
        builder.add_range(paths::ROGUELIKE_SCORE, &range);
        filters_applied.push(format!("roguelikeScore:{}", roguelike_score));
    }

    if let Some(ref sandbox_score) = params.sandbox_score
        && let Some(range) = RangeFilter::parse(sandbox_score)
    {
        builder.add_range(paths::SANDBOX_SCORE, &range);
        filters_applied.push(format!("sandboxScore:{}", sandbox_score));
    }

    if let Some(ref medal_score) = params.medal_score
        && let Some(range) = RangeFilter::parse(medal_score)
    {
        builder.add_range(paths::MEDAL_SCORE, &range);
        filters_applied.push(format!("medalScore:{}", medal_score));
    }

    if let Some(ref base_score) = params.base_score
        && let Some(range) = RangeFilter::parse(base_score)
    {
        builder.add_range(paths::BASE_SCORE, &range);
        filters_applied.push(format!("baseScore:{}", base_score));
    }

    Ok(filters_applied)
}

/// Validate search term for security and sanity
fn validate_search_term(term: &str, field_name: &str) -> Result<(), ApiError> {
    if term.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "{} search term cannot be empty",
            field_name
        )));
    }
    if term.len() > 100 {
        return Err(ApiError::BadRequest(format!(
            "{} search term too long (max 100 chars)",
            field_name
        )));
    }
    Ok(())
}

/// Validate UID format
fn validate_uid(uid: &str) -> Result<(), ApiError> {
    if uid.is_empty() {
        return Err(ApiError::BadRequest("UID cannot be empty".into()));
    }
    if uid.len() > 30 {
        return Err(ApiError::BadRequest("UID too long".into()));
    }
    // Allow alphanumeric and # (for nick numbers)
    if !uid
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '#' || c == ':')
    {
        return Err(ApiError::BadRequest("Invalid UID format".into()));
    }
    Ok(())
}

/// Validate server value
fn validate_server(server: &str) -> Result<(), ApiError> {
    if !VALID_SERVERS.contains(&server) {
        return Err(ApiError::BadRequest(format!(
            "Invalid server. Must be one of: {:?}",
            VALID_SERVERS
        )));
    }
    Ok(())
}

/// Validate grade value
fn validate_grade(grade: &str) -> Result<(), ApiError> {
    if !VALID_GRADES.contains(&grade) {
        return Err(ApiError::BadRequest(format!(
            "Invalid grade. Must be one of: {:?}",
            VALID_GRADES
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_server_valid() {
        assert!(validate_server("en").is_ok());
        assert!(validate_server("jp").is_ok());
        assert!(validate_server("cn").is_ok());
    }

    #[test]
    fn test_validate_server_invalid() {
        assert!(validate_server("invalid").is_err());
        assert!(validate_server("").is_err());
    }

    #[test]
    fn test_validate_grade_valid() {
        assert!(validate_grade("S").is_ok());
        assert!(validate_grade("A").is_ok());
        assert!(validate_grade("F").is_ok());
    }

    #[test]
    fn test_validate_grade_invalid() {
        assert!(validate_grade("X").is_err());
        assert!(validate_grade("s").is_err()); // case sensitive
    }

    #[test]
    fn test_validate_uid_valid() {
        assert!(validate_uid("12345678").is_ok());
        assert!(validate_uid("user#1234").is_ok());
    }

    #[test]
    fn test_validate_uid_invalid() {
        assert!(validate_uid("").is_err());
        let long_uid = "a".repeat(50);
        assert!(validate_uid(&long_uid).is_err());
    }

    #[test]
    fn test_validate_search_term() {
        assert!(validate_search_term("myrtle", "nickname").is_ok());
        assert!(validate_search_term("", "nickname").is_err());
        let long_term = "a".repeat(150);
        assert!(validate_search_term(&long_term, "nickname").is_err());
    }
}
