# dbchat

Chattez avec votre base de données en langage naturel.

```
dbchat postgres://user:pass@localhost/mydb
dbchat> "donne-moi les 5 derniers clients"
```

## Base de test (Docker)

```bash
cd test-db && docker compose up -d
```

Connexion :
```bash
dbchat mysql://dbchat:dbchat@localhost:3306/boutique
```

Documentation complète du schéma et exemples → [`test-db/README.md`](test-db/README.md)

## Installation

### Via curl (recommandé)

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

## Utilisation

```bash
# Utilise la dernière configuration connue
dbchat

# Connexion + mode interactif
dbchat postgres://user:pass@localhost/mydb
dbchat mysql://user:pass@localhost:3306/mydb
dbchat sqlite:///data/mydb.db

# One-shot
dbchat postgres://... -q "quel est le CA total ?"
dbchat postgres://... -q "top 10 produits" -f json

# Avec surcharge du modèle/provider
dbchat postgres://... --provider anthropic --model claude-haiku-4-5
dbchat postgres://... --provider openrouter --model openrouter/free
```

## Commandes interactives

| Commande | Description |
|----------|-------------|
| `votre question` | Question en langage naturel |
| `/tables` | Liste les tables |
| `/schema` | Schéma détaillé (colonnes, types, clés) |
| `/context` | Contexte envoyé au LLM |
| `/verbose` | Active/désactive le mode verbose |
| `/history` | Historique des questions |
| `/config` | Configuration courante |
| `/refresh` | Re-scanne le schéma |
| `/clear` | Efface l'écran |
| `/exit` | Quitte |

## Configuration

Fichier : `~/.config/dbchat/config.toml`

```toml
[llm]
provider = "google"            # google | openai | anthropic | ollama | openai-compatible
model = "gemini-3.1-flash-lite"
api_key = "env:GOOGLE_API_KEY" # ou une cle directe
# api_url = "https://api.deepseek.com" # requis pour openai-compatible / OpenRouter

[db]
engine = "Postgres"
uri = "postgres://user:pass@localhost/mydb"
max_connections = 5
query_timeout_secs = 30
read_only = true
safe_mode = true

[display]
locale = "fr"                  # fr | en (auto-détecté via LANG)
format = "table"               # table | json | csv
show_sql = true
verbose = false
```

```bash
dbchat config         # Assistant interactif
dbchat config init    # Crée/réinitialise la config par défaut
dbchat config show    # Affiche la config courante
```

Le menu interactif permet de configurer la connexion BD, le LLM et les paramètres
opérationnels de sécurité (`read_only`, `safe_mode`, `max_rows`, timeout).

Modèles via API key :

```bash
dbchat config
# puis: LLM -> Gratuits / free tier
# ou:   LLM -> Payants courants
```

Le wizard configure automatiquement le provider, le modèle, l'URL API si besoin,
et la variable d'environnement à utiliser pour la clé.

| Choix | Modèle | Clé |
|-------|--------|-----|
| Gratuit / free tier | `gemini-3.1-flash-lite` | `GOOGLE_API_KEY` |
| Gratuit OpenRouter | `openrouter/free` | `OPENROUTER_API_KEY` |
| Gratuit OpenRouter | `google/gemma-4-31b-it:free` | `OPENROUTER_API_KEY` |
| Gratuit OpenRouter | `cohere/north-mini-code:free` | `OPENROUTER_API_KEY` |
| Payant low-cost | `deepseek-v4-flash` | `DEEPSEEK_API_KEY` |
| Payant courant | `gpt-5.4-mini`, `gpt-5.5` | `OPENAI_API_KEY` |
| Payant courant | `claude-haiku-4-5`, `claude-sonnet-5` | `ANTHROPIC_API_KEY` |
| Payant courant | `gemini-3.5-flash` | `GOOGLE_API_KEY` |

Exemple gratuit :

```toml
[llm]
provider = "google"
model = "gemini-3.1-flash-lite"
api_key = "env:GOOGLE_API_KEY"
```

Exemple OpenRouter gratuit :

```toml
[llm]
provider = "openai-compatible"
model = "openrouter/free"
api_url = "https://openrouter.ai/api/v1"
api_key = "env:OPENROUTER_API_KEY"
```

Exemple low-cost :

```toml
[llm]
provider = "openai-compatible"
model = "deepseek-v4-flash"
api_url = "https://api.deepseek.com"
api_key = "env:DEEPSEEK_API_KEY"
```

`deepseek-v4-flash` utilise l'API DeepSeek au format compatible OpenAI. Les
modèles `:free` passent par OpenRouter et peuvent changer selon la disponibilité
du provider.

## Options CLI

```
  -q, --query <QUERY>        Mode one-shot
  -f, --format <FORMAT>      table, json, csv
  -v, --verbose              Mode verbose
      --model <MODEL>        Surcouche du modèle
      --provider <PROVIDER>  Surcouche du provider
      --read-only            Bloque les requêtes destructives
      --no-color             Désactive les couleurs
```
