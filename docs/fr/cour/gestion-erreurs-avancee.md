# Gestion d'erreurs avancée — thiserror et anyhow

> Des erreurs typées pour les bibliothèques, des erreurs avec contexte pour les applications, et leur traduction en réponses HTTP dans Runique.

## Objectifs

- Savoir pourquoi les erreurs écrites à la main deviennent lourdes
- Générer `Display`, `Error`, `source` et `From` avec `thiserror`
- Utiliser `anyhow` : `Result`, `anyhow!`, `bail!`, `ensure!`, `context`, `downcast`
- Choisir entre `thiserror` et `anyhow`, et les combiner
- Organiser les erreurs par couche (données, métier, HTTP)
- Transformer ses erreurs en réponses Runique (`AppResult`, `AppError`, `ErrorContext`)
- Ne jamais exposer un détail interne dans un message destiné au visiteur

Prérequis : le cours `gestion-des-erreurs` (`Result`, `?`, `From`, trait `Error`).

---

## Table des matières

1. Les limites des solutions simples
2. `thiserror` : déclarer une erreur
3. Les attributs de `thiserror`
4. `anyhow` : une erreur pour toute l'application
5. Le contexte avec `anyhow`
6. Lire une erreur `anyhow`
7. `thiserror` ou `anyhow`
8. Erreurs par couche
9. Erreurs dans les handlers Runique
10. Sécurité des messages d'erreur
11. La même fonction écrite de plusieurs façons
12. Exercices
13. Aide-mémoire

---

## 1. Les limites des solutions simples

### Une `String` comme erreur

```rust
fn trouver(id: u32) -> Result<String, String> {
    if id == 0 {
        return Err("introuvable".to_string());
    }
    Ok("Ada".into())
}

// L'appelant doit comparer des textes pour réagir :
match trouver(0) {
    Err(e) if e == "introuvable" => { /* fragile : une faute de frappe casse le test */ }
    _ => {}
}
```

### `Box<dyn Error>`

Pratique, mais l'appelant ne voit plus **quelles** erreurs sont possibles : il doit tenter des `downcast_ref` au hasard.

### L'implémentation manuelle

```rust
#[derive(Debug)]
enum ErreurDepot {
    Base(sea_orm::DbErr),
    Introuvable(i32),
}

impl std::fmt::Display for ErreurDepot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base(_) => write!(f, "erreur de base de données"),
            Self::Introuvable(id) => write!(f, "élément {id} introuvable"),
        }
    }
}

impl std::error::Error for ErreurDepot {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Base(e) => Some(e),
            Self::Introuvable(_) => None,
        }
    }
}

impl From<sea_orm::DbErr> for ErreurDepot {
    fn from(e: sea_orm::DbErr) -> Self { Self::Base(e) }
}
```

Une trentaine de lignes pour deux variantes, à maintenir à la main à chaque ajout. C'est ce que `thiserror` génère.

---

## 2. `thiserror` : déclarer une erreur

```toml
[dependencies]
thiserror = "2"
```

```rust
use thiserror::Error;

#[derive(Debug, Error)]
enum ErreurDepot {
    #[error("erreur de base de données")]
    Base(#[from] sea_orm::DbErr),

    #[error("élément {0} introuvable")]
    Introuvable(i32),
}
```

Le même code que la section 1, en six lignes. `thiserror` ne fait que générer du code à la compilation : il n'apparaît pas dans l'API publique, et une bibliothèque peut l'utiliser sans l'imposer à ses utilisateurs.

---

## 3. Les attributs de `thiserror`

### 3.1 — `#[error("...")]` : le message

```rust
#[derive(Debug, thiserror::Error)]
enum ErreurFormulaire {
    #[error("le champ est obligatoire")]                         // texte fixe
    Vide,

    #[error("valeur {0} hors limites")]                          // champ de tuple par position
    HorsLimites(i64),

    #[error("{champ} : {longueur} caractères, maximum {max}")]   // champs nommés
    TropLong { champ: String, longueur: usize, max: usize },

    #[error("valeur invalide : {0:?}")]                          // avec Debug
    Invalide(String),

    #[error("{}", .0.to_uppercase())]                            // expression sur un champ
    Brut(String),
}
```

On peut aussi mettre `#[error]` sur une **struct** :

```rust
#[derive(Debug, thiserror::Error)]
#[error("quota dépassé : {utilise}/{limite}")]
struct Quota { utilise: u32, limite: u32 }
```

