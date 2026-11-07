
# Runique Framework — Project Status

Ce document consolide l'état réel du dépôt à partir des sources de référence :

- `Cargo.toml` (version workspace)
- `README.md`
- `CHANGELOG.md`

---

## Snapshot (au 7 novembre 2026)

- **Version workspace** : `4.0.0` (**pas encore publiée** sur crates.io)
- **derive_form** : `4.0.0`
- **runique_dsl** : `0.2.0`
- **Licence** : MIT
- **Branche** : `main`
- **Stack** : Axum 0.8.9 + SeaORM 2.0.4 + Tera 2.4 · Rust edition 2024 · Rust 1.94
- **Chantier principal livré** : admin `ModelAdmin` + documentation alignée 4.0

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
- `ValidationForm<F>` (`forms/validation_form.rs`) : `try_form(form, request) -> Result<ValidationForm<F>, F>`, preuve par le type qu'un handler a validé avant d'agir — additif, ne remplace pas `RuniqueForm::is_valid()` ; dispatch par défaut sur `Method::is_safe()`, hooks `register_dynamic_fields`/`allow_get`/`allow_post` surchargeables
- Protection CSRF intégrée (token masqué anti-BREACH, comparaison temps constant) ; échec CSRF pose désormais un message explicite (`csrf.invalid_or_missing`) au lieu d'échouer silencieusement, et `js/csrf.js` rafraîchit le token juste avant chaque soumission de formulaire
- Tracing structuré par domaine (arbre `RuniqueLog`) sur tout le pipeline (field, set_value, validate, finalize, render)
- Tous les types de champs : Text, Numeric, Boolean, Choice, Radio, Checkbox, Date, Time, DateTime, Duration, File, Color, Slug, UUID, JSON, IP, Hidden, Honeypot
- `save()` / `delete()` n'existent que sur le formulaire validé (`ValidationForm`) — aucune persistance sans validation préalable

### Routing
- `urlpatterns!{}` avec segments typés, GET/POST séparés
- URL registry nommée, balise `{% link "nom" %}`

### Templates
- Moteur Tera + helpers de contexte (`{% csrf %}`, `{% static %}`, `{% link %}`, `{% media %}`)
- Autoescape actif sur `.html`/`.xml`

### Admin panel (stable bêta)
- Builder typé `ModelAdmin<Entity>` — plus de macro `admin!{}`, plus de génération `src/admins/`
- Déclaration dans `src/admin.rs`, câblage via `.with_admin(admin::config)` ; `cargo run` suffit
- CRUD générique : list, detail, create, edit, delete, actions groupées (`.action(colonne, valeur, libellé)`), suppression groupée
- Colonnes SeaORM typées : `display`, `filter`, `search`, `sortable`, `ordering`
- Rangs (superuser, dirigeant, admin, lambda), droits par compte pour les admins, groupes pour les lambdas, grille de droits
- Routes admin déclarées dans `urlpatterns!` (`admin "…" => …`, page à part ou remplacement du détail)
- `OWNER` sur `ModelMeta` (meta `model!`) : vérification d'appartenance pour `can_update_own`/`can_delete_own`
- Historique des actions admin (log, batch_id, diff old/new)
- Templates surchargeables par bloc (HTML, CSS ou les deux) ou par page, pour tous les modèles ou pour un seul
- Comptes : `created_at` et `updated_at` remplis à l'enregistrement, y compris pour un compte créé dans l'admin
- Filtre `display_time` : dates et heures au même format dans les listes, les fiches et l'historique

