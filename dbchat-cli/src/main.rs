use clap::{Parser, Subcommand};

mod config_wizard;
mod render;
mod repl;
mod ui;

#[derive(Parser)]
#[command(name = "dbchat")]
#[command(about = "Chat with your database using natural language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<DbCommand>,

    /// Connection string (e.g. postgres://user:pass@localhost/db)
    #[arg()]
    connection_string: Option<String>,

    /// One-shot query mode
    #[arg(short, long)]
    query: Option<String>,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    format: String,

    /// Read-only mode (block destructive queries)
    #[arg(long)]
    read_only: bool,

    /// Disable colors
    #[arg(long, global = true)]
    no_color: bool,

    /// Verbose mode (show prompt & raw response)
    #[arg(short, long)]
    verbose: bool,

    /// Model override (e.g. gemini-3.1-flash-lite, gpt-5.4-mini)
    #[arg(long)]
    model: Option<String>,

    /// LLM provider (opencode, openrouter, deepseek, openai, anthropic, ollama, google, openai-compatible)
    #[arg(long)]
    provider: Option<String>,

    /// Config directory override
    #[arg(long)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum DbCommand {
    /// Connect to a PostgreSQL database
    Postgres { uri: String },
    /// Connect to a MySQL database
    Mysql { uri: String },
    /// Connect to a SQLite database
    Sqlite { path: String },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
}

#[derive(clap::Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Initialize default configuration
    Init,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let cli = Cli::parse();
    validate_secure_env_file()?;

    if let Some(DbCommand::Config { .. }) = &cli.command {
        return handle_config(&cli);
    }

    let cli_uri = resolve_uri(&cli)?;

    // Load config file, then override with CLI args
    let mut config = if cli_uri.is_some() {
        dbchat_core::AppConfig::load()
    } else {
        dbchat_core::AppConfig::load_existing().unwrap_or_default()
    };

    if let Some(uri) = cli_uri {
        if let Ok(uri_config) = dbchat_core::AppConfig::from_uri(&uri) {
            config.db = uri_config.db;
        }
    } else if config.has_database_uri() {
        config_wizard::print_config_summary(&config);
    } else {
        println!("Aucune configuration de connexion trouvee.");
        config = config_wizard::run_config_menu(config)?;
        if !config.has_database_uri() {
            println!("Aucune connexion BD configuree. Relancez `dbchat config`.");
            return Ok(());
        }
    }

    if cli.read_only {
        config.db.read_only = true;
    }
    if cli.verbose {
        config.display.verbose = true;
    }
    if cli.no_color {
        // NO_COLOR will be checked at render time
    }
    if let Some(ref provider) = cli.provider {
        apply_provider_override(&mut config, provider)?;
    }
    if let Some(ref model) = cli.model {
        config.llm.model = model.clone();
    }
    resolve_llm_api_key(&mut config)?;

    let locale = config.display.locale.clone();
    let uri = config.db.uri.clone();

    if let Some(query) = &cli.query {
        let mut dbchat = dbchat_core::DbChat::connect(config).await?;
        let response = dbchat.chat(query).await?;
        render::render_response(&response, &cli.format, &locale);
        return Ok(());
    }

    let mut dbchat = dbchat_core::DbChat::connect(config).await?;
    println!(
        "\x1b[1;32m✓\x1b[0m {} \x1b[36m{uri}\x1b[0m",
        locale.t("Connecté à", "Connected to"),
        uri = redact_uri(&uri),
    );
    println!(
        "\x1b[34mℹ\x1b[0m {} {}",
        locale.t("tables trouvées.", "tables found."),
        dbchat.schema.tables.len()
    );
    println!();

    repl::run_repl(&mut dbchat).await?;

    Ok(())
}

fn handle_config(cli: &Cli) -> color_eyre::Result<()> {
    if matches!(cli.command, Some(DbCommand::Config { action: None })) {
        let config = dbchat_core::AppConfig::load_existing().unwrap_or_default();
        let _ = config_wizard::run_config_menu(config)?;
        return Ok(());
    }

    let path = dbchat_core::AppConfig::config_path();
    println!(
        "\x1b[1mConfig file:\x1b[0m \x1b[36m{}\x1b[0m",
        path.display()
    );

    let is_init = matches!(
        cli.command,
        Some(DbCommand::Config {
            action: Some(ConfigAction::Init)
        })
    );

    if path.exists() && !is_init {
        let content = std::fs::read_to_string(&path)?;
        println!("\x1b[2m── Content ──\x1b[0m");
        println!("{}", redact_config_content(&content));
    } else {
        let default = dbchat_core::AppConfig::default();
        let content = toml::to_string_pretty(&default).unwrap();
        dbchat_core::AppConfig::ensure_config_dir().map_err(|err| color_eyre::eyre::eyre!(err))?;
        std::fs::write(&path, &content)?;
        println!("\x1b[32m✓\x1b[0m Created: {}", path.display());
        println!("\x1b[2m── Default config ──\x1b[0m");
        println!("{content}");
    }
    Ok(())
}