### 3.2 — `#[from]` : conversion automatique pour `?`

```rust
#[derive(Debug, thiserror::Error)]
enum ErreurImport {
    #[error("lecture du fichier impossible")]
    Lecture(#[from] std::io::Error),

    #[error("ligne mal formée")]
    Format(#[from] std::num::ParseIntError),
}

fn importer(chemin: &str) -> Result<Vec<i64>, ErreurImport> {
    let texte = std::fs::read_to_string(chemin)?;          // io::Error -> Lecture
    let v = texte.lines().map(|l| l.parse::<i64>()).collect::<Result<_, _>>()?;   // -> Format
    Ok(v)
}
```

`#[from]` génère `impl From<std::io::Error> for ErreurImport` **et** déclare le champ comme `source`. Une seule variante par type source : deux variantes `#[from] std::io::Error` seraient ambiguës.

### 3.3 — `#[source]` : garder la cause sans conversion automatique

```rust
#[derive(Debug, thiserror::Error)]
enum ErreurConfig {
    #[error("lecture de {chemin}")]
    Lecture {
        chemin: String,
        #[source]
        cause: std::io::Error,
    },
}

fn lire(chemin: &str) -> Result<String, ErreurConfig> {
    std::fs::read_to_string(chemin).map_err(|cause| ErreurConfig::Lecture {
        chemin: chemin.to_string(),
        cause,
    })
}
```

À quoi ça sert : quand la variante a besoin d'**autres** données que la cause (ici le chemin), `#[from]` ne peut pas les deviner. Un champ nommé `source` est pris comme cause automatiquement, sans attribut.

### 3.4 — `#[error(transparent)]` : déléguer entièrement

```rust
#[derive(Debug, thiserror::Error)]
enum ErreurApi {
    #[error("ressource introuvable")]
    Introuvable,

    #[error(transparent)]
    Autre(#[from] anyhow::Error),      // Display et source sont ceux de l'erreur enveloppée
}
```

À quoi ça sert : une variante « tout le reste » qui ne rajoute pas de message, ou un type public opaque qui cache l'erreur interne :

```rust
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct MonErreur(#[from] ErreurInterne);    // l'enum interne peut changer sans rupture d'API

#[derive(Debug, thiserror::Error)]
enum ErreurInterne {
    #[error("x")]
    X,
}
```

### 3.5 — Récapitulatif

| Attribut | Effet |
|---|---|
| `#[error("...")]` | génère `Display` |
| `#[from]` sur un champ | génère `From` + marque la cause |
| `#[source]` sur un champ | marque la cause, sans `From` |
| champ nommé `source` | marque la cause automatiquement |
| `#[error(transparent)]` | `Display` et `source` délégués au champ unique |

---

## 4. `anyhow` : une erreur pour toute l'application

```toml
[dependencies]
anyhow = "1"
```

### 4.1 — `anyhow::Result`

```rust
use anyhow::Result;            // Result<T, anyhow::Error>

fn charger_port() -> Result<u16> {
    let t = std::fs::read_to_string("port.txt")?;    // n'importe quelle erreur Error + Send + Sync
    let p = t.trim().parse::<u16>()?;
    Ok(p)
}

fn main() -> Result<()> {
    let port = charger_port()?;
    println!("{port}");
    Ok(())
}
```

`anyhow::Error` accepte toute erreur qui implémente `std::error::Error + Send + Sync + 'static`. Il garde la cause d'origine, une pile d'appels si `RUST_BACKTRACE=1`, et le contexte ajouté en chemin.

### 4.2 — Créer une erreur ponctuelle : trois macros

```rust
use anyhow::{anyhow, bail, ensure, Result};

fn verifier_age(age: i32) -> Result<u32> {
    // anyhow! : construire une erreur
    if age < 0 {
        return Err(anyhow!("âge négatif : {age}"));
    }

    // bail! : construire ET renvoyer (équivaut à return Err(anyhow!(..)))
    if age > 150 {
        bail!("âge improbable : {age}");
    }

    // ensure! : renvoyer une erreur si la condition est fausse
    ensure!(age >= 18, "il faut être majeur (âge : {age})");

    Ok(age as u32)
}
```

Les trois écritures donnent la même erreur ; `bail!` et `ensure!` sont plus courts pour les vérifications en début de fonction.

---

## 5. Le contexte avec `anyhow`

### 5.1 — `context` : un message fixe

