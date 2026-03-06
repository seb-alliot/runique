
# 📊 Runique Framework — Project Status

Ce document consolide l'état réel du dépôt à partir des sources de référence :

- `Cargo.toml` (version workspace)
- `README.md`
- `CHANGELOG.md`
- `ROADMAP.md`
- `couverture_test.md`

---

## 🧾 Snapshot (au 3 mars 2026)

- **Version workspace** : `1.1.41`
- **Licence** : MIT
- **Branche de travail** : `vue_admin`
- **Tests reportés** : **1523 / 1523** ✅
- **Couverture (rapport du 2026-03-04)** :
  - Fonctions : **76.66%**
  - Lignes : **71.04%**
  - Régions : **67.22%**
- **Commande couverture** : `cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only`

---

## 🧱 Périmètre du workspace

Crates membres déclarées dans le workspace :

- `runique` (crate framework principale)
- `demo-app` (application de test du framework)
- `demo-app/migration` (migration liée à l'app de test)

Le statut ci-dessous porte sur la crate **`runique`** (source produit).
`demo-app` sert uniquement à valider/tester le framework pendant le développement.

---

## ✅ Fonctionnalités en place

- **Forms** : système de formulaires typés, validation, rendu, protection CSRF intégrée.
- **Routing** : macros et enregistrement de routes.
- **Templates** : moteur Tera + helpers de contexte.
- **ORM / Migration** : intégration SeaORM, `makemigrations`, => possible mais a évité pour evité une desynchronisation avec sea-orm -> `migration up/down/status`.
- **Sécurité** : middleware CSRF, CSP, allowed hosts, sanitization, auth session.
- **Flash messages** : système de messages temporaires en session.
- **CLI `runique`** : `new`, `start`, `create-superuser` => ashe du mot de passe via Argon2, reflection en court pour de la flexibilité, `makemigrations`, `migration` => utilise la cli de sea-orm.
- **I18n (socle)** : module de traduction `utils::trad::switch_lang` avec `Lang` (FR/EN), dictionnaires JSON embarqués et formatage de messages.

### Modules exportés (crate `runique`)

- `app`, `config`, `context`, `engine`, `flash`, `forms`, `macros`, `middleware`, `migration`, `admin`, `errors`, `utils`
- `db` est conditionnel à la feature **`orm`**.

### Compatibilité API héritée

Des alias de compatibilité restent exposés (`config_runique`, `formulaire`, `middleware_runique`, etc.), ce qui facilite la transition d'anciens projets.

---

## ⚙️ Features Cargo & base technique

- Features par défaut : `orm` + `all-databases`
- Backends DB activables : `sqlite`, `postgres`, `mysql`, `mariadb`
- Stack principale : Axum + Tower + Tokio + Tera + SeaORM (optionnel via feature)
- Sécurité mot de passe : `argon2`, `bcrypt`, `scrypt`, `password-hash`

---

## 🧭 État de la vue Admin (bêta)

La vue admin est **opérationnelle en bêta** sur un modèle déclaratif + génération de code :

- Déclaration via macro `admin!` dans `src/admin.rs`
- Parsing de la macro (`syn`) + génération de `src/admins/`
- Watcher via `runique start` pour régénération automatique
- Routes/handlers CRUD générés (base fonctionnelle)

### Limites connues (assumées à ce stade)

- Permissions surtout globales par ressource
- Peu de granularité fine par opération
- `src/admins/` régénéré (écrasement des modifications manuelles)
- **CSRF** : protection fiable dans le flux formulaire (`Prisme` / `csrf_gate`), mais middleware encore permissif pour certains endpoints mutateurs hors flux formulaire.

### État pratique du workflow

- `runique start` détecte `.with_admin(...)` dans `src/main.rs`
- Si admin activé : lancement du watcher + génération
- Si admin non activé : message explicite, pas de daemon lancé

---

## 🧪 Qualité & tests

### État actuel

- **Pass rate** : 100% (1523/1523 réussi)
- **Couverture fonctionnelle** : 76.66%
- **Objectif roadmap avant publication** : 85%+
- **Note** : la couverture reportée ignore les fichiers correspondant à `admin` (cf. commande ci-dessus)

### Zones faibles identifiées

Fichiers critiques encore bas ou à 0% selon `couverture_test.md` :

- `engine/core.rs`
- `errors/error.rs`
- `migration/utils/parser_seaorm.rs`
- `forms/fields/datetime.rs`
- `forms/fields/file.rs`
- plusieurs modules dépendants d'une stack HTTP complète (`context/template.rs`, extractors, etc.)

### Ce qui progresse fortement (selon `couverture_test.md`)

- `db/config.rs` : ~22% → **93%**
- `migration/makemigrations.rs` : ~22% → **76%**
- `migration/migrate.rs` : 0% → **60%**

---

## 📌 Roadmap consolidée

### Fait

- Pipeline migration complet et stabilisé
- Refonte/stabilisation form system
- Amélioration de la couverture (baseline en hausse)

### En cours

- Vue Admin bêta (ergonomie, permissions, sécurité de workflow)
- Simplification et durcissement de certains points middleware (notamment CSP)
- Intégration i18n applicative : socle FR/EN déjà présent, raccordement global (config/runtime) encore en progression

### À faire

- I18n configurable de bout en bout (sélection runtime centralisée)
- Tracing d'erreurs plus poussé
- Montée couverture vers 85%+
- Préparation publication crates.io (docs + couverture cible)

---

## 🆕 Changements récents (Unreleased)

Éléments visibles dans `CHANGELOG.md` :

- pipeline migration complet annoncé et stabilisé
- support large des types de colonnes + FK/index/nullable/unique
- tests E2E DB sur Postgres/MariaDB/SQLite
- correctifs sur `runique start` et rendu password côté admin

---

## 🚀 Niveau de maturité

- **Framework cœur** : stable et utilisable en production sur le socle principal
- **Admin** : bêta utilisable, encore en phase d'itération
- **Publication externe** : préparation encore en cours (principalement couverture + doc fine)

---

## ⚠️ Écarts / incohérences à surveiller

- **Version** : `1.1.41` ;
- **Statut admin** : la doc technique admin décrit une base fonctionnelle, mais la roadmap la garde en chantier.
- **Couverture** : le pourcentage global est bon en progression, mais encore en-dessous de la cible publication.
- **CSRF** : pas de faille systématique si le framework est utilisé comme prévu ; le point sensible est le non-respect du contrat d’utilisation sur les routes mutantes hors flux Prisme.

---

## 🛠️ Correctifs à apporter

### Priorité haute (sécurité / robustesse)

- **Admin permissions** : appliquer réellement les permissions déclarées par ressource dans les handlers CRUD générés (pas seulement `is_staff` / `is_superuser`).
- **CSRF (piste potentielle)** : stabiliser la sécurité par **respect forcé du contrat d’utilisation** (`methode http -> prisme -> handler`) sur les méthodes mutantes, avec lecture body unique dans Prisme et sans relecture middleware.
- **CSP** : réduire les directives permissives par défaut (`unsafe-inline` / `unsafe-eval`) et harmoniser la stratégie nonce.
- **Runtime safety** : remplacer les `panic!/unwrap/expect` sur chemins runtime par des erreurs propagées (`Result` + erreurs typées).

### Priorité moyenne (cohérence technique)

- **I18n end-to-end** : brancher le socle i18n existant (FR/EN + JSON) sur une sélection runtime centralisée (config/session/request).
- **Daemon admin** : clarifier et stabiliser le cycle de vie (lancement, arrêt, reporting d'erreurs).
- **Variables d'environnement** : uniformiser le nommage et les messages d'erreur (ex: allowed hosts).
- **Génération admin** : unifier les chemins/contrats de génération (`src/admins/` vs autres chemins documentés).

### Priorité basse (qualité continue)

- **Couverture ciblée** : renforcer les zones encore faibles (`engine/core.rs`, `errors/error.rs`, `migration/utils/parser_seaorm.rs`, `forms/fields/datetime.rs`, `forms/fields/file.rs`).
- **Doctests/docs publication** : convertir les exemples `ignore/no_run` en exemples exécutables et aligner les docs crates.io.
- **Dette de compatibilité** : planifier la dépréciation progressive des alias hérités tout en conservant la rétrocompatibilité.

---

## 🔗 Références

- Repository : [github.com/seb-alliot/runique](https://github.com/seb-alliot/runique)
- Status coverage : `couverture_test.md`
- Changelog : `CHANGELOG.md`
- Roadmap : `ROADMAP.md`
- Documentation : `docs/en` et `docs/fr`

---

**Dernière mise à jour** : 4 mars 2026
**Statut global** : ✅ Stable sur le cœur, 🟡 Admin bêta en cours d'évolution
Des erreurs muettes peuvent survenir, pensez a me les remonter si vous en trouvez.
Mérci !
