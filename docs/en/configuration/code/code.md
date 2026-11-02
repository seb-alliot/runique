# Accessing the configuration in code

## Loading the configuration

```rust
use runique::config::RuniqueConfig;

let config = RuniqueConfig::from_env();

println!("Debug: {}", config.debug);
println!("Port: {}", config.server.port);
```

`RuniqueConfig::from_env()` reads `.env` and the environment once, at startup. Its main fields:

| Field | Content |
| --- | --- |
| `config.debug` | `DEBUG` |
| `config.server` | `ip_server`, `port`, `secret_key`, `public_url` (set by `.with_public_url(...)`) |
| `config.static_files` | Folders and prefixes: templates, static, media |
| `config.security` | HTTPS, HSTS, ACME |
| `config.password` | The `PasswordConfig` |
| `config.timezone` | `TZ` |

---

## In a handler

```rust
async fn my_handler(mut request: Request) -> AppResult<Response> {
    let config = &request.engine.config;

    context_update!(request => {
        "debug" => config.debug,
        "port" => config.server.port,
    });
    request.render("page.html")
}
```

> Never hand `config.server.secret_key` to a template or a log: it signs the CSRF tokens and the sessions.

---

## Conditional configuration

```rust
if request.engine.config.debug {
    // Debug mode: detailed error pages, templates read again on every request
} else {
    // Production mode: generic errors
}
```

Outside a handler, `is_debug()` (in the prelude) gives the same answer.

---

## See also

| Section | Description |
| --- | --- |
| [The .env file](/docs/en/configuration/variables) | Complete examples |
| [Builder](/docs/en/configuration/builder) | Programmatic configuration |

## Back to summary

- [Configuration](/docs/en/configuration)
