# Runique — Contexte projet pour Claude Code

## Projet

Framework web Rust inspiré de Django. Workspace :
- `runique/` — lib principale
- `demo-app/` — application de démonstration
- `demo-app/migration/` — migrations SeaORM

**Stack :** Axum 0.8.7 · SeaORM 2.0.0-rc.32 · Tera 1.20.1 · Rust 1.85 (edition 2024)

## Version courante

Voir `[workspace.package] version` dans `Cargo.toml` racine.

## Builder API (point d'entrée)

```rust
use runique::app::builder::RuniqueAppBuilder as builder;

builder::new(config)
    .routes(url::routes())
    .with_database(db)
    .statics()
    .middleware(|m| {
        m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)
            .with_session_cleanup_interval(5)
            .with_allowed_hosts(|h| h.enabled(!is_debug()).host("localhost:3000"))
            .with_csp(|c| {
                c.policy(SecurityPolicy::strict())
                 .with_header_security(true)
                 .with_upgrade_insecure(!is_debug())
            })
    })
    .with_admin(|a| {
        a.site_title("Administration")
         .auth(RuniqueAdminAuth::new())
         .page_size(15)
    })
    .build().await?
    .run().await?
```

## Ordre des middlewares (slots, requête entrante)

```
Extensions(0) → Compression(5) → ErrorHandler(10) → Custom(20+)
→ CSP/Headers(30) → Cache(40) → Session(50) → SessionUpgrade(55)
→ CSRF(60) → HostValidation(70) → Handler
```

CSRF est **toujours activé** (slot 60, non configurable).

## API sécurité — builders

```rust
// Rate limiting
RateLimiter::new().max_requests(100).retry_after(60).spawn_cleanup(...)

// Anti brute-force login (dans handler, après Prisme)
LoginGuard::new().max_attempts(5).lockout_secs(300).spawn_cleanup(...)

// Host validation (dans middleware builder)
.with_allowed_hosts(|h| h.enabled(bool).host("monsite.fr"))
```

`effective_key(username, ip)` : clé par username si non vide, sinon `"anonym:{ip}"`.

## RuniqueEnv & is_debug

- `utils/env.rs` — `RuniqueEnv` enum `Development|Production` en `LazyLock`
- `is_debug() -> bool` — `DEBUG=true` dans `.env` → dev, absent → prod
- `css_token() -> String` — cache-buster 4 chiffres pour assets
- `init_logging()` — lit `is_debug()`, `RUST_LOG` override possible

## Tests

- Point d'entrée : `runique/tests/mod.rs`
- Helpers : `runique/tests/helpers/` (server, request, assert, db, db_postgres, db_mariadb)
- Helpers clés : `build_engine()`, `request::get()`, `assert_body_str()`, `fresh_db()`
- `cargo test --tests` — intégration seule (exclut `#[cfg(test)]` inline)
- Docker requis : Postgres (5433), MariaDB (3307)
- 2 tests ignored : bug SQLx Windows UTF-8 (validés sur Linux/CI)

## Couverture

- Rapport : `docs/couverture_test.md`
- Commande : `cargo llvm-cov --package runique --ignore-filename-regex "admin" --summary-only`
- Cible : ~80% (fichiers à 0% = HTTP stack requis, non testables en unit)

## Fichiers à 0% couverture (nécessitent HTTP stack complet)

`app/builder.rs`, `app/error_build.rs`, `app/runique_app.rs`, `app/staging/`,
`app/templates.rs`, `bin/runique.rs`, `context/request/extractor.rs`,
`context/template.rs`, `forms/extractor.rs`, `forms/model_form/mod.rs`,
`migration/migrate.rs`

## derive_form

Crate de macros procédurales pour les formulaires. Publié sur crates.io : `derive_form = "1.1.34"`.
Le dossier `runique/derive_form/` est toujours présent localement.

## Audit sécurité (appliqué)

- `admin_main.rs:212-224` : `check_csrf()` → comparaison constant-time via `subtle::ct_eq`
- `forms/form.rs:60` : erreurs Tera échappées via `html_escape()`
- `middleware/errors/error.rs:505-511` : `html_escape` en `pub(crate)`
- P2 ouvert : `tracing::warn!()` dans `aegis.rs` (parse failures form)

## Conventions

- `impl_form_access!()` et `impl_from_error!()` couvertes indirectement
- Filtres Tera privés testés via `register_asset_filters` + rendu templates
- `password` fields skippés par `Forms::fill()` → utiliser `add_value()` directement
- CSRF admin : token brut comparé brut (≠ Prisme qui masque/démasque)
- `AllowedExtensions` : `FileField::image()`, `document()`, `any()`, `.allowed_extensions(vec![...])`

## Environnement développeur

- `python3` ne fonctionne pas sur ce poste Windows — utiliser `uv run` (uv installed) ou `node`
- Shell : bash (syntaxe Unix, pas PowerShell)
