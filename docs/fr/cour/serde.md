# Sérialisation — serde

> Transformer des structures Rust en JSON, TOML ou formulaires, et inversement : dériver, renommer, ignorer, valider, choisir la représentation des enums, et écrire ses propres conversions.

## Objectifs

- Comprendre le rôle de serde et de ses formats (`serde_json`, `toml`, `serde_urlencoded`)
- Dériver `Serialize` et `Deserialize` et savoir ce qu'ils exigent
- Lire et écrire du JSON typé ou dynamique (`serde_json::Value`)
- Personnaliser les noms, les valeurs par défaut, les champs ignorés ou facultatifs
- Choisir la représentation d'une enum (quatre formes)
- Écrire une conversion sur mesure (`with`, `serialize_with`, `try_from`)
- Refuser les entrées inattendues et ne jamais exposer un champ sensible
- Utiliser serde dans un handler Runique

## Table des matières

1. Qu'est-ce que serde
2. Ajouter serde au projet
3. `#[derive(Serialize, Deserialize)]`
4. JSON avec `serde_json`
5. TOML et formulaires
6. Personnaliser les champs
7. Valeurs absentes, `null` et `Option`
8. Sérialisation d'enums
9. Conversions sur mesure
10. Types génériques et serde
11. Sécurité : entrées et sorties
12. Exemples concrets avec Runique
13. La même conversion écrite de plusieurs façons
14. Exercices pratiques
15. Aide-mémoire

---

## 1. Qu'est-ce que serde

serde (*SERialize / DEserialize*) sépare deux choses :

- **ce qu'est une donnée** : tes structs et enums, qui dérivent `Serialize` et `Deserialize` ;
- **le format** : JSON, TOML, YAML, formulaire, MessagePack... chacun fourni par une crate.

```
struct Utilisateur ──Serialize──► modèle de données serde ──serde_json──► {"nom":"Ada"}
                    ◄─Deserialize──                        ◄─────────────
```

Une struct qui dérive serde fonctionne avec **tous** les formats, sans changer une ligne. Tout le travail est fait à la compilation : pas de réflexion à l'exécution, et des performances proches d'un code écrit à la main.

---

## 2. Ajouter serde au projet

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }   # derive : active #[derive(Serialize, Deserialize)]
serde_json = "1"                                   # format JSON
toml = "0.8"                                       # format TOML (si besoin)
```

Les crates de l'écosystème proposent souvent une feature `serde` : `chrono = { version = "0.4", features = ["serde"] }`, `uuid = { version = "1", features = ["serde"] }`, `rust_decimal` avec `serde-float` ou `serde-str`. Sans elle, leurs types ne sont pas sérialisables (voir `cargo-dependances`, features).

---

## 3. `#[derive(Serialize, Deserialize)]`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Utilisateur {
    id: u32,
    nom: String,
    email: String,
    actif: bool,
    roles: Vec<String>,
}
```

Condition : **chaque champ** doit lui-même être sérialisable. Les types standard le sont (`String`, nombres, `bool`, `Vec`, `HashMap` à clés texte ou nombres, `Option`, tuples, tableaux, `Box`...).

```rust
struct Connexion;                         // pas de Serialize

// #[derive(Serialize)]
// struct Etat { c: Connexion }           // ERREUR : the trait `Serialize` is not implemented for `Connexion`
```

Seul `Serialize` ou seul `Deserialize` suffit selon le besoin : une réponse d'API n'a besoin que de `Serialize`, un payload reçu que de `Deserialize`.

---

## 4. JSON avec `serde_json`

### 4.1 Sérialiser (Rust → JSON)

```rust
let u = Utilisateur {
    id: 1,
    nom: "Ada".into(),
    email: "ada@ex.com".into(),
    actif: true,
    roles: vec!["admin".into()],
};

