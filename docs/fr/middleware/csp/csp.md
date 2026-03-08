# CSP & Headers de sécurité

## Content Security Policy (CSP)

### Fonctionnement

- **Nonce** généré automatiquement par requête
- Injecté dans le contexte Tera sous `csp_nonce`
- Headers CSP ajoutés à chaque réponse

### Usage dans les templates

```html
<!-- Scripts inline sécurisés -->
<script {% csp_nonce %}>
    console.log("Script avec nonce CSP");
</script>

<!-- Ou avec la variable directement -->
<script nonce="{{ csp_nonce }}">
    console.log("Alternative");
</script>
```

### Profils CSP

| Profil | Description |
|--------|-------------|
| `CspConfig::strict()` | Politique stricte (production) |
| `CspConfig::permissive()` | Politique permissive (développement) |
| `CspConfig::default()` | Profil par défaut |

---

## Headers de sécurité

Runique injecte automatiquement des headers de sécurité standards :

| Header | Valeur | Protection |
|--------|--------|------------|
| `X-Content-Type-Options` | `nosniff` | Empêche le MIME sniffing |
| `X-Frame-Options` | `DENY` | Empêche le clickjacking |
| `X-XSS-Protection` | `1; mode=block` | Protection XSS navigateur |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Limite les referrers |
| `Content-Security-Policy` | Dynamique (avec nonce) | CSP |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csrf/csrf.md) | Protection CSRF |
| [Hosts & cache](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/hosts-cache/hosts-cache.md) | Validation des hosts |

## Retour au sommaire

- [Middleware & Sécurité](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md)
