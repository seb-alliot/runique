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
| `{% form.xxx.js %}` | `{{ xxx \| form(field='js') \| safe }}` | Scripts JS du form (rendu champ par champ) |

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

## En situation

Les tags se combinent dans un template réel :

```html
{% extends "base.html" %}

{% block content %}
  <link rel="stylesheet" href="{% static "css/contact.css" %}">

  {% messages %}

  <form method="post" action="{% link "contact" %}">
    {% form.contact_form %}
    <button type="submit">Envoyer</button>
  </form>

  <img src="{% media avatar %}" alt="avatar">
{% endblock %}
```

> `{% static %}` / `{% media %}` acceptent une chaîne littérale ou une variable Tera (`avatar` ci-dessus).
> Voir [Tags Django-like](/docs/fr/template/tags) pour le détail de chaque tag.

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
