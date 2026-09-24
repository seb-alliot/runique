# Protection CSRF

## Fonctionnement

- Token généré **automatiquement** pour chaque session
- Pattern **synchronizer token côté session** : le token est stocké côté serveur en session, exposé uniquement via un champ hidden ou le header `X-CSRF-Token` — pas de cookie séparé lisible en JS (le cookie de session lui-même est `HttpOnly`)
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

## Expiration du token & rafraîchissement automatique

Le token CSRF vit dans la session. Les sessions **anonymes** (visiteur non connecté) expirent
par défaut après **5 minutes d'inactivité** (`with_anonymous_session_duration`, voir
[Sessions](/docs/fr/middleware/sessions)) — un onglet resté ouvert plus longtemps sur un
formulaire public (`/login`, `/inscription`, etc.) porte alors un token périmé au moment de
la soumission.

Runique gère ça à deux niveaux :

- **Message explicite** : un échec CSRF pose une erreur globale sur le formulaire (clé i18n
  `csrf.invalid_or_missing`), affichée via `{% messages %}` — jamais un échec silencieux.
- **Rafraîchissement avant soumission** : `js/csrf.js` intercepte chaque soumission de
  formulaire portant un champ `csrf_token`, fait un GET léger vers la page courante pour
  récupérer un token frais (chaque réponse porte un header `X-CSRF-Token`), met à jour le
  champ, puis soumet — fonctionne même si la session a totalement expiré (le serveur en
  recrée une). L'utilisateur ne perd jamais sa saisie et ne voit l'échec CSRF que si le
  réseau est indisponible au moment de soumettre.

Ce script n'est **pas chargé automatiquement** dans les templates d'une application — à
inclure explicitement dans votre layout de base :

```html
<script {% csp %} src="{{ "js/csrf.js" | runique_static }}" defer></script>
```

(le panel admin le charge déjà lui-même, via `runique_static`).

---

## Chemins exemptés (webhooks, APIs)

Certains endpoints reçoivent des POST sans token CSRF — webhooks Stripe, callbacks tiers, APIs JSON appelées par d'autres serveurs.
Utilisez `.csrf_exempt()` pour bypasser la validation CSRF sur des chemins spécifiques :

```rust
.middleware(|m| {
    m.csrf_exempt(vec!["/webhook/stripe", "/api/callback"])
})
```

Le CSRF est toujours actif — il n'y a pas de méthode `.with_csrf()` à appeler, seule l'exemption existe.

La correspondance est **exacte** — `/webhook/stripe` n'exempte pas `/webhook/stripe/sub`.

> Après avoir exempté un chemin, vérifiez l'authenticité de la requête par d'autres moyens dans votre handler
> (ex : HMAC-SHA256 sur l'en-tête `Stripe-Signature` pour les webhooks Stripe).

`csrf_exempt()` ne saute que la *vérification* — le token CSRF est toujours généré et disponible, donc un handler sur un chemin exempté peut continuer à utiliser `Request`/`RuniqueContext` normalement (session, contexte de template, …) en plus de sa propre logique de vérification.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/fr/middleware/csp) | Content Security Policy |
| [Builder](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
