use dialoguer::{Confirm, Input, Password};
use std::io::Write;
use std::path::Path;

use dbchat_core::config::{
    AppConfig, DEFAULT_OPENCODE_API_KEY_ENV, DEFAULT_OPENCODE_ZEN_CHAT_COMPLETIONS_URL,
    DEFAULT_OPENCODE_ZEN_MODEL, DbEngine, LlmProvider,
};

use crate::ui::menu::{Flash, TerminalUi};
use crate::ui::theme::{SECRET_MASK, TerminalTheme, mask_secret};

pub const OPENCODE_ZEN_CHAT_COMPLETIONS_URL: &str = DEFAULT_OPENCODE_ZEN_CHAT_COMPLETIONS_URL;
pub const OPENCODE_ZEN_FREE_MODEL: &str = DEFAULT_OPENCODE_ZEN_MODEL;
pub const DEEPSEEK_API_BASE_URL: &str = "https://api.deepseek.com";
pub const DEEPSEEK_FLASH_MODEL: &str = "deepseek-v4-flash";
pub const OPENROUTER_API_BASE_URL: &str = "https://openrouter.ai/api/v1";
pub const OPENROUTER_FREE_MODEL: &str = "openrouter/free";
pub const GEMINI_FLASH_LITE_MODEL: &str = "gemini-3.1-flash-lite";
const GEMINI_FLASH_MODEL: &str = "gemini-3.5-flash";
const OLLAMA_CHAT_URL: &str = "http://localhost:11434/api/chat";

#[derive(Clone, Copy)]
struct LlmPreset {
    label: &'static str,
    provider: LlmProvider,
    model: &'static str,
    api_url: Option<&'static str>,
    api_key_env: &'static str,
}

const FREE_LLM_PRESETS: &[LlmPreset] = &[
    LlmPreset {
        label: "OpenCode Zen DeepSeek V4 Flash Free (Recommended)",
        provider: LlmProvider::OpenAICompatible,
        model: OPENCODE_ZEN_FREE_MODEL,
        api_url: Some(OPENCODE_ZEN_CHAT_COMPLETIONS_URL),
        api_key_env: DEFAULT_OPENCODE_API_KEY_ENV,
    },
    LlmPreset {
        label: "Gemini Flash-Lite (free tier)",
        provider: LlmProvider::Google,
        model: GEMINI_FLASH_LITE_MODEL,
        api_url: None,
        api_key_env: "GOOGLE_API_KEY",
    },
    LlmPreset {
        label: "OpenRouter Free Router",
        provider: LlmProvider::OpenAICompatible,
        model: OPENROUTER_FREE_MODEL,
        api_url: Some(OPENROUTER_API_BASE_URL),
        api_key_env: "OPENROUTER_API_KEY",
    },
    LlmPreset {
        label: "OpenRouter Gemma 4 31B (free)",
        provider: LlmProvider::OpenAICompatible,
        model: "google/gemma-4-31b-it:free",
        api_url: Some(OPENROUTER_API_BASE_URL),
        api_key_env: "OPENROUTER_API_KEY",
    },
    LlmPreset {
        label: "OpenRouter North Mini Code (free)",
        provider: LlmProvider::OpenAICompatible,
        model: "cohere/north-mini-code:free",
        api_url: Some(OPENROUTER_API_BASE_URL),
        api_key_env: "OPENROUTER_API_KEY",
    },
];

const PAID_LLM_PRESETS: &[LlmPreset] = &[
    LlmPreset {
        label: "DeepSeek V4 Flash (low-cost)",
        provider: LlmProvider::OpenAICompatible,
        model: DEEPSEEK_FLASH_MODEL,
        api_url: Some(DEEPSEEK_API_BASE_URL),
        api_key_env: "DEEPSEEK_API_KEY",
    },
    LlmPreset {
        label: "OpenAI GPT-5.4 mini",
        provider: LlmProvider::OpenAI,
        model: "gpt-5.4-mini",
        api_url: None,
        api_key_env: "OPENAI_API_KEY",
    },
    LlmPreset {
        label: "OpenAI GPT-5.5",
        provider: LlmProvider::OpenAI,
        model: "gpt-5.5",
        api_url: None,
        api_key_env: "OPENAI_API_KEY",
    },
    LlmPreset {
        label: "Anthropic Claude Sonnet 5",
        provider: LlmProvider::Anthropic,
        model: "claude-sonnet-5",
        api_url: None,
        api_key_env: "ANTHROPIC_API_KEY",
    },
    LlmPreset {
        label: "Anthropic Claude Haiku 4.5",
        provider: LlmProvider::Anthropic,
        model: "claude-haiku-4-5",
        api_url: None,
        api_key_env: "ANTHROPIC_API_KEY",
    },
    LlmPreset {
        label: "Google Gemini 3.5 Flash",
        provider: LlmProvider::Google,
        model: GEMINI_FLASH_MODEL,
        api_url: None,
        api_key_env: "GOOGLE_API_KEY",
    },
];

