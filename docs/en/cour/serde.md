# Serialization — serde

> Turn Rust structures into JSON, TOML or forms, and back: derive, rename, skip, validate, choose the representation of enums, and write your own conversions.

## Objectives

- Understand the role of serde and its formats (`serde_json`, `toml`, `serde_urlencoded`)
- Derive `Serialize` and `Deserialize` and know what they require
- Read and write typed or dynamic JSON (`serde_json::Value`)
- Customize names, default values, skipped or optional fields
- Choose the representation of an enum (four forms)
- Write a custom conversion (`with`, `serialize_with`, `try_from`)
- Reject unexpected input and never expose a sensitive field
- Use serde in a Runique handler

## Table of contents

1. What serde is
2. Adding serde to the project
3. `#[derive(Serialize, Deserialize)]`
4. JSON with `serde_json`
5. TOML and forms
6. Customizing fields
7. Missing values, `null` and `Option`
8. Serializing enums
9. Custom conversions
10. Generic types and serde
11. Security: inputs and outputs
12. Concrete examples with Runique
13. The same conversion written in several ways
14. Practical exercises
15. Cheat sheet

---

## 1. What serde is

serde (*SERialize / DEserialize*) separates two things:

- **what a piece of data is**: your structs and enums, which derive `Serialize` and `Deserialize`;
- **the format**: JSON, TOML, YAML, form, MessagePack... each provided by a crate.

```
struct User ──Serialize──► serde data model ──serde_json──► {"name":"Ada"}
             ◄─Deserialize──                 ◄─────────────
```

A struct that derives serde works with **all** formats, without changing a line. All the work is done at compile time: no reflection at runtime, and performance close to hand-written code.

---

## 2. Adding serde to the project

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }   # derive: enables #[derive(Serialize, Deserialize)]
serde_json = "1"                                   # JSON format
toml = "0.8"                                       # TOML format (if needed)
```

Ecosystem crates often offer a `serde` feature: `chrono = { version = "0.4", features = ["serde"] }`, `uuid = { version = "1", features = ["serde"] }`, `rust_decimal` with `serde-float` or `serde-str`. Without it, their types cannot be serialized (see `cargo-dependances`, features).

---

## 3. `#[derive(Serialize, Deserialize)]`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
    active: bool,
    roles: Vec<String>,
}
```

Condition: **every field** must itself be serializable. Standard types are (`String`, numbers, `bool`, `Vec`, `HashMap` with text or number keys, `Option`, tuples, arrays, `Box`...).

```rust
struct Connection;                        // no Serialize

// #[derive(Serialize)]
// struct State { c: Connection }         // ERROR: the trait `Serialize` is not implemented for `Connection`
```

`Serialize` alone or `Deserialize` alone is enough depending on the need: an API response only needs `Serialize`, a received payload only `Deserialize`.

---

## 4. JSON with `serde_json`

### 4.1 Serializing (Rust → JSON)

```rust
let u = User {
    id: 1,
    name: "Ada".into(),
    email: "ada@ex.com".into(),
    active: true,
    roles: vec!["admin".into()],
};

let compact = serde_json::to_string(&u)?;          // {"id":1,"name":"Ada",...}
let pretty = serde_json::to_string_pretty(&u)?;    // indented
let bytes: Vec<u8> = serde_json::to_vec(&u)?;
serde_json::to_writer(std::io::stdout(), &u)?;     // straight into a stream, no intermediate String
```

### 4.2 Deserializing (JSON → Rust)

```rust
let text = r#"{"id":2,"name":"Linus","email":"l@ex.com","active":false,"roles":[]}"#;

// 1. Annotate the variable
let u: User = serde_json::from_str(text)?;

// 2. Turbofish
let u = serde_json::from_str::<User>(text)?;

// 3. From bytes or a reader
let u: User = serde_json::from_slice(text.as_bytes())?;
let f = std::fs::File::open("u.json")?;
let u: User = serde_json::from_reader(std::io::BufReader::new(f))?;
```

A missing field, an extra one (depending on the options, section 11), or one of the wrong type produces an error that gives the line and column: `missing field \`email\` at line 1 column 30`.