let compact = serde_json::to_string(&u)?;          // {"id":1,"nom":"Ada",...}
let lisible = serde_json::to_string_pretty(&u)?;   // indenté
let octets: Vec<u8> = serde_json::to_vec(&u)?;
serde_json::to_writer(std::io::stdout(), &u)?;     // directement dans un flux, sans String intermédiaire
```

### 4.2 Désérialiser (JSON → Rust)

```rust
let texte = r#"{"id":2,"nom":"Linus","email":"l@ex.com","actif":false,"roles":[]}"#;

// 1. Annoter la variable
let u: Utilisateur = serde_json::from_str(texte)?;

// 2. Turbofish
let u = serde_json::from_str::<Utilisateur>(texte)?;

// 3. Depuis des octets ou un lecteur
let u: Utilisateur = serde_json::from_slice(texte.as_bytes())?;
let f = std::fs::File::open("u.json")?;
let u: Utilisateur = serde_json::from_reader(std::io::BufReader::new(f))?;
```

Un champ manquant, en trop (selon les options, section 11), ou d'un mauvais type produit une erreur qui indique la ligne et la colonne : `missing field \`email\` at line 1 column 30`.

### 4.3 La valeur dynamique `serde_json::Value`

```rust
use serde_json::{json, Value};

let v: Value = serde_json::from_str(r#"{"nom":"Ada","langages":["rust","c"],"age":36}"#)?;

let nom = v["nom"].as_str();                     // Some("Ada")
let premier = v["langages"][0].as_str();         // Some("rust")
let absent = &v["inexistant"];                   // Value::Null, pas de panique
let age = v.get("age").and_then(Value::as_u64);  // Some(36)

let construit = json!({
    "statut": "ok",
    "total": 3,
    "elements": [1, 2, 3],
});
```

| | struct typée | `Value` |
|---|---|---|
| Vérification de la forme | à la désérialisation | à chaque accès |
| Autocomplétion, erreurs de compilation | oui | non |
| Pour | toute donnée dont la forme est connue | JSON de forme inconnue, transformation générique, tests |

### 4.4 Passer d'une forme à l'autre

```rust
let v: Value = serde_json::to_value(&u)?;            // struct -> Value
let u2: Utilisateur = serde_json::from_value(v)?;    // Value -> struct
```

---

## 5. TOML et formulaires

### 5.1 TOML : fichiers de configuration

```rust
#[derive(Debug, Deserialize)]
struct Config {
    serveur: Serveur,
    base: Base,
}

#[derive(Debug, Deserialize)]
struct Serveur { hote: String, port: u16 }

#[derive(Debug, Deserialize)]
struct Base { url: String, connexions: u32 }

let texte = r#"
[serveur]
hote = "0.0.0.0"
port = 8080

[base]
url = "postgres://localhost/app"
connexions = 10
"#;

let c: Config = toml::from_str(texte)?;
```

Les tables TOML (`[serveur]`) correspondent aux structs imbriquées. La même struct se lirait depuis un JSON `{"serveur": {...}, "base": {...}}` sans modification.

### 5.2 Formulaires HTML

```rust
#[derive(Deserialize)]
struct Recherche { q: String, page: Option<u32> }

let r: Recherche = serde_urlencoded::from_str("q=rust+web&page=2")?;
```

C'est le format des formulaires HTML et des paramètres d'URL. Les extracteurs `Form<T>` et `Query<T>` d'axum l'utilisent.

---

## 6. Personnaliser les champs

### 6.1 `rename` et `rename_all`

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]           // tous les champs : cree_le -> creeLe
struct Article {
    id: u32,
    cree_le: String,
    #[serde(rename = "titreArticle")]        // un champ précis
    titre: String,
}
```

Valeurs de `rename_all` : `lowercase`, `UPPERCASE`, `camelCase`, `PascalCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, `SCREAMING-KEBAB-CASE`.

### 6.2 Noms différents à l'entrée et à la sortie

```rust
#[derive(Serialize, Deserialize)]
struct Produit {
    #[serde(rename(serialize = "price", deserialize = "prix"))]
    prix: f64,
}
```

### 6.3 `alias` : accepter plusieurs noms à la lecture

```rust
#[derive(Deserialize)]
struct Contact {
    #[serde(alias = "mail", alias = "e-mail")]
    email: String,
}
```

