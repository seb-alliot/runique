# Middlewares: Pitfalls and Solutions
> Real lessons from building Runique: layer order, the request body you can only read once, forms, flash messages, and what high-level frameworks do not tell you.

## Objectives

- Understand Tower's onion model and why layer order is critical
- Know the different ways to stack layers with axum, and what they mean
- Understand how Runique enforces order with a slot system
- Understand why the HTTP body can only be read once, and how Runique relies on it
- Tell `layer` and `route_layer` apart, and know where to put a middleware specific to some routes
- Write a middleware with `from_fn`
- Choose between a flash message and a message shown right away

## Table of contents

1. The onion model
2. Layer order with axum: the different syntaxes
3. The Runique solution: slots
4. Session before CSRF: the silent bug
5. Writing a middleware
6. The HTTP body can only be read once
7. Forms: display on GET, process on POST
8. Global middlewares and route middlewares
9. Fake route and redirect to the admin login
10. Flash messages: redirect or direct render
11. Exercises
12. Cheat sheet

---

## 1. The onion model

A middleware (a "layer" in Tower) **wraps** the next service. The request goes through the layers from the outside to the inside, and the response goes back through them in the other direction.

```
request  ──►  ┌ ErrorHandler ┌ Session ┌ CSRF ┌ handler ┐ CSRF ┐ Session ┐ ErrorHandler ┐  ──► response
              └──────────────┴─────────┴──────┴─────────┘
              outside                                              inside
```

A layer can:

- act **before**: read headers, reject the request (403, 413, 429), add data;
- act **after**: modify the response (security headers, compression), catch an error;
- or both (measure the duration, open then save a session).

Consequence: a layer can only use what **more outer** layers have prepared. CSRF reads the session: the session must be **outside** CSRF.

---

## 2. Layer order with axum: the different syntaxes

### 2.1 Successive `.layer()` calls: the last one is the outermost

```rust
use axum::Router;

let app = Router::new()
    .route("/", axum::routing::get(handler))
    .layer(csrf_layer)           // added first: the INNERMOST
    .layer(session_layer)        // wraps the previous one
    .layer(error_layer);         // added last: the OUTERMOST

// Request: errors -> session -> csrf -> handler
```

Each `.layer()` wraps **everything built before it**. The stack is therefore read from bottom to top. This is the classic pitfall: reading from top to bottom, you think CSRF runs first.

### 2.2 `ServiceBuilder`: from top to bottom

```rust
use tower::ServiceBuilder;

let app = Router::new()
    .route("/", axum::routing::get(handler))
    .layer(
        ServiceBuilder::new()
            .layer(error_layer)      // the outermost
            .layer(session_layer)
            .layer(csrf_layer),      // the innermost
    );

// Request: errors -> session -> csrf -> handler
```

The same execution order, written in reading order. Both forms are correct; mixing the two in one project is the best way to make a mistake.

| Syntax | Read | First layer crossed |
|---|---|---|
| `.layer(a).layer(b).layer(c)` | from bottom to top | `c` |
| `ServiceBuilder::new().layer(a).layer(b).layer(c)` | from top to bottom | `a` |

### 2.3 The real bug

This pitfall caused a bug while building Runique: CSRF ended up **outside** the session. It looked for a token in a session that was not loaded yet, and rejected every POST request with a 403, without a clear message.

> The bug is not always visible right away: if CSRF is disabled or bypassed in development, it can reach production without ever having been visible.

---

## 3. The Runique solution: slots

Rather than depend on writing order, Runique gives each middleware a **fixed slot number**. At build time, the layers are sorted by slot and applied in the right order, whatever order the developer configures them in.

