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
| `Permissions-Policy` | Preset sécurisé (voir ci-dessous) | Refuse ~20 features sensibles (caméra, micro, géoloc, USB, Bluetooth, paiement, capteurs…) ; autorise en same-origin WebAuthn, fullscreen et picture-in-picture |
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

**Reverse proxy (Nginx, Caddy, Cloudflare…)** — Runique envoie tous ces headers sur chaque réponse dynamique. Un reverse proxy configuré avec `proxy_hide_header` ou des headers `add_header` en doublon peut les écraser silencieusement. En production, ne déclarez pas ces headers dans Nginx — laissez-les passer tels quels depuis l'application.

Pour les fichiers statiques servis directement par Nginx (assets, media), les headers ne passent pas par Runique : il faut les déclarer explicitement dans le bloc `location` concerné :

```nginx
location /media/ {
    add_header X-Content-Type-Options "nosniff" always;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "DENY" always;
}
```

**HSTS (`Strict-Transport-Security`)** — Ce header n'est émis **que si Runique sert réellement du HTTPS** : soit `enforce_https`, soit ACME activé (`should_emit_hsts()`). L'émettre en HTTP simple est inutile (le navigateur l'ignore) et risqué (lock-in HTTPS d'un an sur un domaine pas encore prêt). Si votre proxy (nginx, Caddy, Cloudflare…) termine le TLS sans que Runique le sache, déclarez le header côté proxy.

La valeur est **configurable** (source unique, mêmes réglages partout : middleware, pages d'erreur) :

| Variable d'env | Défaut | Rôle |
| --- | --- | --- |
| `HSTS_MAX_AGE` | `31536000` (1 an) | Durée `max-age` en secondes |
| `HSTS_INCLUDE_SUBDOMAINS` | `true` | Ajoute `includeSubDomains` — ⚠️ casse tout sous-domaine non-HTTPS |
| `HSTS_PRELOAD` | `false` | Ajoute `preload` — **opt-in** : engagement quasi-irréversible (soumission à la liste des navigateurs). Requiert `includeSubDomains` + `max-age ≥ 1 an`, sinon warning au boot et ignoré pour le preload |

Les fichiers statiques ne portent plus le header eux-mêmes : HSTS est *host-scoped*, une fois émis par une page dynamique le navigateur l'applique à tout l'hôte (assets inclus).

**`Permissions-Policy` — preset par défaut** (source : `PermissionsPolicy::default`). Toutes ces features sont **refusées** (`=()`) : `accelerometer`, `ambient-light-sensor`, `bluetooth`, `camera`, `gyroscope`, `hid`, `magnetometer`, `microphone`, `midi`, `serial`, `usb`, `geolocation`, `idle-detection`, `display-capture`, `payment`, `interest-cohort`, `local-fonts`, `sync-xhr`, `xr-spatial-tracking`, `window-management`. **Autorisées en same-origin** (`=(self)`) : `publickey-credentials-create`, `publickey-credentials-get` (WebAuthn / passkeys), `fullscreen`, `picture-in-picture`. Personnalisable via `.with_permissions_policy(|p| …)`.

**COEP (`Cross-Origin-Embedder-Policy: require-corp`)** — Ce header est requis pour utiliser `SharedArrayBuffer` et certaines APIs haute performance. Il peut bloquer le chargement de ressources cross-origin (images, scripts, fonts) qui ne renvoient pas le header `Cross-Origin-Resource-Policy`. Si vous chargez des ressources depuis des CDN tiers, vérifiez leur compatibilité ou désactivez COEP via une `SecurityPolicy` personnalisée.

**`X-XSS-Protection`** — Header legacy, ignoré par les navigateurs modernes (Chrome, Firefox). Conservé pour la compatibilité avec les navigateurs plus anciens.

---

## Retour

- [CSP — Vue d'ensemble](/docs/fr/middleware/csp)
