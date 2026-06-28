# UML — Extensions de requête, MiddlewareConfig, helpers Tera

## RequestExtensions — injection des extensions

[`context/request_extensions.rs`](../../../runique/src/context/request_extensions.rs)

```mermaid
classDiagram
    class RequestExtensions {
        +Option~AEngine~ engine
        +Option~CsrfToken~ csrf_token
        +Option~ATera~ tera
        +Option~ARuniqueConfig~ config
        +Option~CspNonce~ csp_nonce
        +Option~CurrentUser~ current_user
        +with_engine/with_csrf_token/with_tera/with_config/with_csp_nonce/with_current_user()
        +inject(parts) / inject_request(req)
    }
    note for RequestExtensions "Pose dans req.extensions les valeurs que\nRequest::from_request lira (cf. context/request-pipeline)"
```

C'est le **producteur** des extensions que `Request` consomme : le contrat slot→extension de
[request-pipeline.md](request-pipeline.md) passe par ce builder.

## MiddlewareConfig — toggles

[`middleware/config.rs`](../../../runique/src/middleware/config.rs)

```mermaid
classDiagram
    class MiddlewareConfig {
        +bool enable_csp
        +bool enable_header_security
        +bool enable_host_validation
        +bool enable_debug_errors
        +bool enable_cache
        +bool exclusive_login
        +default() / from_env() / production() / development() / api()
    }
```

Défauts : `enable_debug_errors=true` (handler d'erreurs toujours monté — cf. faux positif E1),
`enable_host_validation=true`, `enable_csp=true`, `enable_cache=true`,
`enable_header_security=false`, `exclusive_login=false`.

## Helpers Tera & cache dev (fonctions)

[`context/tera/`](../../../runique/src/context/tera/) · [`middleware/dev/cache.rs`](../../../runique/src/middleware/dev/cache.rs)

```mermaid
flowchart LR
    subgraph Tera helpers
      F["{% form ... %}` / filtre form"]
      S["{% static \"...\" %}`"]
      M["{% media var %}`"]
      U["{% url \"name\" %}` reverse"]
    end
    DEV["dev_no_cache_middleware<br/>(Cache-Control: no-store en dev)"]
```

## Anomalies / flux suspects

### 🟡 CX2 — `enable_header_security=false` par défaut → CSP seule sans headers durcis
Par défaut `enable_header_security=false` : c'est `csp_middleware` (CSP seule) qui s'applique,
pas `security_headers_middleware` (CSP + HSTS/X-Frame/etc.). En prod, les en-têtes de sécurité
additionnels ne sont **pas** posés sauf activation explicite. À documenter / envisager `true`
dans le preset `production()`.

### Rappel CX1 — couplage extraction ↔ slots
`RequestExtensions` doit poser engine/session/csrf sinon `Request::from_request` → 500
(cf. [request-pipeline.md](request-pipeline.md)).