À quoi ça sert : renommer un champ sans casser les anciens clients ou les anciens fichiers.

### 6.4 `skip` : exclure un champ

```rust
#[derive(Serialize, Deserialize)]
struct Session {
    utilisateur: String,
    #[serde(skip)]                           // ni écrit ni lu ; à la lecture : Default::default()
    cache: Vec<u8>,
    #[serde(skip_serializing)]               // lu mais jamais écrit
    mot_de_passe: String,
    #[serde(skip_deserializing)]             // écrit mais jamais lu
    calcule: u32,
}
```

### 6.5 `skip_serializing_if` : omettre une valeur vide

```rust
#[derive(Serialize)]
struct Reponse {
    donnees: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    erreur: Option<String>,                  // absent du JSON si None, au lieu de "erreur": null
    #[serde(skip_serializing_if = "Vec::is_empty")]
    avertissements: Vec<String>,
}
```

### 6.6 `default` : une valeur si le champ est absent

```rust
#[derive(Deserialize)]
struct Options {
    #[serde(default)]                        // absent -> Default::default() (0, "", false, vec vide)
    verbeux: bool,
    #[serde(default = "port_defaut")]        // absent -> valeur d'une fonction
    port: u16,
}

fn port_defaut() -> u16 { 8080 }

#[derive(Deserialize, Default)]
#[serde(default)]                            // sur la struct : tout champ absent prend la valeur de Default
struct Reglages { taille: u32, theme: String }
```

### 6.7 `flatten` : intégrer les champs d'une autre struct

```rust
#[derive(Serialize, Deserialize)]
struct Pagination { page: u32, par_page: u32 }

#[derive(Serialize, Deserialize)]
struct ListeArticles {
    articles: Vec<String>,
    #[serde(flatten)]
    pagination: Pagination,                  // {"articles":[..],"page":1,"par_page":20}
}

#[derive(Deserialize)]
struct AvecExtras {
    nom: String,
    #[serde(flatten)]
    autres: std::collections::HashMap<String, serde_json::Value>,   // récupère tous les champs inconnus
}
```

---

## 7. Valeurs absentes, `null` et `Option`

```rust
#[derive(Deserialize, Debug)]
struct Profil {
    nom: String,
    bio: Option<String>,
}
```

| JSON reçu | `bio` vaut |
|---|---|
| `{"nom":"a","bio":"x"}` | `Some("x")` |
| `{"nom":"a","bio":null}` | `None` |
| `{"nom":"a"}` | `None` (un `Option` absent est accepté) |

### Distinguer « absent » de `null`

Pour une mise à jour partielle (PATCH), « champ absent » (ne pas toucher) et `null` (effacer) n'ont pas le même sens :

```rust
#[derive(Deserialize, Debug)]
struct MiseAJour {
    #[serde(default, deserialize_with = "present")]
    bio: Option<Option<String>>,
}

fn present<'de, D>(d: D) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(Option::deserialize(d)?))     // appelé seulement si le champ est présent
}

// {}            -> bio = None          : ne pas modifier
// {"bio":null}  -> bio = Some(None)    : effacer
// {"bio":"x"}   -> bio = Some(Some(x)) : remplacer
```

---

## 8. Sérialisation d'enums

Une même enum, quatre représentations JSON possibles.

```rust
#[derive(Serialize, Deserialize)]
enum Evenement {
    Connexion { utilisateur: String },
    Achat { montant: u32 },
    Deconnexion,
}
```

### 8.1 Par défaut : étiquette externe

```rust
// {"Connexion":{"utilisateur":"ada"}}
// {"Achat":{"montant":30}}
// "Deconnexion"
```

### 8.2 `#[serde(tag = "type")]` : étiquette interne

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Evenement2 {
    Connexion { utilisateur: String },
    Achat { montant: u32 },
    Deconnexion,
}
// {"type":"connexion","utilisateur":"ada"}
// {"type":"deconnexion"}
```

La forme la plus courante dans les API web. Ne marche pas avec des variantes tuple contenant un simple nombre ou texte (`Achat(u32)`).

### 8.3 `#[serde(tag = "type", content = "data")]` : étiquette adjacente

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum Message {
    Texte(String),
    Nombre(i64),
}
// {"type":"Texte","data":"bonjour"}
```

Fonctionne avec tous les types de variantes.

### 8.4 `#[serde(untagged)]` : sans étiquette

