use comfy_table::{Cell, CellAlignment, Table, presets::UTF8_FULL};
use dbchat_core::ChatResponse;
use dbchat_core::config::Locale;
use dbchat_core::error::DbChatError;

pub fn render_response(response: &ChatResponse, format: &str, locale: &Locale) {
    match format {
        "json" => render_json(response),
        "csv" => render_csv(response),
        _ => render_table_default(response, locale),
    }
}

fn render_json(response: &ChatResponse) {
    match response {
        ChatResponse::Result {
            sql,
            result,
            elapsed,
        } => {
            let rows: Vec<serde_json::Value> = result
                .values()
                .iter()
                .map(|row| {
                    let cols = result.columns();
                    let mut map = serde_json::Map::new();
                    for (i, col) in cols.iter().enumerate() {
                        map.insert(
                            col.clone(),
                            row.get(i).cloned().unwrap_or(serde_json::Value::Null),
                        );
                    }
                    serde_json::Value::Object(map)
                })
                .collect();
            let output = serde_json::json!({
                "sql": sql,
                "elapsed_secs": elapsed.as_secs_f64(),
                "rows_affected": result.rows_affected(),
                "rows": rows,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        ChatResponse::ConfirmDestructive(sql) => {
            println!(
                "{}",
                serde_json::json!({ "warning": "destructive_query", "sql": sql })
            );
        }
        ChatResponse::Info(msg) => {
            println!("{}", serde_json::json!({ "info": msg }));
        }
    }
}

fn render_csv(response: &ChatResponse) {
    match response {
        ChatResponse::Result { result, .. } => {
            if !result.is_select() {
                println!("rows_affected,{}", result.rows_affected());
                return;
            }
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            let _ = wtr.write_record(&result.columns());
            for row in result.values() {
                let record: Vec<String> = row
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::Null => String::new(),
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect();
                let _ = wtr.write_record(&record);
            }
            let _ = wtr.flush();
        }
        ChatResponse::ConfirmDestructive(sql) => {
            eprintln!("\x1b[31m⚠ Destructive: {sql}\x1b[0m");
        }
        ChatResponse::Info(msg) => {
            println!("{msg}");
        }
    }
}

fn render_table_default(response: &ChatResponse, locale: &Locale) {
    match response {
        ChatResponse::Result {
            sql,
            result,
            elapsed,
        } => {
            if locale.t("", "") == "" {
                // always show SQL
            }
            if result.is_select() {
                let columns = result.columns();
                let values = result.values();
                let rows = values.len();
                let timing = format!("{:.3}s", elapsed.as_secs_f64());
                let label = locale.t("▶ Résultat:", "▶ Result:");
                let rows_label = locale.t("lignes", "rows");
                println!(
                    "\x1b[1;32m{label}\x1b[0m \x1b[1m{rows}\x1b[0m {rows_label} (\x1b[2m{timing}\x1b[0m)"
                );
                if rows > 0 && !columns.is_empty() {
                    render_table(&columns, &values);
                } else {
                    let empty = locale.t("∅ Aucun résultat", "∅ No results");
                    println!("  \x1b[2m{empty}\x1b[0m");
                }
            } else {
                let affected = result.rows_affected();
                let timing = format!("{:.3}s", elapsed.as_secs_f64());
                let label = locale.t("ligne(s) affectée(s)", "row(s) affected");
                println!(
                    "\x1b[1;32m✓\x1b[0m \x1b[1m{affected}\x1b[0m {label} (\x1b[2m{timing}\x1b[0m)"
                );
            }

            if !sql.is_empty() {
                let sql_label = locale.t("SQL:", "SQL:");
                println!("\x1b[2m{sql_label}\x1b[0m \x1b[33m{sql}\x1b[0m");
            }
        }
        ChatResponse::ConfirmDestructive(sql) => {
            let warn = locale.t(
                "⚠ ATTENTION Requête destructive détectée:",
                "⚠ WARNING Destructive query detected:",
            );
            println!("\x1b[1;31m{warn}\x1b[0m");
            println!("  \x1b[33m{sql}\x1b[0m");
            let hint = locale.t(
                "Utilisez --read-only ou confirmez avec /sql <requête>",
                "Use --read-only or confirm with /sql <query>",
            );
            println!("  \x1b[2m{hint}\x1b[0m");
        }
        ChatResponse::Info(msg) => {
            println!("\x1b[34mℹ {msg}\x1b[0m");
        }
    }
}

pub fn render_table(columns: &[String], values: &[Vec<serde_json::Value>]) {
    if columns.is_empty() || values.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    let header: Vec<Cell> = columns
        .iter()
        .map(|c| {
            Cell::new(c)
                .set_alignment(CellAlignment::Center)
                .fg(comfy_table::Color::Cyan)
                .add_attribute(comfy_table::Attribute::Bold)
        })
        .collect();
    table.set_header(header);

    for row in values.iter().take(100) {
        let cells: Vec<Cell> = row
            .iter()
            .map(|v| {
                let display = match v {
                    serde_json::Value::Null => "NULL".to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                Cell::new(&display)
            })
            .collect();
        table.add_row(cells);
    }

    if values.len() > 100 {
        let more = values.len() - 100;
        table.add_row(vec![
            Cell::new(format!("... et {more} lignes supplémentaires"))
                .set_alignment(CellAlignment::Center)
                .fg(comfy_table::Color::DarkGrey),
        ]);
    }

    println!("{table}");
}

pub fn render_error(err: &DbChatError) {
    match err {
        DbChatError::Database(e) => {
            eprintln!("\x1b[1;31m❌ Erreur SQL:\x1b[0m \x1b[31m{e}\x1b[0m");
        }
        DbChatError::Timeout(secs) => {
            eprintln!(
                "\x1b[1;31m⏱ Délai d'attente dépassé ({secs}s).\x1b[0m Essayez une requête plus simple."
            );
        }
        DbChatError::Llm(e) => {
            eprintln!("\x1b[1;31m🤖 Erreur LLM:\x1b[0m \x1b[31m{e}\x1b[0m");
        }
        DbChatError::Connection(e) => {
            eprintln!("\x1b[1;31m🔌 Connexion impossible:\x1b[0m \x1b[31m{e}\x1b[0m");
        }
        DbChatError::DestructiveQuery => {
            eprintln!("\x1b[1;31m🛡 Requête destructive bloquée en mode lecture seule.\x1b[0m");
        }
        _ => {
            eprintln!("\x1b[1;31m✗ Erreur:\x1b[0m \x1b[31m{err}\x1b[0m");
        }
    }
}

pub fn render_help(locale: &Locale) {
    let fr = r#"
┌──────────────────────────────────────────────────────────┐
│                    dbchat Aide                          │
├──────────────────────────────────────────────────────────┤
│ <question>      Poser une question en langage naturel    │
│ /tables         Lister les tables de la base             │
│ /schema         Afficher le schéma détaillé              │
│ /context        Voir le contexte envoyé au LLM           │
│ /refresh        Re-scanner le schéma                     │
│ /verbose        Activer/couper le mode verbose           │
│ /history        Voir l'historique des questions          │
│ /config         Voir la configuration courante           │
│ /clear          Effacer l'écran                          │
│ /help           Afficher cette aide                      │
│ /exit           Quitter                                  │
└──────────────────────────────────────────────────────────┘
"#;
    let en = r#"
┌──────────────────────────────────────────────────────────┐
│                    dbchat Help                          │
├──────────────────────────────────────────────────────────┤
│ <question>      Ask a question in natural language       │
│ /tables         List database tables                    │
│ /schema         Show detailed schema                    │
│ /context        Show context sent to LLM                │
│ /refresh        Re-scan schema                          │
│ /verbose        Toggle verbose mode                     │
│ /history        Show question history                   │
│ /config         Show current configuration              │
│ /clear          Clear screen                            │
│ /help           Show this help                          │
│ /exit           Quit                                    │
└──────────────────────────────────────────────────────────┘
"#;
    println!("{}", locale.t(fr, en));
}

pub fn print_green(s: impl AsRef<str>) {
    println!("\x1b[32m{}\x1b[0m", s.as_ref());
}
pub fn print_red(s: impl AsRef<str>) {
    eprintln!("\x1b[31m{}\x1b[0m", s.as_ref());
}
pub fn print_bold(s: impl AsRef<str>) {
    println!("\x1b[1m{}\x1b[0m", s.as_ref());
}
