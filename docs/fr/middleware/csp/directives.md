# Directives CSP & Variables d'environnement

Chaque directive CSP est configurable via variable d'environnement dans le `.env`, sans modifier le code. Les valeurs sont des listes séparées par des virgules.

---

## Directives configurables

| Directive CSP | Variable d'env | Défaut |
|---------------|---------------|--------|
| `default-src` | `RUNIQUE_POLICY_CSP_DEFAULT` | `'self'` |
| `script-src` | `RUNIQUE_POLICY_CSP_SCRIPTS` | `'self'` |
| `style-src` | `RUNIQUE_POLICY_CSP_STYLES` | `'self'` |
| `img-src` | `RUNIQUE_POLICY_CSP_IMAGES` | `'self'` |
| `font-src` | `RUNIQUE_POLICY_CSP_FONTS` | `'self'` |
| Nonce actif | `RUNIQUE_POLICY_CSP_STRICT_NONCE` | `true` |

> Les directives `connect-src`, `frame-ancestors`, `base-uri` et `form-action` ne sont pas encore surchargeables par variable d'env. Utilisez une `SecurityPolicy` personnalisée si besoin.

---

## Exemples courants

### Autoriser un CDN pour les scripts

```env
RUNIQUE_POLICY_CSP_SCRIPTS='self',https://cdn.jsdelivr.net
```

### Autoriser les images base64 inline (avatars, éditeurs rich-text)

```env
RUNIQUE_POLICY_CSP_IMAGES='self',data:
```

### Autoriser les polices Google Fonts

```env
RUNIQUE_POLICY_CSP_FONTS='self',https://fonts.gstatic.com
RUNIQUE_POLICY_CSP_STYLES='self',https://fonts.googleapis.com
```

### Désactiver le nonce (non recommandé)

```env
RUNIQUE_POLICY_CSP_STRICT_NONCE=false
```

---

## Comportement du nonce sur `script-src` et `style-src`

Quand le nonce est actif (`RUNIQUE_POLICY_CSP_STRICT_NONCE=true`, par défaut) :

- `'nonce-{valeur}'` est ajouté automatiquement à `script-src` et `style-src`
- `'unsafe-inline'` est **retiré automatiquement** de ces directives si présent

Cela garantit que les scripts inline sans nonce sont bloqués, même si `'unsafe-inline'` est configuré manuellement.

```
# Header généré avec nonce actif :
Content-Security-Policy: default-src 'self'; script-src 'self' 'nonce-abc123'; ...
```

---

## Directives fixes (non configurables par env)

Ces directives sont définies dans le profil et ne sont pas modifiables par variable d'env :

| Directive | Valeur (default/strict) | Rôle |
|-----------|------------------------|------|
| `connect-src` | `'self'` | Limite les connexions XHR/fetch/WebSocket |
| `frame-ancestors` | `'none'` | Interdit l'intégration dans des iframes (clickjacking) |
| `base-uri` | `'self'` | Empêche l'injection de balise `<base>` |
| `form-action` | `'self'` | Interdit les soumissions de formulaires vers des domaines externes |

Pour les modifier, utiliser une `SecurityPolicy` personnalisée :

```rust
use runique::middleware::SecurityPolicy;

RuniqueApp::new()
    .with_security_csp(SecurityPolicy {
        connect_src: vec!["'self'".into(), "wss://ws.example.com".into()],
        frame_ancestors: vec!["'self'".into()],
        ..SecurityPolicy::default()
    })
    .build()
    .await?;
```

---

## Retour

- [CSP — Vue d'ensemble](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/csp.md)