pub fn run_config_menu(mut config: AppConfig) -> color_eyre::Result<AppConfig> {
    let ui = TerminalUi::new();

    match run_config_menu_loop(&ui, &mut config) {
        Ok(()) => Ok(config),
        Err(err) if is_interrupted_error(&err) => {
            println!();
            ui.flash(&Flash::warning("Configuration cancelled"));
            Ok(config)
        }
        Err(err) => Err(err),
    }
}

fn run_config_menu_loop(ui: &TerminalUi, config: &mut AppConfig) -> color_eyre::Result<()> {
    let mut flash = None;

    loop {
        ui.reset("DB connection > LLM > Query safety")?;
        print_current_config(ui, config);
        if let Some(message) = &flash {
            ui.flash(message);
        }

        let items = [
            "1. Database connection",
            "2. LLM",
            "3. Query safety",
            "4. Show config",
            "5. Test configuration",
            "6. Quit",
        ];

        ui.footer();
        let choice = ui.select("What would you like to configure?", &items, 0)?;
        flash = None;

        match choice {
            Some(0) => {
                if configure_database(ui, config)? {
                    save_config(config)?;
                    flash = Some(Flash::success("Database connection saved"));
                }
            }
            Some(1) => {
                if configure_llm(ui, config)? {
                    save_config(config)?;
                    flash = Some(Flash::success("LLM saved"));
                }
            }
            Some(2) => {
                if configure_query_safety(ui, config)? {
                    save_config(config)?;
                    flash = Some(Flash::success("Query safety saved"));
                }
            }
            Some(3) => show_config_screen(ui, config)?,
            Some(4) => test_config_screen(ui, config)?,
            Some(5) | None => break,
            _ => {}
        }
    }

    Ok(())
}

pub fn print_config_summary(config: &AppConfig) {
    let theme = TerminalTheme::new();
    println!("{}", theme.info("Active config:"));
    for line in detailed_config_lines(config) {
        println!("  {line}");
    }
    println!();
}

fn print_current_config(ui: &TerminalUi, config: &AppConfig) {
    println!("{}", ui.theme().bold("Current configuration:"));
    for line in current_config_lines(config) {
        println!("- {line}");
    }
    println!();
}

fn current_config_lines(config: &AppConfig) -> Vec<String> {
    vec![
        format!("DB: {}", database_status(config)),
        format!("LLM: {}", llm_display_name(config)),
        format!("API key: {}", api_key_status(config)),
        format!("Query safety: {}", query_safety_status(config)),
    ]
}

fn detailed_config_lines(config: &AppConfig) -> Vec<String> {
    vec![
        format!("Path: {}", AppConfig::config_path().display()),
        format!("DB: {} {}", config.db.engine, database_uri_display(config)),
        format!(
            "LLM: {} / {}",
            llm_provider_name(&config.llm.provider),
            config.llm.model
        ),
        format!(
            "API URL: {}",
            config.llm.api_url.as_deref().unwrap_or("provider default")
        ),
        format!("API key: {}", api_key_status(config)),
        format!(
            "Ops: read_only={}, safe_mode={}, max_rows={}, timeout={}s",
            config.db.read_only,
            config.db.safe_mode,
            config.db.max_rows,
            config.db.query_timeout_secs
        ),
    ]
}

