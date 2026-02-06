# 🎨 Templates

## Moteur Tera

Runique utilise **Tera** comme moteur de templates, avec une couche de syntaxe inspirée de Django. Les templates sont écrits en HTML standard enrichi de balises Tera et de **tags Django-like** que Runique transforme automatiquement.

```rust
use runique::prelude::*;

pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Bienvenue sur Runique",
        "description" => "Un framework web moderne",
    });

    request.render("index.html")
}
```

---

## L'objet Request

Chaque handler reçoit un `Request` qui contient tout le nécessaire :

```rust
pub struct Request {
    pub engine: AEngine,       // Arc<RuniqueEngine> (DB, Tera, Config)
    pub session: Session,      // Session tower-sessions
    pub notices: Message,      // Flash messages
    pub csrf_token: CsrfToken, // Token CSRF
    pub context: Context,      // Contexte Tera
    pub method: Method,        // Méthode HTTP
}
```

**Méthodes principales :**

| Méthode | Description |
|---------|-------------|
| `request.render("page.html")` | Rendu du template avec le contexte courant |
| `request.is_get()` | Vérifie si la méthode est GET |
| `request.is_post()` | Vérifie si la méthode est POST |
| `request.is_put()` | Vérifie si la méthode est PUT |
| `request.is_delete()` | Vérifie si la méthode est DELETE |

> 💡 **Hot-reload** : En mode `DEBUG=true`, les templates sont rechargés à chaque requête (pas de cache Tera).

---

## Macro context_update!

Syntaxe simplifiée pour injecter des variables dans le contexte du template :

```rust
context_update!(request => {
    "title" => "Ma page",
    "user" => &form,
    "items" => &vec!["a", "b", "c"],
});

request.render("ma_page.html")
```

Chaque paire `"clé" => valeur` appelle `request.context.insert("clé", &valeur)`.

---

## Tags Django-like (Syntaxe sucrée)

Runique pré-traite les templates pour transformer une syntaxe Django-like en syntaxe Tera standard. Vous pouvez utiliser les deux formes, mais les **tags Django-like** sont recommandés pour la lisibilité.

### {% static %} — Assets statiques

```html
<link rel="stylesheet" href='{% static "css/main.css" %}'>
<script src='{% static "js/app.js" %}'></script>
<img src='{% static "images/logo.png" %}' alt="Logo">
```

**Transformé en :** `{{ "css/main.css" | static }}`

**Résultat :** `/static/css/main.css`

---

### {% media %} — Fichiers médias (uploads)

```html
<img src='{% media "avatars/photo.jpg" %}' alt="Photo de profil">
<a href='{% media "documents/cv.pdf" %}'>Télécharger le CV</a>
```

**Transformé en :** `{{ "avatars/photo.jpg" | media }}`

**Résultat :** `/media/avatars/photo.jpg`

---

### {% csrf %} — Protection CSRF

```html
<form method="post" action="/inscription">
    {% csrf %}
    <!-- Génère automatiquement un <input type="hidden" name="csrf_token" value="..."> -->
    <!-- + le script de validation JS -->

    <button type="submit">Envoyer</button>
</form>
```

**Transformé en :** `{% include "csrf/csrf_field.html" %}`

> ⚠️ **Non nécessaire** dans les formulaires Runique (`{% form.xxx %}`) — le token CSRF est injecté automatiquement. N'utilisez `{% csrf %}` que pour des formulaires HTML écrits manuellement.

---

### {% messages %} — Flash messages

```html
{% messages %}
```

**Transformé en :** `{% include "message/message_include.html" %}`

Affiche automatiquement tous les messages flash (succès, erreur, info, warning) avec les classes CSS correspondantes. Voir le [guide Flash Messages](09-flash-messages.md) pour plus de détails.

---

### {% csp_nonce %} — Nonce CSP

```html
<script {% csp_nonce %}>
    console.log("Script sécurisé avec nonce CSP");
</script>
```

**Transformé en :** `{% include "csp/csp_nonce.html" %}`

Injecte l'attribut `nonce="..."` sur une balise `<script>` ou `<style>` pour la politique CSP (Content Security Policy).

---

### {% link %} — Liens vers des routes nommées

```html
<a href='{% link "index" %}'>Accueil</a>
<a href='{% link "user_detail" id="42" %}'>Profil utilisateur</a>
```

**Transformé en :** `{{ link(link='index') }}`

Résout le nom d'une route enregistrée dans le registre d'URLs (voir [Routage](04-routing.md) et la macro `urlpatterns!`).

---

### {% form.xxx %} — Rendu de formulaire complet

```html
<form method="post" action="/inscription">
    {% form.inscription_form %}
    <button type="submit">S'inscrire</button>
</form>
```

**Transformé en :** `{{ inscription_form | form | safe }}`

Rend l'intégralité du formulaire : tous les champs HTML, les erreurs de validation, le token CSRF, et les scripts JS nécessaires.

---

### {% form.xxx.champ %} — Rendu d'un champ isolé