### 4.3 The dynamic value `serde_json::Value`

```rust
use serde_json::{json, Value};

let v: Value = serde_json::from_str(r#"{"name":"Ada","languages":["rust","c"],"age":36}"#)?;

let name = v["name"].as_str();                   // Some("Ada")
let first = v["languages"][0].as_str();          // Some("rust")
let missing = &v["nonexistent"];                 // Value::Null, no panic
let age = v.get("age").and_then(Value::as_u64);  // Some(36)

let built = json!({
    "status": "ok",
    "total": 3,
    "items": [1, 2, 3],
});
```

| | typed struct | `Value` |
|---|---|---|
| Shape checking | at deserialization | on every access |
| Autocompletion, compile errors | yes | no |
| For | any data whose shape is known | JSON of unknown shape, generic transformation, tests |

### 4.4 Going from one form to the other

```rust
let v: Value = serde_json::to_value(&u)?;          // struct -> Value
let u2: User = serde_json::from_value(v)?;         // Value -> struct
```

---

## 5. TOML and forms

### 5.1 TOML: configuration files

```rust
#[derive(Debug, Deserialize)]
struct Config {
    server: Server,
    database: Database,
}

#[derive(Debug, Deserialize)]
struct Server { host: String, port: u16 }

#[derive(Debug, Deserialize)]
struct Database { url: String, connections: u32 }

let text = r#"
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://localhost/app"
connections = 10
"#;

let c: Config = toml::from_str(text)?;
```

TOML tables (`[server]`) correspond to nested structs. The same struct could be read from a JSON `{"server": {...}, "database": {...}}` without modification.

### 5.2 HTML forms

```rust
#[derive(Deserialize)]
struct Search { q: String, page: Option<u32> }

let r: Search = serde_urlencoded::from_str("q=rust+web&page=2")?;
```

This is the format of HTML forms and URL parameters. axum's `Form<T>` and `Query<T>` extractors use it.

---

## 6. Customizing fields

### 6.1 `rename` and `rename_all`

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]           // all fields: created_at -> createdAt
struct Article {
    id: u32,
    created_at: String,
    #[serde(rename = "articleTitle")]        // one specific field
    title: String,
}
```

Values of `rename_all`: `lowercase`, `UPPERCASE`, `camelCase`, `PascalCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, `SCREAMING-KEBAB-CASE`.

### 6.2 Different names on input and output

```rust
#[derive(Serialize, Deserialize)]
struct Product {
    #[serde(rename(serialize = "price", deserialize = "cost"))]
    cost: f64,
}
```

### 6.3 `alias`: accept several names when reading

```rust
#[derive(Deserialize)]
struct Contact {
    #[serde(alias = "mail", alias = "e-mail")]
    email: String,
}
```

What it is for: renaming a field without breaking old clients or old files.

### 6.4 `skip`: exclude a field

```rust
#[derive(Serialize, Deserialize)]
struct Session {
    user: String,
    #[serde(skip)]                           // neither written nor read; when reading: Default::default()
    cache: Vec<u8>,
    #[serde(skip_serializing)]               // read but never written
    password: String,
    #[serde(skip_deserializing)]             // written but never read
    computed: u32,
}
```

### 6.5 `skip_serializing_if`: omit an empty value

```rust
#[derive(Serialize)]
struct Response {
    data: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,                   // absent from the JSON if None, instead of "error": null
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<String>,
}
```

### 6.6 `default`: a value if the field is missing

```rust
#[derive(Deserialize)]
struct Options {
    #[serde(default)]                        // missing -> Default::default() (0, "", false, empty vec)
    verbose: bool,
    #[serde(default = "default_port")]       // missing -> value from a function
    port: u16,
}

fn default_port() -> u16 { 8080 }

#[derive(Deserialize, Default)]
#[serde(default)]                            // on the struct: any missing field takes the Default value
struct Settings { size: u32, theme: String }
```

