use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DbEngine {
    Postgres,
    Mysql,
    Sqlite,
}

impl std::fmt::Display for DbEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbEngine::Postgres => write!(f, "PostgreSQL"),
            DbEngine::Mysql => write!(f, "MySQL"),
            DbEngine::Sqlite => write!(f, "SQLite"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub engine: DbEngine,
    pub uri: String,
    pub max_connections: u32,
    pub query_timeout_secs: u64,
    pub read_only: bool,
    pub max_rows: u64,
    pub safe_mode: bool,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            engine: DbEngine::Postgres,
            uri: String::new(),
            max_connections: 5,
            query_timeout_secs: 30,
            read_only: true,
            max_rows: 1000,
            safe_mode: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Ollama,
    Google,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub model: String,
    pub temperature: f64,
    pub api_key: Option<String>,
    pub api_url: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            model: String::from("gpt-4o-mini"),
            temperature: 0.0,
            api_key: None,
            api_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Table
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Locale {
    Fr,
    En,
}

impl Locale {
    pub fn detect() -> Self {
        std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .map(|lang| if lang.to_lowercase().starts_with("fr") { Locale::Fr } else { Locale::En })
            .unwrap_or(Locale::En)
    }

    pub fn t<'a>(&self, fr: &'a str, en: &'a str) -> &'a str {
        match self {
            Locale::Fr => fr,
            Locale::En => en,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub theme: Theme,
    pub locale: Locale,
    pub show_sql: bool,
    pub show_timing: bool,
    pub format: OutputFormat,
    pub verbose: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        let locale = Locale::detect();
        Self {
            theme: Theme::Dark,
            locale,
            show_sql: true,
            show_timing: true,
            format: OutputFormat::Table,
            verbose: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub db: DbConfig,
    pub llm: LlmConfig,
    pub display: DisplayConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            db: DbConfig::default(),
            llm: LlmConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn from_uri(uri: &str) -> Result<Self, String> {
        let engine = if uri.starts_with("postgres://") || uri.starts_with("postgresql://") {
            DbEngine::Postgres
        } else if uri.starts_with("mysql://") {
            DbEngine::Mysql
        } else if uri.starts_with("sqlite://") {
            DbEngine::Sqlite
        } else {
            return Err(format!("URI non reconnue : {uri}"));
        };

        Ok(Self {
            db: DbConfig {
                engine,
                uri: uri.to_string(),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str(&content) {
                return config;
            }
        }
        let config = AppConfig::default();
        let _ = config.save();
        config
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn config_path() -> PathBuf {
        if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(dir).join("dbchat").join("config.toml")
        } else if let Some(home) = std::env::var("HOME").ok() {
            PathBuf::from(home).join(".config").join("dbchat").join("config.toml")
        } else {
            PathBuf::from("./dbchat.toml")
        }
    }

    pub fn resolve_llm_api_key(&mut self) {
        if self.llm.api_key.is_none() {
            self.llm.api_key = match self.llm.provider {
                LlmProvider::OpenAI => std::env::var("OPENAI_API_KEY").ok(),
                LlmProvider::Anthropic => std::env::var("ANTHROPIC_API_KEY").ok(),
                LlmProvider::Ollama => Some(String::new()),
                LlmProvider::Google => std::env::var("GOOGLE_API_KEY").ok(),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_detect() {
        let locale = Locale::detect();
        // Just check it doesn't panic
        let _ = locale.t("français", "english");
    }

    #[test]
    fn test_from_uri_postgres() {
        let config = AppConfig::from_uri("postgres://user:pass@localhost/db").unwrap();
        assert_eq!(config.db.engine, DbEngine::Postgres);
    }

    #[test]
    fn test_from_uri_mysql() {
        let config = AppConfig::from_uri("mysql://user:pass@localhost/db").unwrap();
        assert_eq!(config.db.engine, DbEngine::Mysql);
    }

    #[test]
    fn test_from_uri_sqlite() {
        let config = AppConfig::from_uri("sqlite:///path/to/db.db").unwrap();
        assert_eq!(config.db.engine, DbEngine::Sqlite);
    }

    #[test]
    fn test_from_uri_invalid() {
        assert!(AppConfig::from_uri("mongodb://localhost/db").is_err());
    }
}