```rust
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Identifiant {
    Numero(u64),
    Nom(String),
}
// 42      -> Numero(42)
// "ada"   -> Nom("ada")
```

serde essaie chaque variante **dans l'ordre** et garde la première qui correspond. Pratique pour accepter plusieurs formes, mais les erreurs sont peu parlantes (« data did not match any variant ») et l'ordre compte.

### 8.5 Enum simple comme texte

```rust
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Statut { Brouillon, Publie, Archive }
// "brouillon", "publie", "archive"
```

| Forme | JSON | Pour |
|---|---|---|
| externe (défaut) | `{"Variante": {...}}` | échanges Rust ↔ Rust |
| interne `tag` | `{"type": "v", ...}` | API web |
| adjacente `tag` + `content` | `{"type": "v", "data": ...}` | variantes de formes très différentes |
| `untagged` | la valeur seule | accepter plusieurs formats d'entrée |

---

## 9. Conversions sur mesure

### 9.1 `with` : un module de conversion

```rust
mod secondes {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(d.as_secs())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        Ok(Duration::from_secs(u64::deserialize(d)?))
    }
}

#[derive(Serialize, Deserialize)]
struct Cache {
    #[serde(with = "secondes")]
    duree: std::time::Duration,           // {"duree": 300}
}
```

### 9.2 `serialize_with` / `deserialize_with` : un seul sens

```rust
#[derive(Deserialize)]
struct Inscription {
    #[serde(deserialize_with = "minuscules")]
    email: String,
}

fn minuscules<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(String::deserialize(d)?.trim().to_lowercase())
}
```

### 9.3 `try_from` : valider à la désérialisation

```rust
#[derive(Deserialize, Debug)]
#[serde(try_from = "String")]
struct Email(String);

impl TryFrom<String> for Email {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.contains('@') { Ok(Email(s)) } else { Err(format!("email invalide : {s}")) }
    }
}

#[derive(Deserialize, Debug)]
struct Compte { email: Email }

let r: Result<Compte, _> = serde_json::from_str(r#"{"email":"pas-un-email"}"#);   // Err : email invalide
```

Un JSON invalide ne peut alors **jamais** produire un `Email` : le principe « parse, don't validate » (voir `structures-enums`) appliqué à serde. `into = "String"` fait l'inverse à la sérialisation.

### 9.4 Implémenter `Serialize` à la main

```rust
use serde::ser::{Serialize, SerializeStruct, Serializer};

struct Point { x: i32, y: i32 }

impl Serialize for Point {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("Point", 3)?;
        st.serialize_field("x", &self.x)?;
        st.serialize_field("y", &self.y)?;
        st.serialize_field("distance", &((self.x.pow(2) + self.y.pow(2)) as f64).sqrt())?;   // champ calculé
        st.end()
    }
}
```

Rarement nécessaire : les attributs couvrent presque tous les cas. Implémenter `Deserialize` à la main est nettement plus long (un *visitor*) ; on le réserve aux formats vraiment particuliers.

### 9.5 Crates utiles

- `serde_with` : conversions prêtes à l'emploi (`DisplayFromStr`, durées, base64, valeurs vides en `None`).
- `chrono` et `time` avec leur feature `serde` : dates en RFC 3339.

---

## 10. Types génériques et serde

```rust
#[derive(Serialize, Deserialize, Debug)]
struct Reponse<T> {
    succes: bool,
    donnees: Option<T>,
    erreur: Option<String>,
}

impl<T> Reponse<T> {
    fn ok(d: T) -> Self { Reponse { succes: true, donnees: Some(d), erreur: None } }
    fn err(e: impl Into<String>) -> Self { Reponse { succes: false, donnees: None, erreur: Some(e.into()) } }
}

let r = Reponse::ok(vec![1, 2, 3]);
let json = serde_json::to_string(&r)?;     // {"succes":true,"donnees":[1,2,3],"erreur":null}
```

