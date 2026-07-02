use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Column, MySql, Pool, Postgres, Row, Sqlite};

use crate::config::DbEngine;
use crate::error::{DbChatError, Result};

pub enum DbConnector {
    Postgres(Pool<Postgres>),
    MySql(Pool<MySql>),
    Sqlite(Pool<Sqlite>),
}

macro_rules! row_value {
    ($row:expr, $idx:expr) => {{
        let r = $row;
        let i = $idx;
        if let Ok(v) = r.try_get::<String, _>(i) {
            serde_json::Value::String(v)
        } else if let Ok(v) = r.try_get::<i64, _>(i) {
            serde_json::Value::Number(v.into())
        } else if let Ok(v) = r.try_get::<f64, _>(i) {
            serde_json::Number::from_f64(v)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        } else if let Ok(v) = r.try_get::<bool, _>(i) {
            serde_json::Value::Bool(v)
        } else if let Ok(v) = r.try_get::<i32, _>(i) {
            serde_json::Value::Number(serde_json::Number::from(v))
        } else if let Ok(v) = r.try_get::<i16, _>(i) {
            serde_json::Value::Number(serde_json::Number::from(v))
        } else if let Ok(v) = r.try_get::<serde_json::Value, _>(i) {
            v
        } else {
            serde_json::Value::Null
        }
    }};
}

macro_rules! build_result_from_rows {
    ($rows:expr) => {{
        let rows = $rows;
        let columns: Vec<String> = rows
            .first()
            .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect())
            .unwrap_or_default();

        let values: Vec<Vec<serde_json::Value>> = rows
            .iter()
            .map(|r| {
                columns
                    .iter()
                    .map(|col_name| {
                        let idx = r
                            .columns()
                            .iter()
                            .position(|c| c.name() == col_name.as_str());
                        match idx {
                            Some(i) => row_value!(r, i),
                            None => serde_json::Value::Null,
                        }
                    })
                    .collect()
            })
            .collect();

        let rows_affected = values.len() as u64;
        QueryExecResult::Select {
            columns,
            values,
            rows_affected,
        }
    }};
}

impl DbConnector {
    pub async fn connect(engine: &DbEngine, uri: &str, max_connections: u32) -> Result<Self> {
        match engine {
            DbEngine::Postgres => {
                let pool = PgPoolOptions::new()
                    .max_connections(max_connections)
                    .connect(uri)
                    .await
                    .map_err(|e| DbChatError::Connection(format!("PostgreSQL: {e}")))?;
                Ok(DbConnector::Postgres(pool))
            }
            DbEngine::Mysql => {
                let uri = ensure_mysql_utf8(uri);
                let pool = MySqlPoolOptions::new()
                    .max_connections(max_connections)
                    .connect(&uri)
                    .await
                    .map_err(|e| DbChatError::Connection(format!("MySQL: {e}")))?;
                Ok(DbConnector::MySql(pool))
            }
            DbEngine::Sqlite => {
                let pool = SqlitePoolOptions::new()
                    .max_connections(max_connections)
                    .connect(uri)
                    .await
                    .map_err(|e| DbChatError::Connection(format!("SQLite: {e}")))?;
                Ok(DbConnector::Sqlite(pool))
            }
        }
    }

    pub fn engine(&self) -> DbEngine {
        match self {
            DbConnector::Postgres(_) => DbEngine::Postgres,
            DbConnector::MySql(_) => DbEngine::Mysql,
            DbConnector::Sqlite(_) => DbEngine::Sqlite,
        }
    }

