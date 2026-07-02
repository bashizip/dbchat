pub const SECRET_MASK: &str = "********";

#[derive(Debug, Clone, Copy)]
pub struct TerminalTheme {
    color_enabled: bool,
}

impl TerminalTheme {
    pub fn new() -> Self {
        Self {
            color_enabled: std::env::var_os("NO_COLOR").is_none(),
        }
    }

    pub fn bold(&self, text: impl AsRef<str>) -> String {
        self.paint("1", text)
    }

    pub fn muted(&self, text: impl AsRef<str>) -> String {
        self.paint("2", text)
    }

    pub fn info(&self, text: impl AsRef<str>) -> String {
        self.paint("36", text)
    }

    pub fn success(&self, text: impl AsRef<str>) -> String {
        self.paint("32", text)
    }

    pub fn warning(&self, text: impl AsRef<str>) -> String {
        self.paint("33", text)
    }

    pub fn error(&self, text: impl AsRef<str>) -> String {
        self.paint("31", text)
    }

    pub fn check(&self) -> String {
        self.success("✓")
    }

    pub fn cross(&self) -> String {
        self.error("✗")
    }

    pub fn warn_icon(&self) -> String {
        self.warning("!")
    }

    fn paint(&self, code: &str, text: impl AsRef<str>) -> String {
        if self.color_enabled {
            format!("\x1b[{code}m{}\x1b[0m", text.as_ref())
        } else {
            text.as_ref().to_string()
        }
    }
}

impl Default for TerminalTheme {
    fn default() -> Self {
        Self::new()
    }
}

pub fn mask_secret(value: Option<&str>) -> &'static str {
    match value {
        Some(value) if !value.trim().is_empty() => SECRET_MASK,
        _ => "non configurée",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_non_empty_secret() {
        assert_eq!(mask_secret(Some("sk-secret")), SECRET_MASK);
        assert_eq!(mask_secret(Some("   ")), "non configurée");
        assert_eq!(mask_secret(None), "non configurée");
    }
}
