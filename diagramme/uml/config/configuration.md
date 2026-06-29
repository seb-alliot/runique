# UML — config & db

## config — `RuniqueConfig` et sous-configs

[`config/`](../../../runique/src/config/)

```mermaid
classDiagram
    class RuniqueConfig {
        +ServerConfig server
        +MiddlewareConfig middleware
        +SecurityConfig security
        +PasswordConfig password
        +StaticConfig static_files
        +RuniqueLog log
        +String base_dir
        +bool debug
        +String timezone
        +from_env() Self
    }
    class ServerConfig {
        +String ip_server / domain_server
        +u16 port
        +String secret_key
        +from_env()
    }
    class SecurityConfig {
        +bool strict_csp / rate_limiting / enforce_https
        +Vec~String~ allowed_hosts
        +bool acme_enabled
        +Option~String~ acme_domain / acme_email
        +String acme_certs_dir
        +from_env()
    }
    class StaticConfig {
        +String base_dir
        +String static_runique_path / url
        +String media_runique_path / media_runique
        +Vec~String~ templates_dir
        +String media_root / static_url / media_url
        +String og_image
        +u64 max_upload_mb
        +usize max_text_field_kb
        +from_env()
    }
    class RuniqueRouter
    RuniqueConfig *-- ServerConfig
    RuniqueConfig *-- SecurityConfig
    RuniqueConfig *-- StaticConfig
    RuniqueConfig *-- MiddlewareConfig
    RuniqueConfig *-- PasswordConfig
    RuniqueConfig *-- RuniqueLog
```

## db — connexion SeaORM

[`db/`](../../../runique/src/db/)

```mermaid
classDiagram
    class DatabaseConfig {
        +String url
        +DatabaseEngine engine
        +u32 max_connections / min_connections
        +Duration connect_timeout / acquire_timeout
        +Duration idle_timeout / max_lifetime
        +bool sqlx_logging
        +connect() async
    }
    class DatabaseConfigBuilder {
        +url() / engine() / max_connections() …
        +build() DatabaseConfig
    }
    class DatabaseEngine {
        <<enum>> Postgres / Mysql / Mariadb / Sqlite
    }
    DatabaseConfigBuilder ..> DatabaseConfig : build()
    DatabaseConfig *-- DatabaseEngine
```

## Anomalies / flux suspects

### 🟡 CFG1 — `ServerConfig.secret_key` : warning si absent, pas d'échec — ✅ CORRIGÉ
**Corrigé (2.1.21).** `secret_key_is_weak()` (vide ‖ défaut ‖ < 32 car., plancher HMAC-SHA256) ;
`cross_validate` refuse le démarrage en prod pour toute clé faible. Debug : warning. Test `secret_key_tests`.

### 🟢 CFG2 — `from_env` par sous-config (pas d'anomalie)
Chaque sous-config charge ses propres variables → séparation claire, défauts documentés.
`max_upload_mb` (StaticConfig) = garde-fou DoS streaming, distinct du `max_size` par champ.
