# dbchat

Chat with your database using natural language.

```
dbchat postgres://user:pass@localhost/mydb
dbchat> "show me the last 5 customers"
```

## Test database (Docker)

```bash
cd test-db && docker compose up -d
```

Connect:
```bash
dbchat mysql://dbchat:dbchat@localhost:3306/boutique
```

Full schema documentation and examples → [`test-db/README.md`](test-db/README.md)

## Installation

### Via curl (recommended)

```bash
curl -sSfL https://raw.githubusercontent.com/bashizip/dbchat/main/scripts/install.sh | bash
```

Installation dans un répertoire personnalisé :

```bash
curl -sSfL https://raw.githubusercontent.com/bashizip/dbchat/main/scripts/install.sh | bash -s -- latest ~/.local/bin
```

### Via cargo

```bash
cargo install --path dbchat-cli
```

## Usage

```bash
# Use the last known configuration
dbchat

# Connect and enter interactive mode
dbchat postgres://user:pass@localhost/mydb
dbchat mysql://user:pass@localhost:3306/mydb
dbchat sqlite:///data/mydb.db

# One-shot query
dbchat postgres://... -q "what is the total revenue?"
dbchat postgres://... -q "top 10 products" -f json

# With model/provider override
dbchat postgres://... --provider anthropic --model claude-haiku-4-5
dbchat postgres://... --provider openrouter --model openrouter/free
```

## Interactive commands

| Command | Description |
|---------|-------------|
| `your question` | Ask a question in natural language |
| `/tables` | List database tables |
| `/schema` | Show detailed schema (columns, types, keys) |
| `/context` | Show context sent to LLM |
| `/verbose` | Toggle verbose mode |
| `/history` | Show question history |
| `/config` | Show current configuration |
| `/refresh` | Re-scan schema |
| `/clear` | Clear screen |
| `/exit` | Quit |

## Configuration

File: `~/.config/dbchat/config.toml`

```toml
[llm]
provider = "openai-compatible" # openai-compatible | google | openai | anthropic | ollama
model = "deepseek-v4-flash-free"
api_key = "env:OPENCODE_API_KEY"
api_url = "https://opencode.ai/zen/v1/chat/completions"

[db]
engine = "Postgres"
uri = "postgres://user:pass@localhost/mydb"
max_connections = 5
query_timeout_secs = 30
read_only = true
safe_mode = true

[display]
format = "table"               # table | json | csv
show_sql = true
verbose = false
```

```bash
dbchat config         # Interactive configuration wizard
dbchat config init    # Create/reset default config
dbchat config show    # Show current configuration
```

The interactive menu lets you configure the database connection, LLM provider, and
query safety settings (`read_only`, `safe_mode`, `max_rows`, timeout).

Models via API key:

```bash
dbchat config
# then: LLM -> Free models
# or:   LLM -> Paid models
```

The wizard automatically configures the provider, model, API URL if needed,
and the environment variable to use for the key.

By default, dbchat uses OpenCode Zen with the free `deepseek-v4-flash-free` model. To enable it:

1. Log in to OpenCode Zen.
2. Copy your Zen API key.
3. Export it in your shell:

```bash
export OPENCODE_API_KEY="your_key"
```

Alternatively, paste the key into `dbchat config`. dbchat stores it in the
`~/.config/dbchat/.env` file (mode `600` on Unix) and only keeps
`api_key = "env:OPENCODE_API_KEY"` in `config.toml`. This `.env` is global to
dbchat; dbchat does not load `.env` files from your project directories.

| Choice | Model | Key |
|--------|-------|-----|
| Recommended free | `deepseek-v4-flash-free` via OpenCode Zen | `OPENCODE_API_KEY` |
| Free tier | `gemini-3.1-flash-lite` | `GOOGLE_API_KEY` |
| Free OpenRouter | `openrouter/free` | `OPENROUTER_API_KEY` |
| Free OpenRouter | `google/gemma-4-31b-it:free` | `OPENROUTER_API_KEY` |
| Free OpenRouter | `cohere/north-mini-code:free` | `OPENROUTER_API_KEY` |
| Low-cost paid | `deepseek-v4-flash` | `DEEPSEEK_API_KEY` |
| Common paid | `gpt-5.4-mini`, `gpt-5.5` | `OPENAI_API_KEY` |
| Common paid | `claude-haiku-4-5`, `claude-sonnet-5` | `ANTHROPIC_API_KEY` |
| Common paid | `gemini-3.5-flash` | `GOOGLE_API_KEY` |

OpenCode Zen indicates that the free model is available for a limited time
and that data sent to the free model may be used to improve the model during
this period.

Default example:

```toml
[llm]
provider = "openai-compatible"
model = "deepseek-v4-flash-free"
api_url = "https://opencode.ai/zen/v1/chat/completions"
api_key = "env:OPENCODE_API_KEY"
```

Free OpenRouter example:

```toml
[llm]
provider = "openai-compatible"
model = "openrouter/free"
api_url = "https://openrouter.ai/api/v1"
api_key = "env:OPENROUTER_API_KEY"
```

Low-cost example:

```toml
[llm]
provider = "openai-compatible"
model = "deepseek-v4-flash"
api_url = "https://api.deepseek.com"
api_key = "env:DEEPSEEK_API_KEY"
```

`deepseek-v4-flash` uses the DeepSeek API in OpenAI-compatible format. The
`:free` models go through OpenRouter and may change based on provider availability.

## CLI Options

```
  -q, --query <QUERY>        One-shot query mode
  -f, --format <FORMAT>      table, json, csv
  -v, --verbose              Verbose mode
      --model <MODEL>        Model override
      --provider <PROVIDER>  Provider override
      --read-only            Block destructive queries
      --no-color             Disable colors
```
