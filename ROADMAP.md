# Roadmap Runique

## 1. Configuration du mot de passe via settings

**Status :** 🟢 Fait

### 1.a. Enum de configuration

- `password_init(PasswordConfig::...)` : **point d’entrée unique** pour l’initialisation globale du hash au démarrage.

- `PasswordConfig::auto()` : mode auto avec config par défaut (valeur de configuration).
- `PasswordConfig::auto_with(Manual::Argon2|Bcrypt|Scrypt)` : mode auto avec algo explicite (valeur de configuration).
- `PasswordConfig::manual(Manual::Argon2|Bcrypt|Scrypt|Custom)` : hash manuel dans la logique métier (valeur de configuration).
- `PasswordConfig::oauth(External::...)` : mode délégué (fournisseur externe, valeur de configuration).
- `PasswordConfig::custom(handler)` : stratégie personnalisée via `PasswordHandler` (valeur de configuration).


> Règle projet : l'initialisation effective du hash doit passer par `password_init(...)`.
> `RuniqueConfig::from_env()` peut garder une valeur par défaut de configuration, mais ne remplace pas ce point d’entrée.

### 1.b. Flow du mot de passe

```text
field password  →  clean_field (contraintes métier)
    →  clean (regroupement logique globale)
    →  finalize (transformation / hash si Auto)
    →  validate (validation finale)
    →  save (persistance)
```

Configuration dans `main.rs` :

```rust
use runique::utils::password::{External, Manual, PasswordConfig};

// Initialisation globale (obligatoire)
password_init(PasswordConfig::auto_with(Manual::Argon2));

// Exemples de valeurs possibles :
// password_init(PasswordConfig::auto());
// password_init(PasswordConfig::manual(Manual::Bcrypt));
// password_init(PasswordConfig::oauth(External::GoogleOAuth));
```

---

## 2. I18n et Tracing

**Status :** 🟡 En cours

### 2.a. I18n (Internationalisation)

- Socle déjà en place : module `utils::trad::switch_lang`, enum `Lang` (FR/EN), JSON embarqués, formatage de messages.
- Finaliser l’intégration runtime (config/session/request).
- Uniformiser l’usage des clés i18n dans middleware/forms/errors.

### 2.b. Tracing d’erreur

- `debug = false` : tracing off.
- `debug = true` : tracing on (console + page debug).

---

## 3. Migration et Vue Admin

**Status :** 🟡 En cours

### 3.a. Système de migration

**Status :** 🟢 Fait — pipeline complet fonctionnel

```text
entities/*.rs  →  makemigrations  →  fichiers sea-orm  →  cargo run -p migration
```

- Tous types supportés (string, int, float, bool, datetime, binary, json...).
- Primary key, foreign keys, indexes, nullable, unique.
- Vérifié sur Postgres, MariaDB, SQLite via Docker.

### 3.b. Vue Admin

**Status :** 🟡 En cours

#### b.1. Refonte du rendu

- Basculer sur les **models** qui gèrent leur propre rendu.
- Les formulaires se basent sur le model (et non l’inverse) si macro attribut connectée.
- Si formulaire fourni dans la macro, lier le formulaire au model pour en récupérer la logique métier.

#### b.2. Formulaires personnalisés

- Permettre l’ajout de formulaires pour récupérer la logique métier de l’API sur les models.

#### b.3. Personnalisation des templates admin

- Personnalisation visuelle.
- Documentation : clés à renseigner fournies dans les templates.

#### b.4. Sécurité / permissions admin

- Appliquer effectivement les permissions déclarées par ressource dans les handlers CRUD générés.
- Clarifier le contrat `is_staff` / `is_superuser` / rôles custom.
- Ajouter des tests d’autorisation par opération CRUD.

---

## 4. Sécurité middleware et stabilité

**Status :** 🟡 En cours

### 4.a. Middleware CSP

- Peaufiner la configuration pour la rendre plus simple et lisible.
- Réduire les directives permissives par défaut (`unsafe-inline`, `unsafe-eval`).
- Harmoniser la gestion des nonces.

### 4.b. CSRF secure-by-default