```rust
// runique/src/app/staging/middleware_staging/applicator.rs (extract)
const SLOT_EXTENSIONS: u16 = 0;        // Engine/Tera/Config injection (outermost)
const SLOT_TRUSTED_PROXIES: u16 = 2;   // Real client IP extraction — before everything
const SLOT_COMPRESSION: u16 = 5;
const SLOT_CORS: u16 = 8;              // Outside ErrorHandler — OPTIONS preflight never reaches CSRF
const SLOT_ERROR_HANDLER: u16 = 10;    // Catches errors of the WHOLE stack
const SLOT_HOST_VALIDATION: u16 = 15;  // After ErrorHandler, before Session/Auth/CSRF cost
const SLOT_HTTPS_REDIRECT: u16 = 17;   // After HostValidation: the redirect is built from a checked Host
const SLOT_CUSTOM_BASE: u16 = 20;      // Dev's custom middlewares start here
const SLOT_OPEN_REDIRECT: u16 = 25;
const SLOT_SECURITY_HEADERS: u16 = 30;
const SLOT_CACHE: u16 = 40;
const SLOT_SESSION: u16 = 50;          // Before CSRF (CSRF depends on it)
const SLOT_SESSION_UPGRADE: u16 = 55;
const SLOT_AUTH: u16 = 57;             // After Session — reads the signed-in account from the DB
const SLOT_CSRF: u16 = 60;             // After Session (reads/writes in session)
const SLOT_ANTI_BOT: u16 = 65;         // After CSRF — injects honeypot field name extension
```

Reading: a **low** slot is **outer**. The request goes through in increasing order:

```
Extensions(0) → TrustedProxies(2) → Compression(5) → CORS(8) → ErrorHandler(10)
→ HostValidation(15) → HttpsRedirect(17) → Custom(20+) → OpenRedirect(25)
→ CSP/Headers(30) → Cache(40) → Session(50) → SessionUpgrade(55) → Auth(57)
→ CSRF(60) → AntiBot(65) → handler
```

Each position has a reason, written in the comment:

| Layer | Why at this position |
|---|---|
| TrustedProxies (2) | the real client address must be known before anything that reads it (limiters, logs) |
| CORS (8) | the browser's `OPTIONS` preflight request must never reach CSRF |
| ErrorHandler (10) | catches the errors (and panics) of the whole inner stack |
| HostValidation (15) | reject an unknown `Host` **before** paying for the session, authentication and CSRF |
| HttpsRedirect (17) | the redirect is built from an already checked `Host`: no open redirect |
| Session (50) before Auth (57) and CSRF (60) | both read the session |

```rust
// The writing order does not matter
RuniqueAppBuilder::new(config)
    .middleware(|m| {
        m.with_csp(|c| c.policy(SecurityPolicy::strict()))                 // slot 30
            .with_allowed_hosts(|h| h.host("mysite.com"))                  // slot 15
            .with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)   // slot 50
    })
```

Technique: sorting by slot turns a rule the developer had to **remember** into a rule the framework **enforces**. It is the same idea as column types (a mistake becomes impossible) or the builder's typestate (course `traits-avances`).

---

## 4. Session before CSRF: the silent bug

### 4.1 Why CSRF depends on the session

CSRF protection relies on a secret token tied to the session, which each form must send back. The CSRF middleware must read this token to validate a POST request.

### 4.2 The bug

```
Request → CSRF (looks for the token in the session) → Session (loaded too late) → handler
```

CSRF finds nothing, considers every request invalid, and systematically returns 403.

### 4.3 The fix

The slots fix `Session(50) → SessionUpgrade(55) → Auth(57) → CSRF(60)`. CSRF is **always active**: there is no method to disable it globally.

The only planned exception is explicit and limited to specific paths, for entry points called by a third-party service that signs its requests:

```rust
RuniqueAppBuilder::new(config)
    .middleware(|m| {
        m.csrf_exempt(vec!["/webhook/stripe", "/webhook/github"])
    })
```

These routes must then check the service's signature themselves (in constant time): the exemption replaces one protection with another, it does not remove it.

---

## 5. Writing a middleware

### 5.1 The same layer written three ways

Add an `X-Response-Time` header to every response.

```rust
use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

// 1. An async function with from_fn (the simplest)
async fn response_time(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let mut response = next.run(req).await;                // before: nothing; after: we modify the response
    let ms = start.elapsed().as_millis().to_string();
    if let Ok(v) = ms.parse() {
        response.headers_mut().insert("x-response-time", v);
    }
    response
}

let app = axum::Router::new()
    .route("/", axum::routing::get(|| async { "ok" }))
    .layer(axum::middleware::from_fn(response_time));

// 2. With shared state: from_fn_with_state
#[derive(Clone)]
struct AppState { prefix: &'static str }

async fn with_state(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    tracing::info!("{} {}", state.prefix, req.uri());
    next.run(req).await
}

// .layer(axum::middleware::from_fn_with_state(AppState { prefix: "web" }, with_state))

// 3. A ready-made tower-http layer
// .layer(tower_http::timeout::TimeoutLayer::new(Duration::from_secs(10)))
```

