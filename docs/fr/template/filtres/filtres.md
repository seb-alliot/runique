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
| `csrf()` | Génère un champ CSRF depuis le contexte | `{{ csrf() }}` |
| `nonce()` | Retourne le nonce CSP | `{{ nonce() }}` |
| `link(link='...')` | Résolution d'URL nommée | `{{ link(link='index') }}` |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Tags Django-like](/docs/fr/template/tags) | Syntaxe sucrée |
| [Syntaxe Tera](/docs/fr/template/syntaxe) | Héritage, boucles, conditions |

## Retour au sommaire

- [Templates](/docs/fr/template)
