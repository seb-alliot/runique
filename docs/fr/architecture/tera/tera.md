# Tags et filtres Tera

## Tags Django-like (syntaxe sucrée)

| Tag | Transformé en | Description |
| --- | ------------- | ----------- |
| `{% static "..." %}` | `{{ "..." \| static }}` | URL d'un fichier statique |
| `{% media "..." %}` | `{{ "..." \| media }}` | URL d'un fichier média |
| `{% csrf %}` | `{% include "csrf/..." %}` | Champ CSRF caché |
| `{% messages %}` | `{% include "message/..." %}` | Affichage messages flash |
| `{% csp_nonce %}` | `{% include "csp/..." %}` | Attribut nonce CSP |
| `{% link "name" %}` | `{{ link(link='name') }}` | URL d'une route nommée |
| `{% form.xxx %}` | `{{ xxx \| form \| safe }}` | Rendu formulaire complet |
| `{% form.xxx.field %}` | `{{ xxx \| form(field='field') \| safe }}` | Rendu d'un champ |

---

## Filtres Tera

| Filtre | Description |
| ------ | ----------- |
| `static` | Préfixe URL statique de l'app |
| `media` | Préfixe URL média de l'app |
| `runique_static` | Assets statiques internes au framework |
| `runique_media` | Médias internes au framework |
| `form` | Rendu de formulaire complet ou par champ |
| `csrf_field` | Génère un input hidden CSRF |

---

## Fonctions Tera

| Fonction | Description |
| -------- | ----------- |
| `csrf()` | Génère un champ CSRF depuis le contexte |
| `nonce()` | Retourne le nonce CSP |
| `link(link='...')` | Résolution d'URL nommée |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Concepts clés](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/concepts/concepts.md) | `RuniqueEngine`, `Request`, `Prisme<T>` |
| [Macros](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/macros/macros.md) | Macros de contexte, flash, routage, erreur |
| [Stack middleware](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/middleware/middleware.md) | Ordre des slots, injection de dépendances |
| [Lifecycle d'une requête](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/lifecycle/lifecycle.md) | Cycle de vie, bonnes pratiques |

## Retour au sommaire

- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/02-architecture.md)