### 6.7 `flatten`: merge the fields of another struct

```rust
#[derive(Serialize, Deserialize)]
struct Pagination { page: u32, per_page: u32 }

#[derive(Serialize, Deserialize)]
struct ArticleList {
    articles: Vec<String>,
    #[serde(flatten)]
    pagination: Pagination,                  // {"articles":[..],"page":1,"per_page":20}
}

#[derive(Deserialize)]
struct WithExtras {
    name: String,
    #[serde(flatten)]
    others: std::collections::HashMap<String, serde_json::Value>,   // collects all unknown fields
}
```

---

## 7. Missing values, `null` and `Option`

```rust
#[derive(Deserialize, Debug)]
struct Profile {
    name: String,
    bio: Option<String>,
}
```

| JSON received | `bio` is |
|---|---|
| `{"name":"a","bio":"x"}` | `Some("x")` |
| `{"name":"a","bio":null}` | `None` |
| `{"name":"a"}` | `None` (a missing `Option` is accepted) |

### Telling "missing" from `null`

For a partial update (PATCH), "missing field" (do not touch) and `null` (clear) do not mean the same thing:

```rust
#[derive(Deserialize, Debug)]
struct Update {
    #[serde(default, deserialize_with = "present")]
    bio: Option<Option<String>>,
}

fn present<'de, D>(d: D) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(Option::deserialize(d)?))     // called only if the field is present
}

// {}            -> bio = None          : do not modify
// {"bio":null}  -> bio = Some(None)    : clear
// {"bio":"x"}   -> bio = Some(Some(x)) : replace
```

---

## 8. Serializing enums

One enum, four possible JSON representations.

```rust
#[derive(Serialize, Deserialize)]
enum Event {
    Login { user: String },
    Purchase { amount: u32 },
    Logout,
}
```

### 8.1 Default: external tag

```rust
// {"Login":{"user":"ada"}}
// {"Purchase":{"amount":30}}
// "Logout"
```

### 8.2 `#[serde(tag = "type")]`: internal tag

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Event2 {
    Login { user: String },
    Purchase { amount: u32 },
    Logout,
}
// {"type":"login","user":"ada"}
// {"type":"logout"}
```

The most common form in web APIs. It does not work with tuple variants that contain a plain number or text (`Purchase(u32)`).

### 8.3 `#[serde(tag = "type", content = "data")]`: adjacent tag

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum Message {
    Text(String),
    Number(i64),
}
// {"type":"Text","data":"hello"}
```

Works with all kinds of variants.

### 8.4 `#[serde(untagged)]`: no tag

```rust
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Identifier {
    Number(u64),
    Name(String),
}
// 42      -> Number(42)
// "ada"   -> Name("ada")
```

serde tries each variant **in order** and keeps the first one that matches. Handy for accepting several forms, but the errors are not very informative ("data did not match any variant") and the order matters.

### 8.5 Simple enum as text

```rust
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Status { Draft, Published, Archived }
// "draft", "published", "archived"
```

| Form | JSON | For |
|---|---|---|
| external (default) | `{"Variant": {...}}` | Rust ↔ Rust exchanges |
| internal `tag` | `{"type": "v", ...}` | web APIs |
| adjacent `tag` + `content` | `{"type": "v", "data": ...}` | variants of very different shapes |
| `untagged` | the value alone | accepting several input formats |

---

## 9. Custom conversions

### 9.1 `with`: a conversion module

```rust
mod seconds {
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
    #[serde(with = "seconds")]
    duration: std::time::Duration,        // {"duration": 300}
}
```

### 9.2 `serialize_with` / `deserialize_with`: one direction only

```rust
#[derive(Deserialize)]
struct Signup {
    #[serde(deserialize_with = "lowercase")]
    email: String,
}

fn lowercase<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(String::deserialize(d)?.trim().to_lowercase())
}
```

### 9.3 `try_from`: validate at deserialization

