use clap::{Parser, Subcommand};

mod render;
mod repl;

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

    /// Model override (e.g. gpt-4o-mini, claude-3-haiku)
    #[arg(long)]
    model: Option<String>,

    /// LLM provider (openai, anthropic, ollama)
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

    if let Some(DbCommand::Config { .. }) = &cli.command {
        return handle_config(&cli);
    }

    let uri = resolve_uri(&cli)?;

    // Load config file, then override with CLI args
    let mut config = dbchat_core::AppConfig::load();
    if let Ok(uri_config) = dbchat_core::AppConfig::from_uri(&uri) {
        config.db = uri_config.db;
    }
    config.resolve_llm_api_key();

    if cli.read_only {
        config.db.read_only = true;
    }
    if cli.verbose {
        config.display.verbose = true;
    }
    if cli.no_color {
        // NO_COLOR will be checked at render time
    }
    if let Some(ref model) = cli.model {
        config.llm.model = model.clone();
    }
    if let Some(ref provider) = cli.provider {
        config.llm.provider = match provider.to_lowercase().as_str() {
            "openai" => dbchat_core::config::LlmProvider::OpenAI,
            "anthropic" => dbchat_core::config::LlmProvider::Anthropic,
            "ollama" => dbchat_core::config::LlmProvider::Ollama,
            "google" => dbchat_core::config::LlmProvider::Google,
            _ => {
                return Err(color_eyre::eyre::eyre!(
                    "Provider inconnu : {provider} (openai, anthropic, ollama, google)"
                ));
            }
        };
    }

    let locale = config.display.locale.clone();

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
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&path, &content)?;
        println!("\x1b[32m✓\x1b[0m Created: {}", path.display());
        println!("\x1b[2m── Default config ──\x1b[0m");
        println!("{content}");
    }
    Ok(())
}

fn redact_uri(uri: &str) -> String {
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

fn redact_config_content(content: &str) -> String {
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
}

fn resolve_uri(cli: &Cli) -> Result<String, color_eyre::eyre::Error> {
    if let Some(uri) = &cli.connection_string {
        return Ok(uri.clone());
    }

    if let Some(ref cmd) = cli.command {
        return match cmd {
            DbCommand::Postgres { uri } => Ok(uri.clone()),
            DbCommand::Mysql { uri } => Ok(uri.clone()),
            DbCommand::Sqlite { path } => Ok(format!("sqlite://{path}")),
            DbCommand::Config { .. } => unreachable!(),
        };
    }

    Err(color_eyre::eyre::eyre!(
        "Aucune URI de connexion fournie.\n\
         Utilisation: dbchat postgres://user:pass@host/db\n\
         Ou: dbchat \"postgres://user:pass@host/db\"\n\
         Ou: dbchat config init"
    ))
}
