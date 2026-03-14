# Profils CSP

Runique propose trois profils prédéfinis. Le profil actif est configuré dans le builder de l'application.

---

## Comparaison des profils

| Directive | `default()` | `strict()` | `permissive()` |
|-----------|:-----------:|:----------:|:--------------:|
| `default-src` | `'self'` | `'self'` | `'self'` |
| `script-src` | `'self'` + nonce | `'self'` + nonce | `'self'` + `'unsafe-inline'` + `'unsafe-eval'` |
| `style-src` | `'self'` + nonce | `'self'` + nonce | `'self'` + `'unsafe-inline'` |
| `img-src` | `'self'` | `'self'` | `'self'` + `data:` + `https:` |
| `font-src` | `'self'` | `'self'` | `'self'` + `data:` |
| `connect-src` | `'self'` | `'self'` | `'self'` |
| `frame-ancestors` | `'none'` | `'none'` | `'self'` |
| `base-uri` | `'self'` | `'self'` | `'self'` |
| `form-action` | `'self'` | `'self'` | `'self'` |
| Nonce | ✅ actif | ✅ actif | ❌ désactivé |

---

## `SecurityPolicy::default()`

Politique recommandée pour la production. Tous les scripts et styles inline sont autorisés **uniquement via nonce**. Pas d'images ou polices externes.

```rust
// Comportement par défaut — aucune configuration nécessaire
RuniqueApp::new()
    .build()
    .await?;
```

Chaque directive est surchargeable via variables d'env sans toucher au code. Voir [Directives & variables d'env](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/directives.md).

---

## `SecurityPolicy::strict()`

Identique à `default()`. À utiliser explicitement pour signaler l'intention de politique stricte dans le code.

```rust
RuniqueApp::new()
    .with_security_csp(SecurityPolicy::strict())
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

```rust
RuniqueApp::new()
    .with_security_csp(SecurityPolicy::permissive())
    .build()
    .await?;
```

---

## Politique personnalisée

Pour une politique sur mesure, construire un `SecurityPolicy` manuellement :

```rust
use runique::middleware::SecurityPolicy;

let policy = SecurityPolicy {
    script_src: vec!["'self'".into(), "https://cdn.example.com".into()],
    img_src: vec!["'self'".into(), "data:".into()],
    ..SecurityPolicy::default()
};

RuniqueApp::new()
    .with_security_csp(policy)
    .build()
    .await?;
```

Les directives non spécifiées héritent des valeurs de `default()`.

---

## Retour

- [CSP — Vue d'ensemble](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/csp.md)
