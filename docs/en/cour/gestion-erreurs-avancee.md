# Advanced error handling — thiserror and anyhow

> Typed errors for libraries, errors with context for applications, and their translation into HTTP responses in Runique.

## Objectives

- Know why hand-written errors become heavy
- Generate `Display`, `Error`, `source` and `From` with `thiserror`
- Use `anyhow`: `Result`, `anyhow!`, `bail!`, `ensure!`, `context`, `downcast`
- Choose between `thiserror` and `anyhow`, and combine them
- Organize errors by layer (data, business, HTTP)
- Turn your errors into Runique responses (`AppResult`, `AppError`, `ErrorContext`)
- Never expose an internal detail in a message meant for the visitor

Prerequisite: the `gestion-des-erreurs` course (`Result`, `?`, `From`, the `Error` trait).

---

## Table of contents

1. The limits of simple solutions
2. `thiserror`: declaring an error
3. The `thiserror` attributes
4. `anyhow`: one error for the whole application
5. Context with `anyhow`
6. Reading an `anyhow` error
7. `thiserror` or `anyhow`
8. Errors by layer
9. Errors in Runique handlers
10. Security of error messages
11. The same function written several ways
12. Exercises
13. Cheat sheet

---

## 1. The limits of simple solutions

### A `String` as an error

```rust
fn find(id: u32) -> Result<String, String> {
    if id == 0 {
        return Err("not found".to_string());
    }
    Ok("Ada".into())
}

// The caller has to compare texts to react:
match find(0) {
    Err(e) if e == "not found" => { /* fragile: a typo breaks the test */ }
    _ => {}
}
```

### `Box<dyn Error>`

Convenient, but the caller no longer sees **which** errors are possible: they have to try `downcast_ref` at random.

### The manual implementation

```rust
#[derive(Debug)]
enum RepoError {
    Database(sea_orm::DbErr),
    NotFound(i32),
}

impl std::fmt::Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(_) => write!(f, "database error"),
            Self::NotFound(id) => write!(f, "item {id} not found"),
        }
    }
}

impl std::error::Error for RepoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(e) => Some(e),
            Self::NotFound(_) => None,
        }
    }
}

impl From<sea_orm::DbErr> for RepoError {
    fn from(e: sea_orm::DbErr) -> Self { Self::Database(e) }
}
```

About thirty lines for two variants, to maintain by hand with every addition. This is what `thiserror` generates.

---

## 2. `thiserror`: declaring an error

```toml
[dependencies]
thiserror = "2"
```

```rust
use thiserror::Error;

#[derive(Debug, Error)]
enum RepoError {
    #[error("database error")]
    Database(#[from] sea_orm::DbErr),

    #[error("item {0} not found")]
    NotFound(i32),
}
```

The same code as in section 1, in six lines. `thiserror` only generates code at compile time: it does not appear in the public API, and a library can use it without imposing it on its users.

---

## 3. The `thiserror` attributes

### 3.1 — `#[error("...")]`: the message

```rust
#[derive(Debug, thiserror::Error)]
enum FormError {
    #[error("the field is required")]                            // fixed text
    Empty,

    #[error("value {0} out of range")]                           // tuple field by position
    OutOfRange(i64),

    #[error("{field}: {length} characters, maximum {max}")]      // named fields
    TooLong { field: String, length: usize, max: usize },

    #[error("invalid value: {0:?}")]                             // with Debug
    Invalid(String),

    #[error("{}", .0.to_uppercase())]                            // expression on a field
    Raw(String),
}
```

You can also put `#[error]` on a **struct**:

```rust
#[derive(Debug, thiserror::Error)]
#[error("quota exceeded: {used}/{limit}")]
struct Quota { used: u32, limit: u32 }
```

### 3.2 — `#[from]`: automatic conversion for `?`

```rust
#[derive(Debug, thiserror::Error)]
enum ImportError {
    #[error("cannot read the file")]
    Read(#[from] std::io::Error),

    #[error("malformed line")]
    Format(#[from] std::num::ParseIntError),
}

fn import(path: &str) -> Result<Vec<i64>, ImportError> {
    let text = std::fs::read_to_string(path)?;          // io::Error -> Read
    let v = text.lines().map(|l| l.parse::<i64>()).collect::<Result<_, _>>()?;   // -> Format
    Ok(v)
}
```

`#[from]` generates `impl From<std::io::Error> for ImportError` **and** declares the field as the `source`. Only one variant per source type: two `#[from] std::io::Error` variants would be ambiguous.

