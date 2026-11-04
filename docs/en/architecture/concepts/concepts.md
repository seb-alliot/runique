# Key concepts

## 1. RuniqueEngine

The app's **main shared state**, built by the builder and reachable from every handler through `request.engine`:

```rust
pub struct RuniqueEngine {
    pub config: RuniqueConfig,        // configuration (.env + builder)
    pub tera: ATera,                  // template engine
    pub db: ADb,                      // database connection (SeaORM)
    pub url_registry: ARlockmap,      // named routes, for {% link %}
    pub security_csp: ASecurityCsp,   // CSP policy
    pub security_hosts: ASecurityHosts,
    pub trusted_proxies: Arc<TrustedProxies>,
    // … sessions, middleware settings, extra connections (with_custom_db)
}
```

The most used fields: `request.engine.db` for queries, `request.engine.config` for the configuration, `request.engine.extension::<T>()` for a connection declared with `.with_custom_db(...)`.

---

## 2. Request — The main extractor

`Request` is Runique's central extractor: it gathers everything a handler needs.

```rust
pub struct Request {
    pub engine: AEngine,               // Arc<RuniqueEngine>
    pub session: Session,              // tower-sessions session
    pub notices: Message,              // flash messages (success!, error!…)
    pub csrf_token: CsrfToken,         // the session's CSRF token
    pub context: Context,              // the page's Tera context
    pub method: Method,                // HTTP method
    pub headers: HeaderMap,
    pub path_params: StrMap,           // path parameters (/{id})
    pub query_params: StrMap,          // query string parameters
    pub user: Option<CurrentUser>,     // signed-in account, read from the database on every request
    pub prisme: Prisme,                // the submitted form's data + CSRF validity
    // …
}
```

`Request` reads the request body: as an extractor that consumes it, it must be the handler's **last parameter**.

**In a handler:**

```rust
pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Home",
    });
    request.render("index.html")
}
```

**Main methods:**

- `request.render("template.html")` — renders with the current context
- `request.form::<F>("name")` — builds form `F` with the submitted data (see below)

---

## 3. Forms: `request.form("name")` then `ValidationForm::try_form`

A form is handled in two steps.

**`request.form("name")` builds the form, without validating it.** The request body was already read when `Request` was extracted (Prisme), and the CSRF token's validity already computed. `request.form("name")`:

1. creates an instance of the form with its fields;
2. fills it with the submitted data;
3. marks it invalid when the method needs a CSRF token (POST, PUT, PATCH, DELETE) and the token is missing or wrong.

**`ValidationForm::try_form(form, &request)` decides whether to validate, and validates.** The form got its name from `request.form("name")` (rendered as a hidden field); it is only filled, then validated, when the request sends **that same name** back. That's what allows **several forms on one page**: only the one submitted is filled and validated, the others stay blank, with no field error, even when they share a field name. It returns `Ok(validated)` or `Err(form)` with its errors, ready to be rendered again.

```rust
pub async fn signup(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form("signup");
    let form = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => match validated.save(&request.engine.db).await {
            Ok(_) => {
                success!(request.notices => "Account created!");
                return Ok(Redirect::to("/").into_response());
            }
            Err(form) => form,
        },
        Err(form) => form,
    };

    context_update!(request => {
        "signup_form" => &form,
    });
    request.render("signup.html")
}
```

> A `ValidationForm` can only be obtained from `try_form`: a function taking a `ValidationForm<F>` proves, by its type, that its caller validated before acting.

---

## See also

| Section | Description |
| --- | --- |
| [Macros](/docs/en/architecture/macros) | Context, flash, routing, error macros |
| [Tera tags & filters](/docs/en/architecture/tera) | Django-like tags, filters, functions |
| [Middleware stack](/docs/en/architecture/middleware) | Slot order, dependency injection |
| [Request lifecycle](/docs/en/architecture/lifecycle) | Lifecycle, best practices |
| [Forms](/docs/en/formulaire) | Fields, validation, rendering |

## Back to summary

- [Architecture](/docs/en/architecture)
