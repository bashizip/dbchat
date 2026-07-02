use crate::render;

pub async fn run_repl(dbchat: &mut dbchat_core::DbChat) -> color_eyre::Result<()> {
    let mut rl = rustyline::DefaultEditor::new()?;
    let loc = dbchat.config.display.locale.clone();
    let mut history: Vec<String> = Vec::new();

    let history_file = dirs_dbchat_history();
    if let Some(ref path) = history_file {
        let _ = rl.load_history(path);
    }

    println!("\x1b[2m{}\x1b[0m", loc.t(
        "Commandes: /help, /tables, /schema, /exit, /clear, /refresh, /context, /verbose, /history",
        "Commands: /help, /tables, /schema, /exit, /clear, /refresh, /context, /verbose, /history",
    ));

    loop {
        let readline = rl.readline("dbchat> ");
        match readline {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(&line);
                history.push(line.clone());

                if line.starts_with('/') {
                    if !handle_command(&line, dbchat, &history, &loc).await {
                        break;
                    }
                    continue;
                }

                match dbchat.chat(&line).await {
                    Ok(response) => {
                        render::render_response(&response, "table", &loc);
                    }
                    Err(e) => {
                        render::render_error(&e);
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!(
                    "\x1b[2m{}\x1b[0m",
                    loc.t("Tapez /exit pour quitter", "Type /exit to quit")
                );
            }
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("{err}");
                break;
            }
        }
    }

    if let Some(ref path) = history_file {
        let _ = rl.save_history(path);
    }

    render::print_green(loc.t("À bientôt !", "See you!"));
    Ok(())
}

async fn handle_command(
    cmd: &str,
    dbchat: &mut dbchat_core::DbChat,
    history: &[String],
    loc: &dbchat_core::config::Locale,
) -> bool {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let command = parts[0].to_lowercase();

    match command.as_str() {
        "/exit" | "/quit" => return false,

        "/help" => {
            render::render_help(loc);
        }

        "/tables" => {
            render::print_bold(loc.t("Tables disponibles:", "Available tables:"));
            for table in &dbchat.schema.tables {
                let col_count = table.columns.len();
                let row_info = table
                    .row_count
                    .map(|c| format!(" ({c})"))
                    .unwrap_or_default();
                let col_label = loc.t("colonnes", "cols");
                println!(
                    "  \x1b[36m{}\x1b[0m {col_count} {col_label}{row_info}",
                    table.name
                );
            }
        }

        "/schema" => {
            for table in &dbchat.schema.tables {
                render::print_bold(format!("{} {}", loc.t("Table:", "Table:"), table.name));
                for col in &table.columns {
                    let mut flags = String::new();
                    if col.is_primary_key {
                        flags.push_str(" PK");
                    }
                    if let Some((ref ft, ref fc)) = col.fk_ref {
                        flags.push_str(&format!(" FK->{ft}.{fc}"));
                    }
                    if col.is_nullable {
                        flags.push_str(" NULL");
                    }
                    println!(
                        "  \x1b[32m{}\x1b[0m \x1b[2m{}\x1b[0m \x1b[33m{flags}\x1b[0m",
                        col.name, col.data_type
                    );
                }
                println!();
            }
        }

        "/clear" => {
            print!("\x1B[2J\x1B[1;1H");
        }

        "/refresh" => {
            println!(
                "\x1b[34m{}\x1b[0m",
                loc.t("Scanning schema...", "Scanning schema...")
            );
            match dbchat.refresh_schema().await {
                Ok(()) => render::print_green(loc.t("Schema updated", "Schema updated")),
                Err(e) => render::render_error(&e),
            }
        }

        "/context" => {
            render::print_bold(loc.t("Context sent to LLM:", "Context sent to LLM:"));
            println!("{}", dbchat.llm.system_prompt());
        }

        "/verbose" => {
            let on = dbchat.toggle_verbose();
            let msg = if on {
                loc.t("Verbose ON", "Verbose ON")
            } else {
                loc.t("Verbose OFF", "Verbose OFF")
            };
            render::print_green(msg);
        }

        "/history" => {
            render::print_bold(loc.t("Question history:", "Question history:"));
            if history.is_empty() {
                println!("  {}", loc.t("(none)", "(none)"));
            } else {
                for (i, q) in history.iter().rev().take(20).enumerate() {
                    println!("  \x1b[2m{}.\x1b[0m {q}", history.len() - i);
                }
            }
        }

        "/config" => {
            let path = dbchat_core::AppConfig::config_path();
            println!("\x1b[1mConfig:\x1b[0m \x1b[36m{}\x1b[0m", path.display());
            println!(
                "  {} \x1b[35m{}\x1b[0m",
                loc.t("Provider:", "Provider:"),
                match dbchat.config.llm.provider {
                    dbchat_core::config::LlmProvider::OpenAI => "OpenAI",
                    dbchat_core::config::LlmProvider::Anthropic => "Anthropic",
                    dbchat_core::config::LlmProvider::Ollama => "Ollama",
                    dbchat_core::config::LlmProvider::Google => "Google",
                    dbchat_core::config::LlmProvider::OpenAICompatible => "OpenAI-compatible",
                }
            );
            println!(
                "  {} \x1b[35m{}\x1b[0m",
                loc.t("Model:", "Model:"),
                dbchat.config.llm.model
            );
            println!(
                "  {} \x1b[35m{}\x1b[0m",
                loc.t("Verbose:", "Verbose:"),
                dbchat.verbose
            );
        }

        _ => {
            render::print_red(format!(
                "{}: {command}",
                loc.t("Unknown command", "Unknown command")
            ));
            println!("  {}", loc.t(
                "Commands: /help, /tables, /schema, /exit, /clear, /refresh, /context, /verbose, /history, /config",
                "Commands: /help, /tables, /schema, /exit, /clear, /refresh, /context, /verbose, /history, /config",
            ));
        }
    }
    true
}

fn dirs_dbchat_history() -> Option<String> {
    let base = dirs_data_dir();
    let _ = std::fs::create_dir_all(&base);
    Some(format!("{base}/history.txt"))
}

fn dirs_data_dir() -> String {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        format!("{dir}/dbchat")
    } else if let Ok(home) = std::env::var("HOME") {
        format!("{home}/.local/share/dbchat")
    } else {
        "./.dbchat".to_string()
    }
}