```rust
#[derive(Deserialize, Debug)]
#[serde(try_from = "String")]
struct Email(String);

impl TryFrom<String> for Email {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.contains('@') { Ok(Email(s)) } else { Err(format!("invalid email: {s}")) }
    }
}

#[derive(Deserialize, Debug)]
struct Account { email: Email }

let r: Result<Account, _> = serde_json::from_str(r#"{"email":"not-an-email"}"#);   // Err: invalid email
```

Invalid JSON can then **never** produce an `Email`: the "parse, don't validate" principle (see `structures-enums`) applied to serde. `into = "String"` does the reverse at serialization.

### 9.4 Implementing `Serialize` by hand

```rust
use serde::ser::{Serialize, SerializeStruct, Serializer};

struct Point { x: i32, y: i32 }

impl Serialize for Point {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("Point", 3)?;
        st.serialize_field("x", &self.x)?;
        st.serialize_field("y", &self.y)?;
        st.serialize_field("distance", &((self.x.pow(2) + self.y.pow(2)) as f64).sqrt())?;   // computed field
        st.end()
    }
}
```

Rarely needed: the attributes cover almost every case. Implementing `Deserialize` by hand is much longer (a *visitor*); keep it for truly unusual formats.

### 9.5 Useful crates

- `serde_with`: ready-made conversions (`DisplayFromStr`, durations, base64, empty values as `None`).
- `chrono` and `time` with their `serde` feature: dates in RFC 3339.

---

## 10. Generic types and serde

```rust
#[derive(Serialize, Deserialize, Debug)]
struct Response<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> Response<T> {
    fn ok(d: T) -> Self { Response { success: true, data: Some(d), error: None } }
    fn err(e: impl Into<String>) -> Self { Response { success: false, data: None, error: Some(e.into()) } }
}

let r = Response::ok(vec![1, 2, 3]);
let json = serde_json::to_string(&r)?;     // {"success":true,"data":[1,2,3],"error":null}
```

The derive automatically adds `T: Serialize` (or `T: Deserialize<'de>`) to the implementations. For a parameter that is not serialized itself (a `PhantomData<T>`), `#[serde(bound = "")]` removes this constraint.

### Borrowing instead of copying

```rust
#[derive(Deserialize)]
struct Line<'a> {
    name: &'a str,                                 // points into the JSON text: no allocation
    #[serde(borrow)]
    note: std::borrow::Cow<'a, str>,               // borrows if possible, allocates if the text contains an escape
}

let source = String::from(r#"{"name":"ada","note":"good"}"#);
let l: Line = serde_json::from_str(&source)?;
```

A `&str` only works if the JSON text contains no escape sequence (`\"`, `\n`); `Cow` handles both cases (see `borrow-avance`).

---

## 11. Security: inputs and outputs

### 11.1 Rejecting unknown fields

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangePassword {
    old: String,
    new: String,
}
// {"old":"a","new":"b","role":"admin"} -> Err: unknown field `role`
```

By default, serde **ignores** extra fields. This is rarely a problem (they are stored nowhere), but `deny_unknown_fields` reports a client error, and documents that the payload is closed.

### 11.2 Mass assignment

```rust
// Dangerous: deserializing the full entity directly from the request
// #[derive(Deserialize)] struct User { id: u32, name: String, is_admin: bool }
// let u: User = serde_json::from_str(body)?;    // the client can send "is_admin": true

