use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Password, Select};
use std::io::Write;
use std::path::Path;

use dbchat_core::config::{
    AppConfig, DEFAULT_OPENCODE_API_KEY_ENV, DEFAULT_OPENCODE_ZEN_CHAT_COMPLETIONS_URL,
    DEFAULT_OPENCODE_ZEN_MODEL, DbEngine, LlmProvider,
};

pub const OPENCODE_ZEN_CHAT_COMPLETIONS_URL: &str = DEFAULT_OPENCODE_ZEN_CHAT_COMPLETIONS_URL;
pub const OPENCODE_ZEN_FREE_MODEL: &str = DEFAULT_OPENCODE_ZEN_MODEL;
pub const DEEPSEEK_API_BASE_URL: &str = "https://api.deepseek.com";
pub const DEEPSEEK_FLASH_MODEL: &str = "deepseek-v4-flash";
pub const OPENROUTER_API_BASE_URL: &str = "https://openrouter.ai/api/v1";
pub const OPENROUTER_FREE_MODEL: &str = "openrouter/free";
pub const GEMINI_FLASH_LITE_MODEL: &str = "gemini-3.1-flash-lite";
const GEMINI_FLASH_MODEL: &str = "gemini-3.5-flash";

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
    let theme = ColorfulTheme::default();

    loop {
        let items = [
            "1. Connexion BD",
            "2. LLM",
            "3. Securite requetes",
            "4. Afficher config",
            "5. Quitter",
        ];
        let choice = Select::with_theme(&theme)
            .with_prompt("Que veux-tu configurer ?")
            .items(&items)
            .default(0)
            .interact()?;

        match choice {
            0 => {
                configure_database(&theme, &mut config)?;
                save_config(&config)?;
            }
            1 => {
                configure_llm(&theme, &mut config)?;
                save_config(&config)?;
            }
            2 => {
                configure_query_safety(&theme, &mut config)?;
                save_config(&config)?;
            }
            3 => print_config_summary(&config),
            _ => break,
        }
    }

    Ok(config)
}

pub fn print_config_summary(config: &AppConfig) {
    println!("\x1b[1mConfig active:\x1b[0m");
    println!("  Path: {}", AppConfig::config_path().display());
    println!(
        "  DB:   {} {}",
        config.db.engine,
        crate::redact_uri(&config.db.uri)
    );
    println!(
        "  LLM:  {} / {}",
        llm_provider_name(&config.llm.provider),
        config.llm.model
    );
    println!(
        "  Ops:  read_only={}, safe_mode={}, max_rows={}, timeout={}s",
        config.db.read_only, config.db.safe_mode, config.db.max_rows, config.db.query_timeout_secs
    );
    println!();
}

fn configure_database(theme: &ColorfulTheme, config: &mut AppConfig) -> color_eyre::Result<()> {
    let engines = ["PostgreSQL", "MySQL", "SQLite"];
    let default = match config.db.engine {
        DbEngine::Postgres => 0,
        DbEngine::Mysql => 1,
        DbEngine::Sqlite => 2,
    };
    let selected = Select::with_theme(theme)
        .with_prompt("Type de base de donnees")
        .items(&engines)
        .default(default)
        .interact()?;

    match selected {
        0 => {
            config.db.engine = DbEngine::Postgres;
            config.db.uri = Input::with_theme(theme)
                .with_prompt("URI PostgreSQL")
                .default(non_empty_or(
                    &config.db.uri,
                    "postgres://user:pass@localhost/db",
                ))
                .interact_text()?;
        }
        1 => {
            config.db.engine = DbEngine::Mysql;
            config.db.uri = Input::with_theme(theme)
                .with_prompt("URI MySQL")
                .default(non_empty_or(
                    &config.db.uri,
                    "mysql://user:pass@localhost:3306/db",
                ))
                .interact_text()?;
        }
        _ => {
            config.db.engine = DbEngine::Sqlite;
            let current = config
                .db
                .uri
                .strip_prefix("sqlite://")
                .unwrap_or(&config.db.uri);
            let path: String = Input::with_theme(theme)
                .with_prompt("Chemin SQLite")
                .default(non_empty_or(current, "./db.sqlite"))
                .interact_text()?;
            config.db.uri = if path.starts_with("sqlite://") {
                path
            } else {
                format!("sqlite://{path}")
            };
        }
    }

    Ok(())
}

