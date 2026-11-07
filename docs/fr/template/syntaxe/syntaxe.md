# Syntaxe Tera

## Héritage de Templates

### Template parent (base.html)

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <title>{% block title %}Mon Site{% endblock %}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
</head>
<body>
    <header>
        <nav>
            <a href='{% link "index" %}'>Accueil</a>
            <a href='{% link "about" %}'>À propos</a>
        </nav>
    </header>

    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>
        <p>&copy; 2026 Mon App</p>
    </footer>
</body>
</html>
```

### Template enfant (index.html)

```html
{% extends "base.html" %}

{% block title %}Accueil{% endblock %}

{% block content %}
    <h2>{{ title }}</h2>
    <p>{{ description }}</p>

    {% if items %}
        <ul>
            {% for item in items %}
                <li>{{ item }}</li>
            {% endfor %}
        </ul>
    {% endif %}
{% endblock %}
```

---

## Boucles

```html
<!-- Boucle simple -->
<ul>
{% for item in items %}
    <li>{{ item.name }} - {{ item.price }}€</li>
{% endfor %}
</ul>

<!-- Avec index -->
{% for item in items %}
    <div class="item-{{ loop.index }}">{{ item }}</div>
{% endfor %}

<!-- Avec first/last -->
{% for item in items %}
    {% if loop.first %}<ul>{% endif %}
    <li>{{ item }}</li>
    {% if loop.last %}</ul>{% endif %}
{% endfor %}
```

---

## Conditions

```html
{% if user %}
    <p>Bienvenue, {{ user.name }} !</p>
{% elif guest %}
    <p>Bienvenue, visiteur !</p>
{% else %}
    <p>Veuillez vous connecter.</p>
{% endif %}

<!-- Tests combinés -->
{% if user and user.is_active %}
    <span class="badge">Actif</span>
{% endif %}

{% if posts | length > 0 %}
    <p>{{ posts | length }} articles trouvés.</p>
{% endif %}
```

---

## Composants (à la place des macros)

Tera 2 n'a plus de `{% macro %}` : il les remplace par des **composants**. Un composant se définit une fois, dans n'importe quel template chargé, et s'utilise depuis tous les autres :

```html
{# templates/components/user_card.html #}
{% component user_card(user: map, badge: string = "") %}
    <div class="user-card">
        <h3>{{ user.name }} {{ badge }}</h3>
        <p>{{ user.email }}</p>
    </div>
{% endcomponent user_card %}
```

```html
{# n'importe quel template #}
{% for u in users %}
    {{ <user_card user={u} badge="★"/> }}
{% endfor %}
```

Les arguments sont nommés ; une expression (variable, nombre) se passe entre accolades (`user={u}`), une chaîne littérale entre guillemets. Un argument sans valeur par défaut est obligatoire.

---

## Macro context_update!

```rust
context_update!(request => {
    "title" => "Ma page",
    "user" => &form,
    "items" => &vec!["a", "b", "c"],
});

request.render("ma_page.html")
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Tags Django-like](/docs/fr/template/tags) | Syntaxe sucrée |
| [Filtres & fonctions](/docs/fr/template/filtres) | Filtres bas niveau |

## Retour au sommaire

- [Templates](/docs/fr/template)
