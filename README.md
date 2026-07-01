# dbchat

Chattez avec votre base de données en langage naturel.

```
dbchat postgres://user:pass@localhost/mydb
dbchat> "donne-moi les 5 derniers clients"
```

## Installation

```bash
cargo install --path .
```

## Utilisation

```bash
# Connexion + mode interactif
dbchat postgres://user:pass@localhost/mydb
dbchat mysql://user:pass@localhost:3306/mydb
dbchat sqlite:///data/mydb.db

# One-shot
dbchat postgres://... -q "quel est le CA total ?"
dbchat postgres://... -q "top 10 produits" -f json

# Avec surcharge du modèle/provider
dbchat postgres://... --provider anthropic --model claude-3-haiku
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
provider = "openai"            # openai | anthropic | ollama
model = "gpt-4o-mini"
api_key = "sk-..."             # ou via OPENAI_API_KEY

[db]
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
dbchat config init    # Crée la config par défaut
dbchat config show    # Affiche la config
```

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