| Form | For |
|---|---|
| `from_fn` | application-specific logic, a few lines |
| `from_fn_with_state` | same, with a configuration or state (limiter, database) |
| `tower-http` layers | standard needs: compression, timeouts, CORS, headers, traces |
| hand-written `impl Layer` + `impl Service` | library, fine control over performance |

### 5.2 Rejecting before the handler

```rust
use axum::http::StatusCode;
use axum::response::IntoResponse;

async fn require_header(req: Request, next: Next) -> Response {
    if req.headers().get("x-api-key").is_none() {
        return StatusCode::UNAUTHORIZED.into_response();   // the handler is never called
    }
    next.run(req).await
}
```

A secret key comparison in a middleware is done in **constant time** (`subtle::ConstantTimeEq`), never with `==`, so the response time does not reveal how many characters are correct.

### 5.3 Passing data to the handler

```rust
#[derive(Clone)]
struct RequestId(String);

async fn set_id(mut req: Request, next: Next) -> Response {
    let id = RequestId(format!("{:x}", simple_rand()));
    req.extensions_mut().insert(id);              // stored in the request
    next.run(req).await
}

async fn handler(axum::Extension(id): axum::Extension<RequestId>) -> String {
    id.0                                          // retrieved by the handler
}

fn simple_rand() -> u64 { 42 }
```

The request **extensions** are a typed table (one value per type): this is the relay between middlewares and handlers. Runique uses it to pass the engine, the session, the CSRF token, the signed-in user.

### 5.4 In Runique: `add_custom`

```rust
RuniqueAppBuilder::new(config)
    .middleware(|m| {
        m.add_custom(|router| router.layer(axum::middleware::from_fn(response_time)))
    })
```

Middlewares added this way take their place from slot 20: **after** the host protections and error handling, **before** the session and CSRF.

---

## 6. The HTTP body can only be read once

### 6.1 Why

A request body is a **stream**: bytes arrive from the network and are consumed when read. There is no going back.

```rust
async fn handler(req: axum::extract::Request) {
    let body = axum::body::to_bytes(req.into_body(), 2 * 1024 * 1024).await;
    // req was consumed by into_body: there is no body left to read
}
```

The type system shows it: `into_body(self)` **takes** the request. Reading it again is impossible, it is a compile error.

### 6.2 The problem with middlewares

If a middleware reads the body (to log, validate...), the handler receives an empty body, unless the body is rebuilt.

### 6.3 The different solutions

| Solution | Principle | Cost |
|---|---|---|
| put everything in memory on arrival (Django, Rails, Express) | the body is available everywhere | the whole body in RAM, even a 500 MB file |
| the middleware reads then **puts back** the body (`Request::from_parts(parts, Body::from(bytes))`) | the handler reads the bytes again | one copy, and the size must be limited |
| **a single read**, in the right place, result shared | the body is parsed once and the result is passed on | none |

### 6.4 The Runique solution

The body is read **only once**, by the `Request` extractor. This is why `Request` is always the **last parameter** of a handler: axum requires the extractor that consumes the body to come last.

The parsed data is kept in the request (`request.prisme`), and `request.form("name")` builds the form from it, without reading the stream again. Middlewares never touch the body; what they need goes through the extensions (section 5.3).

```rust
pub async fn login(request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form("login");   // filled from the already parsed data
    // ...
}
```

Size limits are checked **before** and **during** this single read (`Content-Length` header, then a limit while reading: 2 MB for a form, `RUNIQUE_MAX_UPLOAD_MB` per file, `RUNIQUE_MAX_BODY_MB` for the whole request): a body that is too large is rejected with a 413 without having been loaded into memory.

### 6.5 Password fields

On display (GET), filling ignores `password` fields: a password is never shown again in the page. On submission (POST, PUT, PATCH), `request.form("name")` fills them like the other fields.

