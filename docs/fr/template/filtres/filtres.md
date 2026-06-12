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
| `markdown` | Convertit du Markdown en HTML, sanitisé (anti-XSS) | `{{ page.content \| markdown }}` |

> Le préprocesseur Runique injecte automatiquement `\| safe` — inutile de l'ajouter manuellement.
>
> La sortie est **sanitisée via ammonia** : le HTML brut dangereux (`<script>`, gestionnaires `on*`) et les URL `javascript:` / `data:` des liens et images sont supprimés. Le Markdown légitime (titres, tables, listes, liens, images, code) est préservé — un Markdown rédigé par un utilisateur peut donc être rendu sans risque.

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
| `current_path` | Path de l'URL courante, sans query string (utile pour `rel="canonical"`, `og:url`, navigation active) |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Tags Django-like](/docs/fr/template/tags) | Syntaxe sucrée |
| [Syntaxe Tera](/docs/fr/template/syntaxe) | Héritage, boucles, conditions |

## Retour au sommaire

- [Templates](/docs/fr/template)