fn configure_database(ui: &TerminalUi, config: &mut AppConfig) -> color_eyre::Result<bool> {
    let mut proposed = config.clone();

    ui.reset("DB connection")?;
    let engines = ["PostgreSQL", "MySQL", "SQLite", "Back"];
    let default = match config.db.engine {
        DbEngine::Postgres => 0,
        DbEngine::Mysql => 1,
        DbEngine::Sqlite => 2,
    };

    ui.footer();
    let Some(selected) = ui.select("Database type", &engines, default)? else {
        return Ok(false);
    };
    if selected == engines.len() - 1 {
        return Ok(false);
    }

    ui.reset("DB connection > Settings")?;
    match selected {
        0 => {
            proposed.db.engine = DbEngine::Postgres;
            proposed.db.uri = Input::with_theme(ui.prompt_theme())
                .with_prompt("PostgreSQL URI")
                .default(non_empty_or(
                    &config.db.uri,
                    "postgres://user:pass@localhost/db",
                ))
                .interact_text()?;
        }
        1 => {
            proposed.db.engine = DbEngine::Mysql;
            proposed.db.uri = Input::with_theme(ui.prompt_theme())
                .with_prompt("MySQL URI")
                .default(non_empty_or(
                    &config.db.uri,
                    "mysql://user:pass@localhost:3306/db",
                ))
                .interact_text()?;
        }
        _ => {
            proposed.db.engine = DbEngine::Sqlite;
            let current = config
                .db
                .uri
                .strip_prefix("sqlite://")
                .unwrap_or(&config.db.uri);
            let path: String = Input::with_theme(ui.prompt_theme())
                .with_prompt("SQLite path")
                .default(non_empty_or(current, "./db.sqlite"))
                .interact_text()?;
            proposed.db.uri = if path.starts_with("sqlite://") {
                path
            } else {
                format!("sqlite://{path}")
            };
        }
    }

    ui.reset("DB connection > Summary")?;
    println!("Engine: {}", proposed.db.engine);
    println!("URI   : {}", crate::redact_uri(&proposed.db.uri));
    println!();

    let save = Confirm::with_theme(ui.prompt_theme())
        .with_prompt("Save this connection?")
        .default(true)
        .interact()?;
    if save {
        *config = proposed;
    }
    Ok(save)
}

fn configure_llm(ui: &TerminalUi, config: &mut AppConfig) -> color_eyre::Result<bool> {
    loop {
        ui.reset("LLM")?;
        let sections = [
            "Free models",
            "Common paid models",
            "OpenAI-compatible custom",
            "Local model",
            "Back",
        ];
        let default = llm_tier_default(config);

        ui.footer();
        let Some(selected) = ui.select("Provider / tier", &sections, default)? else {
            return Ok(false);
        };

        let proposed = match selected {
            0 => configure_llm_preset_flow(ui, config, FREE_LLM_PRESETS, "LLM > Free models")?,
            1 => configure_llm_preset_flow(ui, config, PAID_LLM_PRESETS, "LLM > Paid models")?,
            2 => configure_openai_compatible_custom(ui, config)?,
            3 => configure_local_model(ui, config)?,
            _ => return Ok(false),
        };

        if let Some(proposed) = proposed {
            *config = proposed;
            return Ok(true);
        }
    }
}

fn configure_llm_preset_flow(
    ui: &TerminalUi,
    config: &AppConfig,
    presets: &[LlmPreset],
    trail: &str,
) -> color_eyre::Result<Option<AppConfig>> {
    ui.reset(trail)?;
    let mut labels: Vec<String> = presets
        .iter()
        .map(|preset| preset.label.to_string())
        .collect();
    labels.push("Back".to_string());
    let default = preset_index(config, presets).unwrap_or(0);

    ui.footer();
    let Some(selected) = ui.select("Select model", &labels, default)? else {
        return Ok(None);
    };
    if selected == labels.len() - 1 {
        return Ok(None);
    }

    let preset = presets[selected];
    let mut proposed = config.clone();
    apply_llm_preset(&mut proposed, &preset);

    if !configure_api_key(ui, &mut proposed, preset.api_key_env)? {
        return Ok(None);
    }

    if confirm_llm_summary(ui, &proposed)? {
        Ok(Some(proposed))
    } else {
        Ok(None)
    }
}

fn configure_openai_compatible_custom(
    ui: &TerminalUi,
    config: &AppConfig,
) -> color_eyre::Result<Option<AppConfig>> {
    let mut proposed = config.clone();

    ui.reset("LLM > OpenAI-compatible custom")?;
    proposed.llm.provider = LlmProvider::OpenAICompatible;
    proposed.llm.api_url = Some(
        Input::with_theme(ui.prompt_theme())
            .with_prompt("Base URL or /chat/completions URL")
            .default(
                config
                    .llm
                    .api_url
                    .clone()
                    .unwrap_or_else(|| OPENCODE_ZEN_CHAT_COMPLETIONS_URL.to_string()),
            )
            .interact_text()?,
    );
    proposed.llm.model = prompt_model(ui, &config.llm.model, OPENCODE_ZEN_FREE_MODEL)?;
    proposed.llm.temperature = prompt_temperature(ui, config.llm.temperature)?;

    let default_env = proposed
        .llm_api_key_env()
        .unwrap_or(DEFAULT_OPENCODE_API_KEY_ENV);
    if !configure_api_key(ui, &mut proposed, default_env)? {
        return Ok(None);
    }

    if confirm_llm_summary(ui, &proposed)? {
        Ok(Some(proposed))
    } else {
        Ok(None)
    }
}

