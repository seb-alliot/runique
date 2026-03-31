
# 📊 Runique Framework — Project Status

Ce document consolide l'état réel du dépôt à partir des sources de référence :

- `Cargo.toml` (version workspace)
- `README.md`
- `CHANGELOG.md`
- `ROADMAP.md`
- `couverture_test.md`

---

## 🧾 Snapshot (au 15 mars 2026)

- **Version workspace** : `1.1.54`
- **Licence** : MIT
- **Branche de travail** : `i18n` → merge dans `main` pour publication
- **Tests reportés** : **~1 600 / ~1 600** ✅
- **Couverture (rapport du 2026-03-15)** :
  - Fonctions : **82.83%**
  - Lignes : **78.35%**
  - Régions : **75.38%**
- **Commande couverture** : `cargo llvm-cov --tests --package runique --ignore-filename-regex "admin|bin/runique|runique_app" --summary-only`

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
- **I18n** : 8 langues (`en`, `fr`, `de`, `es`, `it`, `pt`, `ja`, `zh`), 14 sections, fallback automatique vers `Lang::En`, stockage via `AtomicU8`, configurable via `RUNIQUE_LANG`.
- **Sécurité renforcée** : rate limiter (`RateLimiter`), login guard (`LoginGuard`), CSRF masqué (protection BREACH), HSTS, nonce CSP, comparaisons temps constant (`subtle`).

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

- **Pass rate** : 100% (~1 600/~1 600 réussi)
- **Couverture fonctionnelle** : 82.83%
- **Objectif roadmap avant publication** : ~85%+
- **Note** : la couverture reportée ignore `admin`, `bin/runique`, `runique_app`

### Zones faibles identifiées

Fichiers critiques encore bas selon `couverture_test.md` :

- `migration/migrate.rs` (22%) — dépend de commandes CLI sea-orm
- `engine/core.rs` (50%)
- `middleware/dev/cache.rs` (60%)
- `forms/fields/file.rs` (61%) — upload multipart
- `middleware/errors/error.rs` (60%)

### Ce qui a progressé fortement (session 2026-03-13 → 2026-03-15)

- `context/template.rs` : 0% → **80.95%**
- `middleware/security/csp.rs` : 66% → **95%**
- `errors/error.rs` : 38% → **77%**
- `context/request_extensions.rs` : 40% → **100%**

---

## 📌 Roadmap consolidée

### Fait

- Pipeline migration complet et stabilisé
- Refonte/stabilisation form system
- I18n complet : 8 langues, 14 sections, `AtomicU8`, `RUNIQUE_LANG`
- Sécurité renforcée : CSRF masqué, CSP builder, HSTS, rate limiter, login guard
- Couverture en forte hausse (76% → 82% fonctions)

### En cours

- Vue Admin bêta (permissions runtime, pagination, `js:` dans `admin!`)
- Montée couverture vers 85%+

### À faire

- Tracing d'erreurs plus poussé
- Doctests/exemples exécutables pour crates.io
- Dépréciation progressive des alias hérités

---

## 🆕 Changements récents

Voir `CHANGELOG.md` pour le détail complet. Points clés de `[1.1.54]` :

- CSP entièrement migrée vers builder (variables d'env supprimées)
- CSRF masqué (protection BREACH), comparaisons temps constant
- Rate limiter + Login guard dans le prelude
- I18n 8 langues livré dans `[1.1.46]`
- Couverture de tests : 82.83% fonctions

---

## 🚀 Niveau de maturité

- **Framework cœur** : stable et utilisable en production sur le socle principal
- **Admin** : bêta utilisable, encore en phase d'itération
- **Publication externe** : préparation encore en cours (principalement couverture + doc fine)

---

## ⚠️ Écarts / incohérences à surveiller

- **Couverture** : 82.83% fonctions, objectif 85% non encore atteint.
- **Admin permissions** : déclarées dans `admin!{}` mais pas encore vérifiées à l’exécution dans `admin_main`.
- **`migration/migrate.rs`** : 22% de couverture — dépend du CLI sea-orm, difficile à tester unitairement.

---

## 🛠️ Correctifs à apporter

### Priorité haute

- **Admin permissions** : appliquer réellement les permissions déclarées par ressource dans les handlers CRUD générés.
- **Runtime safety** : remplacer les `panic!/unwrap/expect` restants sur chemins runtime par des erreurs propagées.

### Priorité moyenne

- **Daemon admin** : clarifier et stabiliser le cycle de vie (lancement, arrêt, reporting d’erreurs).
- **Génération admin** : unifier les chemins/contrats de génération (`src/admins/`).

### Priorité basse

- **Couverture ciblée** : `engine/core.rs`, `migration/migrate.rs`, `forms/fields/file.rs`, `middleware/dev/cache.rs`.
- **Doctests/docs publication** : convertir les exemples `ignore/no_run` en exemples exécutables.
- **Dette de compatibilité** : planifier la dépréciation progressive des alias hérités.

---

## 🔗 Références

- Repository : [github.com/seb-alliot/runique](https://github.com/seb-alliot/runique)
- Status coverage : (https://github.com/seb-alliot/runique/blob/main/docs/couverture_test.md)
- Changelog : [Changelog](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- Roadmap : [Roadmap](https://github.com/seb-alliot/runique/blob/main/ROADMAP.md)
- Documentation : [English](https://github.com/seb-alliot/runique/tree/main/docs/en) et [Francais](https://github.com/seb-alliot/runique/tree/main/docs/fr)

---

**Dernière mise à jour** : 15 mars 2026
**Statut global** : ✅ Stable sur le cœur, 🟡 Admin bêta en cours d'évolution
Des erreurs muettes peuvent survenir, pensez a me les remonter si vous en trouvez.
Mérci !