`add_value(name, value)` sets a value from code (pre-filling, a column the visitor does not choose):

```rust
let mut form: LoginForm = request.form("login");
form.add_value("username", &value);
```

---

## 7. Forms: display on GET, process on POST

### 7.1 The pitfall

It is tempting to write only the POST handler. But the HTML form must first have been **displayed**: this is when the hidden CSRF token and the form name are inserted into the page.

```rust
// Only a POST: the page containing the form was never rendered by Runique,
// so the browser has neither a CSRF token nor a form name to send back.
```

### 7.2 The correct form

```rust
// urlpatterns! { "/login" => [get: views::login_page, post: views::login_post], }

pub async fn login_page(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form("login");      // empty: first visit
    context_update!(request => { "form" => &form });
    request.render("auth/login.html")
}

pub async fn login_post(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form("login");      // filled: this is its own submission
    // validation, then redirect or display again with the errors
    context_update!(request => { "form" => &form });
    request.render("auth/login.html")
}
```

### 7.3 What `request.form("name")` does

- it builds the form with the request's hidden CSRF token;
- it fills it **only if the request is its own submission**: the name comes back in a hidden field. Another form on the same page, or a first display, stays empty: what was typed in another form is never displayed again, nor its CSRF error;
- it caps the size of accepted files at the server limit.

A page can therefore contain several forms (login and signup, for example) without them getting mixed up, each with its own name.

---

## 8. Global middlewares and route middlewares

### 8.1 `layer` or `route_layer`

```rust
let app = axum::Router::new()
    .route("/admin/{*path}", axum::routing::get(handler))
    .route_layer(axum::middleware::from_fn(admin_guard))   // only if a route matches
    .layer(axum::middleware::from_fn(logger));             // all requests, even 404s

async fn handler() {}
async fn admin_guard(req: Request, next: Next) -> Response { next.run(req).await }
async fn logger(req: Request, next: Next) -> Response { next.run(req).await }
```

| | `layer` | `route_layer` |
|---|---|---|
| Applies to | all requests of the router, including those without a route (404) | only requests that match a route |
| For | global security, logs, compression | authentication of an area, limit on a form |

### 8.2 Runique's protections are global

The middlewares configured by the builder (session, CSRF, CSP, host validation...) apply to **all** routes. This is intended: a security protection with exceptions is a potential hole, and a forgotten route must not end up unprotected.

### 8.3 A middleware for some routes

For a need specific to one route, Runique provides router extensions. Real example, the attempt limit on account deletion on runique.io:

```rust
// demo-app/src/url.rs (extract)
.rate_limit(
    "/profil/supprimer",
    "supprimer_compte",
    view!(supprimer_compte),
    3,                       // at most 3 requests...
    300,                     // ...per window of 300 seconds
    vec![Method::POST],      // only submissions count
)
```

`rate_limit` creates the limiter, puts it as a `route_layer` on this single route, and starts its periodic cleanup (course `concurrence`, section 13). For any other layer, `route_layer` on an axum `Router` is still possible.

---

## 9. Fake route and redirect to the admin login

### 9.1 The bug

During development, a non-existent URL did not return 404: it redirected to `/admin/login`.

```
GET /a-page-that-does-not-exist  →  302  →  /admin/login
```

### 9.2 The cause

The admin's authentication guard was set with `layer` on the whole router: it intercepted **all** unauthenticated requests, including those that matched no route (section 8.1), before the router could answer 404.

```
Request → admin guard (redirects if not signed in) → router (never reached)
```

### 9.3 The fix

The guard must only concern the admin routes: it is set as a `route_layer`, on the admin routes only.

```rust
let admin = axum::Router::new()
    .route("/admin/{*path}", axum::routing::get(handler))
    .route_layer(axum::middleware::from_fn(admin_guard));
```

In the current admin, this guard is `declared_route_access` (`runique/src/admin/middleware/admin_middleware.rs`), set on the admin routes only. Public routes are no longer intercepted, and 404s go up normally to the error handler.

---

## 10. Flash messages: redirect or direct render

### 10.1 The intended behaviour

A flash message is **stored in the session** to be displayed on the **next request**, typically after a redirect.