- **Principe directeur :** respect forcé du contrat d’utilisation `methode http -> prisme -> handler` pour stabiliser la sécurité CSRF.
- **Règle 1 (mutations) :** toutes les routes `POST`/`PUT`/`PATCH`/`DELETE` passent obligatoirement par Prisme.
- **Règle 2 (lecture body) :** Prisme reste l’unique lecteur du body (pas de relecture middleware, pas de buffering global).
- **Règle 3 (source token) :** `form-data` / `x-www-form-urlencoded` => token CSRF dans les champs ; `json/ajax` => token CSRF dans le header.
- **Règle 4 (GET) :** `GET` reste classique (normalisation query/headers), sans vérification CSRF.
- **Application progressive :** mode compat (warning) puis mode strict (refus des routes mutantes hors contrat).
- **Effet attendu :** réduction des failles liées au non-respect du contrat, simplification des handlers, stabilité sécurité renforcée.

### 4.c. Stabilité et couverture

- **Tests exhaustifs** : 🟡 76.66% fonctions (objectif 85% minimum).
- **Stress test** : 🔴 À faire (pousser le framework à bout).
- **Audit sécurité** : 🔴 À faire (identifier et corriger les failles).

### 4.d. Robustesse runtime

- Réduire `panic!/unwrap/expect` sur les chemins runtime.
- Propager des erreurs typées (`Result`) sur les points critiques (middleware, daemon, CLI, i18n).

---

## 5. Moteurs de formulaire

**Status :** 🟢 Fait

- Double appel de `is_valid()` corrigé.
- Restructuration de la gestion des mots de passe.

### 5.a. Potentiellement supprimé - `#[derive(DeriveModelForm)]`

**Status :** 🟡 À évaluer

- **Étape 1 — Check viabilité :** inventorier les usages réels (code + docs + exemples) et confirmer que le couple `model!(...)` + `#[form(...)]` couvre tous les cas actuels.
- **Étape 2 — Mesure des pertes occasionnées :** lister précisément ce qui serait perdu (ergonomie, rétrocompatibilité, snippets existants, onboarding) et estimer l’impact migration.
- **Étape 3 — Plan de transition :** préparer une migration douce (dépréciation documentée, alias temporaire éventuel, guide de remplacement).
- **Étape 4 — Validation technique :** vérifier compile/tests/docs après remplacement des usages critiques.
- **Étape 5 — Décision finale :** `GO` suppression ou `NO-GO` maintien selon coût réel vs bénéfice architecture.

---

## 6. Publication crates.io

**Status :** 🔴 À faire

### Étapes avant publication

- 85% couverture minimum (`bin/` exclu) : 🟡 76.66% actuellement.
- Remplacer doctests `ignore`/`no_run` par exemples réels : 🔴 À faire.
- Docs complètes (models, forms, macros procédurales) : 🔴 À faire.
- Publish crates.io : 🔴 À faire.

> Note : `bin/` est exclu du calcul de couverture (CLI non couvrable proprement).
> Cible réaliste : **85-88%** après couverture des modules HTTP via helpers Axum.

---

## 7. Gouvernance globale de la config API

**Status :** 🟡 En cours

### 7.a. Résolution unifiée de configuration

- Définir un ordre de priorité unique pour toute l’API :
    1. Overrides explicites de démarrage
    2. Configuration applicative (`RuniqueConfig`)
    3. Variables d’environnement
    4. Valeurs par défaut framework

### 7.b. Validation au boot (fail-fast)

- Valider toute la config critique avant le démarrage serveur (security, middleware, db, password, admin).
- Refuser le boot en production si incohérence ou valeur manquante.
- Autoriser des fallbacks contrôlés en dev/test avec warning explicite.

### 7.c. Contrat développeur

**Status :** 🟠 En Stand by => Choix de is_valid() en cour de reflexion pour un meilleur usage d'utilisation

- Documenter un point d’entrée clair de l’initialisation globale (pas de logique implicite cachée).
- Éviter les doubles sources de vérité entre config runtime et valeurs par défaut internes.
- Ajouter des tests d’intégration sur la résolution de config (priorités + erreurs de validation).

### 8 Configuration du pool Database

**Status :** 🔴 À faire

- Permettre la configuration via .env

=> En profiter pour faire du découpage de .env ?
    => .env
        => config basique dev
        => redirection
    => .env.conf
        => pool
           lang
           timezone quand implementer
    => .env.security
        => csp
            => interupteur
                => csp activé
        => rate limite quand implementer

    etc