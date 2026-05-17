# Protection CSRF

## Fonctionnement

- Token généré **automatiquement** pour chaque session
- Pattern **Double Submit Cookie** (cookie + champ hidden)
- Vérifié sur les requêtes POST, PUT, PATCH, DELETE
- Ignoré sur les requêtes GET, HEAD, OPTIONS

---

## Dans les formulaires Runique

Quand vous utilisez `{% form.xxx %}`, le CSRF est **inclus automatiquement**. Pas besoin de l'ajouter manuellement.

---

## Dans les formulaires HTML manuels

```html
<form method="post" action="/submit">
    {% csrf %}
    <input type="text" name="data">
    <button type="submit">Envoyer</button>
</form>
```

---

## Pour les requêtes AJAX

```javascript
const csrfToken = document.querySelector('[name="csrf_token"]').value;

fetch('/api/endpoint', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrfToken
    },
    body: JSON.stringify(data)
});
```

---

## Chemins exemptés (webhooks, APIs)

Certains endpoints reçoivent des POST sans token CSRF — webhooks Stripe, callbacks tiers, APIs JSON appelées par d'autres serveurs.
Utilisez `.csrf_exempt()` pour bypasser la validation CSRF sur des chemins spécifiques :

```rust
.middleware(|m| {
    m.with_csrf()
     .csrf_exempt(vec!["/webhook/stripe", "/api/callback"])
})
```

La correspondance est **exacte** — `/webhook/stripe` n'exempte pas `/webhook/stripe/sub`.

> Après avoir exempté un chemin, vérifiez l'authenticité de la requête par d'autres moyens dans votre handler
> (ex : HMAC-SHA256 sur l'en-tête `Stripe-Signature` pour les webhooks Stripe).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/fr/middleware/csp) | Content Security Policy |
| [Builder](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