fn configure_local_model(
    ui: &TerminalUi,
    config: &AppConfig,
) -> color_eyre::Result<Option<AppConfig>> {
    let mut proposed = config.clone();

    ui.reset("LLM > Local model")?;
    proposed.llm.provider = LlmProvider::Ollama;
    proposed.llm.model = prompt_model(ui, &config.llm.model, "llama3")?;
    proposed.llm.api_url = Some(
        Input::with_theme(ui.prompt_theme())
            .with_prompt("Local endpoint")
            .default(
                config
                    .llm
                    .api_url
                    .clone()
                    .unwrap_or_else(|| OLLAMA_CHAT_URL.to_string()),
            )
            .interact_text()?,
    );
    proposed.llm.temperature = prompt_temperature(ui, config.llm.temperature)?;
    proposed.llm.api_key = None;

    if confirm_llm_summary(ui, &proposed)? {
        Ok(Some(proposed))
    } else {
        Ok(None)
    }
}

fn configure_query_safety(ui: &TerminalUi, config: &mut AppConfig) -> color_eyre::Result<bool> {
    let mut proposed = config.clone();

    ui.reset("Query safety")?;
    let choices = ["Edit settings", "Back"];
    ui.footer();
    let Some(selected) = ui.select("Action", &choices, 0)? else {
        return Ok(false);
    };
    if selected == 1 {
        return Ok(false);
    }

    ui.reset("Query safety > Settings")?;
    proposed.db.read_only = Confirm::with_theme(ui.prompt_theme())
        .with_prompt("Block destructive queries?")
        .default(config.db.read_only)
        .interact()?;
    proposed.db.safe_mode = Confirm::with_theme(ui.prompt_theme())
        .with_prompt("Ask confirmation before dangerous queries?")
        .default(config.db.safe_mode)
        .interact()?;
    proposed.db.max_rows = Input::with_theme(ui.prompt_theme())
        .with_prompt("Max rows")
        .default(config.db.max_rows)
        .interact_text()?;
    proposed.db.query_timeout_secs = Input::with_theme(ui.prompt_theme())
        .with_prompt("Query timeout (seconds)")
        .default(config.db.query_timeout_secs)
        .interact_text()?;

    ui.reset("Query safety > Summary")?;
    println!("Read only        : {}", yes_no(proposed.db.read_only));
    println!("Confirmations    : {}", yes_no(proposed.db.safe_mode));
    println!("Max rows         : {}", proposed.db.max_rows);
    println!("Timeout          : {}s", proposed.db.query_timeout_secs);
    println!();

    let save = Confirm::with_theme(ui.prompt_theme())
        .with_prompt("Save these settings?")
        .default(true)
        .interact()?;
    if save {
        *config = proposed;
    }
    Ok(save)
}

