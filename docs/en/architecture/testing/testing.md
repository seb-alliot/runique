# Testing a Runique Application

Runique provides a set of specialized test helpers to write expressive integration tests for your Axum-based application.

---

## 1. Test Setup

To avoid code duplication, Runique encourages using a shared server builder that starts a real instance of your application on a random free port.

```rust
use mon_projet::helpers::server::test_server_addr;

#[tokio::test]
async fn test_homepage_is_ok() {
    // 1. Get the shared server address (starts it on the first call)
    let addr = test_server_addr();
    
    // 2. Use reqwest (or any client) to call it
    let client = reqwest::Client::new();
    let resp = client.get(format!("http://{}/", addr)).await.unwrap();
    
    assert_eq!(resp.status(), 200);
}
```

---

## 2. Using Oneshot Requests

For faster tests that don't require a real TCP server, you can use the `oneshot` builders. These call the Axum router directly in-memory.

```rust
use crate::helpers::{request, server::build_engine, server::build_default_router};

#[tokio::test]
async fn test_ping() {
    // Build the engine and router
    let engine = build_engine().await;
    let app = build_default_router(engine);
    
    // Standard GET
    let resp = request::get(app.clone(), "/").await;
    assert_eq!(resp.status(), 200);
    
    // POST with CSRF header
    let resp = request::post_with_header(app, "/submit", "x-csrf-token", "my-token").await;
    assert_eq!(resp.status(), 200);
}
```

---

## 3. Specialized Assertions

The `assert` helper module provides readable macros to inspect HTTP responses:

| Helper | Usage |
| --- | --- |
| `assert_status(&resp, code)` | Checks the HTTP status code |
| `assert_is_redirect(&resp)` | Checks for any 3xx status |
| `assert_redirect(&resp, "/dest")` | Checks for 3xx AND the exact location |
| `assert_has_header(&resp, "key")` | Checks if a header exists |
| `assert_body_str(resp, "text").await` | Checks if the body contains exactly "text" |

```rust
use crate::helpers::assert::{assert_status, assert_redirect, assert_body_str};

#[tokio::test]
async fn test_login_flow() {
    let app = my_app_router();
    let resp = request::get(app, "/protected").await;
    
    // Check for redirection to login
    assert_redirect(&resp, "/login");
}
```

---

## 4. Database Testing (SQLite isolated)

Runique provides `fresh_db()` which returns a new, isolated SQLite in-memory database for every test.

```rust
use crate::helpers::db;

#[tokio::test]
async fn test_database_persistence() {
    // Get a fresh, isolated DB
    let db = db::fresh_db().await;
    
    // Apply schema and run queries
    db::exec(&db, "CREATE TABLE demo (id INTEGER PRIMARY KEY, val TEXT)").await;
    db::exec(&db, "INSERT INTO demo (val) VALUES ('runique')").await;
    
    // Easy counting
    db::assert_count(&db, "demo", 1).await;
}
```

---

## 5. Health Checks during build

When you build your application using `RuniqueApp::builder(config).build().await`, Runique performs a suite of health checks:

- **Database connectivity**: is the DB reachable?
- **Templates**: are all `.html` files valid Tera syntax?
- **Security**: is the `SECRET_KEY` safe (not default in prod)?
- **Middleware integrity**: are dependencies between middlewares satisfied?

If any check fails, the `build()` method returns a `BuildError::CheckFailed(CheckReport)` which displays a clear diagnostic in the terminal with suggestions.

---

← [**Examples**](/docs/en/exemple) | [**Troubleshooting**](/docs/en/installation/troubleshooting) →
