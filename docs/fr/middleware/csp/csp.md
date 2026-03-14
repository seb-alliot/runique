# Content Security Policy (CSP)

Runique applique une politique CSP automatiquement à chaque réponse via le middleware `security_headers_middleware`. Un nonce unique est généré par requête et injecté dans les templates Tera.

---

## Table des matières

| Section | Description |
| --- | --- |
| [Profils CSP](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/profils.md) | `default()`, `strict()`, `permissive()` — comparaison et cas d'usage |
| [Directives & variables d'env](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/directives.md) | Toutes les directives configurables et leurs variables |
| [Nonce CSP](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/nonce.md) | Fonctionnement du nonce, usage dans les templates |
| [Headers de sécurité](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/headers.md) | Tous les headers injectés automatiquement |

---

## Démarrage rapide

La CSP est activée par défaut. Aucune configuration requise. Dans vos templates :

```html
<script {% csp_nonce %}>
    // Ce script est autorisé par le nonce CSP
    console.log("OK");
</script>
```

Pour personnaliser la politique via `.env` :

```env
RUNIQUE_POLICY_CSP_IMAGES='self',data:
RUNIQUE_POLICY_CSP_SCRIPTS='self',https://cdn.example.com
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csrf/csrf.md) | Protection CSRF |
| [Builder & configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/builder/builder.md) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md)