fn configure_api_key(
    ui: &TerminalUi,
    config: &mut AppConfig,
    default_env: &str,
) -> color_eyre::Result<bool> {
    ui.reset("LLM > API key")?;
    let env_name = current_api_key_env(config, default_env);
    println!("Model   : {}", config.llm.model);
    println!("Provider: {}", llm_provider_name(&config.llm.provider));
    println!("Current : {}", api_key_status(config));
    println!();

    let choices = [
        format!("Use environment variable {env_name}"),
        "Store in dbchat secure file".to_string(),
        "Clear / do not configure key".to_string(),
        "Back".to_string(),
    ];

    ui.footer();
    let Some(selected) = ui.select("API key", &choices, 0)? else {
        return Ok(false);
    };

    match selected {
        0 => {
            ui.reset("LLM > API key > Environment variable")?;
            let env_name: String = Input::with_theme(ui.prompt_theme())
                .with_prompt("Environment variable name")
                .default(env_name)
                .interact_text()?;
            validate_env_name(&env_name)?;
            config.llm.api_key = Some(format!("env:{env_name}"));
            Ok(true)
        }
        1 => {
            ui.reset("LLM > API key > Secure storage")?;
            println!("File  : {}", AppConfig::env_path().display());
            println!("Value : {SECRET_MASK}");
            println!();

            let env_name: String = Input::with_theme(ui.prompt_theme())
                .with_prompt("Environment variable name")
                .default(env_name)
                .interact_text()?;
            validate_env_name(&env_name)?;

            let key = Password::with_theme(ui.prompt_theme())
                .with_prompt("API key")
                .allow_empty_password(true)
                .interact()?;
            if key.trim().is_empty() {
                ui.flash(&Flash::warning("Empty key. Nothing saved."));
                ui.wait_for_enter()?;
                return Ok(false);
            }

            write_secure_env_key(&env_name, &key)?;
            config.llm.api_key = Some(format!("env:{env_name}"));

            if !secure_env_file_is_private(&AppConfig::env_path())? {
                ui.flash(&Flash::warning(
                    "The secret file exists but its permissions could not be verified.",
                ));
            }
            Ok(true)
        }
        2 => {
            config.llm.api_key = None;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn show_config_screen(ui: &TerminalUi, config: &AppConfig) -> color_eyre::Result<()> {
    ui.reset("Show config")?;
    println!("{}", ui.theme().bold("Active configuration:"));
    for line in detailed_config_lines(config) {
        println!("- {line}");
    }
    ui.wait_for_enter()
}

fn test_config_screen(ui: &TerminalUi, config: &AppConfig) -> color_eyre::Result<()> {
    ui.reset("Test configuration")?;
    println!("{}", ui.theme().bold("Local checks:"));
    for check in config_checks(config) {
        let icon = match check.kind {
            CheckKind::Ok => ui.theme().check(),
            CheckKind::Warning => ui.theme().warn_icon(),
            CheckKind::Error => ui.theme().cross(),
        };
        println!("{icon} {}", check.message);
    }
    ui.wait_for_enter()
}

fn confirm_llm_summary(ui: &TerminalUi, config: &AppConfig) -> color_eyre::Result<bool> {
    ui.reset("LLM > Summary")?;
    println!("Provider : {}", llm_provider_name(&config.llm.provider));
    println!("Model    : {}", config.llm.model);
    println!(
        "Endpoint : {}",
        config.llm.api_url.as_deref().unwrap_or("provider default")
    );
    println!("API key  : {}", api_key_status(config));
    println!("Temp.    : {}", config.llm.temperature);
    println!();

    Ok(Confirm::with_theme(ui.prompt_theme())
        .with_prompt("Save this LLM configuration?")
        .default(true)
        .interact()?)
}

fn apply_llm_preset(config: &mut AppConfig, preset: &LlmPreset) {
    config.llm.provider = preset.provider;
    config.llm.model = preset.model.to_string();
    config.llm.temperature = 0.0;
    config.llm.api_url = preset.api_url.map(str::to_string);
    config.llm.api_key = Some(format!("env:{}", preset.api_key_env));
}

fn preset_index(config: &AppConfig, presets: &[LlmPreset]) -> Option<usize> {
    presets.iter().position(|preset| {
        preset.provider == config.llm.provider
            && preset.model == config.llm.model
            && preset.api_url == config.llm.api_url.as_deref()
    })
}

fn llm_tier_default(config: &AppConfig) -> usize {
    if preset_index(config, FREE_LLM_PRESETS).is_some() {
        0
    } else if preset_index(config, PAID_LLM_PRESETS).is_some() {
        1
    } else if config.llm.provider == LlmProvider::Ollama {
        3
    } else {
        2
    }
}

fn prompt_model(ui: &TerminalUi, current: &str, fallback: &str) -> color_eyre::Result<String> {
    Ok(Input::with_theme(ui.prompt_theme())
        .with_prompt("Model")
        .default(non_empty_or(current, fallback))
        .interact_text()?)
}

fn prompt_temperature(ui: &TerminalUi, current: f64) -> color_eyre::Result<f64> {
    Ok(Input::with_theme(ui.prompt_theme())
        .with_prompt("Temperature")
        .default(current)
        .interact_text()?)
}

fn save_config(config: &AppConfig) -> color_eyre::Result<()> {
    config.save().map_err(|err| color_eyre::eyre::eyre!(err))
}

fn is_interrupted_error(err: &color_eyre::Report) -> bool {
    err.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .map(|io| io.kind() == std::io::ErrorKind::Interrupted)
            .unwrap_or(false)
            || cause.to_string().to_lowercase().contains("interrupted")
    })
}

fn database_status(config: &AppConfig) -> String {
    if config.has_database_uri() {
        format!("configured ({})", config.db.engine)
    } else {
        "not configured".to_string()
    }
}

fn database_uri_display(config: &AppConfig) -> String {
    if config.has_database_uri() {
        crate::redact_uri(&config.db.uri)
    } else {
        "not configured".to_string()
    }
}

fn llm_display_name(config: &AppConfig) -> String {
    preset_label(config)
        .map(strip_recommended_suffix)
        .unwrap_or_else(|| {
            format!(
                "{} {}",
                llm_provider_name(&config.llm.provider),
                config.llm.model
            )
        })
}

fn preset_label(config: &AppConfig) -> Option<&'static str> {
    FREE_LLM_PRESETS
        .iter()
        .chain(PAID_LLM_PRESETS.iter())
        .find(|preset| {
            preset.provider == config.llm.provider
                && preset.model == config.llm.model
                && preset.api_url == config.llm.api_url.as_deref()
        })
        .map(|preset| preset.label)
}

fn strip_recommended_suffix(label: &str) -> String {
    label.replace(" (Recommended)", "")
}

