use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use std::fmt::Display;
use std::io::{self, IsTerminal, Write};

use super::theme::TerminalTheme;

#[derive(Debug, Clone, Copy)]
pub enum FlashKind {
    Success,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Flash {
    kind: FlashKind,
    message: String,
}

impl Flash {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            kind: FlashKind::Success,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            kind: FlashKind::Warning,
            message: message.into(),
        }
    }
}

pub struct TerminalUi {
    theme: TerminalTheme,
    prompt_theme: ColorfulTheme,
}

impl TerminalUi {
    pub fn new() -> Self {
        Self {
            theme: TerminalTheme::new(),
            prompt_theme: ColorfulTheme::default(),
        }
    }

    pub fn theme(&self) -> &TerminalTheme {
        &self.theme
    }

    pub fn prompt_theme(&self) -> &ColorfulTheme {
        &self.prompt_theme
    }

    pub fn reset(&self, trail: &str) -> color_eyre::Result<()> {
        self.clear()?;
        self.header(trail);
        Ok(())
    }

    pub fn clear(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        if stdout.is_terminal() {
            write!(stdout, "\x1b[2J\x1b[H")?;
            stdout.flush()?;
        }
        Ok(())
    }

    pub fn header(&self, trail: &str) {
        println!("{}", self.theme.bold("dbchat config"));
        if !trail.trim().is_empty() {
            println!("{}", self.theme.muted(trail));
        }
        println!();
    }

    pub fn footer(&self) {
        println!();
        println!(
            "{}",
            self.theme
                .muted("↑/↓ naviguer · Entrée sélectionner · Esc retour · Ctrl+C quitter")
        );
    }

    pub fn flash(&self, flash: &Flash) {
        let prefix = match flash.kind {
            FlashKind::Success => self.theme.check(),
            FlashKind::Warning => self.theme.warn_icon(),
        };
        println!("{prefix} {}", flash.message);
        println!();
    }

    pub fn select<T: Display>(
        &self,
        prompt: &str,
        items: &[T],
        default: usize,
    ) -> color_eyre::Result<Option<usize>> {
        if items.is_empty() {
            return Ok(None);
        }

        let default = default.min(items.len() - 1);
        let selected = Select::with_theme(&self.prompt_theme)
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact_opt()?;
        Ok(selected)
    }

    pub fn wait_for_enter(&self) -> color_eyre::Result<()> {
        let mut stdout = io::stdout();
        println!();
        write!(stdout, "{}", self.theme.muted("Entrée pour revenir"))?;
        stdout.flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        Ok(())
    }
}

impl Default for TerminalUi {
    fn default() -> Self {
        Self::new()
    }
}
