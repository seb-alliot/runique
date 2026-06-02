# Profils CSP

Runique propose trois profils prédéfinis, utilisables via `.policy(...)` dans le builder.

---

## Comparaison des profils

| Directive | `default()` | `strict()` | `permissive()` |
| --- | :-----------: | :----------: | :--------------: |
| `default-src` | `'none'` | `'none'` | `'none'` |
| `script-src` | `'self'` + nonce | `'self'` + nonce | `'self'` + `'unsafe-inline'` + `'unsafe-eval'` |
| `style-src` | `'self'` + nonce¹ | `'self'` + nonce | `'self'` + `'unsafe-inline'` |
| `img-src` | `'self'` | `'self'` | `'self'` + `data:` + `https:` |
| `font-src` | `'self'` | `'self'` | `'self'` + `data:` |
| `object-src` | `'none'` | `'none'` | `'self'` |
| `media-src` | `'self'` | `'self'` | `'self'` + `https:` |
| `frame-src` | `'none'` | `'none'` | `'self'` |
| `connect-src` | `'self'` | `'self'` | `'self'` |
| `frame-ancestors` | `'none'` | `'none'` | `'self'` |
| `base-uri` | `'self'` | `'self'` | `'self'` |
| `form-action` | `'self'` | `'self'` | `'self'` |
| `upgrade-insecure-requests` | ❌ | ✅ | ❌ |
| Nonce | ✅ actif | ✅ actif | ❌ désactivé |

¹ `default()` inclut `'unsafe-inline'` dans `style-src` pour la compatibilité htmx. Quand le nonce est actif, `'unsafe-inline'` est retiré dynamiquement et remplacé par `'nonce-{val}'`.

---

## `SecurityPolicy::default()`

Politique recommandée pour la production. Tous les scripts et styles inline sont autorisés **uniquement via nonce**. Pas d'images ou polices externes.

```rust,ignore
RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_csp(|c| c)
    })
    .build()
    .await?;
```

---

## `SecurityPolicy::strict()`

Plus restrictif que `default()` : ajoute `upgrade-insecure-requests` et force le nonce. À utiliser en production pour une sécurité maximale.

```rust,ignore
RuniqueApp::builder(config)
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

## `SecurityPolicy::permissive()`

Politique relâchée pour le développement ou les intégrations legacy. **Ne pas utiliser en production.**

- `'unsafe-inline'` et `'unsafe-eval'` activés → CSP ne protège plus contre le XSS
- Nonce désactivé
- `data:` et `https:` autorisés pour les images et polices
- `frame-ancestors 'self'` au lieu de `'none'`

```rust,ignore
RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_csp(|c| {
            c.policy(SecurityPolicy::permissive())
        })
    })
    .build()
    .await?;
```

---

## Politique personnalisée

Pour une politique sur mesure, utiliser les méthodes du builder directement :

```rust,ignore
RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_csp(|c| {
            c.scripts(vec!["'self'", "https://cdn.example.com"])
             .images(vec!["'self'", "data:"])
             .with_nonce(true)
        })
    })
    .build()
    .await?;
```

Ou construire une `SecurityPolicy` manuellement pour les cas avancés :

```rust,ignore
use runique::middleware::SecurityPolicy;

let policy = SecurityPolicy {
    script_src: vec!["'self'".into(), "https://cdn.example.com".into()],
    img_src: vec!["'self'".into(), "data:".into()],
    ..SecurityPolicy::default()
};

RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_csp(|c| c.policy(policy))
    })
    .build()
    .await?;
```

---

## Retour

- [CSP — Vue d'ensemble](/docs/fr/middleware/csp)