// Safe: an input struct that contains ONLY what the client is allowed to set
#[derive(Deserialize)]
struct UpdateProfile {
    name: String,
    bio: Option<String>,
}
```

Never deserialize a request directly into a type that contains fields the client must not control (role, identifier, owner, price).

### 11.3 Do not expose everything

```rust
#[derive(Serialize)]
struct PublicUser<'a> {
    id: u32,
    name: &'a str,
    // no password hash, no email, no tokens
}
```

A dedicated **output** struct guarantees that a field added to the entity later (a token, an address) does not automatically appear in the API.

### 11.4 Size and depth

`serde_json` rejects by default JSON nested more than 128 levels deep (protection against stack exhaustion). The **size** of the body must be limited before deserialization, by the server. In Runique, the `Request` extractor rejects a form or JSON body above 2 MB; axum's `Json` extractor follows the global request limit (`RUNIQUE_MAX_BODY_MB`, 100 MB by default). For a public JSON route, a lower limit is set with `DefaultBodyLimit::max(..)` on the route.

---

## 12. Concrete examples with Runique

### 12.1 Entities already derive serde

The `Model` generated by `model!{}` derives `Serialize` and `Deserialize`:

```rust
// code generated by model!{} (runique/derive_form/src/model/utils/sea_model.rs, excerpt)
#[derive(Clone, Debug, PartialEq, ::sea_orm::DeriveEntityModel, ::serde::Serialize, ::serde::Deserialize)]
#[sea_orm(table_name = #table)]
pub struct Model { /* ... */ }
```

Two consequences:

- the generated code writes `::serde::...`: the project must have `serde` in its dependencies (a project created by `runique new` does);
- you cannot add `#[serde(skip)]` or `rename` on a generated field. To control what goes out (section 11.3), use a response struct.

### 12.2 Responding in JSON

```rust
use runique::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct ArticleSummary {
    id: Pk,
    title: String,
}

#[derive(Serialize)]
struct ArticleList {
    articles: Vec<ArticleSummary>,
    total: usize,
}

pub async fn list_articles(request: Request) -> AppResult<Response> {
    let articles = article::Entity::find()
        .filter(article::Column::Published.eq(true))
        .all(request.db())
        .await?;                                          // DbErr -> AppError automatically

    let articles: Vec<ArticleSummary> = articles
        .into_iter()
        .map(|a| ArticleSummary { id: a.id, title: a.title })
        .collect();
    let total = articles.len();

    Ok(Json(ArticleList { articles, total }).into_response())
}
```

Returning `Vec<article::Model>` directly would also compile (the `Model` is serializable), but it would expose every column, including those added later.

### 12.3 Receiving structured JSON

Runique's `Request` extractor reads a JSON body as a flat table of strings (`StrMap`), for forms. For structured JSON (lists, booleans, nested objects), use axum's `Json` extractor instead of `Request`: both read the body, and a handler can only take one. The database stays reachable through the engine, which Runique places in the extensions of each request.

The CSRF middleware still applies: a JSON POST must carry the `X-CSRF-Token` header, with the token the template receives in `{{ csrf_token }}` (for example placed in a `<meta name="csrf-token">` tag and read back by JavaScript), otherwise it is rejected with 403 before reaching the handler.

```rust
use runique::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateArticle {
    title: String,
    content: String,
    #[serde(default)]
    published: bool,
    #[serde(default)]
    tags: Vec<String>,
}

pub async fn create_article(
    Extension(engine): Extension<AEngine>,
    Json(payload): Json<CreateArticle>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if payload.title.trim().is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let new_article = article::ActiveModel {
        title: Set(payload.title.trim().to_string()),
        content: Set(payload.content),
        published: Set(payload.published),
        ..Default::default()
    };
    let a = new_article
        .insert(&engine.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;   // the database detail is not returned
    Ok(Json(serde_json::json!({ "id": a.id })))
}
```

`CreateArticle` only contains the fields the client is allowed to set: no identifier, no author, no date (section 11.2). An unknown field is rejected.

---

## 13. The same conversion written in several ways

Accept a date in the format `"2026-11-09"` in a JSON.

