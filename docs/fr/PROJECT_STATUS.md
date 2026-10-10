
# Runique Framework — Project Status

Ce document consolide l'état réel du dépôt à partir des sources de référence :

- `Cargo.toml` (version workspace)
- `README.md`
- `CHANGELOG.md`

---

## Snapshot (au 9 octobre 2026)

- **Version workspace** : `3.0.3`
- **derive_form** : `3.0.0`
- **runique_dsl** : `0.1.0`
- **Licence** : MIT
- **Branche** : `main`
- **Stack** : Axum 0.8.9 + SeaORM 2.0.4 + Tera 2.4 · Rust edition 2024 · Rust 1.94

---

## Périmètre du workspace

- `runique` — crate framework principale
- `runique_dsl` — parseur du DSL des modèles, partagé par les macros et la CLI
- `derive_form` — macros procédurales (`model!{}`, `extend!{}`, `#[form]`)
- `demo-app` — application de validation du framework
- `demo-app/migration` — migrations liées à la demo-app

---

## Fonctionnalités en place

### Formulaires
- Système de formulaires typés : `#[form]`, `RuniqueForm`, validation, rendu HTML via Tera
- `FormField::validate`/`finalize` async — I/O réellement non-bloquant (upload de fichier via `tokio::fs`, hash de mot de passe) sous la façade déjà async de `is_valid()`
- `ValidationForm<F>` (`forms/validation_form.rs`) : `try_new(form, request) -> Result<ValidationForm<F>, F>`, preuve par le type qu'un handler a validé avant d'agir — additif, ne remplace pas `RuniqueForm::is_valid()` ; dispatch par défaut sur `Method::is_safe()`, hooks `register_dynamic_fields`/`allow_get`/`allow_post` surchargeables
- Protection CSRF intégrée (token masqué anti-BREACH, comparaison temps constant) ; échec CSRF pose désormais un message explicite (`csrf.invalid_or_missing`) au lieu d'échouer silencieusement, et `js/csrf.js` rafraîchit le token juste avant chaque soumission de formulaire
- Tracing structuré par domaine (arbre `RuniqueLog`) sur tout le pipeline (field, set_value, validate, finalize, render)
- Tous les types de champs : Text, Numeric, Boolean, Choice, Radio, Checkbox, Date, Time, DateTime, Duration, File, Color, Slug, UUID, JSON, IP, Hidden, Honeypot
- Garde `save()` / `save_as()` : retourne `Err` si `is_valid()` n'a pas été appelé ou a retourné `false` — empêche toute persistance sans validation préalable

### Routing
- `urlpatterns!{}` avec segments typés, GET/POST séparés
- URL registry nommée, helper `{% url %}` Tera

### Templates
- Moteur Tera + helpers de contexte (`{% csrf %}`, `{% static %}`, `{% url %}`, `{% media %}`)
- Autoescape actif sur `.html`/`.xml`

### Admin panel (stable bêta)
- DSL `admin!{}` déclaratif → génération de `src/admins/` par le daemon
- `runique start` : génère `src/admins/` une fois, puis lance l'application (pas de watcher)
- CRUD complet généré : list, detail, create, edit, delete, bulk edit, bulk delete, group actions
- `list_display`, `list_filter` (valeurs distinctes paginées), `search!` sur toutes colonnes
- `group_action` : booléens et valeurs enum exactes, fusion multi-entrées même champ
- `bulk_create` : upsert par valeur (split par virgule), auto-génération `edit_form_builder`
- `m2m` : relations many-to-many via table de jonction
- `own_field` : vérification d'appartenance pour `can_update_own`/`can_delete_own`
- Historique des actions admin (log, batch_id, diff old/new)
- Templates surchargeables par ressource