Le derive ajoute automatiquement `T: Serialize` (ou `T: Deserialize<'de>`) aux implémentations. Pour un paramètre qui n'est pas sérialisé lui-même (un `PhantomData<T>`), `#[serde(bound = "")]` retire cette contrainte.

### Emprunter au lieu de copier

```rust
#[derive(Deserialize)]
struct Ligne<'a> {
    nom: &'a str,                                  // pointe dans le texte JSON : aucune allocation
    #[serde(borrow)]
    note: std::borrow::Cow<'a, str>,               // emprunte si possible, alloue si le texte contient un échappement
}

let source = String::from(r#"{"nom":"ada","note":"bien"}"#);
let l: Ligne = serde_json::from_str(&source)?;
```

Un `&str` ne fonctionne que si le texte JSON ne contient aucune séquence d'échappement (`\"`, `\n`) ; `Cow` gère les deux cas (voir `borrow-avance`).

---

## 11. Sécurité : entrées et sorties

### 11.1 Refuser les champs inconnus

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangerMotDePasse {
    ancien: String,
    nouveau: String,
}
// {"ancien":"a","nouveau":"b","role":"admin"} -> Err : unknown field `role`
```

Par défaut, serde **ignore** les champs en trop. C'est rarement un problème (ils ne sont stockés nulle part), mais `deny_unknown_fields` signale une erreur de client, et documente que le payload est fermé.

### 11.2 Affectation de masse

```rust
// Dangereux : désérialiser directement l'entité complète depuis la requête
// #[derive(Deserialize)] struct Utilisateur { id: u32, nom: String, est_admin: bool }
// let u: Utilisateur = serde_json::from_str(corps)?;    // le client peut envoyer "est_admin": true

// Sûr : une struct d'entrée qui ne contient QUE ce que le client a le droit de fixer
#[derive(Deserialize)]
struct ModifierProfil {
    nom: String,
    bio: Option<String>,
}
```

Ne désérialise jamais une requête directement dans un type qui contient des champs que le client ne doit pas contrôler (rôle, identifiant, propriétaire, prix).

### 11.3 Ne pas tout exposer

```rust
#[derive(Serialize)]
struct UtilisateurPublic<'a> {
    id: u32,
    nom: &'a str,
    // ni le hachage du mot de passe, ni l'email, ni les jetons
}
```

Une struct de **sortie** dédiée garantit qu'un champ ajouté plus tard à l'entité (un jeton, une adresse) n'apparaît pas automatiquement dans l'API.

### 11.4 Taille et profondeur

`serde_json` refuse par défaut un JSON imbriqué à plus de 128 niveaux (protection contre l'épuisement de la pile). La **taille** du corps doit être limitée avant la désérialisation, par le serveur. Dans Runique, l'extracteur `Request` refuse un corps de formulaire ou de JSON au-delà de 2 Mo ; l'extracteur `Json` d'axum, lui, suit la limite globale des requêtes (`RUNIQUE_MAX_BODY_MB`, 100 Mo par défaut). Pour une route JSON publique, une limite plus basse se pose avec `DefaultBodyLimit::max(..)` sur la route.

---

## 12. Exemples concrets avec Runique

### 12.1 Les entités dérivent déjà serde

Le `Model` généré par `model!{}` dérive `Serialize` et `Deserialize` :

```rust
// code généré par model!{} (runique/derive_form/src/model/utils/sea_model.rs, extrait)
#[derive(Clone, Debug, PartialEq, ::sea_orm::DeriveEntityModel, ::serde::Serialize, ::serde::Deserialize)]
#[sea_orm(table_name = #table)]
pub struct Model { /* ... */ }
```

Deux conséquences :

- le code généré écrit `::serde::...` : le projet doit avoir `serde` dans ses dépendances (c'est le cas d'un projet créé par `runique new`) ;
- on ne peut pas ajouter `#[serde(skip)]` ni `rename` sur un champ généré. Pour contrôler ce qui sort (section 11.3), on passe par une struct de réponse.

