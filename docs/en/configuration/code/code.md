# Accessing Configuration in Code

## Loading the configuration

```rust
use runique::config_runique::RuniqueConfig;

let config = RuniqueConfig::from_env();

println!("Debug: {}", config.debug);
println!("Port: {}", config.port);
println!("DB: {}", config.database_url);
```

---

## Access in a handler

```rust
use runique::config_runique::RuniqueConfig;

async fn my_handler(template: TemplateContext) -> Response {
    let config = &template.engine.config;

    println!("Debug mode: {}", config.debug);
    println!("Port: {}", config.server.port);
    println!("IP: {}", config.server.ip_server);
    println!("Allowed hosts: {:?}", config.security.allowed_hosts);
    println!("Secret key: {}", config.security.secrete_key);
}
```

---

## Conditional configuration

```rust
if template.engine.config.debug {
    // Debug mode: detailed logs, templates reloaded
} else {
    // Production mode: template cache, no sensitive logs
}

if template.engine.config.security.allowed_hosts.contains("*") {
    // ⚠️ Warning: all hosts are allowed (dangerous in production!)
}
```

---

## Configuration validation

Configuration is validated at startup:

```rust
let config = RuniqueConfig::from_env()
    .expect("Invalid configuration");

// Returns Err() if:
// - DATABASE_URL is missing
// - SECRETE_KEY is missing
// - Variables are invalid
```

---

## See also

| Section | Description |
| --- | --- |
| [Environment variables](/docs/en/configuration/variables) | All `.env` variables |
| [Builder](/docs/en/configuration/builder) | Programmatic configuration |

## Back to summary

- [Configuration](/docs/en/configuration)
