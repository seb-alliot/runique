# Accessing Configuration in Code

## Loading the configuration

```rust
use runique::config::RuniqueConfig;

let config = RuniqueConfig::from_env();

println!("Debug: {}", config.debug);
println!("Port: {}", config.server.port);
```

---

## Access in a handler

```rust
async fn my_handler(mut request: Request) -> AppResult<Response> {
    let config = &request.engine.config;

    println!("Debug mode: {}", config.debug);
    println!("Port: {}", config.server.port);
    println!("IP: {}", config.server.ip_server);
    println!("Secret key: {}", config.server.secret_key);
}
```

---

## Conditional configuration

```rust
if request.engine.config.debug {
    // Debug mode: detailed logs, templates reloaded
} else {
    // Production mode: template cache, no sensitive logs
}
```

---

## See also

| Section | Description |
| --- | --- |
| [Environment variables](/docs/en/configuration/variables) | All `.env` variables |
| [Builder](/docs/en/configuration/builder) | Programmatic configuration |

## Back to summary

- [Configuration](/docs/en/configuration)
