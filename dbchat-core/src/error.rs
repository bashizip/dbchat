use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbChatError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Unsupported database engine: {0}")]
    UnsupportedEngine(String),

    #[error("Query cancelled: {0}")]
    Cancelled(String),

    #[error("Query timeout exceeded ({0}s)")]
    Timeout(u64),

    #[error("Destructive query blocked in read-only mode")]
    DestructiveQuery,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DbChatError>;
