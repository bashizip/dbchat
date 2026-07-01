# dbchat — Spécifications

## 1. Vision
**dbchat** est un outil CLI en Rust qui permet de dialoguer en langage naturel avec une base de données relationnelle. L'utilisateur décrit ce qu'il veut en français/anglais, et dbchat traduit en SQL natif via un LLM, exécute la requête, et présente le résultat de façon riche et lisible.

## 2. Architecture globale

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

## 3. Stack technique

| Composant       | Technologie                                      |
|----------------|--------------------------------------------------|
| Langage        | Rust (edition 2024)                              |
| Async runtime  | Tokio                                            |
| CLI args       | clap v4 (derive)                                 |
| TUI / REPL     | rustyline + crossterm (ou ratatui si temps)      |
| Base de données| SQLx (PostgreSQL, MySQL, SQLite, MSSQL)          |
| LLM            | genai (multi-provider: OpenAI, Anthropic, Ollama)|
| Coloration     | anstyle + anstream                               |
| Tableaux       | tabled (derive Tabled)                           |
| Markdown       | termimad (rendu markdown dans le terminal)       |
| Sérialisation  | serde + toml (configuration)                     |

## 4. Modes de fonctionnement

### 4.1 Mode Connexion
```bash
dbchat postgres://user:pass@localhost/mydb
dbchat mysql://user:pass@localhost/mydb
dbchat sqlite:mydb.db
```

### 4.2 Mode Interactif (REPL)
```bash
dbchat postgres://user:pass@localhost/mydb
> "donne-moi les 5 derniers clients"
> "combien de commandes par mois en 2024 ?"
> "montre les produits les plus vendus"
> /tables    # liste les tables
> /schema    # montre le schéma complet
> /context   # montre le contexte envoyé au LLM
> /exit      # quitte
```

### 4.3 Mode One-shot (batch)
```bash
dbchat --db postgres://... --query "quel est le CA total ?"
dbchat --db postgres://... --query "top 10 produits" --format json
```

## 5. DB Layer (SQLx)

### 5.1 Connecteurs supportés
| Moteur  | URI                                       | Statut |
|---------|-------------------------------------------|--------|
|PostgreSQL| `postgres://user:pass@host:port/db`      | ✅ phase 1 |
|MySQL    | `mysql://user:pass@host:port/db`          | ✅ phase 1 |
|SQLite   | `sqlite://path/to/db.sqlite`             | ✅ phase 2 |
|MSSQL    | `mssql://user:pass@host:port/db`          | 🔄 phase 3 |

### 5.2 Schéma extrait (pour contexte LLM)
Pour chaque table :
- Nom, colonnes (nom, type, nullable, pk, fk)
- Index et contraintes uniques
- Relations (clés étrangères)
- N-échantillons de données (3-5 lignes)
- Statistiques de base (row count)

Tout ceci est envoyé dans le prompt système du LLM pour lui donner le contexte.

### 5.3 Optimisations DB
- Connection pooling configurable (max conns, timeout)
- Préparation des schémas en parallèle
- Cache du schéma en mémoire (invalidé si /refresh)
- Timeout des queries dangereuses (configurable)
- `EXPLAIN` optionnel avant exécution
- Mode read-only protégé (aucun `INSERT/UPDATE/DELETE` sans confirmation)

## 6. LLM Layer (genai)

### 6.1 Providers
| Provider   | Modèle recommandé          | Configuration           |
|------------|---------------------------|-------------------------|
| OpenAI     | gpt-4o-mini / gpt-4o      | `OPENAI_API_KEY`        |
| Anthropic  | claude-3-haiku / sonnet   | `ANTHROPIC_API_KEY`     |
| Ollama     | llama3 / mixtral / qwen   | `OLLAMA_URL` (localhost)|
| Google     | gemini-2.0-flash          | `GOOGLE_API_KEY`        |

### 6.2 Prompt Engineering

**Système**: contexte DB (schéma, dialecte SQL)
**User**: question en langage naturel

```
Tu es un expert SQL pour {dialect} (PostgreSQL/MySQL/...).
Voici le schéma de la base:

{schéma complet avec types, contraintes, relations}

Règles:
- Génère UNIQUEMENT du SQL valide, sans explications.
- Utilise le dialecte {dialect}.
- Ajoute des commentaires SQL pour les parties complexes.
- Si la requête est ambiguë, demande des précisions.
- Ne génère JAMAIS de requêtes destructrices (DROP, DELETE sans WHERE, etc.)
- Si l'utilisateur demande explicitement une suppression/modification, demande confirmation.

Question: {question}
SQL:
```

### 6.3 Sécurité
- Filtrage de requêtes destructrices en mode read-only
- Validation syntaxique avant exécution
- Limitation du nombre de lignes retournées (default 1000)
- Timeout par défaut de 30s

## 7. Format de réponse

### 7.1 Succès (SELECT)
```
┌─────────────────────────────────────────────────────────┐
│ Résultat: 127 lignes (0.042s)                           │
├──────┬────────────┬──────────────┬──────────────────────┤
│  id  │  name      │  email       │  created_at          │
├──────┼────────────┼──────────────┼──────────────────────┤
│  1   │  Jean Dupont│ jean@ex.com  │  2024-03-15 10:30:00│
│  2   │  Marie Curie│ marie@ex.com │  2024-03-14 09:15:00│
└──────┴────────────┴──────────────┴──────────────────────┘
📊 Statistiques:
   •  127 clients actifs
   •  Période: mars 2024 - mars 2025
   •  Moyenne: 3.2 commandes/client
```

### 7.2 Modification (INSERT/UPDATE/DELETE)
```
⚠ Requête de modification détectée:
  "DELETE FROM users WHERE last_login < '2020-01-01'"
→ 15 lignes affectées.
Confirmer ? [y/N]
```

### 7.3 Erreur SQL
```
❌ Erreur SQL: column "emial" does not exist
💡 Suggestion: Vouliez-vous dire "email" ?
```

## 8. Configuration

Fichier `~/.config/dbchat/config.toml`:

```toml
[llm]
provider = "openai"        # openai | anthropic | ollama
model = "gpt-4o-mini"
temperature = 0.0
api_key = "env:OPENAI_API_KEY"  # ou valeur directe

[db]
max_connections = 5
query_timeout_secs = 30
read_only = true
max_rows = 1000
safe_mode = true           # confirmation avant destructive queries
show_explain = false       # affiche le plan d'exécution

[display]
theme = "dark"             # dark | light
locale = "fr"              # fr | en
show_sql = true            # affiche le SQL généré
show_timing = true         # affiche le temps d'exécution
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
  postgres <uri>    Connexion PostgreSQL
  mysql <uri>       Connexion MySQL
  sqlite <path>     Connexion SQLite
  config            Gérer la configuration
  version           Afficher la version
  help              Aide

OPTIONS:
  -q, --query <SQL>    Requête en one-shot (non interactif)
  -f, --format <FMT>   Format de sortie [table, json, csv]
  --read-only          Mode lecture seule stricte
  --no-color           Désactiver les couleurs
  -v, --verbose        Mode verbeux
  --safe               Mode sécurisé (confirmation avant écriture)
```

## 10. Gestion des erreurs

| Erreur                     | Comportement                                      |
|---------------------------|---------------------------------------------------|
| Connexion impossible      | Message clair avec suggestion de vérification     |
| SQL invalide              | Rattrapage: renvoyer l'erreur au LLM pour correction |
| LLM indisponible          | Fallback Ollama ou message d'erreur explicite     |
| Query timeout             | Avertir et proposer une optimisation              |
| Résultat trop volumineux  | Pagination automatique                            |
