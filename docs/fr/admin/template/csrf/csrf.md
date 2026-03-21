# CSRF dans l'admin

## Ce que `admin_template.html` garantit automatiquement

Le fichier `admin_template.html` inscrit plusieurs éléments **hors blocks**, ce qui signifie qu'ils sont présents dans toutes les pages admin quelle que soit la surcharge appliquée.

| Élément | Emplacement dans le HTML | Rôle |
|---|---|---|
| `<meta name="csrf-token" content="{{ csrf_token }}">` | dans `<head>` | Expose le token CSRF pour JavaScript |
| `<script src="…/csrf.js" defer></script>` | avant `</body>` | Intercepteur AJAX automatique |
| Zone `{% block messages %}` | dans le body | Affichage des messages flash (erreur CSRF incluse) |

Ces éléments **ne peuvent pas être supprimés** via une surcharge de block. Ils sont actifs sur toutes les pages héritant du contrat.

## Le token CSRF

### Propriétés

- **Stable par session** : le token ne change pas entre les requêtes d'une même session (synchronizer token pattern). Cela évite les problèmes avec les onglets multiples et le bouton retour.
- **Masqué dans les réponses** : chaque réponse retourne le token sous une forme masquée différente (XOR + encodage aléatoire), ce qui protège contre l'attaque BREACH (compression + chiffrement TLS).
- **Validation uniquement sur les méthodes mutantes** : `POST`, `PUT`, `PATCH`, `DELETE`. Les requêtes `GET` et `HEAD` ne sont pas vérifiées.

### Flux de validation

```
Requête POST
  └─ middleware CSRF
       ├─ lit X-CSRF-Token (header) ou _csrf_token (form field)
       ├─ démasque la valeur reçue
       ├─ compare avec le token stable de la session
       └─ correspondance ? → continue | non → 403 Forbidden
```

## Ce que `csrf.js` fait

Le script `csrf.js` est chargé automatiquement sur toutes les pages admin. Il intercepte les appels `fetch()` natifs et ajoute le header `X-CSRF-Token` :

```javascript
// Comportement interne de csrf.js (simplifié)
const original = window.fetch;
window.fetch = function(url, options = {}) {
    const token = document.querySelector('meta[name="csrf-token"]')?.content;
    if (token && ["POST", "PUT", "PATCH", "DELETE"].includes((options.method || "GET").toUpperCase())) {
        options.headers = { ...options.headers, "X-CSRF-Token": token };
    }
    return original(url, options);
};
```

Cela couvre tous les appels AJAX sans aucune modification manuelle dans le code frontend.

## Le tag `{% csrf %}` pour les formulaires HTML

Pour les formulaires HTML classiques (non gérés via `form_fields`), il faut inclure le tag Tera `{% csrf %}` à l'intérieur du `<form>` :

```html
<form method="POST" action="/admin/login">
    {% csrf %}
    <input type="text" name="username">
    <input type="password" name="password">
    <button type="submit">Connexion</button>
</form>
```

`{% csrf %}` génère un champ caché :

```html
<input type="hidden" name="_csrf_token" value="…token masqué…">
```

Les formulaires rendus via `{{ form_fields.html }}` incluent ce champ automatiquement — le tag `{% csrf %}` n'est nécessaire que pour les formulaires écrits manuellement.

## Checklist pour un template login custom

Si le template de login est personnalisé (hors `admin_template.html`), les trois éléments suivants sont requis :

- [ ] `<meta name="csrf-token" content="{{ csrf_token }}">` dans `<head>`
- [ ] `{% csrf %}` à l'intérieur du `<form method="POST">`
- [ ] `<script src="{{ "js/csrf.js" | runique_static }}" defer></script>` avant `</body>`

### Exemple complet : `templates/auth/login.html`

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="csrf-token" content="{{ csrf_token }}">
    <title>Connexion — Admin</title>
</head>
<body>
    <form method="POST" action="/admin/login">
        {% csrf %}
        <div>
            <label for="username">Identifiant</label>
            <input type="text" id="username" name="username" required>
        </div>
        <div>
            <label for="password">Mot de passe</label>
            <input type="password" id="password" name="password" required>
        </div>
        <button type="submit">Se connecter</button>
    </form>

    <script src="{{ "js/csrf.js" | runique_static }}" defer></script>
</body>
</html>
```

## Ce qui N'est PAS protégé automatiquement

| Situation | Risque | Solution |
|---|---|---|
| Override de `{% block content %}` avec un `<form>` manuel sans `{% csrf %}` | Le champ `_csrf_token` est absent → 403 sur soumission | Ajouter `{% csrf %}` dans le `<form>` |
| Template qui n'hérite pas de `admin_template.html` | Ni la meta ni le script `csrf.js` ne sont présents | Ajouter manuellement les deux éléments (voir checklist) |
| Appel `fetch()` dans un script chargé avant `csrf.js` | L'intercepteur n'est pas encore actif | Charger le script custom après `csrf.js` ou utiliser `{% block scripts %}` |

## Sous-sections

| Section | Description |
| --- | --- |
| [Clés de contexte](/docs/fr/admin/template) | variables injectées par le backend dans chaque template
| [Surcharge](/docs/fr/admin/template) | remplacer le layout ou un composant CRUD

## Revenir au sommaire

| Section | Description |
| --- | --- |
| [Sommaire template](/docs/fr/admin/template) | Admin
| [Sommaire](/docs/fr/admin) | Sommaire template