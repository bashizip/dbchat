use crate::config::{DbEngine, LlmConfig, LlmProvider};
use crate::db::schema::SchemaInfo;
use crate::error::{DbChatError, Result};

#[derive(Debug, Clone)]
pub struct SqlGeneration {
    pub sql: String,
    pub explanation: Option<String>,
    pub raw_response: String,
}

pub struct LlmClient {
    config: LlmConfig,
    schema: Option<SchemaInfo>,
    engine: Option<DbEngine>,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            schema: None,
            engine: None,
        }
    }

    pub fn set_schema(&mut self, schema: SchemaInfo, engine: DbEngine) {
        self.schema = Some(schema);
        self.engine = Some(engine);
    }

    pub fn schema(&self) -> Option<&SchemaInfo> {
        self.schema.as_ref()
    }

    pub fn engine(&self) -> Option<&DbEngine> {
        self.engine.as_ref()
    }

    pub fn system_prompt(&self) -> String {
        self.build_system_prompt()
    }

    fn build_system_prompt(&self) -> String {
        let dialect = match self.engine {
            Some(DbEngine::Postgres) => "PostgreSQL",
            Some(DbEngine::Mysql) => "MySQL",
            Some(DbEngine::Sqlite) => "SQLite",
            None => "SQL",
        };

        let schema_ctx = self
            .schema
            .as_ref()
            .map(|s| s.to_prompt_context(dialect))
            .unwrap_or_default();

        format!(
            r#"Tu es un expert SQL pour le dialecte {dialect}.

Contexte de la base de données :
{schema_ctx}

Règles impératives :
1. Génère UNIQUEMENT du SQL valide.
2. Utilise strictement le dialecte {dialect}.
3. Si la requête peut être destructive (DELETE sans WHERE, DROP, TRUNCATE, ALTER), commence par "DESTRUCTIVE:" suivi de la description.
4. Si la question est ambiguë, commence par "AMBIGUOUS:" suivi de ta question de clarification.
5. Limite les résultats à 100 lignes avec LIMIT.
6. Ne mets PAS d'explications. Retourne UNIQUEMENT le SQL."#,
        )
    }

    pub async fn generate_sql(&self, question: &str) -> Result<SqlGeneration> {
        let system_prompt = self.build_system_prompt();
        let body = self.build_chat_body(&system_prompt, question);
        let response = self.send_request(body).await?;

        let raw = extract_content(&response);
        let trimmed = raw.trim();

        if trimmed.starts_with("DESTRUCTIVE:") {
            return Ok(SqlGeneration {
                sql: String::new(),
                explanation: Some(
                    trimmed
                        .trim_start_matches("DESTRUCTIVE:")
                        .trim()
                        .to_string(),
                ),
                raw_response: raw,
            });
        }
        if trimmed.starts_with("AMBIGUOUS:") {
            return Ok(SqlGeneration {
                sql: String::new(),
                explanation: Some(trimmed.trim_start_matches("AMBIGUOUS:").trim().to_string()),
                raw_response: raw,
            });
        }

        let sql = extract_sql(trimmed);
        Ok(SqlGeneration {
            sql,
            explanation: None,
            raw_response: raw,
        })
    }

    pub async fn explain_error(&self, sql: &str, error: &str, question: &str) -> Result<String> {
        let prompt = format!(
            r#"La requête SQL suivante a généré une erreur :

SQL: {sql}
Erreur: {error}
Question originale: {question}

Explique l'erreur en langage naturel et suggère une correction. Réponds en quelques phrases en français."#
        );
        let body = self.build_chat_body("Tu es un expert SQL.", &prompt);
        let response = self.send_request(body).await?;
        Ok(extract_content(&response))
    }

    pub async fn explain_result(&self, question: &str, sql: &str, summary: &str) -> Result<String> {
        let prompt = format!(
            r#"Question: {question}
SQL exécuté: {sql}
Résumé: {summary}

Explique ces résultats en langage naturel simple. Réponds en français."#
        );
        let body = self.build_chat_body("Tu es un assistant data.", &prompt);
        let response = self.send_request(body).await?;
        Ok(extract_content(&response))
    }

    fn build_chat_body(&self, system: &str, user: &str) -> serde_json::Value {
        match self.config.provider {
            LlmProvider::OpenAI | LlmProvider::Ollama | LlmProvider::OpenAICompatible => {
                serde_json::json!({
                    "model": self.config.model,
                    "messages": [
                        {"role": "system", "content": system},
                        {"role": "user", "content": user}
                    ],
                    "temperature": self.config.temperature
                })
            }
            LlmProvider::Google => {
                serde_json::json!({
                    "systemInstruction": {
                        "parts": [
                            {"text": system}
                        ]
                    },
                    "contents": [
                        {
                            "role": "user",
                            "parts": [
                                {"text": user}
                            ]
                        }
                    ],
                    "generationConfig": {
                        "temperature": self.config.temperature
                    }
                })
            }
            LlmProvider::Anthropic => {
                serde_json::json!({
                    "model": self.config.model,
                    "max_tokens": 1024,
                    "system": system,
                    "messages": [
                        {"role": "user", "content": user}
                    ],
                    "temperature": self.config.temperature
                })
            }
        }
    }

    fn api_url(&self) -> Result<String> {
        if let Some(ref url) = self.config.api_url {
            return Ok(
                if matches!(self.config.provider, LlmProvider::OpenAICompatible) {
                    openai_compatible_chat_url(url)
                } else {
                    url.clone()
                },
            );
        }
        match self.config.provider {
            LlmProvider::OpenAI => Ok("https://api.openai.com/v1/chat/completions".to_string()),
            LlmProvider::Anthropic => Ok("https://api.anthropic.com/v1/messages".to_string()),
            LlmProvider::Ollama => Ok("http://localhost:11434/api/chat".to_string()),
            LlmProvider::Google => Ok(google_generate_content_url(&self.config.model)),
            LlmProvider::OpenAICompatible => Err(DbChatError::Config(
                "api_url is required for OpenAI-compatible LLM providers".to_string(),
            )),
        }
    }

    fn api_key_header(&self) -> (&'static str, String) {
        let key = self.config.api_key.clone().unwrap_or_default();
        match self.config.provider {
            LlmProvider::OpenAI | LlmProvider::OpenAICompatible => {
                ("Authorization", format!("Bearer {key}"))
            }
            LlmProvider::Anthropic => ("x-api-key", key),
            LlmProvider::Ollama => ("Content-Type", "application/json".to_string()),
            LlmProvider::Google => ("x-goog-api-key", key),
        }
    }

    async fn send_request(&self, body: serde_json::Value) -> Result<String> {
        let url = self.api_url()?;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| DbChatError::Llm(format!("HTTP client: {e}")))?;

        let (header_name, header_value) = self.api_key_header();

        let req = client
            .post(&url)
            .header(header_name, &header_value)
            .json(&body);

        let resp = req
            .send()
            .await
            .map_err(|e| DbChatError::Llm(format!("HTTP error: {e}")))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| DbChatError::Llm(format!("Read response: {e}")))?;

        if !status.is_success() {
            return Err(DbChatError::Llm(format!(
                "API error ({}): {}",
                status,
                &text[..text.len().min(200)]
            )));
        }

        Ok(text)
    }
}

