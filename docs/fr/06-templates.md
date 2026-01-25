# 🎨 Templates

## Moteur Tera

Runique utilise **Tera** pour le rendu HTML:

```rust
use runique::request_context::TemplateContext;
use tera::Context;

async fn index(template: TemplateContext) -> Response {
    let mut context = Context::new();
    context.insert("title", "Accueil");
    context.insert("items", &vec!["Produit 1", "Produit 2"]);

    template.render("index.html", &context)
}
```

---

## Macro context!

Syntaxe simplifiée:

```rust
context_update!(template => {
        "title" => "Erreur de base de données",
        "inscription_form" => &form,
    });

    return template.render("inscription_form.html");
```

---

## Filtres Disponibles

### static - Assets statiques

```html
<link rel="stylesheet" href='{% static "css/main.css" %}'>

<script src="{% static "js/main.js" %}"></script>

```

Génère: `/static/css/main.css`

### media - Fichiers médias

```html
<img src='{% media "media.avif" %}' alt="Logo">
```

Génère: `/media/avatars/profile.jpg`

### csrf_field - Token CSRF

```html
<form method="post">
    {% csrf %}
    <!-- Génère automatiquement: -->
    <!-- <input type="hidden" name="csrf_token" value="..."> -->
</form>
Non necessaire normalement car pris en charge nativement par les formulaires
```

### form - Champs de formulaire

```html
{% form.inscription_form %}
<!-- Ou rendre un champ spécifique: -->
{% form.inscription_form.email %}
```

### link - Liens d'URL

```html
<a href={% link "index" %}>Accueil</a>
<a href={% link "index", id="{{ id }}", name="{{ name }}"  %}>Accueil</a>
```

---

## Boucles et Conditions

### Boucles

```html
<ul>
    {% for item in items %}
        <li>{{ item.name }} - ${{ item.price }}</li>
    {% endfor %}
</ul>

<!-- Avec index -->
{% for i, item in items %}
    <div class="item-{{ i }}">{{ item }}</div>
{% endfor %}

<!-- Avec first/last -->
{% for item in items %}
    {% if loop.first %}<ul>{% endif %}
    <li>{{ item }}</li>
    {% if loop.last %}</ul>{% endif %}
{% endfor %}
```

### Conditions

```html
{% if user %}
    <p>Bienvenue, {{ user.name }}!</p>
{% elif guest %}
    <p>Bienvenue, visiteur!</p>
{% else %}
    <p>Veuillez vous connecter</p>
{% endif %}

<!-- Tests -->
{% if user and user.is_active %}
    ...
{% endif %}

{% if posts | length > 0 %}
    ...
{% endif %}
```

---

## Héritage de Templates

### Template parent (base.html)

```html
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}Mon Site{% endblock %}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
</head>
<body>
    <header>
        <h1>Mon Appli</h1>
    </header>

    {% block content %}{% endblock %}

    <footer>
        <p>&copy; 2026</p>
    </footer>
</body>
</html>
```

### Template enfant (index.html)

```html
{% extends "base.html" %}

{% block title %}Accueil{% endblock %}

{% block content %}
    <h2>Bienvenue!</h2>
    <p>{{ message }}</p>
{% endblock %}
```

---

## Macros

```html
{% macro render_user(user) %}
    <div class="user-card">
        <h3>{{ user.name }}</h3>
        <p>{{ user.email }}</p>
    </div>
{% endmacro %}

<!-- Utiliser: -->
{% for user in users %}
    {{ render_user(user) }}
{% endfor %}
```

---

## Gestion des Erreurs

```html
<!-- Afficher les erreurs globales -->
{% if errors %}
    <div class="alert alert-danger">
        {% for error in errors %}
            <p>{{ error }}</p>
        {% endfor %}
    </div>
{% endif %}

<!-- Erreurs de formulaire -->
    <form method="post" action="/inscription">
        {% form.inscription_form %}
        <button type="submit">S'inscrire</button>
    </form>
    => les erreurs sont déjà rendu nativement par les fields

    Sinon

    <!-- Affichage des erreurs globales avant le formulaire -->
    {% if inscription_form.errors %}
        <div class="alert alert-warning mt-3">
            <div class="alert-message">
                <ul>
                    {% for field_name, error_msg in inscription_form.errors %}
                        <li><strong>{{ field_name }} :</strong> {{ error_msg }}</li>
                    {% endfor %}
                </ul>
            </div>
        </div>
    {% endif %}
```

---

## Prochaines étapes

← [**Formulaires**](https://github.com/seb-alliot/runique/blob/main/docs/fr/05-forms.md) | [**ORM & Base de Données**](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md) →