fn configure_llm(theme: &ColorfulTheme, config: &mut AppConfig) -> color_eyre::Result<()> {
    let sections = [
        "Gratuits / free tier",
        "Payants courants",
        "Avance / custom API",
    ];
    let default = if preset_index(config, FREE_LLM_PRESETS).is_some() {
        0
    } else if preset_index(config, PAID_LLM_PRESETS).is_some() {
        1
    } else {
        2
    };
    let selected = Select::with_theme(theme)
        .with_prompt("Type de modele LLM")
        .items(&sections)
        .default(default)
        .interact()?;

    match selected {
        0 => configure_llm_preset(theme, config, FREE_LLM_PRESETS),
        1 => configure_llm_preset(theme, config, PAID_LLM_PRESETS),
        _ => configure_custom_llm(theme, config),
    }
}

fn configure_llm_preset(
    theme: &ColorfulTheme,
    config: &mut AppConfig,
    presets: &[LlmPreset],
) -> color_eyre::Result<()> {
    let labels: Vec<&str> = presets.iter().map(|preset| preset.label).collect();
    let default = preset_index(config, presets).unwrap_or(0);
    let selected = Select::with_theme(theme)
        .with_prompt("Modele")
        .items(&labels)
        .default(default)
        .interact()?;
    apply_llm_preset(config, &presets[selected]);

    let env_name = presets[selected].api_key_env;
    println!("Par defaut, ce modele utilise la variable d'environnement {env_name}");
    configure_api_key(theme, config, env_name)?;

    Ok(())
}