fn openai_compatible_chat_url(base_or_url: &str) -> String {
    let trimmed = base_or_url.trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/chat/completions")
    }
}

fn google_generate_content_url(model: &str) -> String {
    let model = model.trim().trim_start_matches("models/");
    format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent")
}

fn extract_content(json_str: &str) -> String {
    let v: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return json_str.to_string(),
    };

    // OpenAI / Ollama format
    if let Some(content) = v
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|content| content.as_str())
    {
        return content.to_string();
    }

    // Anthropic format
    if let Some(text) = v
        .get("content")
        .and_then(|content_blocks| content_blocks.get(0))
        .and_then(|block| block.get("text"))
        .and_then(|text| text.as_str())
    {
        return text.to_string();
    }

    // Gemini generateContent format
    if let Some(parts) = v
        .get("candidates")
        .and_then(|candidates| candidates.as_array())
        .and_then(|candidates| candidates.first())
        .and_then(|candidate| candidate.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(|parts| parts.as_array())
    {
        let text = parts
            .iter()
            .filter_map(|part| part.get("text").and_then(|text| text.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        if !text.is_empty() {
            return text;
        }
    }

    // Ollama chat format
    if let Some(content) = v
        .get("message")
        .and_then(|msg| msg.get("content"))
        .and_then(|content| content.as_str())
    {
        return content.to_string();
    }

    // Fallback: try to find any content field
    if let Some(content) = v.get("response").and_then(|c| c.as_str()) {
        return content.to_string();
    }

    json_str.to_string()
}

fn extract_sql(text: &str) -> String {
    let trimmed = text.trim();

    if let Some(sql_start) = trimmed.find("```sql") {
        let after = &trimmed[sql_start + 6..];
        if let Some(sql_end) = after.find("```") {
            return after[..sql_end].trim().to_string();
        }
        return after.trim().to_string();
    }

    if let Some(sql_start) = trimmed.find("```") {
        let after = &trimmed[sql_start + 3..];
        if let Some(sql_end) = after.find("```") {
            return after[..sql_end].trim().to_string();
        }
        return after.trim().to_string();
    }

    trimmed
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with("Voici")
                && !t.starts_with("Here")
                && !t.starts_with("Je vous")
                && !t.starts_with("Bien sûr")
                && !t.starts_with("D'accord")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_openai_compatible_base_url() {
        assert_eq!(
            openai_compatible_chat_url("https://api.deepseek.com"),
            "https://api.deepseek.com/chat/completions"
        );
        assert_eq!(
            openai_compatible_chat_url("https://example.com/v1/chat/completions"),
            "https://example.com/v1/chat/completions"
        );
        assert_eq!(
            openai_compatible_chat_url("https://opencode.ai/zen/v1/chat/completions"),
            "https://opencode.ai/zen/v1/chat/completions"
        );
    }

    #[test]
    fn builds_google_generate_content_url_from_model() {
        assert_eq!(
            google_generate_content_url("gemini-3.1-flash-lite"),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.1-flash-lite:generateContent"
        );
        assert_eq!(
            google_generate_content_url("models/gemini-3.5-flash"),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.5-flash:generateContent"
        );
    }

    #[test]
    fn builds_google_generate_content_body() {
        let config = LlmConfig {
            provider: LlmProvider::Google,
            model: "gemini-3.1-flash-lite".to_string(),
            temperature: 0.2,
            ..Default::default()
        };
        let client = LlmClient::new(config);

        let body = client.build_chat_body("system prompt", "user prompt");

        assert_eq!(
            body["systemInstruction"]["parts"][0]["text"],
            "system prompt"
        );
        assert_eq!(body["contents"][0]["parts"][0]["text"], "user prompt");
        assert_eq!(body["generationConfig"]["temperature"].as_f64(), Some(0.2));
        assert!(body.get("model").is_none());
    }

    #[test]
    fn extracts_openai_compatible_content() {
        let response = r#"{"choices":[{"message":{"content":"SELECT 1;"}}]}"#;
        assert_eq!(extract_content(response), "SELECT 1;");
    }

    #[test]
    fn extracts_google_generate_content() {
        let response =
            r#"{"candidates":[{"content":{"parts":[{"text":"SELECT 1;"},{"text":"-- ok"}]}}]}"#;
        assert_eq!(extract_content(response), "SELECT 1;\n-- ok");
    }
}
