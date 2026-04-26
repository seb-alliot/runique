# Serde — Sérialisation et Désérialisation
> Indispensables · Niveau intermédiaire

Convertir des données Rust vers JSON, TOML, YAML et tout format structuré, et inversement.

## Objectifs

- Comprendre le fonctionnement de serde
- Ajouter serde à un projet et choisir les formats
- Dériver `Serialize` et `Deserialize` automatiquement
- Personnaliser la sérialisation avec les attributs `#[serde(...)]`
- Sérialiser des enums et des types génériques
- Appliquer serde dans un contexte SeaORM / API JSON

---

## Table des matières

1. [Qu'est-ce que serde](#1-quest-ce-que-serde)
2. [Ajouter serde au projet](#2-ajouter-serde-au-projet)
3. [derive(Serialize, Deserialize)](#3-deriveserialize-deserialize)
4. [JSON avec serde_json](#4-json-avec-serde_json)
5. [TOML avec toml](#5-toml-avec-toml)
6. [Personnaliser la sérialisation](#6-personnaliser-la-sérialisation)
7. [Sérialisation d'enums](#7-sérialisation-denums)
8. [Types génériques et serde](#8-types-génériques-et-serde)
9. [Exemples concrets avec Runique](#9-exemples-concrets-avec-runique)
10. [Exercices pratiques](#10-exercices-pratiques)
11. [Aide-mémoire](#11-aide-mémoire)

---

## 1. Qu'est-ce que serde

`serde` (SERialization/DEserialization) est le standard de facto pour convertir des structures Rust vers et depuis des formats de données : JSON, TOML, YAML, MessagePack, CBOR, etc.

**Architecture en deux couches :**

```rust
// Couche 1 — serde core
// Définit les traits Serialize et Deserialize.
// Tes types implémentent ces traits.

// Couche 2 — serde_json, toml, serde_yaml...
// Chaque crate de format sait comment lire/écrire son format.
// Elle utilise les traits de serde core pour traverser tes données.
```

Le principal avantage : tu annotas tes types **une seule fois**, et tu peux les sérialiser vers **n'importe quel format** sans modifier le code.

---

## 2. Ajouter serde au projet

```toml
# Cargo.toml

[dependencies]
serde = { version = "1", features = ["derive"] }

# Formats courants (ajouter selon le besoin)
serde_json = "1"
toml        = "0.8"
serde_yaml  = "0.9"
```

La feature `derive` active les macros `#[derive(Serialize, Deserialize)]`. Sans elle, tu devrais tout implémenter à la main.

---

## 3. `#[derive(Serialize, Deserialize)]`

C'est la façon la plus courante d'utiliser serde. La macro génère le code d'(dé)sérialisation pour toi.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Utilisateur {
    id: u32,
    nom: String,
    email: String,
    actif: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Article {
    titre: String,
    contenu: String,
    auteur: Utilisateur,
    tags: Vec<String>,
}
```

Les types standard (`String`, `Vec`, `Option`, `HashMap`, entiers, flottants, booléens) sont tous sérialisables nativement.

```rust
// Option<T> se sérialise en null si None, en valeur si Some
#[derive(Serialize, Deserialize)]
struct Profil {
    nom: String,
    bio: Option<String>,       // null en JSON si absent
    age: Option<u8>,
}
```

---

## 4. JSON avec `serde_json`

### Sérialiser (Rust → JSON)

```rust
use serde_json;

let utilisateur = Utilisateur {
    id: 1,
    nom: "Alice".to_string(),
    email: "alice@example.com".to_string(),
    actif: true,
};

// Vers une String
let json = serde_json::to_string(&utilisateur)?;
// {"id":1,"nom":"Alice","email":"alice@example.com","actif":true}

// Vers une String formatée (indentée)
let json_pretty = serde_json::to_string_pretty(&utilisateur)?;

// Vers un Vec<u8> (pour écrire dans un fichier, une socket...)
let bytes = serde_json::to_vec(&utilisateur)?;
```

### Désérialiser (JSON → Rust)

```rust
let json = r#"{"id": 2, "nom": "Bob", "email": "bob@example.com", "actif": false}"#;

// Depuis une &str ou String
let utilisateur: Utilisateur = serde_json::from_str(json)?;

// Depuis des bytes
let utilisateur: Utilisateur = serde_json::from_slice(&bytes)?;

// Depuis un reader (fichier, réseau...)
let fichier = std::fs::File::open("utilisateur.json")?;
let utilisateur: Utilisateur = serde_json::from_reader(fichier)?;
```

### La valeur dynamique `serde_json::Value`

Quand la structure n'est pas connue à la compilation :

```rust
use serde_json::{json, Value};

// Construire du JSON sans struct
let payload = json!({
    "action": "connexion",
    "donnees": {
        "id": 42,
        "roles": ["admin", "user"]
    }
});

// Naviguer dans un JSON arbitraire
let data: Value = serde_json::from_str(json_inconnu)?;

if let Some(nom) = data["utilisateur"]["nom"].as_str() {
    println!("Nom : {nom}");
}
```

---

## 5. TOML avec `toml`

```toml
# config.toml
[serveur]
hote = "0.0.0.0"
port = 8080

[base_de_donnees]
url = "postgres://localhost/mydb"
pool_max = 10
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    serveur: ConfigServeur,
    base_de_donnees: ConfigDb,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConfigServeur {
    hote: String,
    port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConfigDb {
    url: String,
    pool_max: u32,
}

// Lire un fichier TOML
let contenu = std::fs::read_to_string("config.toml")?;
let config: Config = toml::from_str(&contenu)?;

println!("Port : {}", config.serveur.port);

// Écrire en TOML
let toml_str = toml::to_string_pretty(&config)?;
std::fs::write("config_export.toml", toml_str)?;
```

---

## 6. Personnaliser la sérialisation

Les attributs `#[serde(...)]` permettent d'adapter le comportement sans écrire de code d'(dé)sérialisation manuellement.

### `#[serde(rename)]` — renommer un champ

```rust
#[derive(Serialize, Deserialize)]
struct Commande {
    #[serde(rename = "order_id")]
    id_commande: u64,

    #[serde(rename = "customer_name")]
    nom_client: String,
}
// Sérialisé en : {"order_id": 1, "customer_name": "Alice"}
```

### `#[serde(rename_all)]` — renommer tous les champs

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReponseApi {
    user_id: u32,          // → userId
    nom_complet: String,   // → nomComplet
    date_creation: String, // → dateCreation
}
// Valeurs possibles : "camelCase", "snake_case", "PascalCase",
//                     "SCREAMING_SNAKE_CASE", "kebab-case"
```

### `#[serde(skip)]` — exclure un champ

```rust
#[derive(Serialize, Deserialize)]
struct Utilisateur {
    id: u32,
    nom: String,

    #[serde(skip)]
    mot_de_passe_hash: String, // jamais sérialisé ni désérialisé

    #[serde(skip_serializing)]
    token_interne: String,     // désérialisé depuis JSON, jamais écrit
}
```

### `#[serde(default)]` — valeur par défaut si absent

```rust
#[derive(Serialize, Deserialize)]
struct Parametres {
    langue: String,

    #[serde(default)]              // utilise Default::default() → false
    notifications: bool,

    #[serde(default = "taille_par_defaut")]
    taille_page: u32,
}

fn taille_par_defaut() -> u32 { 20 }
```

### `#[serde(alias)]` — accepter plusieurs noms à la désérialisation

```rust
#[derive(Deserialize)]
struct Payload {
    #[serde(alias = "username", alias = "login")]
    nom_utilisateur: String,
    // Accepte "nom_utilisateur", "username" ou "login" dans le JSON entrant
}
```

---

## 7. Sérialisation d'enums

serde gère toutes les formes d'enums. La représentation JSON peut être contrôlée avec `#[serde(tag)]`.

### Forme par défaut (externally tagged)

```rust
#[derive(Serialize, Deserialize, Debug)]
enum Evenement {
    Connexion { user_id: u32 },
    Message { de: String, texte: String },
    Deconnexion,
}

// Serialisé en :
// {"Connexion":{"user_id":1}}
// {"Message":{"de":"Alice","texte":"Bonjour"}}
// "Deconnexion"
```

### `#[serde(tag = "type")]` — internally tagged

```rust
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Notification {
    Email { destinataire: String, sujet: String },
    Sms   { numero: String, corps: String },
    Push  { token: String },
}

// Serialisé en :
// {"type":"Email","destinataire":"alice@...","sujet":"..."}
// {"type":"Sms","numero":"+33...","corps":"..."}
```

### `#[serde(tag = "type", content = "data")]` — adjacently tagged

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum Reponse {
    Ok(String),
    Erreur { code: u32, message: String },
}

// {"type":"Ok","data":"succès"}
// {"type":"Erreur","data":{"code":404,"message":"Non trouvé"}}
```

### `#[serde(untagged)]` — sans tag

```rust
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Valeur {
    Entier(i64),
    Flottant(f64),
    Texte(String),
    Booleen(bool),
}
// Serde devine le variant selon la forme des données
```

---

## 8. Types génériques et serde

serde s'intègre naturellement avec les génériques, à condition d'ajouter les bounds appropriés.

```rust
use serde::{Deserialize, Serialize};

// T doit implémenter Serialize + Deserialize
#[derive(Serialize, Deserialize, Debug)]
struct Page<T> {
    items: Vec<T>,
    total: u64,
    page: u32,
    par_page: u32,
}

// Utilisation
let page: Page<Utilisateur> = Page {
    items: vec![/* ... */],
    total: 150,
    page: 1,
    par_page: 20,
};

let json = serde_json::to_string(&page)?;
```

Pour les bounds explicites dans les `impl` :

```rust
use serde::{de::DeserializeOwned, Serialize};

fn envoyer_json<T: Serialize>(valeur: &T) -> String {
    serde_json::to_string(valeur).unwrap()
}

fn recevoir_json<T: DeserializeOwned>(json: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

// DeserializeOwned = Deserialize<'static> — pas de lifetime à gérer
```

### Wrapper générique de réponse API

```rust
#[derive(Serialize)]
struct ApiReponse<T: Serialize> {
    succes: bool,
    donnees: Option<T>,
    erreur: Option<String>,
}

impl<T: Serialize> ApiReponse<T> {
    fn ok(donnees: T) -> Self {
        ApiReponse { succes: true, donnees: Some(donnees), erreur: None }
    }

    fn err(message: impl Into<String>) -> ApiReponse<()> {
        ApiReponse { succes: false, donnees: None, erreur: Some(message.into()) }
    }
}
```

---

## 9. Exemples concrets avec Runique

### Entités SeaORM qui dérivent Serialize

Dans Runique, les entités SeaORM sont dans `demo-app/src/entities/`. Pour les exposer via une API JSON, on dérive `Serialize` sur le `Model` :

```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "articles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub titre: String,
    pub contenu: String,
    pub publie: bool,

    #[serde(skip)]                     // ne pas exposer dans l'API
    pub auteur_id: i32,

    #[serde(rename = "createdAt")]     // convention camelCase pour le front
    pub created_at: DateTimeUtc,
}
```

### Réponses JSON dans un handler Axum

```rust
use axum::{extract::State, Json};
use serde::Serialize;

#[derive(Serialize)]
struct ArticleListeReponse {
    articles: Vec<article::Model>,
    total: u64,
}

pub async fn liste_articles(
    State(db): State<DatabaseConnection>,
) -> Result<Json<ArticleListeReponse>, AppError> {
    let articles = Article::find()
        .filter(article::Column::Publie.eq(true))
        .all(&db)
        .await?;

    let total = articles.len() as u64;

    Ok(Json(ArticleListeReponse { articles, total }))
}
```

### Désérialiser un body de requête

```rust
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreerArticlePayload {
    titre: String,
    contenu: String,

    #[serde(default)]
    publie: bool,

    tags: Vec<String>,
}

pub async fn creer_article(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreerArticlePayload>,
) -> Result<Json<article::Model>, AppError> {
    let nouvel_article = article::ActiveModel {
        titre:   Set(payload.titre),
        contenu: Set(payload.contenu),
        publie:  Set(payload.publie),
        ..Default::default()
    };

    let article = nouvel_article.insert(&db).await?;
    Ok(Json(article))
}
```

---

## 10. Exercices pratiques

### Exercice 1 : struct de configuration

Crée une struct `AppConfig` avec les champs : `hote: String`, `port: u16`, `debug: bool` (défaut `false`), `max_connexions: u32` (défaut `100`). Lis-la depuis le TOML suivant :

```toml
hote = "127.0.0.1"
port = 3000
```

### Exercice 2 : enum de statut

Crée un enum `StatutCommande` avec les variants `EnAttente`, `EnCours { depuis: String }`, `Livree`, `Annulee { raison: String }`. Sérialise-le avec `#[serde(tag = "statut")]` et vérifie la sortie JSON.

### Exercice 3 : pagination générique

Écris une struct `PaginatedResponse<T>` avec `items: Vec<T>`, `total: u64`, `page: u32`. Sérialise-la avec une liste de strings, puis une liste d'entiers, en vérifiant que le JSON produit est correct.

### Exercice 4 : alias et renommage

Crée une struct `LoginPayload` qui accepte `"email"` ou `"username"` pour le champ identifiant, et `"password"` ou `"mot_de_passe"` pour le mot de passe. Utilise `#[serde(alias)]`.

---

## 11. Aide-mémoire

| Besoin | Attribut / méthode |
|---|---|
| Renommer un champ | `#[serde(rename = "nom")]` |
| Renommer tous les champs | `#[serde(rename_all = "camelCase")]` |
| Exclure un champ | `#[serde(skip)]` |
| Exclure à la sérialisation | `#[serde(skip_serializing)]` |
| Valeur par défaut | `#[serde(default)]` ou `#[serde(default = "fn")]` |
| Accepter plusieurs noms | `#[serde(alias = "autre")]` |
| Enum avec tag interne | `#[serde(tag = "type")]` |
| Enum sans tag | `#[serde(untagged)]` |
| Sérialiser → JSON String | `serde_json::to_string(&val)?` |
| Désérialiser ← JSON String | `serde_json::from_str::<T>(json)?` |
| JSON formaté | `serde_json::to_string_pretty(&val)?` |
| JSON dynamique | `serde_json::Value` + macro `json!{}` |
| TOML → struct | `toml::from_str::<T>(&contenu)?` |
| struct → TOML | `toml::to_string_pretty(&val)?` |
| Bound générique désér. | `T: DeserializeOwned` |

> **Règle :** Dériez toujours `Serialize` + `Deserialize` ensemble sauf besoin explicite de l'un sans l'autre. Préférez `#[serde(rename_all)]` au niveau de la struct plutôt que des `rename` champ par champ.