### 12.2 Répondre en JSON

```rust
use runique::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct ArticleResume {
    id: Pk,
    titre: String,
}

#[derive(Serialize)]
struct ListeArticles {
    articles: Vec<ArticleResume>,
    total: usize,
}

pub async fn liste_articles(request: Request) -> AppResult<Response> {
    let articles = article::Entity::find()
        .filter(article::Column::Publie.eq(true))
        .all(request.db())
        .await?;                                          // DbErr -> AppError automatiquement

    let articles: Vec<ArticleResume> = articles
        .into_iter()
        .map(|a| ArticleResume { id: a.id, titre: a.titre })
        .collect();
    let total = articles.len();

    Ok(Json(ListeArticles { articles, total }).into_response())
}
```

Renvoyer directement `Vec<article::Model>` compilerait aussi (le `Model` est sérialisable), mais exposerait toutes les colonnes, y compris celles ajoutées plus tard.

### 12.3 Recevoir un JSON structuré

L'extracteur `Request` de Runique lit un corps JSON comme une table plate de textes (`StrMap`), pour les formulaires. Pour un JSON structuré (listes, booléens, objets imbriqués), on utilise l'extracteur `Json` d'axum à la place de `Request` : les deux lisent le corps, un handler ne peut en prendre qu'un. La base reste accessible par l'engine, que Runique place dans les extensions de chaque requête.

Le middleware CSRF s'applique toujours : un POST JSON doit porter l'en-tête `X-CSRF-Token`, avec le jeton que le template reçoit dans `{{ csrf_token }}` (par exemple posé dans une balise `<meta name="csrf-token">` et relu par le JavaScript), sinon il est refusé en 403 avant d'atteindre le handler.

```rust
use runique::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreerArticle {
    titre: String,
    contenu: String,
    #[serde(default)]
    publie: bool,
    #[serde(default)]
    tags: Vec<String>,
}

pub async fn creer_article(
    Extension(engine): Extension<AEngine>,
    Json(payload): Json<CreerArticle>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if payload.titre.trim().is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let nouvel_article = article::ActiveModel {
        titre: Set(payload.titre.trim().to_string()),
        contenu: Set(payload.contenu),
        publie: Set(payload.publie),
        ..Default::default()
    };
    let a = nouvel_article
        .insert(&engine.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;   // le détail de la base n'est pas renvoyé
    Ok(Json(serde_json::json!({ "id": a.id })))
}
```

`CreerArticle` ne contient que les champs que le client a le droit de fixer : ni l'identifiant, ni l'auteur, ni la date (section 11.2). Un champ inconnu est refusé.

---

## 13. La même conversion écrite de plusieurs façons

Accepter une date au format `"2026-11-09"` dans un JSON.

```rust
use serde::{Deserialize, Deserializer};

// 1. Un String, converti plus tard à la main : simple, mais la validation est repoussée
#[derive(Deserialize)]
struct V1 { date: String }

// 2. chrono avec sa feature serde : NaiveDate lit directement "AAAA-MM-JJ"
#[derive(Deserialize)]
struct V2 { date: chrono::NaiveDate }

// 3. deserialize_with : un format sur mesure ("09/11/2026")
#[derive(Deserialize)]
struct V3 {
    #[serde(deserialize_with = "date_fr")]
    date: chrono::NaiveDate,
}

fn date_fr<'de, D: Deserializer<'de>>(d: D) -> Result<chrono::NaiveDate, D::Error> {
    let s = String::deserialize(d)?;
    chrono::NaiveDate::parse_from_str(&s, "%d/%m/%Y").map_err(serde::de::Error::custom)
}

// 4. Un newtype avec try_from : réutilisable partout
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct DateFr(chrono::NaiveDate);

impl TryFrom<String> for DateFr {
    type Error = String;
    fn try_from(s: String) -> Result<Self, String> {
        chrono::NaiveDate::parse_from_str(&s, "%d/%m/%Y")
            .map(DateFr)
            .map_err(|e| format!("date invalide « {s} » : {e}"))
    }
}
```