fn query_safety_status(config: &AppConfig) -> &'static str {
    if config.db.read_only || config.db.safe_mode {
        "enabled"
    } else {
        "disabled"
    }
}

fn api_key_status(config: &AppConfig) -> String {
    api_key_status_for_path(config, &AppConfig::env_path())
}

fn api_key_status_for_path(config: &AppConfig, secure_env_path: &Path) -> String {
    match api_key_source_for_path(config, secure_env_path) {
        ApiKeySource::EnvVar(name) => name,
        ApiKeySource::SecureFile(name) => format!("secure file ({name})"),
        ApiKeySource::InlineSecret => mask_secret(Some("stored")).to_string(),
        ApiKeySource::MissingEnv(name) => format!("{name} (missing)"),
        ApiKeySource::NotConfigured => mask_secret(None).to_string(),
        ApiKeySource::NotRequired => "not required".to_string(),
    }
}

fn api_key_source_for_path(config: &AppConfig, secure_env_path: &Path) -> ApiKeySource {
    if config.llm.provider == LlmProvider::Ollama {
        return ApiKeySource::NotRequired;
    }

    if let Some(value) = config.llm.api_key.as_deref() {
        if let Some(name) = value.strip_prefix("env:") {
            return env_api_key_source(name, secure_env_path);
        }
        if !value.trim().is_empty() {
            return ApiKeySource::InlineSecret;
        }
    }

    config
        .llm_api_key_env()
        .map(|name| env_api_key_source(name, secure_env_path))
        .unwrap_or(ApiKeySource::NotConfigured)
}

