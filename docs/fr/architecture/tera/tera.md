# Tags et filtres Tera

## Tags Django-like (syntaxe sucrée)

| Tag | Transformé en | Description |
| --- | ------------- | ----------- |
| `{% static "..." %}` | `{{ "..." \| static }}` | URL d'un fichier statique |
| `{% media "..." %}` | `{{ "..." \| media }}` | URL d'un fichier média |
| `{% csrf %}` | `{% include "csrf" %}` | Champ CSRF caché |
| `{% messages %}` | `{% include "message" %}` | Affichage messages flash |
| `{% csp %}` | `{% include "csp" %}` | Attribut nonce CSP |
| `{% link "name" %}` | `{{ link(link='name') }}` | URL d'une route nommée |
| `{% form.xxx %}` | `{{ xxx \| form \| safe }}` | Rendu formulaire complet |
| `{% form.xxx.field %}` | `{{ xxx \| form(field='field') \| safe }}` | Rendu d'un champ |

---

## Filtres Tera

| Filtre | Description |
| ------ | ----------- |
| `static` | Préfixe URL statique de l'app |
| `media` | Préfixe URL média de l'app |
| `form` | Rendu de formulaire complet ou par champ |
| `csrf_field` | Génère un input hidden CSRF |

---

## Fonctions Tera

| Fonction | Description |
| -------- | ----------- |
| `link(link='...')` | Résolution d'URL nommée |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Concepts clés](/docs/fr/architecture/concepts) | `RuniqueEngine`, `Request`, `request.form()` |
| [Macros](/docs/fr/architecture/macros) | Macros de contexte, flash, routage, erreur |
| [Stack middleware](/docs/fr/architecture/middleware) | Ordre des slots, injection de dépendances |
| [Lifecycle d'une requête](/docs/fr/architecture/lifecycle) | Cycle de vie, bonnes pratiques |

## Retour au sommaire

- [Architecture](/docs/fr/architecture)