    pub fn dialect(&self) -> &'static str {
        match self {
            DbConnector::Postgres(_) => "PostgreSQL",
            DbConnector::MySql(_) => "MySQL",
            DbConnector::Sqlite(_) => "SQLite",
        }
    }

    pub fn is_connected(&self) -> bool {
        match self {
            DbConnector::Postgres(pool) => !pool.is_closed(),
            DbConnector::MySql(pool) => !pool.is_closed(),
            DbConnector::Sqlite(pool) => !pool.is_closed(),
        }
    }

    pub async fn execute_raw(&self, query: &str) -> Result<QueryExecResult> {
        match self {
            DbConnector::Postgres(pool) => {
                let upper = query.trim().to_uppercase();
                if upper.starts_with("SELECT")
                    || upper.starts_with("WITH")
                    || upper.starts_with("EXPLAIN")
                    || upper.starts_with("SHOW")
                    || upper.starts_with("DESCRIBE")
                    || upper.starts_with("PRAGMA")
                {
                    let rows = sqlx::query(query).fetch_all(pool).await?;
                    Ok(build_result_from_rows!(rows))
                } else {
                    let res = sqlx::query(query).execute(pool).await?;
                    Ok(QueryExecResult::Modify {
                        rows_affected: res.rows_affected(),
                    })
                }
            }
            DbConnector::MySql(pool) => {
                let upper = query.trim().to_uppercase();
                if upper.starts_with("SELECT")
                    || upper.starts_with("WITH")
                    || upper.starts_with("EXPLAIN")
                    || upper.starts_with("SHOW")
                    || upper.starts_with("DESCRIBE")
                    || upper.starts_with("PRAGMA")
                {
                    let rows = sqlx::query(query).fetch_all(pool).await?;
                    Ok(build_result_from_rows!(rows))
                } else {
                    let res = sqlx::query(query).execute(pool).await?;
                    Ok(QueryExecResult::Modify {
                        rows_affected: res.rows_affected(),
                    })
                }
            }
            DbConnector::Sqlite(pool) => {
                let upper = query.trim().to_uppercase();
                if upper.starts_with("SELECT")
                    || upper.starts_with("WITH")
                    || upper.starts_with("EXPLAIN")
                    || upper.starts_with("SHOW")
                    || upper.starts_with("DESCRIBE")
                    || upper.starts_with("PRAGMA")
                {
                    let rows = sqlx::query(query).fetch_all(pool).await?;
                    Ok(build_result_from_rows!(rows))
                } else {
                    let res = sqlx::query(query).execute(pool).await?;
                    Ok(QueryExecResult::Modify {
                        rows_affected: res.rows_affected(),
                    })
                }
            }
        }
    }

    pub async fn close(&self) {
        match self {
            DbConnector::Postgres(pool) => pool.close().await,
            DbConnector::MySql(pool) => pool.close().await,
            DbConnector::Sqlite(pool) => pool.close().await,
        }
    }
}

pub enum QueryExecResult {
    Select {
        columns: Vec<String>,
        values: Vec<Vec<serde_json::Value>>,
        rows_affected: u64,
    },
    Modify {
        rows_affected: u64,
    },
}

impl QueryExecResult {
    pub fn rows_affected(&self) -> u64 {
        match self {
            QueryExecResult::Select { rows_affected, .. } => *rows_affected,
            QueryExecResult::Modify { rows_affected } => *rows_affected,
        }
    }

    pub fn is_select(&self) -> bool {
        matches!(self, QueryExecResult::Select { .. })
    }

    pub fn columns(&self) -> Vec<String> {
        match self {
            QueryExecResult::Select { columns, .. } => columns.clone(),
            QueryExecResult::Modify { .. } => vec![],
        }
    }

    pub fn values(&self) -> Vec<Vec<serde_json::Value>> {
        match self {
            QueryExecResult::Select { values, .. } => values.clone(),
            QueryExecResult::Modify { .. } => vec![],
        }
    }
}

fn ensure_mysql_utf8(uri: &str) -> String {
    if uri.contains("charset=") {
        uri.to_string()
    } else if uri.contains('?') {
        format!("{uri}&charset=utf8mb4")
    } else {
        format!("{uri}?charset=utf8mb4")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_charset_to_mysql_uri() {
        assert_eq!(
            ensure_mysql_utf8("mysql://user:pass@localhost/db"),
            "mysql://user:pass@localhost/db?charset=utf8mb4"
        );
    }

    #[test]
    fn appends_charset_to_uri_with_existing_params() {
        assert_eq!(
            ensure_mysql_utf8("mysql://user:pass@localhost/db?connect_timeout=10"),
            "mysql://user:pass@localhost/db?connect_timeout=10&charset=utf8mb4"
        );
    }

    #[test]
    fn does_not_duplicate_charset() {
        assert_eq!(
            ensure_mysql_utf8("mysql://user:pass@localhost/db?charset=utf8"),
            "mysql://user:pass@localhost/db?charset=utf8"
        );
    }

    #[test]
    fn adds_charset_to_any_uri_without_one() {
        assert_eq!(
            ensure_mysql_utf8("mysql://user:pass@localhost/db"),
            "mysql://user:pass@localhost/db?charset=utf8mb4"
        );
    }
}
