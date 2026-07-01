use sqlx::{Column, Row};

use crate::error::Result;
use crate::db::connector::DbConnector;

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default: Option<String>,
    pub is_foreign_key: bool,
    pub fk_ref: Option<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: Option<u64>,
    pub sample_rows: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone)]
pub struct SchemaInfo {
    pub tables: Vec<TableInfo>,
}

impl SchemaInfo {
    pub fn to_prompt_context(&self, dialect: &str) -> String {
        let mut ctx = format!("Database dialect: {dialect}\n\n");
        ctx.push_str("Tables:\n\n");
        for table in &self.tables {
            ctx.push_str(&format!("  {}\n", table.name));
            ctx.push_str("    Columns:\n");
            for col in &table.columns {
                let pk = if col.is_primary_key { " PK" } else { "" };
                let fk = if let Some((ref t, ref c)) = col.fk_ref {
                    format!(" FK → {t}.{c}")
                } else {
                    String::new()
                };
                let nullable = if col.is_nullable { "" } else { " NOT NULL" };
                ctx.push_str(&format!(
                    "      - {name} {ty}{nullable}{pk}{fk}\n",
                    name = col.name, ty = col.data_type,
                ));
            }
            if let Some(count) = table.row_count {
                ctx.push_str(&format!("    Row count: {count}\n"));
            }
            if !table.sample_rows.is_empty() {
                ctx.push_str("    Sample rows:\n");
                for row in &table.sample_rows {
                    let vals: Vec<String> = row
                        .iter()
                        .map(|v| match v {
                            serde_json::Value::Null => "NULL".to_string(),
                            other => other.to_string(),
                        })
                        .collect();
                    ctx.push_str(&format!("      [{vals}]\n", vals = vals.join(", ")));
                }
            }
            ctx.push('\n');
        }
        ctx
    }
}

macro_rules! sample_row_value {
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
        } else if let Ok(v) = r.try_get::<i32, _>(i) {
            serde_json::Value::Number(serde_json::Number::from(v))
        } else if let Ok(v) = r.try_get::<bool, _>(i) {
            serde_json::Value::Bool(v)
        } else if let Ok(v) = r.try_get::<serde_json::Value, _>(i) {
            v
        } else {
            serde_json::Value::Null
        }
    }};
}

macro_rules! collect_samples {
    ($rows:expr, $col_names:expr) => {{
        let rows = $rows;
        let col_names = $col_names;
        rows.iter()
            .map(|r| {
                col_names
                    .iter()
                    .map(|c| {
                        let name: &str = c;
                        let idx = r.columns().iter().position(|col| col.name() == name);
                        match idx {
                            Some(i) => sample_row_value!(r, i),
                            None => serde_json::Value::Null,
                        }
                    })
                    .collect()
            })
            .collect()
    }};
}

impl DbConnector {
    pub async fn introspect_schema(&self) -> Result<SchemaInfo> {
        match self {
            DbConnector::Postgres(pool) => introspect_postgres_schema(pool).await,
            DbConnector::MySql(pool) => introspect_mysql_schema(pool).await,
            DbConnector::Sqlite(pool) => introspect_sqlite_schema(pool).await,
        }
    }

    pub async fn table_names(&self) -> Result<Vec<String>> {
        match self {
            DbConnector::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT table_name FROM information_schema.tables \
                     WHERE table_schema = 'public' AND table_type = 'BASE TABLE' \
                     ORDER BY table_name",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows.iter().map(|r| r.get::<String, _>(0)).collect())
            }
            DbConnector::MySql(pool) => {
                let rows = sqlx::query(
                    "SELECT table_name FROM information_schema.tables \
                     WHERE table_schema = DATABASE() AND table_type = 'BASE TABLE' \
                     ORDER BY table_name",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows.iter().map(|r| r.get::<String, _>(0)).collect())
            }
            DbConnector::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows.iter().map(|r| r.get::<String, _>(0)).collect())
            }
        }
    }
}

// ─── PostgreSQL ───────────────────────────────────────────

