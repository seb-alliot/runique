# Exemples de templates admin

Trois approches sont possibles. L'héritage Tera n'est **pas obligatoire** — un fichier HTML classique fonctionne aussi, tant que les variables injectées sont utilisées correctement.

---

## Approche 1 — Étendre le layout Runique par défaut

Le cas le plus courant : on garde le layout admin Runique et on personalise uniquement le contenu.

```html
{# templates/mon_theme/users_list.html #}

{% extends "admin/admin_base.html" %}

{% block title %}{{ resource.title }}{% endblock %}

{% block content %}
<h1>{{ resource.title }}</h1>
<p>{{ total }} {{ admin_list_entries_count }}</p>

{% for entry in entries %}
<div class="card">
    <span>#{{ entry.id }}</span>
    <span>{{ entry.username }}</span>
    <a href="/admin/{{ resource_key }}/{{ entry.id }}/edit">{{ admin_list_btn_edit }}</a>
</div>
{% endfor %}
{% endblock %}
```

Déclaration dans le builder (`src/main.rs`) :

```rust
RuniqueApp::builder(config)
    .with_admin(|a| a
        .routes(admins::routes("/admin"))
        .with_state(admins::admin_state())
        .templates(|t| t
            .with_list("templates/mon_theme/users_list.html")
        )
    )
    .build().await?
```

---

## Approche 2 — Étendre un layout personnalisé

On a créé son propre `template.html` (voir [Surcharge du layout](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/template/surcharge/surcharge.md)) et on l'utilise comme base.

```html
{# templates.html #}

{% block content %}
<h1>{{ resource.title }}</h1>

{% if entries %}
<table>
    <tbody>
    {% for entry in entries %}
    <tr>
        <td>{{ entry.id }}</td>
        <td>{{ entry.username }}</td>
        <td>
            <a href="/admin/{{ resource_key }}/{{ entry.id }}/detail">{{ admin_list_btn_detail }}</a>
            <a href="/admin/{{ resource_key }}/{{ entry.id }}/delete" class="danger">
                {{ admin_list_btn_delete }}
            </a>
        </td>
    </tr>
    {% endfor %}
    </tbody>
</table>
{% else %}
<p>{{ admin_list_empty_title }}</p>
{% endif %}
{% endblock %}
```

Déclaration dans le builder (`src/main.rs`) :

```rust
RuniqueApp::builder(config)
    .with_admin(|a| a
        .routes(admins::routes("/admin"))
        .with_state(admins::admin_state())
        .templates(|t| t
            .with_base("template.html")
            .with_list("templates/mon_theme/users_list.html")
        )
    )
    .build().await?
```

---

## Approche 3 — HTML autonome (sans héritage)

Aucun `{% extends %}` n'est nécessaire. Le template est un fichier HTML complet. Utile pour les intégrations avec des frameworks frontend (Alpine.js, HTMX, etc.) ou pour les cas où le layout admin Runique n'est pas souhaité.

```html
{# templates/mon_theme/users_list.html #}
<!DOCTYPE html>
<html lang="{{ lang }}">
<head>
    <meta charset="UTF-8">
    <title>{{ resource.title }} — {{ site_title }}</title>
    <link rel="stylesheet" href="/static/css/mon_theme.css">

    {# CSRF obligatoire pour les actions POST depuis cette page #}
    <meta name="csrf-token" content="{{ csrf_token }}">
    <script src="/static/js/csrf.js" defer></script>
</head>
<body>
    <nav>
        <strong>{{ site_title }}</strong>
        {% for res in resources %}
        <a href="/admin/{{ res.key }}/list"
           {% if res.key == current_resource %}class="active"{% endif %}>
            {{ res.title }}
        </a>
        {% endfor %}
    </nav>

    <main>
        <h1>{{ resource.title }}</h1>

        {# Messages flash #}
        {% messages %}

        <a href="/admin/{{ resource_key }}/create">{{ admin_list_btn_create }}</a>

        {% for entry in entries %}
        <div>{{ entry.id }} — {{ entry.username }}</div>
        {% endfor %}
    </main>
</body>
</html>
```

> Si le template n'étend pas `admin_template.html`, les éléments CSRF ne sont plus injectés automatiquement. Il faut les ajouter manuellement (voir ci-dessus). Voir [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/template/csrf/csrf.md) pour le détail.

---

## Récapitulatif

| Approche | Quand l'utiliser |
| --- | --- |
| Étendre `admin_base.html` | Personnalisation du contenu uniquement |
| Étendre un layout custom | Refonte complète du layout admin |
| HTML autonome | Intégration frontend, ou aucun layout partagé souhaité |

---

## Revenir au sommaire

| Section | Description |
| --- | --- |
| [Surcharge](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/template/surcharge/surcharge.md) | Principe et niveaux d'héritage |
| [Clés de contexte](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/template/clef/context.md) | Variables disponibles par vue |