```rust
use anyhow::{Context, Result};

fn lire_config() -> Result<String> {
    std::fs::read_to_string("config.toml").context("lecture de la configuration")
}
// Affichage : lecture de la configuration
// Cause : No such file or directory (os error 2)
```

Le trait `Context` ajoute `.context()` et `.with_context()` à tout `Result` dont l'erreur implémente `Error`, et aussi à `Option` (un `None` devient une erreur).

### 5.2 — `with_context` : un message calculé seulement en cas d'erreur

```rust
fn lire(chemin: &str) -> Result<String> {
    std::fs::read_to_string(chemin).with_context(|| format!("lecture de {chemin}"))
}
```

`context(format!(...))` construirait la `String` **à chaque appel**, même en cas de succès ; `with_context` ne l'appelle qu'en cas d'erreur.

### 5.3 — Sur une `Option`

```rust
use std::collections::HashMap;

fn port(env: &HashMap<String, String>) -> Result<u16> {
    let texte = env.get("PORT").context("variable PORT absente")?;   // None -> erreur
    texte.parse().with_context(|| format!("PORT invalide : {texte}"))
}
```

### 5.4 — Empiler le contexte à chaque couche

```rust
fn lire_utilisateurs() -> Result<Vec<String>> {
    let t = std::fs::read_to_string("users.csv").context("ouverture de users.csv")?;
    Ok(t.lines().map(String::from).collect())
}

fn demarrer() -> Result<()> {
    let u = lire_utilisateurs().context("chargement des comptes au démarrage")?;
    println!("{} comptes", u.len());
    Ok(())
}
// Error: chargement des comptes au démarrage
//
// Caused by:
//     0: ouverture de users.csv
//     1: No such file or directory (os error 2)
```

Chaque couche dit **ce qu'elle essayait de faire** ; la cause d'origine reste au bout de la chaîne.

---

## 6. Lire une erreur `anyhow`

### 6.1 — Les formats d'affichage

```rust
fn afficher(e: &anyhow::Error) {
    println!("{e}");        // seulement le dernier contexte
    println!("{e:#}");      // toute la chaîne sur une ligne : a: b: c
    println!("{e:?}");      // chaîne complète sur plusieurs lignes + backtrace si activée
}
```

Utilise `{e:#}` dans les logs d'une ligne, `{e:?}` pour le terminal ou un rapport de bug.

### 6.2 — Parcourir la chaîne

```rust
fn causes(e: &anyhow::Error) {
    for (i, cause) in e.chain().enumerate() {
        println!("{i} : {cause}");
    }
    let racine = e.root_cause();
}
```

### 6.3 — Retrouver le type d'origine

```rust
fn reagir(e: &anyhow::Error) {
    if let Some(io) = e.downcast_ref::<std::io::Error>() {
        if io.kind() == std::io::ErrorKind::NotFound {
            println!("fichier absent");
        }
    }
}
```

`downcast_ref` cherche le type dans **toute la chaîne**, contexte compris. Si tu dois souvent faire des `downcast`, c'est le signe qu'une erreur typée (`thiserror`) conviendrait mieux à cet endroit.

---

## 7. `thiserror` ou `anyhow`

| | `thiserror` | `anyhow` |
|---|---|---|
| Nature | génère un type d'erreur à toi | un type d'erreur unique, opaque |
| L'appelant peut faire un `match` | oui | non (seulement `downcast`) |
| Contexte | à prévoir dans les variantes | `.context()` partout |
| Pour | bibliothèques, couches dont l'appelant doit réagir aux cas | applications, binaires, scripts, couches où l'erreur sera seulement affichée |
| Dans une API publique | oui | à éviter : l'utilisateur perd l'information de type |

La règle courante : **`thiserror` dans les bibliothèques, `anyhow` dans les applications.** Les deux se combinent :

```rust
// Une bibliothèque expose des erreurs typées
#[derive(Debug, thiserror::Error)]
pub enum ErreurPaiement {
    #[error("carte refusée")]
    Refusee,
    #[error("service indisponible")]
    Indisponible(#[source] std::io::Error),
}

// L'application les reçoit avec anyhow : la conversion est automatique
fn payer() -> anyhow::Result<()> {
    debiter().context("paiement de la commande 42")?;
    Ok(())
}

fn debiter() -> Result<(), ErreurPaiement> { Err(ErreurPaiement::Refusee) }

// ... et peut encore réagir au cas précis
fn gerer() {
    if let Err(e) = payer() {
        if let Some(ErreurPaiement::Refusee) = e.downcast_ref::<ErreurPaiement>() {
            println!("proposer un autre moyen de paiement");
        }
    }
}
```