### 3.3 — `#[source]`: keeping the cause without automatic conversion

```rust
#[derive(Debug, thiserror::Error)]
enum ConfigError {
    #[error("reading {path}")]
    Read {
        path: String,
        #[source]
        cause: std::io::Error,
    },
}

fn read(path: &str) -> Result<String, ConfigError> {
    std::fs::read_to_string(path).map_err(|cause| ConfigError::Read {
        path: path.to_string(),
        cause,
    })
}
```

What it is for: when the variant needs **other** data than the cause (here the path), `#[from]` cannot guess it. A field named `source` is taken as the cause automatically, without an attribute.

### 3.4 — `#[error(transparent)]`: delegating entirely

```rust
#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("resource not found")]
    NotFound,

    #[error(transparent)]
    Other(#[from] anyhow::Error),      // Display and source are those of the wrapped error
}
```

What it is for: a "everything else" variant that adds no message, or an opaque public type that hides the internal error:

```rust
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct MyError(#[from] InternalError);    // the internal enum can change without breaking the API

#[derive(Debug, thiserror::Error)]
enum InternalError {
    #[error("x")]
    X,
}
```

### 3.5 — Summary

| Attribute | Effect |
|---|---|
| `#[error("...")]` | generates `Display` |
| `#[from]` on a field | generates `From` + marks the cause |
| `#[source]` on a field | marks the cause, without `From` |
| field named `source` | marks the cause automatically |
| `#[error(transparent)]` | `Display` and `source` delegated to the single field |

---

## 4. `anyhow`: one error for the whole application

```toml
[dependencies]
anyhow = "1"
```

### 4.1 — `anyhow::Result`

```rust
use anyhow::Result;            // Result<T, anyhow::Error>

fn load_port() -> Result<u16> {
    let t = std::fs::read_to_string("port.txt")?;    // any Error + Send + Sync error
    let p = t.trim().parse::<u16>()?;
    Ok(p)
}

fn main() -> Result<()> {
    let port = load_port()?;
    println!("{port}");
    Ok(())
}
```

`anyhow::Error` accepts any error that implements `std::error::Error + Send + Sync + 'static`. It keeps the original cause, a call stack if `RUST_BACKTRACE=1`, and the context added along the way.

### 4.2 — Creating a one-off error: three macros

```rust
use anyhow::{anyhow, bail, ensure, Result};

fn check_age(age: i32) -> Result<u32> {
    // anyhow!: build an error
    if age < 0 {
        return Err(anyhow!("negative age: {age}"));
    }

    // bail!: build AND return (same as return Err(anyhow!(..)))
    if age > 150 {
        bail!("unlikely age: {age}");
    }

    // ensure!: return an error if the condition is false
    ensure!(age >= 18, "must be an adult (age: {age})");

    Ok(age as u32)
}
```

The three forms give the same error; `bail!` and `ensure!` are shorter for checks at the start of a function.

---

## 5. Context with `anyhow`

### 5.1 — `context`: a fixed message

```rust
use anyhow::{Context, Result};

fn read_config() -> Result<String> {
    std::fs::read_to_string("config.toml").context("reading the configuration")
}
// Display: reading the configuration
// Cause: No such file or directory (os error 2)
```

The `Context` trait adds `.context()` and `.with_context()` to any `Result` whose error implements `Error`, and also to `Option` (a `None` becomes an error).

### 5.2 — `with_context`: a message computed only on error

```rust
fn read(path: &str) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("reading {path}"))
}
```

`context(format!(...))` would build the `String` **on every call**, even on success; `with_context` only calls it on error.

### 5.3 — On an `Option`

```rust
use std::collections::HashMap;

fn port(env: &HashMap<String, String>) -> Result<u16> {
    let text = env.get("PORT").context("PORT variable missing")?;   // None -> error
    text.parse().with_context(|| format!("invalid PORT: {text}"))
}
```

### 5.4 — Stacking context at each layer

```rust
fn read_users() -> Result<Vec<String>> {
    let t = std::fs::read_to_string("users.csv").context("opening users.csv")?;
    Ok(t.lines().map(String::from).collect())
}

fn start() -> Result<()> {
    let u = read_users().context("loading accounts at startup")?;
    println!("{} accounts", u.len());
    Ok(())
}
// Error: loading accounts at startup
//
// Caused by:
//     0: opening users.csv
//     1: No such file or directory (os error 2)
```

Each layer says **what it was trying to do**; the original cause stays at the end of the chain.

---

## 6. Reading an `anyhow` error

### 6.1 — The display formats