pub(crate) fn redact_uri(uri: &str) -> String {
    let Some(scheme_end) = uri.find("://") else {
        return uri.to_string();
    };
    let credentials_start = scheme_end + 3;
    let Some(at_rel) = uri[credentials_start..].find('@') else {
        return uri.to_string();
    };
    let at = credentials_start + at_rel;
    let credentials = &uri[credentials_start..at];
    if credentials.is_empty() {
        return uri.to_string();
    }

    let user = credentials
        .split_once(':')
        .map(|(user, _)| user)
        .unwrap_or(credentials);
    format!("{}{}:***{}", &uri[..credentials_start], user, &uri[at..])
}

pub(crate) fn redact_config_content(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("api_key") {
                let indent_len = line.len() - trimmed.len();
                format!("{}api_key = \"***\"", &line[..indent_len])
            } else if trimmed.starts_with("uri") {
                let indent_len = line.len() - trimmed.len();
                let value = trimmed
                    .split_once('=')
                    .map(|(_, value)| value.trim().trim_matches('"'))
                    .unwrap_or_default();
                format!("{}uri = \"{}\"", &line[..indent_len], redact_uri(value))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn resolve_uri(cli: &Cli) -> Result<Option<String>, color_eyre::eyre::Error> {
    if let Some(uri) = &cli.connection_string {
        return Ok(Some(uri.clone()));
    }

    if let Some(ref cmd) = cli.command {
        return match cmd {
            DbCommand::Postgres { uri } => Ok(Some(uri.clone())),
            DbCommand::Mysql { uri } => Ok(Some(uri.clone())),
            DbCommand::Sqlite { path } => Ok(Some(format!("sqlite://{path}"))),
            DbCommand::Config { .. } => unreachable!(),
        };
    }

    Ok(None)
}

fn apply_provider_override(
    config: &mut dbchat_core::AppConfig,
    provider: &str,
) -> Result<(), color_eyre::eyre::Error> {
    config.llm.provider = match provider.to_lowercase().as_str() {
        "openai" => {
            config.llm.api_url = None;
            config.llm.api_key = None;
            config.llm.model = "gpt-5.4-mini".to_string();
            dbchat_core::config::LlmProvider::OpenAI
        }
        "anthropic" => {
            config.llm.api_url = None;
            config.llm.api_key = None;
            config.llm.model = "claude-haiku-4-5".to_string();
            dbchat_core::config::LlmProvider::Anthropic
        }
        "ollama" => {
            config.llm.api_url = None;
            config.llm.api_key = None;
            config.llm.model = "llama3".to_string();
            dbchat_core::config::LlmProvider::Ollama
        }
        "google" => {
            config.llm.api_url = None;
            config.llm.api_key = None;
            config.llm.model = config_wizard::GEMINI_FLASH_LITE_MODEL.to_string();
            dbchat_core::config::LlmProvider::Google
        }
        "openai-compatible" | "openai_compatible" | "compatible" => {
            dbchat_core::config::LlmProvider::OpenAICompatible
        }
        "opencode" | "zen" | "opencode-zen" => {
            config.llm.api_url = Some(config_wizard::OPENCODE_ZEN_CHAT_COMPLETIONS_URL.to_string());
            config.llm.model = config_wizard::OPENCODE_ZEN_FREE_MODEL.to_string();
            config.llm.api_key = Some("env:OPENCODE_API_KEY".to_string());
            dbchat_core::config::LlmProvider::OpenAICompatible
        }
        "openrouter" | "openrouter-free" => {
            config.llm.api_url = Some(config_wizard::OPENROUTER_API_BASE_URL.to_string());
            config.llm.model = config_wizard::OPENROUTER_FREE_MODEL.to_string();
            config.llm.api_key = Some("env:OPENROUTER_API_KEY".to_string());
            dbchat_core::config::LlmProvider::OpenAICompatible
        }
        "deepseek" | "deepseek-flash" | "deepseek-flash-free" | "deepseek-v4-flash" => {
            config.llm.api_url = Some(config_wizard::DEEPSEEK_API_BASE_URL.to_string());
            config.llm.model = config_wizard::DEEPSEEK_FLASH_MODEL.to_string();
            config.llm.api_key = Some("env:DEEPSEEK_API_KEY".to_string());
            dbchat_core::config::LlmProvider::OpenAICompatible
        }
        _ => {
            return Err(color_eyre::eyre::eyre!(
                "Provider inconnu : {provider} (opencode, openrouter, deepseek, openai, anthropic, ollama, google, openai-compatible)"
            ));
        }
    };
    Ok(())
}

fn validate_secure_env_file() -> color_eyre::Result<()> {
    let path = dbchat_core::AppConfig::env_path();
    if path.exists() {
        for item in dotenvy::from_path_iter(&path)
            .map_err(|err| color_eyre::eyre::eyre!("Invalid {}: {err}", path.display()))?
        {
            item.map_err(|err| color_eyre::eyre::eyre!("Invalid {}: {err}", path.display()))?;
        }
    }
    Ok(())
}

fn resolve_llm_api_key(config: &mut dbchat_core::AppConfig) -> color_eyre::Result<()> {
    let configured_env = configured_llm_api_key_env(config);
    config.resolve_llm_api_key();

    if config.llm.api_key.is_none()
        && let Some(env_name) = configured_env
    {
        config.llm.api_key = secure_env_value(&env_name)?;
    }

    Ok(())
}

fn configured_llm_api_key_env(config: &dbchat_core::AppConfig) -> Option<String> {
    config
        .llm
        .api_key
        .as_deref()
        .and_then(|value| value.strip_prefix("env:").map(str::to_string))
        .or_else(|| config.llm_api_key_env().map(str::to_string))
}

fn secure_env_value(name: &str) -> color_eyre::Result<Option<String>> {
    secure_env_value_from_path(&dbchat_core::AppConfig::env_path(), name)
}

fn secure_env_value_from_path(
    path: &std::path::Path,
    name: &str,
) -> color_eyre::Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }

    let iter = dotenvy::from_path_iter(path)
        .map_err(|err| color_eyre::eyre::eyre!("Invalid {}: {err}", path.display()))?;
    for item in iter {
        let (key, value) =
            item.map_err(|err| color_eyre::eyre::eyre!("Invalid {}: {err}", path.display()))?;
        if key == name {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

#[cfg(test)]
fn unique_test_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_password_from_database_uri() {
        assert_eq!(
            redact_uri("postgres://user:secret@localhost/db"),
            "postgres://user:***@localhost/db"
        );
    }

    #[test]
    fn redacts_config_secrets() {
        let content = "uri = \"mysql://u:p@localhost/db\"\napi_key = \"sk-test\"";
        assert_eq!(
            redact_config_content(content),
            "uri = \"mysql://u:***@localhost/db\"\napi_key = \"***\""
        );
    }

    #[test]
    fn applies_deepseek_provider_override() {
        let mut config = dbchat_core::AppConfig::default();
        apply_provider_override(&mut config, "deepseek").unwrap();

        assert_eq!(
            config.llm.provider,
            dbchat_core::config::LlmProvider::OpenAICompatible
        );
        assert_eq!(config.llm.model, config_wizard::DEEPSEEK_FLASH_MODEL);
        assert_eq!(
            config.llm.api_url.as_deref(),
            Some(config_wizard::DEEPSEEK_API_BASE_URL)
        );
        assert_eq!(config.llm.api_key.as_deref(), Some("env:DEEPSEEK_API_KEY"));
    }

    #[test]
    fn applies_openrouter_provider_override() {
        let mut config = dbchat_core::AppConfig::default();
        apply_provider_override(&mut config, "openrouter").unwrap();

        assert_eq!(
            config.llm.provider,
            dbchat_core::config::LlmProvider::OpenAICompatible
        );
        assert_eq!(config.llm.model, config_wizard::OPENROUTER_FREE_MODEL);
        assert_eq!(
            config.llm.api_url.as_deref(),
            Some(config_wizard::OPENROUTER_API_BASE_URL)
        );
        assert_eq!(
            config.llm.api_key.as_deref(),
            Some("env:OPENROUTER_API_KEY")
        );
    }

    #[test]
    fn applies_opencode_provider_override_aliases() {
        for provider in ["opencode", "zen", "opencode-zen"] {
            let mut config = dbchat_core::AppConfig::default();
            apply_provider_override(&mut config, provider).unwrap();

            assert_eq!(
                config.llm.provider,
                dbchat_core::config::LlmProvider::OpenAICompatible
            );
            assert_eq!(config.llm.model, config_wizard::OPENCODE_ZEN_FREE_MODEL);
            assert_eq!(
                config.llm.api_url.as_deref(),
                Some(config_wizard::OPENCODE_ZEN_CHAT_COMPLETIONS_URL)
            );
            assert_eq!(config.llm.api_key.as_deref(), Some("env:OPENCODE_API_KEY"));
        }
    }

    #[test]
    fn reads_secure_env_value_from_dotenv_file() {
        let dir = std::env::temp_dir().join(format!(
            "dbchat-main-env-{}-{}",
            std::process::id(),
            unique_test_suffix()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");
        std::fs::write(&path, "OTHER=value\nOPENCODE_API_KEY=\"test secret\"\n").unwrap();

        assert_eq!(
            secure_env_value_from_path(&path, "OPENCODE_API_KEY").unwrap(),
            Some("test secret".to_string())
        );
        assert_eq!(secure_env_value_from_path(&path, "MISSING").unwrap(), None);
    }

    #[test]
    fn standard_provider_override_clears_custom_endpoint() {
        let mut config = dbchat_core::AppConfig::default();
        config.llm.provider = dbchat_core::config::LlmProvider::OpenAICompatible;
        config.llm.api_url = Some("https://api.deepseek.com".to_string());
        config.llm.api_key = Some("env:DEEPSEEK_API_KEY".to_string());

        apply_provider_override(&mut config, "openai").unwrap();

        assert_eq!(
            config.llm.provider,
            dbchat_core::config::LlmProvider::OpenAI
        );
        assert!(config.llm.api_url.is_none());
        assert!(config.llm.api_key.is_none());
    }
}