```rust
use serde::{Deserialize, Deserializer};

// 1. A String, converted later by hand: simple, but validation is postponed
#[derive(Deserialize)]
struct V1 { date: String }

// 2. chrono with its serde feature: NaiveDate reads "YYYY-MM-DD" directly
#[derive(Deserialize)]
struct V2 { date: chrono::NaiveDate }

// 3. deserialize_with: a custom format ("09/11/2026")
#[derive(Deserialize)]
struct V3 {
    #[serde(deserialize_with = "date_dmy")]
    date: chrono::NaiveDate,
}

fn date_dmy<'de, D: Deserializer<'de>>(d: D) -> Result<chrono::NaiveDate, D::Error> {
    let s = String::deserialize(d)?;
    chrono::NaiveDate::parse_from_str(&s, "%d/%m/%Y").map_err(serde::de::Error::custom)
}

// 4. A newtype with try_from: reusable everywhere
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct DateDmy(chrono::NaiveDate);

impl TryFrom<String> for DateDmy {
    type Error = String;
    fn try_from(s: String) -> Result<Self, String> {
        chrono::NaiveDate::parse_from_str(&s, "%d/%m/%Y")
            .map(DateDmy)
            .map_err(|e| format!("invalid date \"{s}\": {e}"))
    }
}
```

| Version | Validation | Reusable | For |
|---|---|---|---|
| 1 | later, by hand | no | prototype |
| 2 | on reading | yes | standard format |
| 3 | on reading | per field | a particular format, in one place |
| 4 | on reading | yes, everywhere | a particular format, used often |

`serde::de::Error::custom` turns any displayable error into a serde error, with the position in the JSON.

---

## 14. Practical exercises

### Exercise 1: configuration struct

Read this TOML into a struct, with a default port of 8080 and a debug mode that is false by default.

```toml
[server]
host = "127.0.0.1"
```

```rust
// Solution:
#[derive(Deserialize, Debug)]
struct File { server: Server }

#[derive(Deserialize, Debug)]
struct Server {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default)]
    debug: bool,
}

fn default_port() -> u16 { 8080 }
```

### Exercise 2: status enum

Make `Status::Pending` be written `"pending"` and an unknown status be rejected. Use a multi-word variant to show the renaming.

```rust
// Solution:
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Status { Pending, InReview, Rejected }

assert_eq!(serde_json::to_string(&Status::InReview)?, r#""in_review""#);
assert!(serde_json::from_str::<Status>(r#""unknown""#).is_err());
```

### Exercise 3: generic pagination

Write `Page<T>` that produces `{"items":[...],"page":1,"total":42}` and omits `"next"` when there is no next page.

```rust
// Solution:
#[derive(Serialize)]
struct Page<T> {
    items: Vec<T>,
    page: u32,
    total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    next: Option<u32>,
}
```

### Exercise 4: alias and renaming

An old API sends `"mail"`, the new one `"email"`. Accept both, always return `"email"`, and lowercase the address.

```rust
// Solution:
#[derive(Serialize, Deserialize)]
struct Contact {
    #[serde(alias = "mail", deserialize_with = "lowercase")]
    email: String,
}
```

---

## 15. Cheat sheet

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]        #[serde(deny_unknown_fields)]
#[serde(tag = "type")]   #[serde(tag = "t", content = "c")]   #[serde(untagged)]
#[serde(try_from = "String")]   #[serde(into = "String")]   #[serde(default)]

#[serde(rename = "x")]   #[serde(alias = "y")]   #[serde(skip)]
#[serde(skip_serializing_if = "Option::is_none")]   #[serde(default = "function")]
#[serde(flatten)]   #[serde(with = "module")]   #[serde(deserialize_with = "f")]   #[serde(borrow)]

serde_json::to_string(&v)?   to_string_pretty   to_vec   to_writer   to_value
serde_json::from_str::<T>(s)?   from_slice   from_reader   from_value
json!({ "a": 1 })   v["key"].as_str()   v.get("key")
toml::from_str::<T>(s)?   serde_urlencoded::from_str::<T>(s)?
```

### Key points

- serde separates the data (derive) from the format (`serde_json`, `toml` crates...).
- Every field must be serializable; third-party crates often have a `serde` feature.
- `Option` accepts a missing field; `default` gives a value when it is missing.
- For web APIs, `#[serde(tag = "type")]` is the most readable enum representation.
- `try_from` validates on reading: invalid JSON never produces an invalid value.
- An input struct only contains what the client can set; an output struct only what it can see.
- In Runique, `Model`s already derive serde: go through a response struct to choose the exposed fields.
