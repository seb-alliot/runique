# Filtres & fonctions Tera

## Filtres d'assets

| Filtre | Description | Exemple |
|--------|-------------|---------|
| `static` | Préfixe URL statique de l'app | `{{ "css/main.css" \| static }}` |
| `media` | Préfixe URL média de l'app | `{{ "photo.jpg" \| media }}` |
| `runique_static` | Assets statiques internes au framework | `{{ "css/error.css" \| runique_static }}` |
| `runique_media` | Médias internes au framework | `{{ "logo.png" \| runique_media }}` |

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
| [Tags Django-like](https://github.com/seb-alliot/runique/blob/main/docs/fr/template/tags/tags.md) | Syntaxe sucrée |
| [Syntaxe Tera](https://github.com/seb-alliot/runique/blob/main/docs/fr/template/syntaxe/syntaxe.md) | Héritage, boucles, conditions |

## Retour au sommaire

- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/template/06-templates.md)
