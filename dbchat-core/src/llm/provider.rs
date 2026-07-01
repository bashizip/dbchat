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
            LlmProvider::OpenAI | LlmProvider::Ollama | LlmProvider::Google => {
                serde_json::json!({
                    "model": self.config.model,
                    "messages": [
                        {"role": "system", "content": system},
                        {"role": "user", "content": user}
                    ],
                    "temperature": self.config.temperature
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

    fn api_url(&self) -> &str {
        if let Some(ref url) = self.config.api_url {
            return url;
        }
        match self.config.provider {
            LlmProvider::OpenAI => "https://api.openai.com/v1/chat/completions",
            LlmProvider::Anthropic => "https://api.anthropic.com/v1/messages",
            LlmProvider::Ollama => "http://localhost:11434/api/chat",
            LlmProvider::Google => "https://generativelanguage.googleapis.com/v1beta/models/",
        }
    }

    fn api_key_header(&self) -> (&'static str, String) {
        let key = self.config.api_key.clone().unwrap_or_default();
        match self.config.provider {
            LlmProvider::OpenAI => ("Authorization", format!("Bearer {key}")),
            LlmProvider::Anthropic => ("x-api-key", key),
            LlmProvider::Ollama => ("Content-Type", "application/json".to_string()),
            LlmProvider::Google => ("x-goog-api-key", key),
        }
    }

    async fn send_request(&self, body: serde_json::Value) -> Result<String> {
        let url = self.api_url();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| DbChatError::Llm(format!("HTTP client: {e}")))?;

        let (header_name, header_value) = self.api_key_header();

        let req = client.post(url).header(header_name, &header_value);
        let req = if matches!(self.config.provider, LlmProvider::Ollama) {
            req.json(&body)
        } else {
            req.json(&body)
        };

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

fn extract_content(json_str: &str) -> String {
    let v: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return json_str.to_string(),
    };

    // OpenAI / Ollama format
    if let Some(choices) = v.get("choices") {
        if let Some(choice) = choices.get(0) {
            if let Some(msg) = choice.get("message") {
                if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                    return content.to_string();
                }
            }
        }
    }

    // Anthropic format
    if let Some(content_blocks) = v.get("content") {
        if let Some(block) = content_blocks.get(0) {
            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                return text.to_string();
            }
        }
    }

    // Ollama chat format
    if let Some(msg) = v.get("message") {
        if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
            return content.to_string();
        }
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
