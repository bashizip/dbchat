# dbchat — Specifications

## 1. Vision
**dbchat** is a Rust CLI tool that lets you chat with a relational database in natural language. You describe what you want in English, and dbchat translates it to native SQL via an LLM, executes the query, and presents the results in a rich, readable format.

## 2. Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    dbchat CLI                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │  Config  │  │  Chat    │  │  Result  │  │  Help  │ │
│  │  Module  │  │  REPL    │  │  Viewer  │  │  Mode  │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────────┘ │
│       │             │             │                    │
├───────┴─────────────┴─────────────┴────────────────────┤
│                    dbchat-core                          │
│  ┌────────────┐  ┌──────────────┐  ┌────────────────┐ │
│  │  DB Layer  │  │  LLM Layer   │  │  Schema        │ │
│  │  (SQLx)    │  │  (genai/rig) │  │  Introspect    │ │
│  └────────────┘  └──────────────┘  └────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

## 3. Technology Stack

| Component       | Technology                                      |
|----------------|--------------------------------------------------|
| Language       | Rust (edition 2024)                              |
| Async runtime  | Tokio                                            |
| CLI args       | clap v4 (derive)                                 |
| TUI / REPL     | rustyline + crossterm                            |
| Database       | SQLx (PostgreSQL, MySQL, SQLite, MSSQL)          |
| LLM            | genai (multi-provider: OpenAI, Anthropic, Ollama)|
| Colors         | anstyle + anstream                               |
| Tables         | tabled (derive Tabled)                           |
| Markdown       | termimad (markdown rendering in terminal)        |
| Serialization  | serde + toml (configuration)                     |

## 4. Operating Modes

### 4.1 Connection Mode
```bash
dbchat postgres://user:pass@localhost/mydb
dbchat mysql://user:pass@localhost/mydb
dbchat sqlite:mydb.db
```

### 4.2 Interactive Mode (REPL)
```bash
dbchat postgres://user:pass@localhost/mydb
> "show me the last 5 customers"
> "how many orders per month in 2024?"
> "show the best selling products"
> /tables    # list tables
> /schema    # show full schema
> /context   # show context sent to LLM
> /exit      # quit
```

### 4.3 One-shot Mode (batch)
```bash
dbchat --db postgres://... --query "what is the total revenue?"
dbchat --db postgres://... --query "top 10 products" --format json
```

## 5. DB Layer (SQLx)

### 5.1 Supported Connectors
| Engine    | URI                                       | Status |
|-----------|-------------------------------------------|--------|
|PostgreSQL | `postgres://user:pass@host:port/db`      | ✅ phase 1 |
|MySQL      | `mysql://user:pass@host:port/db`          | ✅ phase 1 |
|SQLite     | `sqlite://path/to/db.sqlite`             | ✅ phase 2 |
|MSSQL      | `mssql://user:pass@host:port/db`          | 🔄 phase 3 |

### 5.2 Extracted Schema (for LLM context)
For each table:
- Name, columns (name, type, nullable, pk, fk)
- Indexes and unique constraints
- Relationships (foreign keys)
- N data samples (3-5 rows)
- Basic statistics (row count)

All of this is sent in the LLM system prompt to provide context.

### 5.3 DB Optimizations
- Configurable connection pooling (max conns, timeout)
- Parallel schema preparation
- In-memory schema cache (invalidated on /refresh)
- Timeout for dangerous queries (configurable)
- Optional `EXPLAIN` before execution
- Protected read-only mode (no `INSERT/UPDATE/DELETE` without confirmation)

## 6. LLM Layer (genai)

### 6.1 Providers
| Provider   | Recommended Model            | Configuration           |
|------------|------------------------------|-------------------------|
| OpenAI     | gpt-4o-mini / gpt-4o         | `OPENAI_API_KEY`        |
| Anthropic  | claude-3-haiku / sonnet      | `ANTHROPIC_API_KEY`     |
| Ollama     | llama3 / mixtral / qwen      | `OLLAMA_URL` (localhost)|
| Google     | gemini-2.0-flash             | `GOOGLE_API_KEY`        |

### 6.2 Prompt Engineering

**System**: DB context (schema, SQL dialect)
**User**: natural language question

```
You are an SQL expert for {dialect} (PostgreSQL/MySQL/...).
Here is the database schema:

{full schema with types, constraints, relationships}

Rules:
- Generate ONLY valid SQL, no explanations.
- Use the {dialect} dialect.
- Add SQL comments for complex parts.
- If the query is ambiguous, ask for clarification.
- NEVER generate destructive queries (DROP, DELETE without WHERE, etc.)
- If the user explicitly asks for a delete/update, ask for confirmation.

Question: {question}
SQL:
```

### 6.3 Security
- Destructive query filtering in read-only mode
- Syntax validation before execution
- Returned row limit (default 1000)
- Default timeout of 30s

## 7. Response Format

### 7.1 Success (SELECT)
```
┌─────────────────────────────────────────────────────────┐
│ Result: 127 rows (0.042s)                               │
├──────┬────────────┬──────────────┬──────────────────────┤
│  id  │  name      │  email       │  created_at          │
├──────┼────────────┼──────────────┼──────────────────────┤
│  1   │  John Doe  │ john@ex.com  │  2024-03-15 10:30:00│
│  2   │  Jane Smith│ jane@ex.com  │  2024-03-14 09:15:00│
└──────┴────────────┴──────────────┴──────────────────────┘
```

### 7.2 Modification (INSERT/UPDATE/DELETE)
```
⚠ Destructive query detected:
  "DELETE FROM users WHERE last_login < '2020-01-01'"
→ 15 rows affected.
Confirm? [y/N]
```

### 7.3 SQL Error
```
❌ SQL error: column "emial" does not exist
💡 Suggestion: Did you mean "email"?
```

## 8. Configuration

File `~/.config/dbchat/config.toml`:

```toml
[llm]
provider = "openai"        # openai | anthropic | ollama
model = "gpt-4o-mini"
temperature = 0.0
api_key = "env:OPENAI_API_KEY"  # or direct value

[db]
max_connections = 5
query_timeout_secs = 30
read_only = true
max_rows = 1000
safe_mode = true           # confirmation before destructive queries

[display]
theme = "dark"             # dark | light
show_sql = true            # show generated SQL
show_timing = true         # show execution time
format = "table"           # table | json | csv

[history]
enabled = true
max_entries = 1000
file = "~/.local/share/dbchat/history.txt"
```

## 9. CLI Interface (clap)

```bash
dbchat [OPTIONS] [CONNECTION_STRING]

COMMANDS:
  postgres <uri>    Connect to PostgreSQL
  mysql <uri>       Connect to MySQL
  sqlite <path>     Connect to SQLite
  config            Manage configuration
  version           Show version
  help              Show help

OPTIONS:
  -q, --query <SQL>    One-shot query (non-interactive)
  -f, --format <FMT>   Output format [table, json, csv]
  --read-only          Strict read-only mode
  --no-color           Disable colors
  -v, --verbose        Verbose mode
  --safe               Safe mode (confirmation before writes)
```

## 10. Error Handling

| Error                     | Behavior                                           |
|---------------------------|---------------------------------------------------|
| Connection failed         | Clear message with verification suggestion        |
| Invalid SQL               | Recovery: send error back to LLM for correction   |
| LLM unavailable           | Ollama fallback or explicit error message         |
| Query timeout             | Warn and suggest optimization                     |
| Result too large          | Automatic pagination                              |