```rust
fn show(e: &anyhow::Error) {
    println!("{e}");        // only the last context
    println!("{e:#}");      // the whole chain on one line: a: b: c
    println!("{e:?}");      // full chain on several lines + backtrace if enabled
}
```

Use `{e:#}` in one-line logs, `{e:?}` for the terminal or a bug report.

### 6.2 — Walking the chain

```rust
fn causes(e: &anyhow::Error) {
    for (i, cause) in e.chain().enumerate() {
        println!("{i}: {cause}");
    }
    let root = e.root_cause();
}
```

### 6.3 — Getting the original type back

```rust
fn react(e: &anyhow::Error) {
    if let Some(io) = e.downcast_ref::<std::io::Error>() {
        if io.kind() == std::io::ErrorKind::NotFound {
            println!("file missing");
        }
    }
}
```

`downcast_ref` looks for the type in the **whole chain**, context included. If you often need to `downcast`, it is a sign that a typed error (`thiserror`) would fit better at that spot.

---

## 7. `thiserror` or `anyhow`

| | `thiserror` | `anyhow` |
|---|---|---|
| Nature | generates an error type of your own | a single, opaque error type |
| The caller can `match` | yes | no (only `downcast`) |
| Context | to plan in the variants | `.context()` everywhere |
| For | libraries, layers where the caller must react to the cases | applications, binaries, scripts, layers where the error will only be displayed |
| In a public API | yes | avoid: the user loses the type information |

The usual rule: **`thiserror` in libraries, `anyhow` in applications.** The two combine:

```rust
// A library exposes typed errors
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("card declined")]
    Declined,
    #[error("service unavailable")]
    Unavailable(#[source] std::io::Error),
}

// The application receives them with anyhow: the conversion is automatic
fn pay() -> anyhow::Result<()> {
    charge().context("payment of order 42")?;
    Ok(())
}

fn charge() -> Result<(), PaymentError> { Err(PaymentError::Declined) }

// ... and can still react to the specific case
fn handle() {
    if let Err(e) = pay() {
        if let Some(PaymentError::Declined) = e.downcast_ref::<PaymentError>() {
            println!("offer another payment method");
        }
    }
}
```

---

## 8. Errors by layer

Each layer has its own vocabulary: the data layer talks about databases and rows, the business layer about rules, the HTTP layer about status codes.

### 8.1 — Data layer

```rust
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("article {0} not found")]
    NotFound(i32),
    #[error("database error")]
    Database(#[from] sea_orm::DbErr),
}
```

### 8.2 — Business layer

```rust
#[derive(Debug, thiserror::Error)]
pub enum ArticleError {
    #[error("empty title")]
    EmptyTitle,
    #[error("you are not the author of this article")]
    NotAuthor,
    #[error(transparent)]
    Repo(#[from] RepoError),
}

pub async fn publish(id: i32, author: i32) -> Result<(), ArticleError> {
    let article = load(id).await?;            // RepoError -> ArticleError::Repo
    if article.author != author {
        return Err(ArticleError::NotAuthor);
    }
    Ok(())
}

struct Article { author: i32 }
async fn load(id: i32) -> Result<Article, RepoError> { Err(RepoError::NotFound(id)) }
```

### 8.3 — HTTP layer: translating to a status

```rust
fn status(e: &ArticleError) -> u16 {
    match e {
        ArticleError::EmptyTitle => 400,
        ArticleError::NotAuthor => 403,
        ArticleError::Repo(RepoError::NotFound(_)) => 404,
        ArticleError::Repo(RepoError::Database(_)) => 500,
    }
}
```

The `match` is exhaustive: adding a variant forces you to decide its HTTP code.

---

## 9. Errors in Runique handlers

### 9.1 — `AppResult` and `AppError`

A Runique handler returns `AppResult<Response>`, an alias of `Result<Response, Box<AppError>>`. `AppError` is not an enum: it is a struct that carries an `ErrorContext` (HTTP code, title, message, information for the debug page).

```rust
// runique/src/context/template.rs
pub struct AppError {
    pub context: ErrorContext,
}

// runique/src/utils/aliases/definition.rs
pub type AppResult<T> = Result<T, Box<AppError>>;
```

`AppError` implements `IntoResponse`: returning `Err(...)` from a handler produces the error page matching the status (404, 500...), or the debug page in development.

### 9.2 — The conversions already provided

`DbErr`, `anyhow::Error` and session errors go through `?` without writing anything:

```rust
// runique/src/context/template.rs
impl_from_error!(
    anyhow::Error => from_anyhow,
    DbErr => database,
    tower_sessions::session::Error => database
);
```

`impl_from_error!(Type => method, ...)` generates `From<Type> for AppError` and `From<Type> for Box<AppError>`, by calling `ErrorContext::method(&error)`. These errors become 500s: the detail is logged, the visitor sees a generic page.

### 9.3 — Choosing the status yourself

```rust
use runique::prelude::*;

pub async fn article_detail(request: Request) -> AppResult<Response> {
    let id = request.get_path_as::<Pk>("id").unwrap_or_default();
    let Some(article) = article::Entity::find_by_id(id)
        .one(request.db())
        .await?                                      // DbErr -> 500 automatically
    else {
        return Err(Box::new(AppError::new(ErrorContext::not_found(&format!("/articles/{id}")))));
    };
    // ...
}
```

Constructors available on `ErrorContext`: `not_found(path)`, `bad_request(message)`, `database(error)`, `from_anyhow(&error)`, `generic(StatusCode, message)`.

### 9.4 — Shortcut: `get_or_404`

For the most common access (one row by its key, 404 if it does not exist), the `objects` manager of an entity provides `get_or_404(db, id, &request, message)`, which returns the row or directly the error response (404 if absent, 500 if the query fails).

### 9.5 — Plugging your own error into `?`

```rust
use runique::prelude::*;

impl From<ArticleError> for Box<AppError> {
    fn from(e: ArticleError) -> Self {
        let context = match &e {
            ArticleError::EmptyTitle => ErrorContext::bad_request("The title is required."),
            ArticleError::NotAuthor => {
                ErrorContext::generic(StatusCode::FORBIDDEN, "Action reserved for the author.")
            }
            ArticleError::Repo(RepoError::NotFound(id)) => {
                ErrorContext::not_found(&format!("/articles/{id}"))
            }
            ArticleError::Repo(RepoError::Database(db)) => ErrorContext::database(db),
        };
        Box::new(AppError::new(context))
    }
}

pub async fn publish_article(request: Request) -> AppResult<Response> {
    let id = request.get_path_as::<i32>("id").unwrap_or_default();
    publish(id, 1).await?;              // ArticleError -> Box<AppError> thanks to the From
    // ...
}
```

`From<ArticleError> for Box<AppError>` is allowed in your crate because `ArticleError` belongs to you (orphan rule, see `traits-avances`).

### 9.6 — `anyhow` in services

```rust
use anyhow::Context;

pub async fn create_article(
    title: String,
    content: String,
    db: &DatabaseConnection,
) -> anyhow::Result<article::Model> {
    let active = article::ActiveModel {
        title: Set(title.trim().to_string()),
        content: Set(content),
        ..Default::default()
    };
    active
        .insert(db)
        .await
        .with_context(|| format!("inserting the article \"{}\"", title.trim()))
}
```

The handler calls `create_article(...).await?`: the `anyhow::Error` becomes a 500 thanks to the conversions of section 9.2, with the whole context chain in the logs.

### 9.7 — Form error: not an `AppError`

Invalid input is not an HTTP error: you redisplay the form with the messages under the fields. Runique forms (`ValidationForm::try_form`, `is_valid`) handle this case; `AppError` is reserved for what prevents producing the page.

---

## 10. Security of error messages

| Where | What goes there |
|---|---|
| Message shown to the visitor | a generic sentence, useful to them: "Article not found", "Action reserved for the author" |
| Server logs | everything: cause chain, query, identifiers, call stack |
| Debug page | reserved for development (`DEBUG=true`), never in production |

Never return to the visitor: a server file path, an SQL query, a table name, a raw database message, a token, a password, the email address of another user.

Also beware of messages that **reveal** information: "wrong password" versus "unknown user" tells you which accounts exist. For a login, use a single message for both cases.

```rust
#[derive(Debug, thiserror::Error)]
enum LoginError {
    #[error("invalid credentials")]     // same message for both cases
    UnknownUser,
    #[error("invalid credentials")]
    WrongPassword,
}
// The two variants stay distinct for logs and tests.
```

---

## 11. The same function written several ways

Loading a JSON settings file.

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Settings { port: u16 }

// 1. Box<dyn Error>: simple, no context
fn load_v1(path: &str) -> Result<Settings, Box<dyn std::error::Error>> {
    let t = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&t)?)
}