### Sécurité
- CSRF masqué (protection BREACH), token lié à la session, comparaison `subtle::ct_eq`
- `session.cycle_id()` au login — protection fixation de session
- Permissions admin granulaires par opération (`can_create`, `can_update`, `can_delete`, `can_update_own`, `can_delete_own`)
- Whitelist de colonnes SQL générée statiquement — protection injection SQL dans filtres/tri admin
- CSP builder avec nonce, HSTS, host validation
- `RateLimiter` global + par méthode HTTP (`rate_limit_get()`, `rate_limit_post()`, etc.)
- `LoginGuard` — protection contre brute-force login (à activer pour l'admin : `with_login_guard`)
- Compte relu en base à chaque requête (`request.user`) : la session ne garde que l'id, une désactivation ou un retrait de droits s'applique à la requête suivante ; activation garantie par une contrainte `CHECK` (`is_active` ⇒ `activated_at`)
- Uploads (3.0.2) : aucun fichier écrit avant un jeton CSRF valide, dossiers de staging jamais servis, CSP sans script sur `/media`
- Taille des corps de requête bornée (2 Mo par défaut, `RUNIQUE_MAX_UPLOAD_MB` pour relever)
- `AntiBot` — honeypot configurable par scope
- Sanitization HTML (ammonia), argon2/bcrypt/scrypt pour les mots de passe
- Redirections sécurisées (open-redirect guard), cookies `HttpOnly`/`SameSite=Strict`/`Secure`
- Login en temps constant (verify contre dummy-hash — pas d'énumération d'utilisateurs)
- Tokens de reset persistés en DB : hashés SHA-256, single-use, durcis IDOR (mutation par l'id utilisateur lié au token)
- CSRF vérifié à la fois via le champ body `csrf_token` et le header `X-CSRF-Token` (2026-09-02) — nécessaire pour les clients JSON/`fetch` qui ne peuvent pas poser de champ de formulaire caché ; sûr par construction, un `fetch` cross-origin qui pose ce header déclenche un preflight CORS que Runique n'autorise jamais par défaut (`CorsConfig` désactivé, `any_origin() + allow_credentials(true)` = erreur au build)

### ORM / Migrations
- `model!{}` DSL → entité SeaORM + migration SQL + AdminForm
- `extend!{}` — extension de tables framework (ex. `eihwaz_users`)
- `makemigrations` — plan → validate → **commit/rollback atomique** + snapshots ; `DROP COLUMN` sur colonnes supprimées (garde destructive)
- Backends supportés : PostgreSQL, MariaDB, SQLite
- Alias `Pk` : `i32` par défaut, `i64` (`big-pk`) ou `Uuid` via `Uuid::now_v7()` (`pk-uuid`) — features mutuellement exclusives (`compile_error!`). Utilisable sur n'importe quel champ (pas seulement la PK), typiquement une FK, pour rester automatiquement synchronisé avec le type de la table référencée
- `model!{}` — syntaxe de champs unifiée (l'ancienne grammaire `fields: { name: SqlType }` est supprimée) : bloc anonyme unique, 43 types sémantiques, options `readonly`/`label` incluses
- Générateur de migrations durci multi-moteurs (2026-09-01) : gardes runtime pour `CREATE TYPE`/triggers `updated_at` (au lieu d'un choix figé à la génération), casse d'identifiant Postgres corrigée sur `ALTER TYPE`, ordre `TYPE`/`USING` invalide corrigé, `modify_column` sauté sous SQLite (panique sea-query) ; FK toujours inline en `CREATE TABLE`
- `makemigrations` reconnaît désormais un `migration/` initialisé via `sea-orm-cli migrate init` (`lib.rs` reformaté canoniquement, placeholder `todo!()` supprimé), et branche les tables du framework même sans aucun modèle propre au projet (3.0.2) — cf. [Migrations](/docs/fr/installation/migrations)
- Recherche/filtres portables multi-moteurs (2026-09-02) : `CAST(col AS TEXT)` invalide sur MySQL/MariaDB (exige `CHAR`) — corrigé via des helpers `text_cast_type`/`text_eq`/`ilike` détectant le moteur à l'exécution (`db.get_database_backend()`). `search_cond!` prend désormais la connexion `db` en premier argument sur ses 4 formes — cf. [Requêtes](/docs/fr/orm/requetes)

### I18n
- 9 langues (en, fr, de, es, it, pt, ja, zh, ru), stockage `AtomicU8`, `RUNIQUE_LANG`

### Tracing & observabilité
- Arbre `RuniqueLog` par domaine (forms, middleware, session, auth, admin, db, mailer, migration, templates, errors, builder), chaque feuille un `Option<LogLevel>`
- `TraceResult::trace` / `trace_or` — les `Result` avalés loggent leur `file:line` ; les sites sensibles à la sécurité plancher à `WARN` même catégorie désactivée
- Sorties : stdout couleurs, fichiers roulants (JSON/plain, non bloquants), `LogSink` custom (aucun type `tracing` exposé) ; `.external()` délègue le subscriber global à l'app hôte
- Override runtime `RUNIQUE_LOG_FILE`

### CLI
- `runique new`, `runique start`, `runique create-superuser`, `runique makemigrations`, `runique migration up` (retour arrière et état : `sea-orm-cli`), `runique test`

---

## Sécurité — historique des corrections

| Version | Faille | Sévérité |
|---------|--------|----------|
| 2.1.9 | Injection SQL dans les filtres de liste admin | Élevée |
| 2.1.9 | Fixation de session au login (cycle_id manquant) | Moyenne |
| 2.1.9 | Granularité droits write admin (create/update/delete indistincts) | Moyenne |
| 2.1.9 | IDOR — can_update_own/can_delete_own non appliqués | Faible |
| 2.1.15 | Énumération d'utilisateurs via timing attack au login | Moyenne |
| 2.1.15 | Contrôle d'accès manquant sur l'action admin reset-password | Moyenne |
| 2.1.17 | Tokens de reset : mémoire → DB (hashés, single-use, durcis IDOR) | Durcissement |
| 3.0.3 | `bulk_create` : le seul droit de création permettait de modifier des lignes existantes | Élevée |
| 3.0.3 | Un membre du staff pouvait prendre le compte d'un superutilisateur (changement d'email + réinitialisation) | Élevée |
| 3.0.3 | Open redirect via une tabulation, un `Location` non ASCII ou l'en-tête `Refresh` | Moyenne |
| 3.0.2 | XSS stockée via un upload en staging (chemin réaffiché, dossier servi sans CSP) | Élevée |
| 3.0.2 | Fichiers écrits sur disque avant la vérification CSRF (multipart) | Moyenne |
| 3.0.2 | Open redirect via identifiant dans l'URL (`site.com:x@evil.com`) | Moyenne |
| 3.0.2 | Corps urlencoded/JSON sans limite de taille | Moyenne |
| 3.0.2 | Reset-password admin possible depuis une ressource hors comptes | Faible |
| 3.0.2 | Admin : jeton CSRF brut accepté (masquage BREACH contourné) | Durcissement |

---

## État admin — permissions

- `can_read`, `can_create`, `can_update`, `can_delete` : appliqués par opération ✅
- `can_update_own`, `can_delete_own` : appliqués quand `own_field` est déclaré dans `admin!{}` ✅
- Permissions par groupe, relues en base à chaque requête (révocation immédiate) ✅

---

## Correctifs à apporter / roadmap

### Réglé (vérifié le 9 octobre 2026)
- **Filtres admin et injection SQL** : les ressources générées n'acceptent que les colonnes de leur liste blanche, avec des valeurs liées ; la ressource intégrée `users` ignore les filtres de l'URL
- **Tests de non-régression sécurité** : rotation de session et du jeton CSRF au login, contrôles par opération, liste blanche des colonnes, et les correctifs 3.0.2 (chacun vérifié par mutation)

### Priorité haute (3.x)
- **Refonte de l'admin** autour d'un builder typé `ModelAdmin<Entity>`, sans code généré — voir [l'ébauche](https://github.com/seb-alliot/runique/blob/main/ebauche-model-admin.md)

### Priorité basse
- **Couverture** : détail par fichier dans [couverture_test.md](https://github.com/seb-alliot/runique/blob/main/docs/couverture_test.md) (8 octobre : `migrate.rs` 79 %, `engine/core.rs` 92 %, `forms/fields/file.rs` 96 %)

---

## Références

- Repository : [github.com/seb-alliot/runique](https://github.com/seb-alliot/runique)
- Changelog : [CHANGELOG.md](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- Documentation : [English](https://github.com/seb-alliot/runique/tree/main/docs/en) | [Français](https://github.com/seb-alliot/runique/tree/main/docs/fr)

---

**Dernière mise à jour** : 9 octobre 2026
**Statut global** : ✅ Framework stable · 🟡 Admin bêta mature · 🔒 Sécurité : tokens de reset durcis en DB, auth en temps constant · 📖 Documentation API publique complète (docs.rs)
