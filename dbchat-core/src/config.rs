use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_OPENCODE_ZEN_CHAT_COMPLETIONS_URL: &str =
    "https://opencode.ai/zen/v1/chat/completions";
pub const DEFAULT_OPENCODE_ZEN_MODEL: &str = "deepseek-v4-flash-free";
pub const DEFAULT_OPENCODE_API_KEY_ENV: &str = "OPENCODE_API_KEY";

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LlmProvider {
    #[serde(rename = "openai", alias = "OpenAI")]
    OpenAI,
    #[serde(rename = "anthropic", alias = "Anthropic")]
    Anthropic,
    #[serde(rename = "ollama", alias = "Ollama")]
    Ollama,
    #[serde(rename = "google", alias = "Google")]
    Google,
    #[serde(
        rename = "openai-compatible",
        alias = "OpenAICompatible",
        alias = "openai_compatible"
    )]
    OpenAICompatible,
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
            provider: LlmProvider::OpenAICompatible,
            model: String::from(DEFAULT_OPENCODE_ZEN_MODEL),
            temperature: 0.0,
            api_key: Some(format!("env:{DEFAULT_OPENCODE_API_KEY_ENV}")),
            api_url: Some(String::from(DEFAULT_OPENCODE_ZEN_CHAT_COMPLETIONS_URL)),
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
            .map(|lang| {
                if lang.to_lowercase().starts_with("fr") {
                    Locale::Fr
                } else {
                    Locale::En
                }
            })
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
    pub fn load_existing() -> Option<Self> {
        let path = Self::config_path();
        let content = std::fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }

    pub fn has_database_uri(&self) -> bool {
        !self.db.uri.trim().is_empty()
    }

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
        Self::ensure_config_dir()?;
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn config_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(dir).join("dbchat")
        } else if let Some(home) = std::env::var("HOME").ok() {
            PathBuf::from(home).join(".config").join("dbchat")
        } else {
            PathBuf::from("./dbchat")
        }
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn env_path() -> PathBuf {
        Self::config_dir().join(".env")
    }

    pub fn ensure_config_dir() -> Result<(), String> {
        let dir = Self::config_dir();
        ensure_private_dir(&dir).map_err(|e| e.to_string())
    }

    pub fn resolve_llm_api_key(&mut self) {
        if let Some(value) = self.llm.api_key.clone() {
            if let Some(var) = value.strip_prefix("env:") {
                self.llm.api_key = std::env::var(var).ok();
            }
            return;
        }

        self.llm.api_key = match self.llm.provider {
            LlmProvider::OpenAI => std::env::var("OPENAI_API_KEY").ok(),
            LlmProvider::Anthropic => std::env::var("ANTHROPIC_API_KEY").ok(),
            LlmProvider::Ollama => Some(String::new()),
            LlmProvider::Google => std::env::var("GOOGLE_API_KEY").ok(),
            LlmProvider::OpenAICompatible => {
                let api_url = self.llm.api_url.as_deref().unwrap_or_default();
                if api_url.contains("opencode.ai") {
                    std::env::var(DEFAULT_OPENCODE_API_KEY_ENV).ok()
                } else if api_url.contains("openrouter.ai") {
                    std::env::var("OPENROUTER_API_KEY").ok()
                } else if api_url.contains("deepseek.com") {
                    std::env::var("DEEPSEEK_API_KEY").ok()
                } else {
                    std::env::var("OPENAI_COMPATIBLE_API_KEY").ok()
                }
            }
        };
    }

    pub fn llm_api_key_env(&self) -> Option<&'static str> {
        match self.llm.provider {
            LlmProvider::OpenAI => Some("OPENAI_API_KEY"),
            LlmProvider::Anthropic => Some("ANTHROPIC_API_KEY"),
            LlmProvider::Ollama => None,
            LlmProvider::Google => Some("GOOGLE_API_KEY"),
            LlmProvider::OpenAICompatible => {
                let api_url = self.llm.api_url.as_deref().unwrap_or_default();
                if api_url.contains("opencode.ai") {
                    Some(DEFAULT_OPENCODE_API_KEY_ENV)
                } else if api_url.contains("openrouter.ai") {
                    Some("OPENROUTER_API_KEY")
                } else if api_url.contains("deepseek.com") {
                    Some("DEEPSEEK_API_KEY")
                } else {
                    Some("OPENAI_COMPATIBLE_API_KEY")
                }
            }
        }
    }
}

fn ensure_private_dir(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

        let mut builder = std::fs::DirBuilder::new();
        builder.recursive(true);
        builder.mode(0o700);
        builder.create(path)?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(path)
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

    #[test]
    fn test_has_database_uri() {
        let mut config = AppConfig::default();
        assert!(!config.has_database_uri());
        config.db.uri = "mysql://user:pass@localhost/db".to_string();
        assert!(config.has_database_uri());
    }

    #[test]
    fn test_default_llm_uses_opencode_zen() {
        let config = AppConfig::default();

        assert_eq!(config.llm.provider, LlmProvider::OpenAICompatible);
        assert_eq!(config.llm.model, DEFAULT_OPENCODE_ZEN_MODEL);
        assert_eq!(
            config.llm.api_url.as_deref(),
            Some(DEFAULT_OPENCODE_ZEN_CHAT_COMPLETIONS_URL)
        );
        assert_eq!(config.llm.api_key.as_deref(), Some("env:OPENCODE_API_KEY"));
        assert_eq!(config.llm_api_key_env(), Some(DEFAULT_OPENCODE_API_KEY_ENV));
    }

    #[test]
    fn test_env_path_is_next_to_config_path() {
        assert_eq!(
            AppConfig::env_path(),
            AppConfig::config_path().parent().unwrap().join(".env")
        );
    }

    #[test]
    fn test_openai_compatible_serializes() {
        let mut config = AppConfig::default();
        config.llm.provider = LlmProvider::OpenAICompatible;
        let content = toml::to_string(&config).unwrap();
        assert!(content.contains("openai-compatible"));
        let parsed: AppConfig = toml::from_str(&content).unwrap();
        assert_eq!(parsed.llm.provider, LlmProvider::OpenAICompatible);
    }

    #[test]
    fn test_legacy_openai_compatible_deserializes() {
        let content = r#"
[db]
engine = "Postgres"
uri = ""
max_connections = 5
query_timeout_secs = 30
read_only = true
max_rows = 1000
safe_mode = true

[llm]
provider = "OpenAICompatible"
model = "deepseek-v4-flash"
temperature = 0.0

[display]
theme = "Dark"
locale = "En"
show_sql = true
show_timing = true
format = "Table"
verbose = false
"#;
        let parsed: AppConfig = toml::from_str(content).unwrap();
        assert_eq!(parsed.llm.provider, LlmProvider::OpenAICompatible);
    }
}
