# Filtres & fonctions Tera

## Filtres d'assets

| Filtre | Description | Exemple |
|--------|-------------|---------|
| `static` | Préfixe URL statique de l'app | `{{ "css/main.css" \| static }}` |
| `media` | Préfixe URL média de l'app | `{{ "photo.jpg" \| media }}` |

---

## Filtre Markdown

| Filtre     | Description                                      | Exemple                          |
|------------|--------------------------------------------------|----------------------------------|
| `markdown` | Convertit du Markdown en HTML (safe automatique) | `{{ page.content \| markdown }}` |

> Le préprocesseur Runique injecte automatiquement `\| safe` — inutile de l'ajouter manuellement.

---

## Filtre de formulaire

| Filtre | Description | Exemple |
|--------|-------------|---------|
| `form` | Rendu complet du formulaire | `{{ form.nom_form \| form \| safe }}` |
| `form(field='xxx')` | Rendu d'un seul champ | `{{ form.nom_form \| form(field='email') \| safe }}` |
| `csrf_field` | Génère un input hidden CSRF | `{{ csrf_token \| csrf_field \| safe }}` |

---

## Fonctions Tera

| Fonction | Description | Exemple |
|----------|-------------|---------|
| `link(link='...')` | Résolution d'URL nommée | `{{ link(link='index') }}` |

## Variables de contexte auto-injectées

| Variable | Description |
|----------|-------------|
| `csrf_token` | Token CSRF masqué (utilisé par `{% csrf %}` et `\| csrf_field`) |
| `csp_nonce` | Valeur nonce CSP du header (utilisé par `{% csp %}`) |
| `messages` | Flash messages de la requête |
| `user` | Utilisateur authentifié courant (si connecté) |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Tags Django-like](/docs/fr/template/tags) | Syntaxe sucrée |
| [Syntaxe Tera](/docs/fr/template/syntaxe) | Héritage, boucles, conditions |

## Retour au sommaire

- [Templates](/docs/fr/template)
