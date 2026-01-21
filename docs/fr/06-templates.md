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
template.render("index.html", &context! {
    "title" => "Accueil",
    "items" => vec!["A", "B", "C"],
    "user" => user_data
})
```

---

## Filtres Disponibles

### static - Assets statiques

```html
<link rel="stylesheet" href="{{ 'css/main.css' | static }}">
<script src="{{ 'js/app.js' | static }}"></script>
```

Génère: `/static/css/main.css`

### media - Fichiers médias

```html
<img src="{{ user.avatar | media }}" alt="Avatar">
```

Génère: `/media/avatars/profile.jpg`

### csrf_field - Token CSRF

```html
<form method="post">
    {{ '' | csrf_field }}
    <!-- Génère automatiquement: -->
    <!-- <input type="hidden" name="csrf_token" value="..."> -->
</form>
```

### form - Champs de formulaire

```html
{{ form | form }}
<!-- Ou renderer un champ spécifique: -->
{{ form.email | form_field }}
```

### link - Liens d'URL

```html
<a href="{{ 'index' | link }}">Accueil</a>
<a href="{{ 'profile' | link('user_id=5') }}">Profile</a>
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
    <link rel="stylesheet" href="{{ 'css/main.css' | static }}">
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
{% if form.has_error('email') %}
    <span class="error">{{ form.get_error('email') }}</span>
{% endif %}
```

---

## Structure Complète

```
templates/
├── base.html              # Template parent
├── index.html             # Accueil
├── errors/
│   ├── 404.html
│   └── 500.html
├── auth/
│   ├── login.html
│   └── register.html
├── blog/
│   ├── list.html
│   ├── detail.html
│   └── form.html
└── includes/
    ├── header.html
    ├── footer.html
    └── navigation.html
```

---

## Prochaines étapes

← [**Formulaires**](./05-forms.md) | [**ORM & Base de Données**](./07-orm.md) →