```html
<form method="post" action="/inscription">
    <div class="row">
        <div class="col">{% form.inscription_form.username %}</div>
        <div class="col">{% form.inscription_form.email %}</div>
    </div>
    <div class="row">
        {% form.inscription_form.password %}
    </div>
    <button type="submit">S'inscrire</button>
</form>
```

**Transformé en :** `{{ inscription_form | form(field='username') | safe }}`

Rend un seul champ du formulaire. Les scripts JS sont automatiquement injectés après le **dernier champ** rendu.

---

## Filtres Tera

Les filtres sont la forme « bas-niveau » des tags Django-like. Vous pouvez les utiliser directement dans la syntaxe Tera standard si vous préférez.

### Filtres d'assets

| Filtre | Description | Exemple |
|--------|-------------|---------|
| `static` | Préfixe URL statique de l'app | `{{ "css/main.css" \| static }}` |
| `media` | Préfixe URL média de l'app | `{{ "photo.jpg" \| media }}` |
| `runique_static` | Assets statiques internes au framework | `{{ "css/error.css" \| runique_static }}` |
| `runique_media` | Médias internes au framework | `{{ "logo.png" \| runique_media }}` |

### Filtre de formulaire

| Filtre | Description | Exemple |
|--------|-------------|---------|
| `form` | Rendu complet du formulaire | `{{ mon_form \| form \| safe }}` |
| `form(field='xxx')` | Rendu d'un seul champ | `{{ mon_form \| form(field='email') \| safe }}` |
| `csrf_field` | Génère un input hidden CSRF | `{{ csrf_token \| csrf_field \| safe }}` |

### Fonctions Tera

| Fonction | Description | Exemple |
|----------|-------------|---------|
| `csrf()` | Génère un champ CSRF depuis le contexte | `{{ csrf() }}` |
| `nonce()` | Retourne le nonce CSP | `{{ nonce() }}` |
| `link(link='...')` | Résolution d'URL nommée | `{{ link(link='index') }}` |

---

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

## Boucles et Conditions

### Boucles

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

### Conditions

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

## Macros Tera (dans les templates)

Les macros Tera permettent de créer des composants réutilisables :

```html
{% macro render_user(user) %}
    <div class="user-card">
        <h3>{{ user.name }}</h3>
        <p>{{ user.email }}</p>
    </div>
{% endmacro %}

<!-- Utilisation : -->
{% for u in users %}
    {{ self::render_user(user=u) }}
{% endfor %}
```

---

## Gestion des erreurs de formulaire

### Affichage automatique (via {% form.xxx %})

Quand vous utilisez `{% form.inscription_form %}`, les erreurs de validation sont **rendues automatiquement** sous chaque champ concerné.

### Affichage manuel des erreurs globales

```html
{% if inscription_form.errors %}
    <div class="alert alert-warning">
        <ul>
            {% for field_name, error_msg in inscription_form.errors %}
                <li><strong>{{ field_name }} :</strong> {{ error_msg }}</li>
            {% endfor %}
        </ul>
    </div>
{% endif %}
```

---

## Exemple complet : page avec formulaire

```rust
// Handler Rust
pub async fn inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_get() {
        context_update!(request => {
            "title" => "Inscription",
            "inscription_form" => &form,
        });
        return request.render("inscription.html");
    }

    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;
            success!(request.notices => format!("Bienvenue {} !", user.username));
            return Ok(Redirect::to("/").into_response());
        }

        context_update!(request => {
            "title" => "Erreur de validation",
            "inscription_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render("inscription.html");
    }

    request.render("inscription.html")
}
```

```html
<!-- Template inscription.html -->
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
    <h1>{{ title }}</h1>

    {% messages %}

    <form method="post" action="/inscription">
        {% form.inscription_form %}
        <button type="submit" class="btn btn-primary">S'inscrire</button>
    </form>
{% endblock %}
```

---

## Variables auto-injectées

Ces variables sont automatiquement disponibles dans tous les templates :

| Variable | Type | Description |
|----------|------|-------------|
| `csrf_token` | `String` | Token CSRF de la session |
| `csp_nonce` | `String` | Nonce CSP pour les scripts/styles inline |
| `messages` | `Vec<FlashMessage>` | Messages flash de la session précédente |
| `debug` | `bool` | Mode debug actif ou non |

---

## ⚠️ Piège courant : collision de noms de variables

Quand vous utilisez `{% form.user %}` dans un template, Tera applique le filtre `form` sur la variable `user` du contexte. **Cette variable doit être un formulaire Prisme**, pas un objet quelconque.

```rust
// ❌ ERREUR : "user" est un Model SeaORM, pas un formulaire
context_update!(request => {
    "user" => &db_user,  // users::Model → le filtre form va crasher !
});

// ✅ CORRECT : séparer le formulaire et l'entité DB
context_update!(request => {
    "user" => &form,           // UsernameForm (Prisme) → {% form.user %} fonctionne
    "found_user" => &db_user,  // users::Model → {{ found_user.email }}
});
```

---

## Prochaines étapes

← [**Forms**](https://github.com/seb-alliot/runique/blob/main/docs/fr/05-forms.md) | [**ORM & Database**](https://github.com/seb-alliot/runique/blob/main/docs/fr/07-orm.md) →