fn configure_custom_llm(theme: &ColorfulTheme, config: &mut AppConfig) -> color_eyre::Result<()> {
    let providers = [
        "OpenAI custom",
        "Anthropic custom",
        "Google custom",
        "OpenAI-compatible custom",
    ];
    let default = match config.llm.provider {
        LlmProvider::OpenAI => 0,
        LlmProvider::Anthropic => 1,
        LlmProvider::Google => 2,
        LlmProvider::OpenAICompatible | LlmProvider::Ollama => 3,
    };
    let selected = Select::with_theme(theme)
        .with_prompt("Provider custom")
        .items(&providers)
        .default(default)
        .interact()?;

    match selected {
        0 => {
            config.llm.provider = LlmProvider::OpenAI;
            config.llm.api_url = None;
            config.llm.model = prompt_model(theme, &config.llm.model, "gpt-5.4-mini")?;
            configure_api_key(theme, config, "OPENAI_API_KEY")?;
        }
        1 => {
            config.llm.provider = LlmProvider::Anthropic;
            config.llm.api_url = None;
            config.llm.model = prompt_model(theme, &config.llm.model, "claude-haiku-4-5")?;
            configure_api_key(theme, config, "ANTHROPIC_API_KEY")?;
        }
        2 => {
            config.llm.provider = LlmProvider::Google;
            config.llm.api_url = None;
            config.llm.model = prompt_model(theme, &config.llm.model, GEMINI_FLASH_LITE_MODEL)?;
            configure_api_key(theme, config, "GOOGLE_API_KEY")?;
        }
        _ => {
            config.llm.provider = LlmProvider::OpenAICompatible;
            config.llm.api_url = Some(
                Input::with_theme(theme)
                    .with_prompt("Base URL ou URL /chat/completions")
                    .default(
                        config
                            .llm
                            .api_url
                            .clone()
                            .unwrap_or_else(|| OPENCODE_ZEN_CHAT_COMPLETIONS_URL.to_string()),
                    )
                    .interact_text()?,
            );
            config.llm.model = prompt_model(theme, &config.llm.model, OPENCODE_ZEN_FREE_MODEL)?;
            configure_api_key(theme, config, DEFAULT_OPENCODE_API_KEY_ENV)?;
        }
    }

    config.llm.temperature = Input::with_theme(theme)
        .with_prompt("Temperature")
        .default(config.llm.temperature)
        .interact_text()?;

    Ok(())
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

fn configure_query_safety(theme: &ColorfulTheme, config: &mut AppConfig) -> color_eyre::Result<()> {
    config.db.read_only = Confirm::with_theme(theme)
        .with_prompt("Bloquer les requetes destructives ?")
        .default(config.db.read_only)
        .interact()?;
    config.db.safe_mode = Confirm::with_theme(theme)
        .with_prompt("Demander confirmation avant les requetes dangereuses ?")
        .default(config.db.safe_mode)
        .interact()?;
    config.db.max_rows = Input::with_theme(theme)
        .with_prompt("Nombre max de lignes")
        .default(config.db.max_rows)
        .interact_text()?;
    config.db.query_timeout_secs = Input::with_theme(theme)
        .with_prompt("Timeout requetes (secondes)")
        .default(config.db.query_timeout_secs)
        .interact_text()?;
    Ok(())
}

fn configure_api_key(
    theme: &ColorfulTheme,
    config: &mut AppConfig,
    default_env: &str,
) -> color_eyre::Result<()> {
    let choices = [
        "Utiliser une variable d'environnement",
        "Stocker la cle dans le .env securise dbchat",
        "Effacer / ne pas configurer de cle",
    ];
    let default = match config.llm.api_key.as_deref() {
        Some(value) if value.starts_with("env:") => 0,
        Some(_) => 1,
        None => 0,
    };
    let selected = Select::with_theme(theme)
        .with_prompt("Cle API")
        .items(&choices)
        .default(default)
        .interact()?;

    match selected {
        0 => {
            let current = config
                .llm
                .api_key
                .as_deref()
                .and_then(|value| value.strip_prefix("env:"))
                .unwrap_or(default_env);
            let env_name: String = Input::with_theme(theme)
                .with_prompt("Nom de la variable d'environnement")
                .default(current.to_string())
                .interact_text()?;
            validate_env_name(&env_name)?;
            config.llm.api_key = Some(format!("env:{env_name}"));
        }
        1 => {
            let env_name = config
                .llm
                .api_key
                .as_deref()
                .and_then(|value| value.strip_prefix("env:"))
                .unwrap_or(default_env)
                .to_string();
            validate_env_name(&env_name)?;
            let key = Password::with_theme(theme)
                .with_prompt("Cle API")
                .allow_empty_password(true)
                .interact()?;
            if !key.trim().is_empty() {
                write_secure_env_key(&env_name, &key)?;
                config.llm.api_key = Some(format!("env:{env_name}"));
                println!(
                    "Cle stockee dans le fichier securise {}",
                    AppConfig::env_path().display()
                );
            }
        }
        _ => config.llm.api_key = None,
    }

    Ok(())
}

fn validate_env_name(name: &str) -> color_eyre::Result<()> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(color_eyre::eyre::eyre!(
            "Le nom de variable d'environnement ne peut pas etre vide"
        ));
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err(color_eyre::eyre::eyre!("Nom de variable invalide: {name}"));
    }
    if !chars.all(|c| c == '_' || c.is_ascii_alphanumeric()) {
        return Err(color_eyre::eyre::eyre!("Nom de variable invalide: {name}"));
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

fn prompt_model(
    theme: &ColorfulTheme,
    current: &str,
    fallback: &str,
) -> color_eyre::Result<String> {
    Ok(Input::with_theme(theme)
        .with_prompt("Modele")
        .default(non_empty_or(current, fallback))
        .interact_text()?)
}

fn save_config(config: &AppConfig) -> color_eyre::Result<()> {
    config.save().map_err(|err| color_eyre::eyre::eyre!(err))?;
    println!("\x1b[32m✓\x1b[0m Configuration sauvegardee");
    Ok(())
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
