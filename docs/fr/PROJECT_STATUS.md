
# Runique Framework — Project Status

Ce document consolide l'état réel du dépôt à partir des sources de référence :

- `Cargo.toml` (version workspace)
- `README.md`
- `CHANGELOG.md`

---

## Snapshot (au 10 juin 2026)

- **Version workspace** : `2.1.15`
- **derive_form** : `2.1.8`
- **Licence** : MIT
- **Branche** : `main`
- **Stack** : Axum 0.8.7 + SeaORM 2.0.0-rc.38 + Tera 1.20.1 · Rust edition 2024 · Rust 1.88

---

## Périmètre du workspace

- `runique` — crate framework principale
- `derive_form` — proc-macro DSL (model!{}, extend!{})
- `demo-app` — application de validation du framework
- `demo-app/migration` — migrations liées à la demo-app

---

## Fonctionnalités en place

### Formulaires
- Système de formulaires typés : `#[form]`, `RuniqueForm`, validation, rendu HTML via Tera
- Protection CSRF intégrée (token masqué anti-BREACH, comparaison temps constant)
- `FormTracing` structuré + `eprintln!` debug sur tout le pipeline (field, set_value, validate, finalize, render)
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
- Watcher via `runique start` (debounce 300ms, génération initiale au démarrage)
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
- `LoginGuard` — protection contre brute-force login
- `AntiBot` — honeypot configurable par scope
- Sanitization HTML (ammonia), argon2/bcrypt/scrypt pour les mots de passe
- Redirections sécurisées (open-redirect guard), cookies `HttpOnly`/`SameSite=Strict`/`Secure`

### ORM / Migrations
- `model!{}` DSL → entité SeaORM + migration SQL + AdminForm
- `extend!{}` — extension de tables framework (ex. `eihwaz_users`)
- `makemigrations` avec détection des changements destructifs + prompt de confirmation
- Backends supportés : PostgreSQL, MariaDB, SQLite

### I18n
- 8 langues (en, fr, de, es, it, pt, ja, zh), stockage `AtomicU8`, `RUNIQUE_LANG`

### CLI
- `runique new`, `runique start`, `runique create-superuser`, `runique makemigrations`, `runique migration`

---

## Sécurité — historique des corrections

| Version | Faille | Sévérité |
|---------|--------|----------|
| 2.1.9 | Injection SQL dans les filtres de liste admin | Élevée |
| 2.1.9 | Fixation de session au login (cycle_id manquant) | Moyenne |
| 2.1.9 | Granularité droits write admin (create/update/delete indistincts) | Moyenne |
| 2.1.9 | IDOR — can_update_own/can_delete_own non appliqués | Faible |

---

## État admin — permissions

- `can_read`, `can_create`, `can_update`, `can_delete` : appliqués par opération ✅
- `can_update_own`, `can_delete_own` : appliqués quand `own_field` est déclaré dans `admin!{}` ✅
- Permissions par groupe, cache mémoire avec révocation immédiate ✅

---

## Correctifs à apporter / roadmap

### Priorité haute (v2.x)
- **SQLi filtres via `configure {}`** : les filtres des ressources builtin passent par un chemin distinct, à vérifier
- **Tests de non-régression sécurité** : ajouter tests couvrant la whitelist SQL, le cycle_id, les gardes par opération

### Priorité moyenne (v3.0, breaking)
- **Validation séquentielle S1/S2/S3** : `req.form()` → S1 CSRF → S2 règles → S3 données accessibles. Garantie structurelle que CSRF + validation précèdent tout accès aux données (~115 call sites)
- **TypeState form** : variante `validate() -> Result<ValidForm<T>, T>`

### Priorité basse
- **Tokens de reset en mémoire** : non persistants au redémarrage, inopérants multi-instance
- **`makemigrations` DROP COLUMN** : colonnes supprimées non détectées
- **Couverture ciblée** : `migration/migrate.rs` (22%), `engine/core.rs` (50%), `forms/fields/file.rs` (61%)

---

## Références

- Repository : [github.com/seb-alliot/runique](https://github.com/seb-alliot/runique)
- Changelog : [CHANGELOG.md](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- Documentation : [English](https://github.com/seb-alliot/runique/tree/main/docs/en) | [Français](https://github.com/seb-alliot/runique/tree/main/docs/fr)

---

**Dernière mise à jour** : 30 mai 2026
**Statut global** : ✅ Framework stable · 🟡 Admin bêta mature · 🔒 Audit sécurité complété 2026-05-28 · 📖 Documentation API publique complète (docs.rs)