// 2. anyhow + context: for an application
fn load_v2(path: &str) -> anyhow::Result<Settings> {
    use anyhow::Context;
    let t = std::fs::read_to_string(path).with_context(|| format!("reading {path}"))?;
    serde_json::from_str(&t).with_context(|| format!("format of {path}"))
}

// 3. thiserror: for a library, the caller tells the cases apart
#[derive(Debug, thiserror::Error)]
enum SettingsError {
    #[error("reading {path}")]
    Read { path: String, #[source] cause: std::io::Error },
    #[error("format of {path}")]
    Format { path: String, #[source] cause: serde_json::Error },
}

fn load_v3(path: &str) -> Result<Settings, SettingsError> {
    let t = std::fs::read_to_string(path).map_err(|cause| SettingsError::Read {
        path: path.into(),
        cause,
    })?;
    serde_json::from_str(&t).map_err(|cause| SettingsError::Format {
        path: path.into(),
        cause,
    })
}
```

---

## 12. Exercises

### Exercise 1 — error type with `thiserror`

Create `SignupError` with `InvalidEmail(String)`, `PasswordTooShort { min: usize }` and `Database(#[from] sea_orm::DbErr)`.

```rust
// Solution:
#[derive(Debug, thiserror::Error)]
enum SignupError {
    #[error("invalid email address: {0}")]
    InvalidEmail(String),
    #[error("password too short (minimum {min} characters)")]
    PasswordTooShort { min: usize },
    #[error("database error")]
    Database(#[from] sea_orm::DbErr),
}
```

### Exercise 2 — context

Rewrite this function so that the error says which file and which line are the problem.

```rust
fn total(path: &str) -> anyhow::Result<i64> {
    let t = std::fs::read_to_string(path)?;
    let mut s = 0;
    for l in t.lines() {
        s += l.parse::<i64>()?;
    }
    Ok(s)
}
```

```rust
// Solution:
use anyhow::Context;

fn total(path: &str) -> anyhow::Result<i64> {
    let t = std::fs::read_to_string(path).with_context(|| format!("reading {path}"))?;
    let mut s = 0;
    for (i, l) in t.lines().enumerate() {
        s += l.trim().parse::<i64>().with_context(|| format!("{path}, line {}: \"{l}\"", i + 1))?;
    }
    Ok(s)
}
```

### Exercise 3 — towards a Runique response

Write `From<SignupError> for Box<AppError>`: 400 for the two input errors, 500 for the database.

```rust
// Solution:
use runique::prelude::*;

impl From<SignupError> for Box<AppError> {
    fn from(e: SignupError) -> Self {
        let c = match &e {
            SignupError::Database(db) => ErrorContext::database(db),
            other => ErrorContext::bad_request(&other.to_string()),
        };
        Box::new(AppError::new(c))
    }
}
```

`to_string()` is safe here: the messages of the two input variants contain nothing internal. For `Database`, the database message is never shown.

---

## 13. Cheat sheet

```rust
// thiserror
#[derive(Debug, thiserror::Error)]
enum E {
    #[error("text {0}")]             A(i32),
    #[error("{field}")]              B { field: String },
    #[error("cause")]                C(#[from] std::io::Error),
    #[error("x")]                    D { #[source] cause: std::io::Error },
    #[error(transparent)]            F(#[from] anyhow::Error),
}

// anyhow
use anyhow::{anyhow, bail, ensure, Context, Result};
fn f() -> Result<()> {
    let v = op().context("step")?;
    let w = op().with_context(|| format!("step {v}"))?;
    let x = option.context("missing")?;
    ensure!(cond, "message {v}");
    bail!("message");
}
format!("{e}")  format!("{e:#}")  format!("{e:?}")  e.chain()  e.downcast_ref::<T>()

// Runique
AppResult<Response>                       // Result<Response, Box<AppError>>
Err(Box::new(AppError::new(ErrorContext::not_found(path))))
ErrorContext::bad_request(msg)   ErrorContext::generic(StatusCode::FORBIDDEN, msg)
impl From<MyError> for Box<AppError> { .. }
```

### Key points

- `thiserror` generates the code of a typed error; `anyhow` provides a single error with context.
- Library: `thiserror`. Application: `anyhow`. The two combine.
- `#[from]` to convert, `#[source]` to keep a cause along with other data, `transparent` to delegate.
- `with_context` rather than `context(format!(..))`.
- Each layer translates errors into its own vocabulary; the HTTP layer chooses the status.
- In Runique, `DbErr` and `anyhow::Error` become 500s through `?`; a `From<MyError> for Box<AppError>` chooses the status of your own errors.
- The visitor sees a generic message, the logs see everything.
