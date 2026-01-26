//! Safe dynamic SQL query builder for search
//!
//! Builds parameterized SQL queries safely without SQL injection risks.

use super::types::{QueryLogic, RangeFilter};

/// Represents a bound parameter value for SQL queries
#[derive(Debug, Clone)]
pub enum QueryParam {
    String(String),
    Int(i64),
    Float(f64),
}

/// Represents a single filter condition with its SQL and parameters
#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub sql: String,
    pub params: Vec<QueryParam>,
}

/// Query builder for safe dynamic SQL generation
pub struct SearchQueryBuilder {
    conditions: Vec<FilterCondition>,
    logic: QueryLogic,
    sort_expression: String,
    sort_order: String,
    limit: i64,
    offset: i64,
    param_counter: usize,
}

impl SearchQueryBuilder {
    pub fn new(logic: QueryLogic) -> Self {
        Self {
            conditions: Vec::new(),
            logic,
            sort_expression: "COALESCE((score->>'totalScore')::FLOAT, 0)".to_string(),
            sort_order: "DESC".to_string(),
            limit: 25,
            offset: 0,
            param_counter: 0,
        }
    }

    /// Get the next parameter placeholder (e.g., $1, $2, etc.)
    fn next_param(&mut self) -> String {
        self.param_counter += 1;
        format!("${}", self.param_counter)
    }

    /// Add text search with ILIKE (fuzzy substring match)
    pub fn add_text_search(&mut self, json_path: &str, value: &str) {
        let param = self.next_param();
        self.conditions.push(FilterCondition {
            sql: format!("{} ILIKE {}", json_path, param),
            params: vec![QueryParam::String(format!("%{}%", value))],
        });
    }

    /// Add exact match condition for a column
    pub fn add_exact_match_column(&mut self, column: &str, value: &str) {
        let param = self.next_param();
        self.conditions.push(FilterCondition {
            sql: format!("{} = {}", column, param),
            params: vec![QueryParam::String(value.to_string())],
        });
    }

    /// Add exact match condition for a JSONB path
    pub fn add_exact_match_jsonb(&mut self, json_path: &str, value: &str) {
        let param = self.next_param();
        self.conditions.push(FilterCondition {
            sql: format!("{} = {}", json_path, param),
            params: vec![QueryParam::String(value.to_string())],
        });
    }

    /// Add range condition for numeric JSONB fields
    pub fn add_range(&mut self, json_path: &str, range: &RangeFilter) {
        if let Some(min) = range.min {
            let param = self.next_param();
            self.conditions.push(FilterCondition {
                sql: format!("{}::FLOAT >= {}", json_path, param),
                params: vec![QueryParam::Float(min)],
            });
        }
        if let Some(max) = range.max {
            let param = self.next_param();
            self.conditions.push(FilterCondition {
                sql: format!("{}::FLOAT <= {}", json_path, param),
                params: vec![QueryParam::Float(max)],
            });
        }
    }

    /// Add range condition for integer JSONB fields
    pub fn add_int_range(&mut self, json_path: &str, range: &RangeFilter) {
        if let Some(min) = range.min {
            let param = self.next_param();
            self.conditions.push(FilterCondition {
                sql: format!("{}::INT >= {}", json_path, param),
                params: vec![QueryParam::Int(min as i64)],
            });
        }
        if let Some(max) = range.max {
            let param = self.next_param();
            self.conditions.push(FilterCondition {
                sql: format!("{}::INT <= {}", json_path, param),
                params: vec![QueryParam::Int(max as i64)],
            });
        }
    }

    /// Set sorting parameters
    pub fn set_sort(&mut self, expression: &str, order: &str) {
        self.sort_expression = expression.to_string();
        self.sort_order = if order.eq_ignore_ascii_case("asc") {
            "ASC".to_string()
        } else {
            "DESC".to_string()
        };
    }

    /// Set pagination parameters
    pub fn set_pagination(&mut self, limit: i64, offset: i64) {
        self.limit = limit.clamp(1, 100);
        self.offset = offset.max(0);
    }

    /// Add privacy filter to exclude users with publicProfile set to false
    /// Users with no publicProfile setting (default) are treated as public
    pub fn add_privacy_filter(&mut self) {
        self.conditions.push(FilterCondition {
            sql: "(settings->>'publicProfile' IS NULL OR (settings->>'publicProfile')::BOOLEAN = true)".to_string(),
            params: vec![],
        });
    }

    /// Check if any conditions have been added
    pub fn has_conditions(&self) -> bool {
        !self.conditions.is_empty()
    }

    /// Build the final SELECT query and collect all parameters
    pub fn build(&self) -> (String, Vec<QueryParam>) {
        self.build_with_fields(false, false, false)
    }