| Version | Validation | Réutilisable | Pour |
|---|---|---|---|
| 1 | plus tard, à la main | non | prototype |
| 2 | à la lecture | oui | format standard |
| 3 | à la lecture | par champ | un format particulier, à un endroit |
| 4 | à la lecture | oui, partout | un format particulier, utilisé souvent |

`serde::de::Error::custom` transforme n'importe quelle erreur affichable en erreur serde, avec la position dans le JSON.

---

## 14. Exercices pratiques

### Exercice 1 : struct de configuration

Lis ce TOML dans une struct, avec un port par défaut de 8080 et un mode debug faux par défaut.

```toml
[serveur]
hote = "127.0.0.1"
```

```rust
// Solution :
#[derive(Deserialize, Debug)]
struct Fichier { serveur: Serveur }

#[derive(Deserialize, Debug)]
struct Serveur {
    hote: String,
    #[serde(default = "port_defaut")]
    port: u16,
    #[serde(default)]
    debug: bool,
}

fn port_defaut() -> u16 { 8080 }
```

### Exercice 2 : enum de statut

Fais en sorte que `Statut::EnAttente` s'écrive `"en_attente"` et qu'un statut inconnu soit refusé.

```rust
// Solution :
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Statut { EnAttente, Validee, Refusee }

assert_eq!(serde_json::to_string(&Statut::EnAttente)?, r#""en_attente""#);
assert!(serde_json::from_str::<Statut>(r#""inconnu""#).is_err());
```

### Exercice 3 : pagination générique

Écris `Page<T>` qui produit `{"elements":[...],"page":1,"total":42}` et omet `"suivante"` quand il n'y a pas de page suivante.

```rust
// Solution :
#[derive(Serialize)]
struct Page<T> {
    elements: Vec<T>,
    page: u32,
    total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    suivante: Option<u32>,
}
```

### Exercice 4 : alias et renommage

Une ancienne API envoie `"mail"`, la nouvelle `"email"`. Accepte les deux, renvoie toujours `"email"`, et passe l'adresse en minuscules.

```rust
// Solution :
#[derive(Serialize, Deserialize)]
struct Contact {
    #[serde(alias = "mail", deserialize_with = "minuscules")]
    email: String,
}
```

---

## 15. Aide-mémoire

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]        #[serde(deny_unknown_fields)]
#[serde(tag = "type")]   #[serde(tag = "t", content = "c")]   #[serde(untagged)]
#[serde(try_from = "String")]   #[serde(into = "String")]   #[serde(default)]

#[serde(rename = "x")]   #[serde(alias = "y")]   #[serde(skip)]
#[serde(skip_serializing_if = "Option::is_none")]   #[serde(default = "fonction")]
#[serde(flatten)]   #[serde(with = "module")]   #[serde(deserialize_with = "f")]   #[serde(borrow)]

serde_json::to_string(&v)?   to_string_pretty   to_vec   to_writer   to_value
serde_json::from_str::<T>(s)?   from_slice   from_reader   from_value
json!({ "a": 1 })   v["cle"].as_str()   v.get("cle")
toml::from_str::<T>(s)?   serde_urlencoded::from_str::<T>(s)?
```

### Points clés

- serde sépare la donnée (derive) du format (crate `serde_json`, `toml`...).
- Chaque champ doit être sérialisable ; les crates tierces ont souvent une feature `serde`.
- `Option` accepte un champ absent ; `default` donne une valeur quand il manque.
- Pour les API web, `#[serde(tag = "type")]` est la représentation d'enum la plus lisible.
- `try_from` valide à la lecture : un JSON invalide ne produit jamais une valeur invalide.
- Une struct d'entrée ne contient que ce que le client peut fixer ; une struct de sortie que ce qu'il peut voir.
- Dans Runique, les `Model` dérivent déjà serde : passe par une struct de réponse pour choisir les champs exposés.
