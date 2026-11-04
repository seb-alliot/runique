# Protection CSRF

## Fonctionnement

- Token généré **automatiquement** pour chaque session
- Pattern **synchronizer token côté session** : le token est stocké côté serveur en session, exposé uniquement via un champ hidden ou le header `X-CSRF-Token` — pas de cookie séparé lisible en JS (le cookie de session lui-même est `HttpOnly`)
- Vérifié sur les requêtes POST, PUT, PATCH, DELETE
- Ignoré sur les requêtes GET, HEAD, OPTIONS

---

## Dans les formulaires Runique

Un formulaire construit avec `RuniqueForm` contient déjà le jeton. `{% form.xxx %}` le pose dans la page. Pas besoin d'ajouter `{% csrf %}`.

## Où le jeton est vérifié

`{% csrf %}` ne fait que poser le jeton sur le formulaire, dans un champ caché `csrf_token`. Il ne vérifie rien.

La vérification se fait dans `request.form("nom")`. Sans jeton valide, le formulaire est invalide et `save()` ne part pas. C'est le chemin de tout `RuniqueForm`.

Le middleware lit l'en-tête `X-CSRF-Token`. Un corps JSON sans cet en-tête est refusé (403). Pour `application/x-www-form-urlencoded` et `multipart/form-data`, il laisse passer le corps : le champ est lu par `request.form("nom")`.

Un handler qui lit le corps et écrit sans `request.form("nom")` ne reçoit pas cette vérification, même si `{% csrf %}` est dans la page. Le cookie de session est `SameSite=Strict`.

**Une action POST sans formulaire** (déconnexion, bouton « supprimer ») vérifie le jeton elle-même, calculé à l'extraction de `Request` :

```rust
pub async fn logout_view(request: Request) -> AppResult<Response> {
    if !request.prisme.csrf_valid {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    logout(&request.session, None).await?;
    Ok(Redirect::to("/").into_response())
}
```

Un extracteur Axum natif (`Form<T>`, `Json<T>` sans en-tête, `Multipart`) ne passe pas par cette vérification : une route qui en utilise un pour écrire doit contrôler le jeton elle-même, ou s'appuyer sur le refus des formulaires d'un autre site (« Formulaires envoyés depuis un autre site », plus bas).

---

## Dans les formulaires HTML manuels

Le tag pose le jeton. Le handler doit appeler `request.form("nom")` pour le vérifier.

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

Runique **ajoute ce script lui-même** : `request.render()` insère une seule balise avant `</body>`, avec le nonce CSP de la requête, dans toute page qui contient un champ CSRF, quelle qu'en soit l'origine (un formulaire Runique, `{% csrf %}`, le filtre `csrf_field` ou la fonction `csrf_token`). Une page à plusieurs formulaires reçoit une seule balise. Rien n'est ajouté à une page sans champ CSRF, à une page qui charge déjà `js/csrf.js`, ni à un fragment sans `</body>` (réponse HTMX).

Seule une page **sans formulaire** qui envoie des `fetch` en POST, ou une réponse HTML construite sans `request.render()`, doit l'inclure elle-même :

```html
<script {% csp %} src="{% runique_static "js/csrf.js" %}" defer></script>
```

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

## Formulaires envoyés depuis un autre site

Désactivé par défaut. À activer si vous écrivez des handlers qui lisent leur formulaire avec un extracteur Axum (`Form<T>`, `Multipart`) au lieu de `request.form("nom")` : personne ne vérifie alors leur jeton.

```rust
.middleware(|m| m.with_cross_site_form_guard())
```

Le middleware refuse (403) un formulaire envoyé depuis la page d'un **autre site** ou d'un **sous-domaine**, avant le handler. Il le sait par l'en-tête `Sec-Fetch-Site`, posé par le navigateur : une page ne peut ni l'écrire ni le retirer.

| Requête | Effet |
| --- | --- |
| Formulaire envoyé depuis votre site (`same-origin`) | passe, le jeton est vérifié comme avant |
| Formulaire envoyé depuis un autre site (`cross-site`) ou un sous-domaine (`same-site`) | **403** |
| Sans en-tête `Sec-Fetch-Site` (ancien navigateur, curl, tests) | passe, laissé au jeton |
| Avec l'en-tête `X-CSRF-Token` | décidé par cet en-tête, comme avant |
| GET, HEAD | jamais contrôlés |
| Chemin de `csrf_exempt()` | non contrôlé |

Les formulaires Runique et l'admin sont envoyés depuis votre site : ils ne changent pas. Un formulaire Runique posté depuis un autre site était déjà refusé par son jeton (page réaffichée avec l'erreur) ; il reçoit maintenant un 403 plus tôt, sans rien écrire dans les deux cas.

Ce n'est pas un remplacement du jeton : un client sans cet en-tête n'est arrêté que par lui. Un service tiers qui poste un formulaire chez vous (retour de paiement d'un prestataire) se met dans `csrf_exempt()`.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/fr/middleware/csp) | Content Security Policy |
| [Builder](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
