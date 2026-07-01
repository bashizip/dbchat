# Roadmap dbchat

## Phase 0: Fondations (semaine 1)
**Objectif: projet Rust compilable avec l'architecture modulaire**

- [x] Initialisation du projet Cargo (workspace lib + bin)
- [x] Structures de données core (DBConfig, LLMConfig, AppConfig)
- [x] CLI skeleton avec clap (sous-commandes, parsing URI)
- [x] Connexion SQLx basique (PostgreSQL)
- [x] Module d'erreurs unifié (thiserror)
- [x] `cargo run -- --help` fonctionnel

## Phase 1a: DB Layer (semaine 1-2)
**Objectif: connecter, introspecter, exécuter du SQL brut**

- [ ] Schema introspection: tables, colonnes, types, PK, FK
- [ ] Connection pooling configurable
- [ ] Exécution de queries avec timeout
- [ ] Support PostgreSQL (complet)
- [ ] Support MySQL
- [ ] Pagination des résultats
- [ ] Statistiques basiques (row count par table)
- [ ] N-échantillons (3-5 lignes par table)
- [ ] Module `Explain` (affichage du plan)

## Phase 1b: LLM Layer (semaine 2)
**Objectif: traduction NL → SQL via LLM**

- [ ] Provider OpenAI (streaming)
- [ ] Provider Anthropic (streaming)
- [ ] Provider Ollama (local, streaming)
- [ ] Construction dynamique du prompt système (schema + dialect)
- [ ] Extraction du SQL depuis la réponse LLM
- [ ] Validation / parsing du SQL généré
- [ ] Gestion des erreurs SQL: renvoi au LLM pour correction (max 2 itérations)
- [ ] Détection de requêtes destructrices (filtre sécurité)

## Phase 1c: CLI interactif (semaine 2-3)
**Objectif: session interactive complète**

- [ ] REPL avec rustyline (historique, édition, completion)
- [ ] Commandes slash: /tables, /schema, /help, /exit, /clear
- [ ] Affichage coloré des résultats (tabled)
- [ ] Rendu markdown léger (termimad)
- [ ] Mode one-shot (`-q "question"`)
- [ ] Gestion des signaux (Ctrl+C safe)
- [ ] Barre de progression / spinner pendant LLM

## Phase 2: Richesses (semaine 3-4)
**Objectif: expérience utilisateur premium**

- [x] Support SQLite (déjà implémenté Phase 1)
- [x] Configuration persistante (`~/.config/dbchat/config.toml`)
- [x] Multi-langue (fr + en détection auto via LANG)
- [x] Thèmes couleur (dark / light — support dans les configs)
- [x] Format JSON / CSV en sortie (`-f json`, `-f csv`)
- [x] Session history (replay / history)
- [x] Mode verbose pour debug (`/verbose`, `-v`)
- [ ] Autocomplétion des noms de tables dans le REPL (différé Phase 3)

## Phase 3: Avancé (semaine 4-6)
**Objectif: assistant DB intelligent**

- [ ] Support MSSQL
- [ ] Visualisation ASCII (bar charts dans le terminal)
- [ ] Suggestions de questions ("vous pourriez demander...")
- [ ] Mode `--watch` (requête répétée toutes les N secondes)
- [ ] Export des résultats en fichier (CSV, JSON, Parquet)
- [ ] Agrégations intelligentes (détection auto de GROUP BY)
- [ ] `EXPLAIN` visuel (arbre du plan en ASCII)
- [ ] Multi-turn context (questions de suivi)
- [ ] Connexion via socket SSH tunnel

## Phase 4: Production (semaine 6-8)
**Objectif: outil robuste et distribué**

- [ ] Tests d'intégration avec DB de test
- [ ] CI/CD (GitHub Actions, tests, lint)
- [ ] Package managers: Homebrew, cargo, Docker
- [ ] Auto-update check
- [ ] Documentation complète (man page, README)
- [ ] Benchmarking et optimisations perf
- [ ] Crash reporting (panic = message utile)
- [ ] Release v1.0.0
