# Gestion d'erreurs avancée — thiserror et anyhow
> Indispensables · Niveau intermédiaire · Prérequis : [gestion-des-erreurs.md](gestion-des-erreurs.md)

Concevoir des types d'erreurs expressifs avec `thiserror`, gérer les erreurs contextuelles dans les applications avec `anyhow`, et propager correctement les erreurs entre couches.

## Objectifs

- Comprendre les limites de `Box<dyn Error>` et des strings comme erreurs
- Créer des types d'erreurs ergonomiques avec `thiserror`
- Ajouter du contexte aux erreurs avec `anyhow`
- Savoir quand utiliser l'un ou l'autre
- Propager les erreurs entre couches (service → handler)
- Intégrer la gestion d'erreurs dans un projet Runique

---

## Table des matières

1. [Les limites de Box\<dyn Error\> et des strings](#1-les-limites-de-boxdyn-error-et-des-strings)
2. [Le crate thiserror](#2-le-crate-thiserror)
3. [#\[error\], #\[from\], #\[source\]](#3-error-from-source)
4. [Le crate anyhow](#4-le-crate-anyhow)
5. [anyhow::Result, .context(), .with_context()](#5-anyhowresult-context-with_context)
6. [Quand utiliser thiserror vs anyhow](#6-quand-utiliser-thiserror-vs-anyhow)
7. [Propager les erreurs entre couches](#7-propager-les-erreurs-entre-couches)
8. [Erreurs dans les handlers Runique](#8-erreurs-dans-les-handlers-runique)
9. [Exercices pratiques](#9-exercices-pratiques)
10. [Aide-mémoire](#10-aide-mémoire)

---

## 1. Les limites de `Box<dyn Error>` et des strings

### Strings comme erreurs — fragile

```rust
// Retourner String comme erreur : vite pénible
fn parser_age(texte: &str) -> Result<u8, String> {
    texte.parse::<u8>().map_err(|e| e.to_string())
}

// L'appelant ne peut pas distinguer les cas d'erreur autrement
// qu'en faisant du matching sur le texte — fragile et non idiomatique
match parser_age("abc") {
    Err(msg) if msg.contains("invalid") => { /* ... */ }
    Err(msg) => { /* ... */ }
    Ok(age) => { /* ... */ }
}
```

### `Box<dyn Error>` — pratique mais imprécis

```rust
use std::error::Error;

fn charger_donnees(chemin: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let contenu = std::fs::read(chemin)?;    // io::Error converti auto
    Ok(contenu)
}

// Problèmes :
// — L'appelant ne sait pas quel type d'erreur peut survenir
// — Impossible de faire du pattern matching sur le type exact
// — Allocation heap à chaque erreur
// — Pas d'information de contexte ("lors de quelle opération ?")
```

### L'implémentation manuelle — verbeux

```rust
use std::fmt;

#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    Metier(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Io(e)      => write!(f, "Erreur IO : {e}"),
            AppError::Parse(e)   => write!(f, "Erreur de parsing : {e}"),
            AppError::Metier(m)  => write!(f, "Erreur métier : {m}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e)    => Some(e),
            AppError::Parse(e) => Some(e),
            AppError::Metier(_) => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self { AppError::Io(e) }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> Self { AppError::Parse(e) }
}
// 40 lignes de boilerplate pour 3 variants. thiserror réduit ça à 8 lignes.
```

---

## 2. Le crate `thiserror`

`thiserror` génère automatiquement les implémentations `Display`, `Error` et `From` via des macros derive.

```toml
# Cargo.toml
[dependencies]
thiserror = "2"
```

```rust
use thiserror::Error;

// Équivalent aux 40 lignes précédentes — en 8 lignes
#[derive(Error, Debug)]
enum AppError {
    #[error("Erreur IO : {0}")]
    Io(#[from] std::io::Error),

    #[error("Erreur de parsing : {0}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("Erreur métier : {0}")]
    Metier(String),
}
```

La macro `#[derive(Error)]` génère :
- `impl std::fmt::Display` à partir des `#[error("...")]`
- `impl std::error::Error`
- `impl From<X>` pour chaque `#[from]`

---

## 3. `#[error]`, `#[from]`, `#[source]`

### `#[error("...")]` — le message affiché

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum DbError {
    // Accès aux champs par position
    #[error("Connexion refusée à l'adresse {0}")]
    ConnexionRefusee(String),

    // Accès aux champs nommés par nom
    #[error("Enregistrement introuvable : table={table}, id={id}")]
    Introuvable { table: String, id: i64 },

    // Afficher l'erreur source avec {0}
    #[error("Erreur de requête : {0}")]
    Requete(#[from] sea_orm::DbErr),

    // Accès à self pour les calculs
    #[error("Timeout après {0}ms")]
    Timeout(u64),
}
```

### `#[from]` — conversion automatique

```rust
#[derive(Error, Debug)]
enum ServiceError {
    // #[from] génère impl From<sea_orm::DbErr> for ServiceError
    #[error("Erreur base de données")]
    Db(#[from] sea_orm::DbErr),

    // #[from] génère impl From<std::io::Error> for ServiceError
    #[error("Erreur fichier")]
    Fichier(#[from] std::io::Error),

    #[error("Validation échouée : {champ} — {message}")]
    Validation { champ: String, message: String },
}

// Grâce à #[from], l'opérateur ? convertit automatiquement
fn charger_utilisateur(id: i64, db: &Db) -> Result<Utilisateur, ServiceError> {
    let user = db.find_by_id(id)?;   // DbErr → ServiceError::Db auto
    Ok(user)
}
```

### `#[source]` — chaîner les erreurs sans conversion automatique

```rust
#[derive(Error, Debug)]
enum ConfigError {
    #[error("Fichier de config introuvable")]
    FichierManquant,

    // #[source] expose l'erreur sous-jacente via Error::source()
    // mais ne génère pas de From (tu l'encapsules manuellement)
    #[error("Format TOML invalide")]
    FormatInvalide(#[source] toml::de::Error),

    #[error("Clé manquante : {0}")]
    CleManquante(String),
}

// Différence entre #[from] et #[source] :
// #[from]   → From impl généré + source() implémenté
// #[source] → source() implémenté seulement (pas de From auto)
```

---

## 4. Le crate `anyhow`

`anyhow` est conçu pour les **applications**, là où tu veux propager n'importe quelle erreur avec du contexte, sans te soucier des types exacts.

```toml
# Cargo.toml
[dependencies]
anyhow = "1"
```

```rust
use anyhow::{Context, Result, bail, ensure, anyhow};

// anyhow::Result<T> est un alias pour Result<T, anyhow::Error>
// anyhow::Error accepte n'importe quel type qui implémente Error

fn lire_config(chemin: &str) -> Result<Config> {
    let contenu = std::fs::read_to_string(chemin)?;  // io::Error accepté
    let config: Config = toml::from_str(&contenu)?;  // toml::Error accepté
    Ok(config)
}
```

### Créer des erreurs anyhow ponctuelles

```rust
use anyhow::{anyhow, bail, ensure, Result};

fn valider_age(age: i32) -> Result<()> {
    // bail! — retourne immédiatement une erreur
    if age < 0 {
        bail!("L'âge ne peut pas être négatif : {age}");
    }

    // ensure! — comme assert! mais retourne Err au lieu de paniquer
    ensure!(age <= 150, "Âge irréaliste : {age}");

    // anyhow! — construit une erreur sans la retourner
    if age == 0 {
        return Err(anyhow!("Âge zéro non accepté"));
    }

    Ok(())
}
```

---

## 5. `anyhow::Result`, `.context()`, `.with_context()`

### `.context()` — ajouter un message statique

```rust
use anyhow::{Context, Result};

fn charger_utilisateur(id: u32) -> Result<Utilisateur> {
    let chemin = format!("users/{id}.json");

    let contenu = std::fs::read_to_string(&chemin)
        .context("Impossible de lire le fichier utilisateur")?;

    let utilisateur: Utilisateur = serde_json::from_str(&contenu)
        .context("Format JSON invalide pour l'utilisateur")?;

    Ok(utilisateur)
}
// En cas d'erreur, la chaîne complète est affichée :
// Error: Format JSON invalide pour l'utilisateur
// Caused by:
//     expected value at line 1 column 1
```

### `.with_context()` — message calculé à la demande

```rust
fn traiter_fichiers(chemins: &[&str]) -> Result<()> {
    for chemin in chemins {
        let contenu = std::fs::read_to_string(chemin)
            // La closure n'est appelée qu'en cas d'erreur — pas de formatage inutile
            .with_context(|| format!("Lecture de '{chemin}' échouée"))?;

        traiter_contenu(&contenu)
            .with_context(|| format!("Traitement de '{chemin}' échoué"))?;
    }

    Ok(())
}
```

### Chaîner le contexte sur plusieurs niveaux

```rust
fn initialiser_app() -> Result<()> {
    charger_config("config.toml")
        .context("Initialisation de l'application échouée")?;
    Ok(())
}

// Sortie en cas d'erreur :
// Error: Initialisation de l'application échouée
// Caused by:
//     0: Format TOML invalide
//     1: expected an equals, found a newline at line 3
```

---

## 6. Quand utiliser thiserror vs anyhow

| Situation | Recommandation |
|---|---|
| Tu écris une **bibliothèque** | `thiserror` — types précis, l'appelant peut matcher |
| Tu écris une **application** | `anyhow` — simplicité, contexte, moins de boilerplate |
| Couche **domaine / service** | `thiserror` — erreurs métier typées |
| Couche **handler / main** | `anyhow` ou type custom avec `thiserror` |
| L'appelant doit **distinguer** les cas d'erreur | `thiserror` |
| Tu veux juste **propager et logger** | `anyhow` |
| Tu veux un **backtrace** automatique | `anyhow` (avec `RUST_BACKTRACE=1`) |

```rust
// Règle simple :
//   thiserror → quand le TYPE de l'erreur a de la valeur pour l'appelant
//   anyhow    → quand seul le MESSAGE a de la valeur (logs, affichage)
```

---

## 7. Propager les erreurs entre couches

Dans une application en couches (repository → service → handler), les types d'erreurs se transforment à chaque niveau.

### Couche repository — erreurs techniques

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("Enregistrement introuvable : id={0}")]
    Introuvable(i64),

    #[error("Erreur base de données")]
    Db(#[from] sea_orm::DbErr),
}
```

### Couche service — erreurs métier

```rust
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Utilisateur introuvable")]
    UtilisateurInexistant,

    #[error("Permission refusée")]
    Interdit,

    #[error("Erreur interne")]
    Interne(#[from] RepoError),
}

pub async fn obtenir_utilisateur(
    id: i64,
    repo: &UserRepo,
) -> Result<Utilisateur, ServiceError> {
    let user = repo.find_by_id(id).await.map_err(|e| match e {
        RepoError::Introuvable(_) => ServiceError::UtilisateurInexistant,
        autre => ServiceError::Interne(autre),
    })?;

    Ok(user)
}
```

### De thiserror vers anyhow

```rust
use anyhow::{Context, Result};

// Dans main() ou dans un code d'orchestration de haut niveau,
// on peut convertir les erreurs typées vers anyhow pour simplifier
async fn demarrer_app() -> Result<()> {
    let db = connecter_db()
        .await
        .context("Connexion à la base de données échouée")?;

    let config = charger_config("config.toml")
        .context("Chargement de la configuration échoué")?;

    // Les deux types d'erreurs (DbError, ConfigError) sont absorbés
    // par anyhow — on ne se soucie plus des types, seulement du contexte
    Ok(())
}
```

---

## 8. Erreurs dans les handlers Runique

Dans Runique, les handlers Axum retournent `Result<Response, AppError>`. Le type `AppError` est défini avec `thiserror` et convertit ses variants en réponses HTTP.

```rust
use thiserror::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Ressource introuvable")]
    NotFound,

    #[error("Non autorisé")]
    Unauthorized,

    #[error("Requête invalide : {0}")]
    BadRequest(String),

    #[error("Erreur interne")]
    Internal(#[from] sea_orm::DbErr),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound      => StatusCode::NOT_FOUND,
            AppError::Unauthorized  => StatusCode::UNAUTHORIZED,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_)   => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}
```

### La macro `impl_from_error!` de Runique

Runique fournit la macro `impl_from_error!` pour brancher des types d'erreurs externes sur `AppError` sans écrire les `impl From` manuellement :

```rust
// Dans runique, cette macro génère les impl From<X> for AppError
impl_from_error!(
    AppError,
    [(sea_orm::DbErr, AppError::Internal)]
);

// Ce qui permet d'utiliser ? directement depuis les handlers :
pub async fn detail_article(
    State(db): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<Json<article::Model>, AppError> {
    let article = Article::find_by_id(id)
        .one(&db)
        .await?                                    // DbErr → AppError::Internal auto
        .ok_or(AppError::NotFound)?;               // None → 404

    Ok(Json(article))
}
```

### Pattern : enrichir l'erreur avec du contexte

```rust
use anyhow::Context;

// Dans un service (pas un handler), on peut utiliser anyhow
// pour ajouter du contexte avant de remonter l'erreur
pub async fn creer_article(
    payload: CreerArticlePayload,
    db: &DatabaseConnection,
) -> anyhow::Result<article::Model> {
    let actif = article::ActiveModel {
        titre:   Set(payload.titre.trim().to_string()),
        contenu: Set(payload.contenu),
        publie:  Set(false),
        ..Default::default()
    };

    actif.insert(db)
        .await
        .with_context(|| format!("Insertion de l'article '{}' échouée", payload.titre))
}
```

---

## 9. Exercices pratiques

### Exercice 1 : type d'erreur avec thiserror

Crée un enum `ValidationError` avec les variants :
- `Vide { champ: String }`
- `TropLong { champ: String, max: usize, actuel: usize }`
- `FormatInvalide { champ: String, attendu: String }`

Utilise `#[error("...")]` avec des messages incluant les noms de champs. Écris une fonction `valider_nom(nom: &str) -> Result<(), ValidationError>` qui retourne les erreurs appropriées.

### Exercice 2 : chaîne de couches

Crée deux types d'erreurs `RepoError` et `ServiceError`. `ServiceError` doit avoir un variant `Repo(#[from] RepoError)`. Écris une fonction de service qui appelle une fonction de repo et propage l'erreur avec `?`.

### Exercice 3 : anyhow avec contexte

Réécris cette fonction en utilisant `anyhow::Result` et `.with_context()` pour donner du contexte à chaque étape :

```rust
fn traiter_import(chemin: &str) -> Result<Vec<Enregistrement>, Box<dyn std::error::Error>> {
    let contenu = std::fs::read_to_string(chemin)?;
    let enregistrements: Vec<Enregistrement> = serde_json::from_str(&contenu)?;
    Ok(enregistrements)
}
```

### Exercice 4 : AppError complet

Crée un `AppError` pour un handler Axum avec les variants `NotFound`, `Unauthorized`, `BadRequest(String)`, `DbError(#[from] sea_orm::DbErr)`. Implémente `IntoResponse` pour mapper chaque variant vers le bon code HTTP.

---

## 10. Aide-mémoire

| Besoin | Solution |
|---|---|
| Type d'erreur précis pour une lib | `#[derive(thiserror::Error)]` |
| Message d'erreur | `#[error("message {champ}")]` |
| Conversion automatique depuis un autre type | `#[from]` |
| Chaîner l'erreur source sans From | `#[source]` |
| Propager toutes erreurs dans une app | `anyhow::Result<T>` |
| Ajouter contexte (message statique) | `.context("message")?` |
| Ajouter contexte (message calculé) | `.with_context(\|\| format!(...))?` |
| Retourner erreur immédiatement | `bail!("message")` |
| Assert qui retourne Err | `ensure!(condition, "message")` |
| Construire une erreur ad hoc | `anyhow!("message")` |
| Accéder à l'erreur source | `err.source()` |
| Chaîne complète de causes | `anyhow` affiche automatiquement `Caused by:` |
| Backtrace | `RUST_BACKTRACE=1` avec `anyhow` |

> **Règle d'or :** `thiserror` pour les types que l'on publie ou qui ont une valeur sémantique pour l'appelant ; `anyhow` pour le code applicatif où seul le message compte. Les deux se combinent : une couche service retourne `Result<T, MonErreur>` (thiserror), un handler l'absorbe via `?` ou `.context()` dans un `anyhow::Result`.