```rust
success!(request.notices => "Profile updated.");
return Ok(Redirect::to("/profil").into_response());
// stored in session → displayed on /profil, then erased
```

This is the "POST, then redirect, then GET" pattern: reloading the page after the redirect does not send the form a second time.

### 10.2 The pitfall: a direct render

```rust
warning!(request.notices => "Invalid form.");
return request.render("forms/login.html");   // no redirect
// the message is in the session: it will show on the NEXT page, not on this one
```

### 10.3 The solution: `flash_now!`

```rust
context_update!(request => {
    "messages" => flash_now!(warning => "Invalid form."),
});
return request.render("forms/login.html");
```

`flash_now!` directly builds the messages (`Vec<FlashMessage>`) to put in the context of the current render, without going through the session:

```rust
// runique/src/macros/context/flash.rs (extract)
macro_rules! flash_now {
    ($msg_type:ident => $content:expr) => {{
        vec![$crate::flash::FlashMessage::$msg_type($content)]
    }};
    // + a form that accepts several messages
}
```

### 10.4 Choosing

| After the action | Syntax |
|---|---|
| redirect | `success!` / `info!` / `warning!` / `error!` on `request.notices` |
| display the same page again (form errors) | `flash_now!` in the context |

### 10.5 The rejected idea

Automatic detection (store in session if the response is a redirect, inject otherwise) would require inspecting the response **after** the handler, then modifying the render already produced. The explicit choice between two syntaxes stays simpler and more predictable.

---

## 11. Exercises

### Exercise 1 — execution order

In which order does the request go through these layers?

```rust
Router::new()
    .route("/", get(h))
    .layer(a)
    .layer(b)
    .layer(ServiceBuilder::new().layer(c).layer(d));
```

Answer: `c`, `d`, `b`, `a`, then the handler. The `ServiceBuilder` is added last, so it is the outermost, and inside it, `c` comes before `d`.

### Exercise 2 — API key middleware

Write a middleware that rejects with 401 a request without an `x-api-key` header equal to a known key, comparing in constant time.

```rust
// Solution:
use subtle::ConstantTimeEq;

async fn api_key(req: Request, next: Next) -> Response {
    const KEY: &[u8] = b"demo-secret-key";
    let provided = req.headers().get("x-api-key").map(|v| v.as_bytes()).unwrap_or(b"");
    if provided.ct_eq(KEY).unwrap_u8() == 0 {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(req).await
}
```

`ct_eq` compares all bytes whatever the position of the first difference: the duration does not reveal the correct start of the key. (A real key is read from the configuration, never hard-coded.)

### Exercise 3 — `layer` or `route_layer`

A guard that redirects to the login page is set with `.layer()` on the public router. What is the symptom, and the fix?

Answer: every non-existent URL redirects to the login instead of returning 404 (section 9). Fix: `.route_layer()`, on the protected routes only.

## 12. Cheat sheet

```text
.layer(a).layer(b)                     b is outside a (read from bottom to top)
ServiceBuilder::new().layer(a).layer(b)   a is outside b (read from top to bottom)
layer                                  all requests, even without a route
route_layer                            only the routes that match
low slot = outer                       Session(50) < Auth(57) < CSRF(60)
```

```rust
axum::middleware::from_fn(f)                async fn f(req: Request, next: Next) -> Response
axum::middleware::from_fn_with_state(s, f)
req.extensions_mut().insert(v)              Extension(v): Extension<T>
m.add_custom(|router| router.layer(..))     m.csrf_exempt(vec!["/webhook/x"])
.rate_limit(path, name, view!(h), max, window_s, vec![Method::POST])
request.form("name")                        flash_now!(warning => "..")
```

### Key points

- A layer only sees what the outer layers have prepared.
- `.layer()` reads from bottom to top; `ServiceBuilder` from top to bottom.
- Runique sorts its layers by slot: the writing order does not matter.
- CSRF is always active; an exemption only applies to paths signed by a third party.
- The body is read once, by `Request`, always the last parameter; middlewares go through the extensions.
- `route_layer` for what only concerns some routes, otherwise 404s are intercepted.
- Flash message for a redirect, `flash_now!` for a direct render.
