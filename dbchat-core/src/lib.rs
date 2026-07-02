pub mod config;
pub mod db;
pub mod error;
pub mod llm;

pub use config::AppConfig;
pub use db::QueryExecResult;
pub use db::{ColumnInfo, SchemaInfo, TableInfo};

use db::DbConnector;
use error::{DbChatError, Result};
use llm::LlmClient;

pub struct DbChat {
    pub db: DbConnector,
    pub llm: LlmClient,
    pub schema: SchemaInfo,
    pub config: AppConfig,
    pub verbose: bool,
    pub last_system_prompt: Option<String>,
    pub last_raw_response: Option<String>,
}

impl DbChat {
    pub async fn connect(config: AppConfig) -> Result<Self> {
        let db = DbConnector::connect(&config.db.engine, &config.db.uri, config.db.max_connections)
            .await?;

        let schema = db.introspect_schema().await?;
        let mut llm = LlmClient::new(config.llm.clone());
        llm.set_schema(schema.clone(), config.db.engine.clone());

        let verbose = config.display.verbose;

        Ok(Self {
            db,
            llm,
            schema,
            config,
            verbose,
            last_system_prompt: None,
            last_raw_response: None,
        })
    }

    pub async fn chat(&mut self, question: &str) -> Result<ChatResponse> {
        if self.verbose {
            let prompt = self.llm.system_prompt();
            self.last_system_prompt = Some(prompt);
        }

        let generation = self.llm.generate_sql(question).await?;
        self.last_raw_response = Some(generation.raw_response.clone());

        if let Some(ref explanation) = generation.explanation {
            return Ok(ChatResponse::Info(explanation.clone()));
        }

        if generation.sql.is_empty() {
            return Ok(ChatResponse::Info("Failed to generate SQL query.".to_string()));
        }

        if is_destructive(&generation.sql) {
            if self.config.db.read_only {
                return Err(DbChatError::DestructiveQuery);
            }
            if self.config.db.safe_mode {
                return Ok(ChatResponse::ConfirmDestructive(generation.sql.clone()));
            }
        }

        let sql = apply_select_limit(&generation.sql, self.config.db.max_rows);

        let start = std::time::Instant::now();
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.db.query_timeout_secs),
            self.db.execute_raw(&sql),
        )
        .await
        .map_err(|_| DbChatError::Timeout(self.config.db.query_timeout_secs))??;

        let elapsed = start.elapsed();

        Ok(ChatResponse::Result {
            sql,
            result,
            elapsed,
        })
    }

    pub async fn refresh_schema(&mut self) -> Result<()> {
        self.schema = self.db.introspect_schema().await?;
        self.llm
            .set_schema(self.schema.clone(), self.config.db.engine.clone());
        Ok(())
    }

    pub fn toggle_verbose(&mut self) -> bool {
        self.verbose = !self.verbose;
        self.verbose
    }
}

pub enum ChatResponse {
    Result {
        sql: String,
        result: QueryExecResult,
        elapsed: std::time::Duration,
    },
    ConfirmDestructive(String),
    Info(String),
}

fn is_destructive(sql: &str) -> bool {
    let upper = sql.trim().to_uppercase();
    upper.starts_with("DROP")
        || upper.starts_with("TRUNCATE")
        || upper.starts_with("DELETE")
        || upper.starts_with("UPDATE")
        || upper.starts_with("INSERT")
        || upper.starts_with("ALTER")
        || upper.starts_with("CREATE")
        || upper.starts_with("REPLACE")
}

fn apply_select_limit(sql: &str, max_rows: u64) -> String {
    let trimmed = sql.trim().trim_end_matches(';').trim();
    if max_rows == 0 || !is_select_like(trimmed) || has_limit_clause(trimmed) {
        return trimmed.to_string();
    }
    format!("{trimmed} LIMIT {max_rows}")
}

fn is_select_like(sql: &str) -> bool {
    let upper = sql.trim_start().to_uppercase();
    upper.starts_with("SELECT") || upper.starts_with("WITH")
}

fn has_limit_clause(sql: &str) -> bool {
    sql.split_whitespace()
        .any(|token| token.eq_ignore_ascii_case("LIMIT"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_limit_to_selects_without_limit() {
        assert_eq!(
            apply_select_limit("SELECT * FROM users;", 50),
            "SELECT * FROM users LIMIT 50"
        );
    }

    #[test]
    fn keeps_existing_limit() {
        assert_eq!(
            apply_select_limit("SELECT * FROM users LIMIT 10", 50),
            "SELECT * FROM users LIMIT 10"
        );
    }

    #[test]
    fn does_not_limit_modifying_sql() {
        assert_eq!(
            apply_select_limit("UPDATE users SET name = 'x'", 50),
            "UPDATE users SET name = 'x'"
        );
    }
}