fn env_api_key_source(name: &str, secure_env_path: &Path) -> ApiKeySource {
    if std::env::var(name)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return ApiKeySource::EnvVar(name.to_string());
    }

    match secure_env_value_from_path(secure_env_path, name) {
        Ok(Some(value)) if !value.trim().is_empty() => ApiKeySource::SecureFile(name.to_string()),
        _ => ApiKeySource::MissingEnv(name.to_string()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ApiKeySource {
    EnvVar(String),
    SecureFile(String),
    InlineSecret,
    MissingEnv(String),
    NotConfigured,
    NotRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckKind {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfigCheck {
    kind: CheckKind,
    message: String,
}

fn config_checks(config: &AppConfig) -> Vec<ConfigCheck> {
    let mut checks = Vec::new();

    if !config.has_database_uri() {
        checks.push(ConfigCheck {
            kind: CheckKind::Error,
            message: "DB: not configured. Configure a database connection.".to_string(),
        });
    } else if AppConfig::from_uri(&config.db.uri).is_ok() {
        checks.push(ConfigCheck {
            kind: CheckKind::Ok,
            message: format!("DB: {} configured", config.db.engine),
        });
    } else {
        checks.push(ConfigCheck {
            kind: CheckKind::Error,
            message: "DB: unknown URI scheme. Fix the database connection.".to_string(),
        });
    }

    if config.llm.model.trim().is_empty() {
        checks.push(ConfigCheck {
            kind: CheckKind::Error,
            message: "LLM: missing model. Choose a model.".to_string(),
        });
    } else {
        checks.push(ConfigCheck {
            kind: CheckKind::Ok,
            message: format!("LLM: {}", llm_display_name(config)),
        });
    }

    match api_key_source_for_path(config, &AppConfig::env_path()) {
        ApiKeySource::EnvVar(name) => checks.push(ConfigCheck {
            kind: CheckKind::Ok,
            message: format!("API key: {name} available in environment"),
        }),
        ApiKeySource::SecureFile(name) => checks.push(ConfigCheck {
            kind: CheckKind::Ok,
            message: format!("API key: secure file ({name})"),
        }),
        ApiKeySource::InlineSecret => checks.push(ConfigCheck {
            kind: CheckKind::Ok,
            message: format!("API key: {SECRET_MASK}"),
        }),
        ApiKeySource::MissingEnv(name) => checks.push(ConfigCheck {
            kind: CheckKind::Warning,
            message: format!("API key: {name} missing. Export it or save via LLM configuration."),
        }),
        ApiKeySource::NotConfigured => checks.push(ConfigCheck {
            kind: CheckKind::Warning,
            message: "API key: not configured.".to_string(),
        }),
        ApiKeySource::NotRequired => checks.push(ConfigCheck {
            kind: CheckKind::Ok,
            message: "API key: not required for local model".to_string(),
        }),
    }

    if config.db.read_only || config.db.safe_mode {
        checks.push(ConfigCheck {
            kind: CheckKind::Ok,
            message: "Query safety: enabled".to_string(),
        });
    } else {
        checks.push(ConfigCheck {
            kind: CheckKind::Warning,
            message: "Query safety: disabled. Enable read_only or safe_mode.".to_string(),
        });
    }

    checks
}

fn current_api_key_env(config: &AppConfig, default_env: &str) -> String {
    config
        .llm
        .api_key
        .as_deref()
        .and_then(|value| value.strip_prefix("env:"))
        .or_else(|| config.llm_api_key_env())
        .unwrap_or(default_env)
        .to_string()
}

fn validate_env_name(name: &str) -> color_eyre::Result<()> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(color_eyre::eyre::eyre!(
            "Environment variable name cannot be empty"
        ));
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err(color_eyre::eyre::eyre!("Invalid variable name: {name}"));
    }
    if !chars.all(|c| c == '_' || c.is_ascii_alphanumeric()) {
        return Err(color_eyre::eyre::eyre!("Invalid variable name: {name}"));
    }
    Ok(())
}

fn write_secure_env_key(name: &str, value: &str) -> color_eyre::Result<()> {
    AppConfig::ensure_config_dir().map_err(|err| color_eyre::eyre::eyre!(err))?;
    write_env_key(&AppConfig::env_path(), name, value)?;
    Ok(())
}

fn write_env_key(path: &Path, name: &str, value: &str) -> color_eyre::Result<()> {
    let new_line = format!("{name}={}", quote_dotenv_value(value));
    let mut updated = false;
    let mut lines = if path.exists() {
        std::fs::read_to_string(path)?
            .lines()
            .map(|line| {
                if dotenv_line_matches_key(line, name) {
                    updated = true;
                    new_line.clone()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    if !updated {
        lines.push(new_line);
    }

    write_private_file(path, &(lines.join("\n") + "\n"))?;
    Ok(())
}

fn write_private_file(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(content.as_bytes())?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        file.write_all(content.as_bytes())
    }
}

fn secure_env_file_is_private(path: &Path) -> color_eyre::Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path)?.permissions().mode() & 0o777;
        Ok(mode & 0o077 == 0)
    }

    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(true)
    }
}

fn secure_env_value_from_path(path: &Path, name: &str) -> color_eyre::Result<Option<String>> {
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

fn dotenv_line_matches_key(line: &str, key: &str) -> bool {
    let trimmed = line.trim_start();
    let trimmed = trimmed.strip_prefix("export ").unwrap_or(trimmed);
    trimmed
        .split_once('=')
        .map(|(name, _)| name.trim() == key)
        .unwrap_or(false)
}

fn quote_dotenv_value(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{escaped}\"")
}

fn non_empty_or(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn llm_provider_name(provider: &LlmProvider) -> &'static str {
    match provider {
        LlmProvider::OpenAI => "OpenAI",
        LlmProvider::Anthropic => "Anthropic",
        LlmProvider::Ollama => "Ollama",
        LlmProvider::Google => "Google",
        LlmProvider::OpenAICompatible => "OpenAI-compatible",
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_presets_are_api_key_based() {
        assert!(
            FREE_LLM_PRESETS
                .iter()
                .all(|preset| preset.provider != LlmProvider::Ollama)
        );
        assert!(
            FREE_LLM_PRESETS
                .iter()
                .all(|preset| !preset.api_key_env.is_empty())
        );
    }

    #[test]
    fn presets_include_simple_free_and_common_paid_models() {
        let free_models: Vec<&str> = FREE_LLM_PRESETS.iter().map(|preset| preset.model).collect();
        assert_eq!(FREE_LLM_PRESETS[0].model, OPENCODE_ZEN_FREE_MODEL);
        assert!(free_models.contains(&GEMINI_FLASH_LITE_MODEL));
        assert!(free_models.contains(&OPENROUTER_FREE_MODEL));
        assert!(free_models.contains(&"google/gemma-4-31b-it:free"));

        let paid_models: Vec<&str> = PAID_LLM_PRESETS.iter().map(|preset| preset.model).collect();
        assert!(paid_models.contains(&DEEPSEEK_FLASH_MODEL));
        assert!(paid_models.contains(&"gpt-5.4-mini"));
        assert!(paid_models.contains(&"claude-sonnet-5"));
    }

    #[test]
    fn preset_sets_provider_endpoint_and_env_api_key() {
        let mut config = AppConfig::default();
        config.llm.temperature = 0.7;
        let preset = FREE_LLM_PRESETS
            .iter()
            .find(|preset| preset.model == OPENROUTER_FREE_MODEL)
            .unwrap();

        apply_llm_preset(&mut config, preset);

        assert_eq!(config.llm.provider, LlmProvider::OpenAICompatible);
        assert_eq!(config.llm.model, OPENROUTER_FREE_MODEL);
        assert_eq!(config.llm.temperature, 0.0);
        assert_eq!(config.llm.api_url.as_deref(), Some(OPENROUTER_API_BASE_URL));
        assert_eq!(
            config.llm.api_key.as_deref(),
            Some("env:OPENROUTER_API_KEY")
        );
    }

    #[test]
    fn preset_sets_opencode_endpoint_and_env_api_key() {
        let mut config = AppConfig::default();

        apply_llm_preset(&mut config, &FREE_LLM_PRESETS[0]);

        assert_eq!(config.llm.provider, LlmProvider::OpenAICompatible);
        assert_eq!(config.llm.model, OPENCODE_ZEN_FREE_MODEL);
        assert_eq!(
            config.llm.api_url.as_deref(),
            Some(OPENCODE_ZEN_CHAT_COMPLETIONS_URL)
        );
        assert_eq!(config.llm.api_key.as_deref(), Some("env:OPENCODE_API_KEY"));
    }

    #[test]
    fn current_config_overview_masks_inline_api_key() {
        let mut config = AppConfig::default();
        config.llm.api_key = Some("sk-test-secret".to_string());

        let rendered = current_config_lines(&config).join("\n");

        assert!(rendered.contains(SECRET_MASK));
        assert!(!rendered.contains("sk-test-secret"));
    }

    #[test]
    fn detailed_config_masks_database_password_and_inline_api_key() {
        let mut config = AppConfig::default();
        config.db.uri = "mysql://user:password@localhost/db".to_string();
        config.llm.api_key = Some("sk-test-secret".to_string());

        let rendered = detailed_config_lines(&config).join("\n");

        assert!(rendered.contains("mysql://user:***@localhost/db"));
        assert!(rendered.contains(SECRET_MASK));
        assert!(!rendered.contains("password"));
        assert!(!rendered.contains("sk-test-secret"));
    }

    #[test]
    fn api_key_status_can_report_secure_env_file_without_secret() -> color_eyre::Result<()> {
        let dir = std::env::temp_dir().join(format!(
            "dbchat-secure-status-{}-{}",
            std::process::id(),
            unique_test_suffix()
        ));
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(".env");
        std::fs::write(&path, "DBCHAT_TEST_STATUS_KEY=\"hidden secret\"\n")?;

        let mut config = AppConfig::default();
        config.llm.api_key = Some("env:DBCHAT_TEST_STATUS_KEY".to_string());

        let status = api_key_status_for_path(&config, &path);

        assert_eq!(status, "secure file (DBCHAT_TEST_STATUS_KEY)");
        assert!(!status.contains("hidden secret"));
        Ok(())
    }

    #[test]
    fn config_checks_report_missing_api_key_without_value() {
        let mut config = AppConfig::default();
        config.db.uri = "postgres://user:pass@localhost/db".to_string();
        config.llm.api_key = Some("env:DBCHAT_TEST_MISSING_API_KEY_DO_NOT_SET".to_string());

        let rendered = config_checks(&config)
            .into_iter()
            .map(|check| check.message)
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("DBCHAT_TEST_MISSING_API_KEY_DO_NOT_SET missing"));
        assert!(!rendered.contains("pass"));
    }

    #[test]
    fn secure_env_writer_updates_key_without_dropping_others() -> color_eyre::Result<()> {
        let dir = std::env::temp_dir().join(format!(
            "dbchat-env-writer-{}-{}",
            std::process::id(),
            unique_test_suffix()
        ));
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(".env");
        std::fs::write(
            &path,
            "KEEP=this-stays\nOPENCODE_API_KEY=\"old\"\n# comment\n",
        )?;

        write_env_key(&path, "OPENCODE_API_KEY", "new secret")?;

        let content = std::fs::read_to_string(&path)?;
        assert!(content.contains("KEEP=this-stays"));
        assert!(content.contains("# comment"));
        assert!(content.contains("OPENCODE_API_KEY=\"new secret\""));
        assert!(!content.contains("old"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path)?.permissions().mode() & 0o777,
                0o600
            );
        }

        Ok(())
    }

    #[test]
    fn secure_env_writer_quotes_special_characters() -> color_eyre::Result<()> {
        let dir = std::env::temp_dir().join(format!(
            "dbchat-env-writer-quotes-{}-{}",
            std::process::id(),
            unique_test_suffix()
        ));
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(".env");

        write_env_key(&path, "OPENCODE_API_KEY", "sk-\"quoted\"\\value")?;

        let content = std::fs::read_to_string(&path)?;
        assert_eq!(content, "OPENCODE_API_KEY=\"sk-\\\"quoted\\\"\\\\value\"\n");
        Ok(())
    }

    fn unique_test_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
}