    /// Build SELECT query with optional full field inclusion
    pub fn build_with_fields(
        &self,
        include_data: bool,
        include_score: bool,
        include_settings: bool,
    ) -> (String, Vec<QueryParam>) {
        let mut all_params: Vec<QueryParam> = Vec::new();

        let where_clause = if self.conditions.is_empty() {
            "TRUE".to_string()
        } else {
            let connector = self.logic.sql_connector();

            let condition_strs: Vec<String> = self
                .conditions
                .iter()
                .map(|c| {
                    all_params.extend(c.params.clone());
                    format!("({})", c.sql)
                })
                .collect();

            condition_strs.join(connector)
        };

        // Add pagination params at the end
        let limit_param = all_params.len() + 1;
        let offset_param = all_params.len() + 2;
        all_params.push(QueryParam::Int(self.limit));
        all_params.push(QueryParam::Int(self.offset));

        let select_fields = format!(
            r#"uid,
            server,
            updated_at,
            data->'status'->>'nickName' as nickname,
            (data->'status'->>'level')::BIGINT as level,
            data->'status'->>'secretary' as secretary,
            data->'status'->>'secretarySkinId' as secretary_skin_id,
            (score->>'totalScore')::FLOAT as total_score,
            score->'grade'->>'grade' as grade{}{}{}
            "#,
            if include_data {
                ", data"
            } else {
                ", NULL::jsonb as data"
            },
            if include_score {
                ", score"
            } else {
                ", NULL::jsonb as score"
            },
            if include_settings {
                ", settings"
            } else {
                ", NULL::jsonb as settings"
            }
        );

        let query = format!(
            r#"SELECT {select_fields} FROM users
WHERE {}
ORDER BY {} {} NULLS LAST
LIMIT ${} OFFSET ${}"#,
            where_clause, self.sort_expression, self.sort_order, limit_param, offset_param
        );

        (query, all_params)
    }

    /// Build count query for pagination (without ORDER BY, LIMIT, OFFSET)
    pub fn build_count(&self) -> (String, Vec<QueryParam>) {
        let mut all_params: Vec<QueryParam> = Vec::new();

        let where_clause = if self.conditions.is_empty() {
            "TRUE".to_string()
        } else {
            let connector = self.logic.sql_connector();

            let condition_strs: Vec<String> = self
                .conditions
                .iter()
                .map(|c| {
                    all_params.extend(c.params.clone());
                    format!("({})", c.sql)
                })
                .collect();

            condition_strs.join(connector)
        };

        let query = format!("SELECT COUNT(*) FROM users WHERE {}", where_clause);

        (query, all_params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        let builder = SearchQueryBuilder::new(QueryLogic::And);
        let (query, params) = builder.build();
        assert!(query.contains("WHERE TRUE"));
        assert_eq!(params.len(), 2); // limit and offset
    }

    #[test]
    fn test_text_search() {
        let mut builder = SearchQueryBuilder::new(QueryLogic::And);
        builder.add_text_search("data->'status'->>'nickName'", "myrtle");
        let (query, params) = builder.build();
        assert!(query.contains("ILIKE $1"));
        assert_eq!(params.len(), 3); // search term + limit + offset
        if let QueryParam::String(s) = &params[0] {
            assert_eq!(s, "%myrtle%");
        } else {
            panic!("Expected string param");
        }
    }

    #[test]
    fn test_and_logic() {
        let mut builder = SearchQueryBuilder::new(QueryLogic::And);
        builder.add_exact_match_column("server", "en");
        builder.add_exact_match_jsonb("score->'grade'->>'grade'", "S");
        let (query, _) = builder.build();
        assert!(query.contains(" AND "));
    }

    #[test]
    fn test_or_logic() {
        let mut builder = SearchQueryBuilder::new(QueryLogic::Or);
        builder.add_exact_match_column("server", "en");
        builder.add_exact_match_column("server", "jp");
        let (query, _) = builder.build();
        assert!(query.contains(" OR "));
    }

    #[test]
    fn test_range_filter() {
        let mut builder = SearchQueryBuilder::new(QueryLogic::And);
        let range = RangeFilter {
            min: Some(100.0),
            max: Some(120.0),
        };
        builder.add_int_range("(data->'status'->>'level')", &range);
        let (query, params) = builder.build();
        assert!(query.contains("::INT >= $1"));
        assert!(query.contains("::INT <= $2"));
        assert_eq!(params.len(), 4); // min, max, limit, offset
    }

    #[test]
    fn test_count_query() {
        let mut builder = SearchQueryBuilder::new(QueryLogic::And);
        builder.add_exact_match_column("server", "en");
        let (query, params) = builder.build_count();
        assert!(query.starts_with("SELECT COUNT(*)"));
        assert!(!query.contains("ORDER BY"));
        assert!(!query.contains("LIMIT"));
        assert_eq!(params.len(), 1); // just the server param
    }
}
