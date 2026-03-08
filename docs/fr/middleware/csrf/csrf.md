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

## Voir aussi

| Section | Description |
| --- | --- |
| [CSP & headers](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/csp.md) | Content Security Policy |
| [Builder](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/builder/builder.md) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md)