async fn introspect_postgres_schema(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<SchemaInfo> {
    let table_rows: Vec<sqlx::postgres::PgRow> = sqlx::query(
        "SELECT table_name FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE' \
         ORDER BY table_name",
    )
    .fetch_all(pool)
    .await?;

    let mut tables = Vec::new();
    for row in &table_rows {
        let table_name: String = row.get("table_name");
        let columns = introspect_postgres_columns(pool, &table_name).await?;
        let row_count = get_row_count_postgres(pool, &table_name).await;
        let sample_rows = get_sample_data_postgres(pool, &table_name, &columns).await;
        tables.push(TableInfo { name: table_name, columns, row_count, sample_rows });
    }
    Ok(SchemaInfo { tables })
}

async fn introspect_postgres_columns(
    pool: &sqlx::Pool<sqlx::Postgres>,
    table: &str,
) -> Result<Vec<ColumnInfo>> {
    let rows = sqlx::query(
        r#"
        SELECT c.column_name, c.data_type, c.is_nullable, c.column_default,
               CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_pk
        FROM information_schema.columns c
        LEFT JOIN (
            SELECT ku.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage ku
                ON tc.constraint_name = ku.constraint_name AND tc.table_schema = ku.table_schema
            WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_name = $1 AND tc.table_schema = 'public'
        ) pk ON pk.column_name = c.column_name
        WHERE c.table_name = $1 AND c.table_schema = 'public'
        ORDER BY c.ordinal_position
        "#,
    )
    .bind(table)
    .fetch_all(pool)
    .await?;

    let mut columns = Vec::new();
    for r in &rows {
        columns.push(ColumnInfo {
            name: r.get("column_name"),
            data_type: r.get("data_type"),
            is_nullable: r.get::<String, _>("is_nullable") == "YES",
            is_primary_key: r.get("is_pk"),
            default: r.get("column_default"),
            is_foreign_key: false,
            fk_ref: None,
        });
    }
    Ok(columns)
}

async fn get_row_count_postgres(pool: &sqlx::Pool<sqlx::Postgres>, table: &str) -> Option<u64> {
    let q = format!("SELECT COUNT(*) FROM \"{table}\"");
    sqlx::query_scalar::<_, i64>(&q).fetch_optional(pool).await.ok().flatten().map(|c| c as u64)
}

async fn get_sample_data_postgres(
    pool: &sqlx::Pool<sqlx::Postgres>,
    table: &str,
    columns: &[ColumnInfo],
) -> Vec<Vec<serde_json::Value>> {
    if columns.is_empty() { return vec![]; }
    let col_names: Vec<String> = columns.iter().map(|c| format!("\"{}\"", c.name)).collect();
    let q = format!("SELECT {} FROM \"{}\" LIMIT 3", col_names.join(", "), table);
    if let Ok(rows) = sqlx::query(&q).fetch_all(pool).await {
        let names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
        collect_samples!(rows, names)
    } else {
        vec![]
    }
}

// ─── MySQL ────────────────────────────────────────────────

async fn introspect_mysql_schema(pool: &sqlx::Pool<sqlx::MySql>) -> Result<SchemaInfo> {
    let table_rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(
        "SELECT table_name FROM information_schema.tables \
         WHERE table_schema = DATABASE() AND table_type = 'BASE TABLE' \
         ORDER BY table_name",
    )
    .fetch_all(pool)
    .await?;

    let mut tables = Vec::new();
    for row in &table_rows {
        let table_name: String = row.get("table_name");
        let columns = introspect_mysql_columns(pool, &table_name).await?;
        let row_count = get_row_count_mysql(pool, &table_name).await;
        let sample_rows = get_sample_data_mysql(pool, &table_name, &columns).await;
        tables.push(TableInfo { name: table_name, columns, row_count, sample_rows });
    }
    Ok(SchemaInfo { tables })
}

async fn introspect_mysql_columns(
    pool: &sqlx::Pool<sqlx::MySql>,
    table: &str,
) -> Result<Vec<ColumnInfo>> {
    let rows = sqlx::query(
        r#"
        SELECT c.column_name, c.data_type, c.is_nullable, c.column_default,
               CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_pk
        FROM information_schema.columns c
        LEFT JOIN (
            SELECT ku.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage ku
                ON tc.constraint_name = ku.constraint_name AND tc.table_schema = ku.table_schema
            WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_name = ? AND tc.table_schema = DATABASE()
        ) pk ON pk.column_name = c.column_name
        WHERE c.table_name = ? AND c.table_schema = DATABASE()
        ORDER BY c.ordinal_position
        "#,
    )
    .bind(table)
    .bind(table)
    .fetch_all(pool)
    .await?;

    let mut columns = Vec::new();
    for r in &rows {
        columns.push(ColumnInfo {
            name: r.get("column_name"),
            data_type: r.get("data_type"),
            is_nullable: r.get::<String, _>("is_nullable") == "YES",
            is_primary_key: r.get("is_pk"),
            default: r.get("column_default"),
            is_foreign_key: false,
            fk_ref: None,
        });
    }
    Ok(columns)
}

async fn get_row_count_mysql(pool: &sqlx::Pool<sqlx::MySql>, table: &str) -> Option<u64> {
    let q = format!("SELECT COUNT(*) FROM `{table}`");
    sqlx::query_scalar::<_, i64>(&q).fetch_optional(pool).await.ok().flatten().map(|c| c as u64)
}

async fn get_sample_data_mysql(
    pool: &sqlx::Pool<sqlx::MySql>,
    table: &str,
    columns: &[ColumnInfo],
) -> Vec<Vec<serde_json::Value>> {
    if columns.is_empty() { return vec![]; }
    let col_names: Vec<String> = columns.iter().map(|c| format!("`{}`", c.name)).collect();
    let q = format!("SELECT {} FROM `{}` LIMIT 3", col_names.join(", "), table);
    if let Ok(rows) = sqlx::query(&q).fetch_all(pool).await {
        let names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
        collect_samples!(rows, names)
    } else {
        vec![]
    }
}

// ─── SQLite ───────────────────────────────────────────────

async fn introspect_sqlite_schema(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<SchemaInfo> {
    let table_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    let mut tables = Vec::new();
    for row in &table_rows {
        let table_name: String = row.get("name");
        let columns = introspect_sqlite_columns(pool, &table_name).await?;
        let row_count = get_row_count_sqlite(pool, &table_name).await;
        let sample_rows = get_sample_data_sqlite(pool, &table_name, &columns).await;
        tables.push(TableInfo { name: table_name, columns, row_count, sample_rows });
    }
    Ok(SchemaInfo { tables })
}

async fn introspect_sqlite_columns(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    table: &str,
) -> Result<Vec<ColumnInfo>> {
    let q = format!("PRAGMA table_info(\"{table}\")");
    let rows = sqlx::query(&q).fetch_all(pool).await?;

    let mut columns = Vec::new();
    for r in &rows {
        columns.push(ColumnInfo {
            name: r.get("name"),
            data_type: r.get("type"),
            is_nullable: !r.get::<bool, _>("notnull"),
            is_primary_key: r.get("pk"),
            default: r.get("dflt_value"),
            is_foreign_key: false,
            fk_ref: None,
        });
    }
    Ok(columns)
}

async fn get_row_count_sqlite(pool: &sqlx::Pool<sqlx::Sqlite>, table: &str) -> Option<u64> {
    let q = format!("SELECT COUNT(*) FROM \"{table}\"");
    sqlx::query_scalar::<_, i64>(&q).fetch_optional(pool).await.ok().flatten().map(|c| c as u64)
}

async fn get_sample_data_sqlite(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    table: &str,
    columns: &[ColumnInfo],
) -> Vec<Vec<serde_json::Value>> {
    if columns.is_empty() { return vec![]; }
    let col_names: Vec<String> = columns.iter().map(|c| format!("\"{}\"", c.name)).collect();
    let q = format!("SELECT {} FROM \"{}\" LIMIT 3", col_names.join(", "), table);
    if let Ok(rows) = sqlx::query(&q).fetch_all(pool).await {
        let names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
        collect_samples!(rows, names)
    } else {
        vec![]
    }
}