---

## 8. Erreurs par couche

Chaque couche a son vocabulaire : la couche de données parle de base et de lignes, la couche métier de règles, la couche HTTP de codes de statut.

### 8.1 — Couche données

```rust
#[derive(Debug, thiserror::Error)]
pub enum ErreurDepot {
    #[error("article {0} introuvable")]
    Introuvable(i32),
    #[error("erreur de base de données")]
    Base(#[from] sea_orm::DbErr),
}
```

### 8.2 — Couche métier

```rust
#[derive(Debug, thiserror::Error)]
pub enum ErreurArticle {
    #[error("titre vide")]
    TitreVide,
    #[error("vous n'êtes pas l'auteur de cet article")]
    PasAuteur,
    #[error(transparent)]
    Depot(#[from] ErreurDepot),
}

pub async fn publier(id: i32, auteur: i32) -> Result<(), ErreurArticle> {
    let article = charger(id).await?;            // ErreurDepot -> ErreurArticle::Depot
    if article.auteur != auteur {
        return Err(ErreurArticle::PasAuteur);
    }
    Ok(())
}

struct Article { auteur: i32 }
async fn charger(id: i32) -> Result<Article, ErreurDepot> { Err(ErreurDepot::Introuvable(id)) }
```

### 8.3 — Couche HTTP : traduire en statut

```rust
fn statut(e: &ErreurArticle) -> u16 {
    match e {
        ErreurArticle::TitreVide => 400,
        ErreurArticle::PasAuteur => 403,
        ErreurArticle::Depot(ErreurDepot::Introuvable(_)) => 404,
        ErreurArticle::Depot(ErreurDepot::Base(_)) => 500,
    }
}
```

Le `match` est exhaustif : ajouter une variante oblige à décider de son code HTTP.

---

## 9. Erreurs dans les handlers Runique

### 9.1 — `AppResult` et `AppError`

Un handler Runique renvoie `AppResult<Response>`, alias de `Result<Response, Box<AppError>>`. `AppError` n'est pas une enum : c'est une struct qui porte un `ErrorContext` (code HTTP, titre, message, informations pour la page de debug).

```rust
// runique/src/context/template.rs
pub struct AppError {
    pub context: ErrorContext,
}

// runique/src/utils/aliases/definition.rs
pub type AppResult<T> = Result<T, Box<AppError>>;
```

`AppError` implémente `IntoResponse` : renvoyer `Err(...)` depuis un handler produit la page d'erreur correspondant au statut (404, 500...), ou la page de debug en développement.

### 9.2 — Les conversions déjà fournies

`DbErr`, `anyhow::Error` et les erreurs de session passent par `?` sans rien écrire :

```rust
// runique/src/context/template.rs
impl_from_error!(
    anyhow::Error => from_anyhow,
    DbErr => database,
    tower_sessions::session::Error => database
);
```

`impl_from_error!(Type => methode, ...)` génère `From<Type> for AppError` et `From<Type> for Box<AppError>`, en appelant `ErrorContext::methode(&erreur)`. Ces erreurs deviennent des 500 : le détail est journalisé, le visiteur voit une page générique.

### 9.3 — Choisir le statut soi-même

```rust
use runique::prelude::*;

pub async fn detail_article(request: Request) -> AppResult<Response> {
    let id = request.get_path_as::<Pk>("id").unwrap_or_default();
    let Some(article) = article::Entity::find_by_id(id)
        .one(request.db())
        .await?                                      // DbErr -> 500 automatiquement
    else {
        return Err(Box::new(AppError::new(ErrorContext::not_found(&format!("/articles/{id}")))));
    };
    // ...
}
```

Constructeurs disponibles sur `ErrorContext` : `not_found(chemin)`, `bad_request(message)`, `database(erreur)`, `from_anyhow(&erreur)`, `generic(StatusCode, message)`.

### 9.4 — Raccourci : `get_or_404`

Pour l'accès le plus courant (une ligne par sa clé, 404 si elle n'existe pas), le gestionnaire `objects` d'une entité fournit `get_or_404(db, id, &request, message)`, qui renvoie la ligne ou directement la réponse d'erreur (404 si absente, 500 si la requête échoue).

### 9.5 — Brancher sa propre erreur sur `?`