### Sécurité
- CSRF masqué (protection BREACH), token lié à la session, comparaison `subtle::ct_eq`
- Formulaire envoyé depuis un autre site ou un sous-domaine refusé avant le handler (`Sec-Fetch-Site`)
- Contenu des sessions chiffré en base (XChaCha20-Poly1305, clé dérivée de `SECRET_KEY`)
- `session.cycle_id()` au login — protection fixation de session
- Droits par opération (create, read, read_own, update, update_own, delete, delete_own) et par colonne, tranchés par une seule porte (`auth/gate`)
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
- `model!{}` DSL → entité SeaORM + migration SQL + formulaire du modèle (`ModelForm`)
- `extend!{}` — extension de tables framework (ex. `eihwaz_users`)
- `makemigrations` — plan → validate → **commit/rollback atomique** + snapshots ; `DROP COLUMN` sur colonnes supprimées (garde destructive)
- Backends supportés : PostgreSQL, MariaDB, SQLite
- Alias `Pk` : `i32` par défaut, `i64` (`big-pk`) ou `Uuid` via `Uuid::now_v7()` (`pk-uuid`) — features mutuellement exclusives (`compile_error!`). Utilisable sur n'importe quel champ (pas seulement la PK), typiquement une FK, pour rester automatiquement synchronisé avec le type de la table référencée
- `model!{}` — syntaxe de champs unifiée (l'ancienne grammaire `fields: { name: SqlType }` est supprimée) : bloc anonyme unique, 43 types sémantiques, options `readonly`/`label` incluses
- Générateur de migrations durci multi-moteurs (2026-09-01) : gardes runtime pour `CREATE TYPE`/triggers `updated_at` (au lieu d'un choix figé à la génération), casse d'identifiant Postgres corrigée sur `ALTER TYPE`, ordre `TYPE`/`USING` invalide corrigé, `modify_column` sauté sous SQLite (panique sea-query) ; FK toujours inline en `CREATE TABLE`
- `makemigrations` crée le crate `migration/` s'il manque et branche les tables du framework même sans aucun modèle propre au projet ; un changement de type de colonne n'est pas migré (avertissement après `--force`) — cf. [Migrations](/docs/fr/installation/migrations)
- Recherche/filtres portables multi-moteurs (2026-09-02) : `CAST(col AS TEXT)` invalide sur MySQL/MariaDB (exige `CHAR`) — corrigé via des helpers `text_cast_type`/`text_eq`/`ilike` détectant le moteur à l'exécution (`db.get_database_backend()`). `search_cond!` prend désormais la connexion `db` en premier argument sur ses 4 formes — cf. [Requêtes](/docs/fr/orm/requetes)
- Fichiers envoyés : écriture vidée sur le disque (`flush`) avant de considérer l'envoi comme terminé

### I18n
- 9 langues (en, fr, de, es, it, pt, ja, zh, ru), stockage `AtomicU8`, `RUNIQUE_LANG`

### Tracing & observabilité
- Arbre `RuniqueLog` par domaine (forms, middleware, session, auth, admin, db, mailer, migration, templates, errors, builder), chaque feuille un `Option<LogLevel>`
- `TraceResult::trace` / `trace_or` — les `Result` avalés loggent leur `file:line` ; les sites sensibles à la sécurité plancher à `WARN` même catégorie désactivée
- Sorties : stdout couleurs, fichiers roulants (JSON/plain, non bloquants), `LogSink` custom (aucun type `tracing` exposé) ; `.external()` délègue le subscriber global à l'app hôte
- Override runtime `RUNIQUE_LOG_FILE`

### CLI
- `runique new` (projet + crate `migration/`), `runique create-superuser`, `runique makemigrations`, `runique migration up | down | status` (plus besoin de `sea-orm-cli`), `runique test` — pas de `runique start` (`cargo run` + `ModelAdmin`)

---

## Sécurité — historique des corrections

| Version | Faille | Sévérité |
|---------|--------|----------|
| 4.0.0 | Timing « mot de passe oublié » (réponse plus rapide pour adresse inconnue) | Faible |
| 4.0.0 | Message admin superuser trop révélateur | Faible |
| 4.0.0 | `csrf.js` envoyait le jeton CSRF hors origine | Moyenne |
| 4.0.0 | `password` / `is_active` exposés dans les formulaires `extend!` | Élevée |
| 4.0.0 | Page debug template : lecture hors `templates/` | Moyenne |
| 4.0.0 | Erreurs globales de formulaire non échappées (XSS) | Moyenne |
| 4.0.0 | Contenu des sessions stocké en clair en base | Moyenne |
| 4.0.0 | Reset laissant valides les autres sessions d'un store personnalisé | Moyenne |
| 4.0.0 | Formulaire d'un autre site atteignant un handler qui ne vérifie pas le jeton | Moyenne |
| 4.0.0 | Une erreur de suppression en base pouvait laisser une session réutilisable ; révocation des sessions hors de la transaction du reset | Moyenne |
| 4.0.0 | Remplacement non atomique du jeton de reset : deux jetons valides en concurrence | Moyenne |
| 4.0.0 | Hash au préfixe non reconnu : vérification plus rapide, donc mesurable | Faible |
| 4.0.0 | Jeton CSRF en double dans un corps de formulaire (partie corrigée) | Faible |
| 4.0.0 | Alias d'enum chaîne ambigus interprétés comme une autre variante | Faible |
| 4.0.0 | Fichiers validés avant d'être entièrement écrits : un fichier au-delà de la limite d'un champ pouvait passer | Moyenne |
| 4.0.0 | Compte créé sans son activation dans un projet généré par `runique new` | Faible |
| 4.0.0 | Scripts `add_js` sans hash SRI ni nonce CSP | Durcissement |
| 3.0.3 | `bulk_create` : le seul droit de création permettait de modifier des lignes existantes | Élevée |
| 3.0.3 | Un membre du staff pouvait prendre le compte d'un superutilisateur (changement d'email + réinitialisation) | Élevée |
| 3.0.3 | Open redirect via une tabulation, un `Location` non ASCII ou l'en-tête `Refresh` | Moyenne |
| 3.0.2 | XSS stockée via un upload en staging (chemin réaffiché, dossier servi sans CSP) | Élevée |
| 3.0.2 | Fichiers écrits sur disque avant la vérification CSRF (multipart) | Moyenne |
| 3.0.2 | Open redirect via identifiant dans l'URL (`site.com:x@evil.com`) | Moyenne |
| 3.0.2 | Corps urlencoded/JSON sans limite de taille | Moyenne |
| 3.0.2 | Reset-password admin possible depuis une ressource hors comptes | Faible |
| 3.0.2 | Admin : jeton CSRF brut accepté (masquage BREACH contourné) | Durcissement |
| 2.1.17 | Tokens de reset : mémoire → DB (hashés, single-use, durcis IDOR) | Durcissement |
| 2.1.15 | Énumération d'utilisateurs via timing attack au login | Moyenne |
| 2.1.15 | Contrôle d'accès manquant sur l'action admin reset-password | Moyenne |
| 2.1.9 | Injection SQL dans les filtres de liste admin | Élevée |
| 2.1.9 | Fixation de session au login (cycle_id manquant) | Moyenne |
| 2.1.9 | Granularité droits write admin (create/update/delete indistincts) | Moyenne |
| 2.1.9 | IDOR — can_update_own/can_delete_own non appliqués | Faible |

---

## État admin — permissions

- Rangs : superuser (tout, contournement compris), dirigeant (tout), admin (sa grille, réglée par un dirigeant), lambda (union de ses groupes, jamais les tables du framework) ✅
- Droits `*_own` appliqués quand `ModelMeta::OWNER` est défini (meta `model!`) ✅
- Compte et droits relus en base à chaque requête (révocation immédiate) ✅
- Détail : [Permissions](/docs/fr/admin/permission)

---

## Suite du projet

Ordre et contenu des prochaines versions : [ROADMAP.fr.md](https://github.com/seb-alliot/runique/blob/main/ROADMAP.fr.md).

Couverture de tests au 7 novembre 2026 : 92,57 % des lignes, détail par fichier dans [couverture_test.md](https://github.com/seb-alliot/runique/blob/main/docs/couverture_test.md).

Non planifié pour l'instant : uploads typés (`StagedFile`, hors valeurs texte).

---

## Chantier makemigrations

### Déjà en place
- [x] Plan, validation, commit ou rollback atomique, snapshots
- [x] Renommage simple d'une colonne (`renamed_from`)
- [x] Refus sans `--force` des opérations destructives : `DROP COLUMN`, changement de type, nullable vers `NOT NULL`, longueur ou décimale réduite, `DROP FK`, nouvelle FK `CASCADE`

### À faire
- [ ] Suppression de table quand le modèle disparaît : refus sans `--force`, `DROP TABLE` dans `up`, recréation dans `down`, snapshot retiré, ordre inverse des FK
- [ ] Changements destructeurs : indiquer ce qu'ils touchent, relations comprises
- [ ] Vrai `ALTER` lors d'un changement de type (`USING` sur PostgreSQL et MySQL, snapshot fidèle ; reconstruction de table sous SQLite en option)
- [ ] Passage de nullable à `NOT NULL` avec défaut : remplir les `NULL` existants avant la contrainte
- [ ] Colonne à la fois renommée et modifiée (aujourd'hui seul le renommage est appliqué, et le snapshot enregistre la modification comme faite)
- [ ] Détection d'un changement de valeur par défaut
- [ ] Noms de contraintes lus en base au lieu d'être supposés
- [ ] Limite de longueur des identifiants selon le moteur (63 pour PostgreSQL, 64 pour MariaDB/MySQL)
- [ ] Création et suppression automatiques des tables de relation du many-to-many (DSL, migrations, macro)
- [ ] Respect de `cascade`, `restrict` et `set_null` sur ces tables
- [ ] Renommage de table (`renamed_from`), et message explicite pour un modèle renommé

---

## Références

- Repository : [github.com/seb-alliot/runique](https://github.com/seb-alliot/runique)
- Changelog : [CHANGELOG.md](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- Roadmap : [ROADMAP.fr.md](https://github.com/seb-alliot/runique/blob/main/ROADMAP.fr.md)
- Documentation : [English](https://github.com/seb-alliot/runique/tree/main/docs/en) | [Français](https://github.com/seb-alliot/runique/tree/main/docs/fr)

---

**Dernière mise à jour** : 7 novembre 2026
**Statut global** : ✅ Framework stable · 🟡 Admin bêta mature (`ModelAdmin`) · 🔒 Sécurité : tokens reset en DB, auth temps constant, correctifs 4.0 inclus · 📖 Doc 4.0 alignée · 📦 crates.io 4.0.0 encore non publiée · ➡️ Prochain : file patch sécu `4.0.x` (puis mineures 4.x)
