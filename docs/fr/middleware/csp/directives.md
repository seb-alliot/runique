# Directives CSP

Chaque directive CSP est configurable via le builder — plus via variables d'environnement.

---

## Directives disponibles

| Méthode builder | Directive CSP | Défaut |
| --- | --- | --- |
| `.default_src(vec![...])` | `default-src` | `'self'` |
| `.scripts(vec![...])` | `script-src` | `'self'` |
| `.styles(vec![...])` | `style-src` | `'self'` |
| `.images(vec![...])` | `img-src` | `'self'` |
| `.fonts(vec![...])` | `font-src` | `'self'` |
| `.connect(vec![...])` | `connect-src` | `'self'` |
| `.objects(vec![...])` | `object-src` | `'none'` |
| `.media(vec![...])` | `media-src` | `'self'` |
| `.frames(vec![...])` | `frame-src` | `'none'` |
| `.frame_ancestors(vec![...])` | `frame-ancestors` | `'none'` |
| `.base_uri(vec![...])` | `base-uri` | `'self'` |
| `.form_action(vec![...])` | `form-action` | `'self'` |

### Toggles

| Méthode builder | Défaut | Description |
| --- | --- | --- |
| `.with_nonce(bool)` | `true` | Nonce par requête injecté dans `script-src` et `style-src` |
| `.with_header_security(bool)` | `false` | HSTS, X-Frame-Options, COEP, COOP, CORP… |
| `.with_upgrade_insecure(bool)` | `false` | `upgrade-insecure-requests` |

### Presets

| Méthode builder | Description |
| --- | --- |
| `.policy(SecurityPolicy::default())` | Politique par défaut — `'self'` partout, nonce actif |
| `.policy(SecurityPolicy::strict())` | Strict — nonce obligatoire, `upgrade-insecure-requests`, `frame-ancestors 'none'` |
| `.policy(SecurityPolicy::permissive())` | Permissif — `unsafe-eval` autorisé, images depuis `https:` |

---

## Exemples courants

### Minimal — CSP activée sans personnalisation

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| c)
    })
    .build()
    .await?;
```

### CDN pour scripts et styles (ex. Bootstrap)

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.scripts(vec!["'self'", "https://cdn.jsdelivr.net"])
             .styles(vec!["'self'", "https://cdn.jsdelivr.net"])
        })
    })
    .build()
    .await?;
```

### Google Fonts + images base64

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.fonts(vec!["'self'", "https://fonts.gstatic.com"])
             .styles(vec!["'self'", "https://fonts.googleapis.com"])
             .images(vec!["'self'", "data:"])
        })
    })
    .build()
    .await?;
```

### WebSocket + iframes

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.connect(vec!["'self'", "wss://ws.example.com"])
             .frames(vec!["'self'"])
             .frame_ancestors(vec!["'self'"])
        })
    })
    .build()
    .await?;
```

### Configuration complète (production)

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.with_header_security(true)
             .with_nonce(true)
             .with_upgrade_insecure(true)
             .scripts(vec!["'self'", "https://cdn.jsdelivr.net"])
             .styles(vec!["'self'", "https://cdn.jsdelivr.net", "https://fonts.googleapis.com"])
             .fonts(vec!["'self'", "https://fonts.gstatic.com"])
             .images(vec!["'self'", "data:", "https://cdn.example.com"])
             .connect(vec!["'self'", "wss://ws.example.com"])
        })
    })
    .build()
    .await?;
```

### Preset strict avec headers de sécurité

```rust,ignore
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| {
            c.policy(SecurityPolicy::strict())
             .with_header_security(true)
        })
    })
    .build()
    .await?;
```

---

## Comportement du nonce sur `script-src` et `style-src`

Quand le nonce est actif (`.with_nonce(true)`) :

- `'nonce-{valeur}'` est ajouté automatiquement à `script-src` et `style-src`
- `'unsafe-inline'` est **retiré automatiquement** de ces directives si présent

Cela garantit que les scripts inline sans nonce sont bloqués, même si `'unsafe-inline'` est configuré manuellement.

```text
# Header généré avec nonce actif :
Content-Security-Policy: default-src 'self'; script-src 'self' 'nonce-abc123'; style-src 'self' 'nonce-abc123'; ...
```

---

## Retour

- [CSP — Vue d'ensemble](/docs/fr/middleware/csp)