```rust
use runique::prelude::*;

impl From<ErreurArticle> for Box<AppError> {
    fn from(e: ErreurArticle) -> Self {
        let contexte = match &e {
            ErreurArticle::TitreVide => ErrorContext::bad_request("Le titre est obligatoire."),
            ErreurArticle::PasAuteur => {
                ErrorContext::generic(StatusCode::FORBIDDEN, "Action réservée à l'auteur.")
            }
            ErreurArticle::Depot(ErreurDepot::Introuvable(id)) => {
                ErrorContext::not_found(&format!("/articles/{id}"))
            }
            ErreurArticle::Depot(ErreurDepot::Base(db)) => ErrorContext::database(db),
        };
        Box::new(AppError::new(contexte))
    }
}

pub async fn publier_article(request: Request) -> AppResult<Response> {
    let id = request.get_path_as::<i32>("id").unwrap_or_default();
    publier(id, 1).await?;              // ErreurArticle -> Box<AppError> grâce au From
    // ...
}
```

`From<ErreurArticle> for Box<AppError>` est autorisé dans ton crate parce que `ErreurArticle` t'appartient (règle de l'orphelin, voir `traits-avances`).

### 9.6 — `anyhow` dans les services

```rust
use anyhow::Context;

pub async fn creer_article(
    titre: String,
    contenu: String,
    db: &DatabaseConnection,
) -> anyhow::Result<article::Model> {
    let actif = article::ActiveModel {
        titre: Set(titre.trim().to_string()),
        contenu: Set(contenu),
        ..Default::default()
    };
    actif
        .insert(db)
        .await
        .with_context(|| format!("insertion de l'article « {} »", titre.trim()))
}
```

Le handler appelle `creer_article(...).await?` : l'`anyhow::Error` devient une 500 grâce aux conversions de la section 9.2, avec toute la chaîne de contexte dans les logs.

### 9.7 — Erreur de formulaire : pas une `AppError`

Une saisie invalide n'est pas une erreur HTTP : on réaffiche le formulaire avec les messages sous les champs. Les formulaires Runique (`ValidationForm::try_form`, `is_valid`) gèrent ce cas ; `AppError` est réservé à ce qui empêche de produire la page.

---

## 10. Sécurité des messages d'erreur

| Où | Ce qu'on y met |
|---|---|
| Message affiché au visiteur | une phrase générique, utile pour lui : « Article introuvable », « Action réservée à l'auteur » |
| Logs du serveur | tout : chaîne de causes, requête, identifiants, pile d'appels |
| Page de debug | réservée au développement (`DEBUG=true`), jamais en production |

À ne jamais renvoyer au visiteur : chemin de fichier du serveur, requête SQL, nom de table, message brut de la base, jeton, mot de passe, adresse e-mail d'un autre utilisateur.

Attention aussi aux messages qui **révèlent** une information : « mot de passe incorrect » contre « utilisateur inconnu » permet de savoir quels comptes existent. Pour une connexion, un seul message pour les deux cas.

```rust
#[derive(Debug, thiserror::Error)]
enum ErreurConnexion {
    #[error("identifiants invalides")]     // même message pour les deux cas
    UtilisateurInconnu,
    #[error("identifiants invalides")]
    MauvaisMotDePasse,
}
// Les deux variantes restent distinctes pour les logs et les tests.
```

---

## 11. La même fonction écrite de plusieurs façons

Charger un fichier JSON de paramètres.

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Parametres { port: u16 }

// 1. Box<dyn Error> : simple, sans contexte
fn charger_v1(chemin: &str) -> Result<Parametres, Box<dyn std::error::Error>> {
    let t = std::fs::read_to_string(chemin)?;
    Ok(serde_json::from_str(&t)?)
}

// 2. anyhow + contexte : pour une application
fn charger_v2(chemin: &str) -> anyhow::Result<Parametres> {
    use anyhow::Context;
    let t = std::fs::read_to_string(chemin).with_context(|| format!("lecture de {chemin}"))?;
    serde_json::from_str(&t).with_context(|| format!("format de {chemin}"))
}

