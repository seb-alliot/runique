# Headers de sécurité

Le middleware `security_headers_middleware` injecte automatiquement un ensemble de headers de sécurité à chaque réponse, en plus du header CSP. Il s'active via `.with_header_security(true)` dans le builder.

---

## Headers injectés

| Header | Valeur | Protection |
| --- | --- | --- |
| `Content-Security-Policy` | Dynamique (avec nonce par requête) | Restreint les sources autorisées pour scripts, styles, images, etc. |
| `X-Content-Type-Options` | `nosniff` | Empêche le navigateur de deviner le type MIME — bloque les attaques MIME sniffing |
| `X-Frame-Options` | `DENY` | Interdit l'intégration de la page dans une iframe — protège contre le clickjacking |
| `X-XSS-Protection` | `1; mode=block` | Active le filtre XSS des navigateurs legacy (IE/Edge ancien) |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Envoie le referrer complet en same-origin, seulement l'origine en cross-origin, rien en HTTP→HTTPS |
| `Permissions-Policy` | `geolocation=(), microphone=(), camera=()` | Désactive l'accès à la géolocalisation, au micro et à la caméra |
| `Cross-Origin-Embedder-Policy` | `require-corp` | Exige que les ressources cross-origin soient explicitement autorisées (CORP) |
| `Cross-Origin-Opener-Policy` | `same-origin` | Isole le contexte de navigation — empêche les attaques cross-origin via `window.opener` |
| `Cross-Origin-Resource-Policy` | `same-origin` | Interdit le chargement des ressources depuis d'autres origines |
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` | Force HTTPS pendant 1 an, sous-domaines inclus (HSTS) |

---

## Activation

### CSP seul (sans headers de sécurité additionnels)

```rust
.middleware(|m| {
    m.with_csp(|c| c)
})
```

### CSP + tous les headers de sécurité

```rust
.middleware(|m| {
    m.with_csp(|c| {
        c.with_header_security(true)
         .with_nonce(true)
    })
})
```

### Preset strict complet

```rust
.middleware(|m| {
    m.with_csp(|c| {
        c.policy(SecurityPolicy::strict())
         .with_header_security(true)
    })
})
```

---

## Notes

**HSTS (`Strict-Transport-Security`)** — Ce header est toujours envoyé, même si l'application tourne en HTTP derrière un reverse proxy. Le navigateur le respecte uniquement sur les connexions HTTPS. En production, assurez-vous que votre proxy (nginx, Caddy, Cloudflare…) termine le TLS.

**COEP (`Cross-Origin-Embedder-Policy: require-corp`)** — Ce header est requis pour utiliser `SharedArrayBuffer` et certaines APIs haute performance. Il peut bloquer le chargement de ressources cross-origin (images, scripts, fonts) qui ne renvoient pas le header `Cross-Origin-Resource-Policy`. Si vous chargez des ressources depuis des CDN tiers, vérifiez leur compatibilité ou désactivez COEP via une `SecurityPolicy` personnalisée.

**`X-XSS-Protection`** — Header legacy, ignoré par les navigateurs modernes (Chrome, Firefox). Conservé pour la compatibilité avec les navigateurs plus anciens.

---

## Retour

- [CSP — Vue d'ensemble](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/csp.md)
