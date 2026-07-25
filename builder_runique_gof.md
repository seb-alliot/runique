# `RuniqueAppBuilder` — Pattern GoF : Builder

`RuniqueAppBuilder` est une application du pattern **Builder** (GoF), avec deux particularités : un **directeur implicite** (le `main`) et un **builder hiérarchique** (des sous-builders par sous-système). L'idiome Rust — consommer `self` et renvoyer `Self` — garantit qu'aucun état de construction n'est partagé.

Fichiers de référence :
- `runique/src/app/builder/mod.rs` — `RuniqueAppBuilder` + méthodes fluent
- `runique/src/app/builder/build.rs` — `build()` : validation → assemblage
- `runique/src/app/staging/*` — sous-builders (`CoreStaging`, `MiddlewareStaging`, `AdminStaging`, `StaticStaging`)
- Class diagram complet : [diagramme/uml/app/builder-staging.md](diagramme/uml/app/builder-staging.md)

---

## Intention GoF

Séparer la **construction** d'un objet complexe de sa **représentation**, afin de construire **pas à pas** et de faire varier les parties.

**Problème résolu** : assembler une application web (DB, routes, middlewares, statics, admin, reset password, mailer, logs…) où presque tout est optionnel et où l'ordre d'assemblage a des contraintes (middlewares triés par slot, admin mergé avant la stack middleware…). Un constructeur à N paramètres serait ingérable ; on veut une API incrémentale validée à la fin.

## Participants

| Rôle GoF | Élément Runique |
|----------|-----------------|
| `Builder` | `RuniqueAppBuilder` (état partiel accumulé) |
| étapes de construction | `with_database()`, `routes()`, `middleware()`, `with_admin()`, `static_files()`, `with_log()`… — `(mut self, …) -> Self` |
| `ConcreteBuilder` | l'unique impl `RuniqueAppBuilder` |
| `Product` | `RuniqueApp` (produit fini, distinct du builder) |
| `Director` | **implicite** — le `main.rs` (le client) orchestre les appels |
| assemblage + invariants | `build()` : `validate()` + `all_ready()` → construit `RuniqueApp` |

---

## Les étapes d'accumulation

Chaque méthode `with_*` / `routes` / `middleware` prend `mut self`, écrit dans un champ du builder, et renvoie `self` (chaînage). Elles n'assemblent rien — elles **accumulent** :

- `new(config)` — initialise le builder avec la `RuniqueConfig`.
- `with_database(db)` / `with_database_config(cfg)` — pose la connexion (déjà ouverte) ou la config à connecter dans `build()`.
- `routes(router)` — enregistre le `Router` Axum applicatif.
- `middleware(|m| …)` — configure le sous-builder `MiddlewareStaging` via une closure.
- `with_admin(|a| …)` — configure le sous-builder `AdminStaging` (registry, auth, prefix).
- `static_files(|s| …)` / `statics()` / `no_statics()` — configure `StaticStaging`.
- `with_log(|l| …)` — configure le `RuniqueLog`.
- `with_password_reset(...)`, `with_mailer(...)`, `with_session_duration(...)`, `with_error_handler(...)` — options ponctuelles.

## Les étapes exactes de `build()` (l'assemblage)

`build()` est `async` et exécute une séquence **ordonnée** (`runique/src/app/builder/build.rs`) :

1. **Tracing d'abord** : `log_init(config.log)` + `init_subscriber()` — pour que `get_log()` fonctionne dès l'init des templates et du staging middleware.
2. **Validation** : `self.validate()` puis `all_ready()` → `BuildError` si un composant n'est pas prêt.
3. **Connexion DB** : `self.core.connect().await` (si une config DB est fournie).
4. **Destructuration** du staging (extensions, url_registry, middleware, statics, router).
5. **Core** — ordre strict : `TemplateLoader::init` (Tera) → `password_init` → construction de `RuniqueEngine` (config, tera, db, features, CSP, hosts, permissions-policy, trusted-proxies, session stores).
6. **URLs** : `add_urls(&engine)`.
7. **Password reset** : merge du router de reset (si configuré) + enregistrement des noms d'URL.
8. **Admin** : purge des droits orphelins au boot (`prune_orphan_droits`), merge du router admin, `robots.txt` optionnel.
9. **Middleware staging** : tri des middlewares par slot puis application (l'ordre des slots est la contrainte clé).
10. **Fichiers statiques** : `attach_static_files` (ServeDir + headers de sécurité par asset).
11. Branche le `session_db_store`.

→ Renvoie `RuniqueApp { router, engine }`. `run()` fait ensuite `axum::serve`.

---

## Particularité 1 — le Director implicite

Le GoF canonique a **Director + Builder + Product**. Ici, pas de Director explicite : c'est le **`main.rs` (le client)** qui séquence lui-même les étapes. C'est la forme moderne du Builder (« fluent builder »).

## Particularité 2 — un Builder hiérarchique (builder de builders)

`.middleware(|m| m…)`, `.with_admin(|a| a…)`, `.static_files(|s| s…)` prennent une **closure `FnOnce(Staging) -> Staging`**. Chaque sous-système a donc son propre sous-builder :

- `CoreStaging` — DB, config, URL registry
- `MiddlewareStaging` — collecte + tri par slot
- `AdminStaging` — registry, auth, config admin
- `StaticStaging` — fichiers statiques, media

La construction d'un gros objet est déléguée à des builders spécialisés, eux-mêmes fluents.

## Idiome Rust

- Chaque étape **consomme `self` et renvoie `Self`** (move semantics) : aucun état de construction partagé ni mutable en dehors du flux ; le compilateur interdit de réutiliser un builder à moitié consommé.
- Les sous-config par **closure de portée** (`impl FnOnce(X) -> X`) plutôt que d'exposer un sous-builder mutable : la closure encadre la mutation.
- `build()` est `async` (connexion DB, init Tera) et renvoie `Result<RuniqueApp, BuildError>` : la validation des invariants est un **point unique**.

---

## Tradeoff

- **Coût** : du boilerplate (une méthode par option) et une validation **différée** à `build()` — certaines erreurs de config apparaissent au runtime de construction, pas à la compilation.
- **Gain** : API incrémentale et lisible, config optionnelle sans télescopage de constructeurs, ordre d'assemblage maîtrisé (tri des middlewares, merge admin avant la stack), et un seul endroit qui valide la cohérence.