// 3. thiserror : pour une bibliothèque, l'appelant distingue les cas
#[derive(Debug, thiserror::Error)]
enum ErreurParametres {
    #[error("lecture de {chemin}")]
    Lecture { chemin: String, #[source] cause: std::io::Error },
    #[error("format de {chemin}")]
    Format { chemin: String, #[source] cause: serde_json::Error },
}

fn charger_v3(chemin: &str) -> Result<Parametres, ErreurParametres> {
    let t = std::fs::read_to_string(chemin).map_err(|cause| ErreurParametres::Lecture {
        chemin: chemin.into(),
        cause,
    })?;
    serde_json::from_str(&t).map_err(|cause| ErreurParametres::Format {
        chemin: chemin.into(),
        cause,
    })
}
```

---

## 12. Exercices

### Exercice 1 — type d'erreur avec `thiserror`

Crée `ErreurInscription` avec `EmailInvalide(String)`, `MotDePasseCourt { min: usize }` et `Base(#[from] sea_orm::DbErr)`.

```rust
// Solution :
#[derive(Debug, thiserror::Error)]
enum ErreurInscription {
    #[error("adresse email invalide : {0}")]
    EmailInvalide(String),
    #[error("mot de passe trop court (minimum {min} caractères)")]
    MotDePasseCourt { min: usize },
    #[error("erreur de base de données")]
    Base(#[from] sea_orm::DbErr),
}
```

### Exercice 2 — contexte

Réécris cette fonction pour que l'erreur dise quel fichier et quelle ligne posent problème.

```rust
fn total(chemin: &str) -> anyhow::Result<i64> {
    let t = std::fs::read_to_string(chemin)?;
    let mut s = 0;
    for l in t.lines() {
        s += l.parse::<i64>()?;
    }
    Ok(s)
}
```

```rust
// Solution :
use anyhow::Context;

fn total(chemin: &str) -> anyhow::Result<i64> {
    let t = std::fs::read_to_string(chemin).with_context(|| format!("lecture de {chemin}"))?;
    let mut s = 0;
    for (i, l) in t.lines().enumerate() {
        s += l.trim().parse::<i64>().with_context(|| format!("{chemin}, ligne {} : « {l} »", i + 1))?;
    }
    Ok(s)
}
```

### Exercice 3 — vers une réponse Runique

Écris `From<ErreurInscription> for Box<AppError>` : 400 pour les deux erreurs de saisie, 500 pour la base.

```rust
// Solution :
use runique::prelude::*;

impl From<ErreurInscription> for Box<AppError> {
    fn from(e: ErreurInscription) -> Self {
        let c = match &e {
            ErreurInscription::Base(db) => ErrorContext::database(db),
            autre => ErrorContext::bad_request(&autre.to_string()),
        };
        Box::new(AppError::new(c))
    }
}
```

`to_string()` est sûr ici : les messages des deux variantes de saisie ne contiennent rien d'interne. Pour `Base`, le message de la base n'est jamais montré.

---

## 13. Aide-mémoire

```rust
// thiserror
#[derive(Debug, thiserror::Error)]
enum E {
    #[error("texte {0}")]            A(i32),
    #[error("{champ}")]              B { champ: String },
    #[error("cause")]                C(#[from] std::io::Error),
    #[error("x")]                    D { #[source] cause: std::io::Error },
    #[error(transparent)]            F(#[from] anyhow::Error),
}

// anyhow
use anyhow::{anyhow, bail, ensure, Context, Result};
fn f() -> Result<()> {
    let v = op().context("étape")?;
    let w = op().with_context(|| format!("étape {v}"))?;
    let x = option.context("absent")?;
    ensure!(cond, "message {v}");
    bail!("message");
}
format!("{e}")  format!("{e:#}")  format!("{e:?}")  e.chain()  e.downcast_ref::<T>()

// Runique
AppResult<Response>                       // Result<Response, Box<AppError>>
Err(Box::new(AppError::new(ErrorContext::not_found(chemin))))
ErrorContext::bad_request(msg)   ErrorContext::generic(StatusCode::FORBIDDEN, msg)
impl From<MonErreur> for Box<AppError> { .. }
```

### Points clés

- `thiserror` génère le code d'une erreur typée ; `anyhow` fournit une erreur unique avec contexte.
- Bibliothèque : `thiserror`. Application : `anyhow`. Les deux se combinent.
- `#[from]` pour convertir, `#[source]` pour garder une cause avec d'autres données, `transparent` pour déléguer.
- `with_context` plutôt que `context(format!(..))`.
- Chaque couche traduit les erreurs dans son vocabulaire ; la couche HTTP choisit le statut.
- Dans Runique, `DbErr` et `anyhow::Error` deviennent des 500 par `?` ; un `From<MonErreur> for Box<AppError>` choisit le statut de ses propres erreurs.
- Le visiteur voit un message générique, les logs voient tout.
