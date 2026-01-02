# Guide des templates - Rusti Framework

Rusti utilise **Tera** comme moteur de templates avec un système de **préprocessing personnalisé** pour ajouter des tags Django-like.

## Table des matières

1. [Syntaxe Tera de base](#syntaxe-tera-de-base)
2. [Tags personnalisés Rusti](#tags-personnalisés-rusti)
3. [Préprocessing](#préprocessing)
4. [Contexte et variables](#contexte-et-variables)
5. [Héritage de templates](#héritage-de-templates)
6. [Exemples complets](#exemples-complets)

---

## Syntaxe Tera de base

Tera est un moteur de templates inspiré de Jinja2/Django.

### Variables

```html
{{ variable }}
{{ user.name }}
{{ user.age }}
```

### Filtres

```html
{{ text|upper }}
{{ text|lower }}
{{ text|truncate(length=100) }}
{{ date|date(format="%d/%m/%Y") }}
{{ price|floatformat(decimal_places=2) }}
```

### Conditions

```html
{% if user.is_authenticated %}
    <p>Bienvenue {{ user.username }} !</p>
{% else %}
    <p>Veuillez vous connecter.</p>
{% endif %}
```

### Boucles

```html
{% for post in posts %}
    <article>
        <h2>{{ post.title }}</h2>
        <p>{{ post.content }}</p>
    </article>
{% endfor %}
```

### Blocs et héritage

```html
{% extends "base.html" %}

{% block title %}Mon titre{% endblock %}

{% block content %}
    <p>Contenu de ma page</p>
{% endblock %}
```

---

## Tags personnalisés Rusti

Rusti ajoute des **tags personnalisés** via un système de préprocessing qui transforme les tags avant que Tera ne les traite.

### 1. Tag `{% static %}`

Génère l'URL d'un fichier statique.

**Syntaxe :**

```html
{% static 'chemin/vers/fichier' %}
```

**Exemples :**

```html
<!-- CSS -->
<link rel="stylesheet" href="{% static 'css/style.css' %}">

<!-- JavaScript -->
<script src="{% static 'js/app.js' %}"></script>

<!-- Images -->
<img src="{% static 'images/logo.png' %}" alt="Logo">

<!-- Fonts -->
<link rel="stylesheet" href="{% static 'fonts/custom-font.woff2' %}">
```

**Configuration (.env) :**

```env
STATIC_URL=/static/
STATIC_ROOT=static/
```

**Résultat après préprocessing :**

```html
<link rel="stylesheet" href="/static/css/style.css">
```

### 2. Tag `{% media %}`

Génère l'URL d'un fichier media (uploadé par les utilisateurs).

**Syntaxe :**

```html
{% media 'chemin/vers/fichier' %}
{% media variable %}
```

**Exemples :**

```html
<!-- Image uploadée -->
<img src="{% media user.avatar %}" alt="Avatar">

<!-- Document uploadé -->
<a href="{% media document.file %}">Télécharger le document</a>

<!-- Vidéo -->
<video src="{% media video.path %}" controls></video>
```

**Configuration (.env) :**

```env
MEDIA_URL=/media/
MEDIA_ROOT=media/
```

### 3. Tag `{% csrf %}`

Génère le champ caché du token CSRF.

**Syntaxe :**

```html
{% csrf %}
```

**Exemple :**

```html
<form method="post" action="/submit/">
    {% csrf %}

    <input type="text" name="username">
    <input type="password" name="password">
    <button type="submit">Se connecter</button>
</form>
```

**Résultat après préprocessing :**

```html
<input type="hidden" name="csrftoken" value="abc123...xyz789">
```

**⚠️ Important :** Le middleware `CsrfMiddleware` doit être activé pour que ce tag fonctionne.

### 4. Tag `{% messages %}`

Affiche les messages flash (success, error, warning, info).

**Syntaxe :**

```html
{% messages %}
```

**Exemple :**

```html
<body>
    <header>
        <h1>Mon App</h1>
    </header>

    <!-- Affichage automatique des messages -->
    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>
</body>
```

**Résultat généré :**

```html
<div class="messages">
    <div class="alert alert-success">Opération réussie !</div>
    <div class="alert alert-error">Une erreur s'est produite.</div>
</div>
```

**Utilisation dans les handlers :**

```rust
use rusti::prelude::*;

async fn create_user(
    Form(form): Form<UserForm>,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        let _ = message.error("Données invalides").await;
        return redirect("/register");
    }

    // Créer l'utilisateur...

    let _ = message.success("Compte créé avec succès !").await;
    redirect("/dashboard")
}
```

### 5. Tag `{% link %}`

Génère des URLs en utilisant le **reverse routing** (noms de routes).

**Syntaxe :**

```html
{% link 'nom_route' %}
{% link 'nom_route' param1=valeur1 param2=valeur2 %}
```

**Exemples :**

```html
<!-- Route sans paramètre -->
<a href="{% link 'index' %}">Accueil</a>

<!-- Route avec paramètre -->
<a href="{% link 'post_detail' id=post.id %}">Voir l'article</a>

<!-- Plusieurs paramètres -->
<a href="{% link 'user_post' user_id=user.id post_id=post.id %}">
    Article de cet utilisateur
</a>
```

**Configuration des routes :**

```rust
use rusti::prelude::*;

fn routes() -> Router {
    urlpatterns![
        path!("", index, "index"),
        path!("posts/<id>/", post_detail, "post_detail"),
        path!("users/<user_id>/posts/<post_id>/", user_post, "user_post"),
    ]
}
```

**Résultat après préprocessing :**

```html
<a href="/">Accueil</a>
<a href="/posts/42/">Voir l'article</a>
<a href="/users/10/posts/42/">Article de cet utilisateur</a>
```

### 6. Tag `{{ csp }}`

**⚠️ IMPORTANT : Génère le nonce CSP UNIQUEMENT si `use_nonce: true` dans la configuration CSP.**

**Syntaxe :**

```html
{{ csp }}
```

**Exemple :**

```html
<script nonce="{{ csp }}">
    // Code JavaScript inline
    console.log("Script autorisé avec nonce CSP");
</script>

<style nonce="{{ csp }}">
    /* CSS inline autorisé */
    body { background: #f0f0f0; }
</style>
```

**Configuration CSP avec nonce :**

```rust
use rusti::prelude::*;
use rusti::middleware::CspConfig;

let csp_config = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string()],
    use_nonce: true,  // ✅ Active la génération de nonce
    ..Default::default()
};

RustiApp::new(settings).await?
    .middleware(CspMiddleware::new(csp_config))
    .routes(routes())
    .run()
    .await?;
```

**Résultat après préprocessing :**

```html
<script nonce="abc123xyz789">
    console.log("Script autorisé");
</script>
```

**Si `use_nonce: false` :**

Le tag `{{ csp }}` générera une **chaîne vide** :

```html
<script nonce="">
    console.log("Pas de nonce généré");
</script>
```

**Quand utiliser le nonce CSP :**

| Cas d'usage | Utiliser nonce ? |
|-------------|------------------|
| Scripts inline nécessaires | ✅ Oui |
| Styles inline nécessaires | ✅ Oui |
| Uniquement scripts externes | ❌ Non (utilisez `script-src 'self'`) |
| Mode strict sans inline | ❌ Non |

**Bonnes pratiques :**

1. **Privilégier les fichiers externes** plutôt que les scripts inline
2. **Activer `use_nonce: true`** uniquement si vous avez du code inline
3. **Ne pas hardcoder les nonces** - toujours utiliser `{{ csp }}`
4. **Ajouter le nonce** sur chaque balise `<script>` et `<style>` inline

---

## Préprocessing

Rusti utilise un système de **préprocessing** qui transforme les tags personnalisés en tags Tera standards **avant** que Tera ne les traite.

### Ordre de traitement

1. **Préprocessing Rusti** → Transforme les tags personnalisés
2. **Tera** → Traite les tags Tera standards
3. **Rendu HTML** → Résultat final envoyé au client

### Exemple de transformation

**Template original :**

```html
<link rel="stylesheet" href="{% static 'css/style.css' %}">
<img src="{% media user.avatar %}" alt="Avatar">
<form method="post">
    {% csrf %}
    <button type="submit">Envoyer</button>
</form>
```

**Après préprocessing (avant Tera) :**

```html
<link rel="stylesheet" href="{{ settings.static_url }}css/style.css">
<img src="{{ settings.media_url }}{{ user.avatar }}" alt="Avatar">
<form method="post">
    {{ csrf_input() }}
    <button type="submit">Envoyer</button>
</form>
```

**Après Tera (HTML final) :**

```html
<link rel="stylesheet" href="/static/css/style.css">
<img src="/media/avatars/user123.jpg" alt="Avatar">
<form method="post">
    <input type="hidden" name="csrftoken" value="abc123...xyz">
    <button type="submit">Envoyer</button>
</form>
```

---

## Contexte et variables

### Passage de variables

```rust
use rusti::prelude::*;

async fn index(template: Template) -> Response {
    template.render("index.html", context! {
        title: "Bienvenue",
        username: "Alice",
        posts: vec![
            Post { id: 1, title: "Premier article".to_string() },
            Post { id: 2, title: "Second article".to_string() },
        ],
    })
}
```

### Variables globales

Certaines variables sont **automatiquement disponibles** dans tous les templates :

| Variable | Description |
|----------|-------------|
| `settings.static_url` | URL de base pour les fichiers statiques |
| `settings.media_url` | URL de base pour les fichiers media |
| `csrf_token()` | Fonction pour générer le token CSRF |
| `csrf_input()` | Fonction pour générer le champ hidden CSRF |
| `get_messages()` | Fonction pour récupérer les messages flash |
| `csp_nonce()` | Fonction pour récupérer le nonce CSP |

**Exemple :**

```html
<!-- Accès aux settings -->
<p>Static URL : {{ settings.static_url }}</p>
<p>Media URL : {{ settings.media_url }}</p>

<!-- CSRF token -->
{{ csrf_input() }}

<!-- Messages flash -->
{% for msg in get_messages() %}
    <div class="alert">{{ msg.message }}</div>
{% endfor %}

<!-- CSP nonce (si use_nonce: true) -->
<script nonce="{{ csp_nonce() }}">
    console.log("Script avec nonce");
</script>
```

---

## Héritage de templates

### Template de base (templates/base.html)

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Mon App{% endblock %}</title>

    <link rel="stylesheet" href="{% static 'css/style.css' %}">
    {% block extra_css %}{% endblock %}
</head>
<body>
    <header>
        <h1>Mon Application</h1>
        <nav>
            <a href="{% link 'index' %}">Accueil</a>
            <a href="{% link 'post_list' %}">Articles</a>
            <a href="{% link 'contact' %}">Contact</a>
        </nav>
    </header>

    <!-- Messages flash -->
    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>
        <p>© 2026 Mon App</p>
    </footer>

    <script src="{% static 'js/app.js' %}"></script>
    {% block extra_js %}{% endblock %}
</body>
</html>
```

### Template enfant (templates/posts/list.html)

```html
{% extends "base.html" %}

{% block title %}Articles{% endblock %}

{% block extra_css %}
<link rel="stylesheet" href="{% static 'css/posts.css' %}">
{% endblock %}

{% block content %}
<h2>Tous les articles</h2>

{% for post in posts %}
<article>
    <h3>{{ post.title }}</h3>
    <p>{{ post.content|truncate(length=200) }}</p>
    <a href="{% link 'post_detail' id=post.id %}">Lire la suite</a>
</article>
{% endfor %}

{% if posts|length == 0 %}
<p>Aucun article pour le moment.</p>
{% endif %}
{% endblock %}

{% block extra_js %}
<script src="{% static 'js/posts.js' %}"></script>
{% endblock %}
```

---

## Exemples complets

### Exemple 1 : Formulaire de connexion

```html
{% extends "base.html" %}

{% block title %}Connexion{% endblock %}

{% block content %}
<h2>Connexion</h2>

<form method="post" action="/login/">
    {% csrf %}

    <div class="form-group">
        <label for="username">Nom d'utilisateur</label>
        <input type="text" id="username" name="username" required>
    </div>

    <div class="form-group">
        <label for="password">Mot de passe</label>
        <input type="password" id="password" name="password" required>
    </div>

    <button type="submit">Se connecter</button>
</form>

<p>
    <a href="{% link 'register' %}">Créer un compte</a> |
    <a href="{% link 'password_reset' %}">Mot de passe oublié ?</a>
</p>
{% endblock %}
```

### Exemple 2 : Profil utilisateur avec avatar

```html
{% extends "base.html" %}

{% block title %}Profil de {{ user.username }}{% endblock %}

{% block content %}
<div class="profile">
    <div class="profile-header">
        {% if user.avatar %}
            <img src="{% media user.avatar %}" alt="Avatar de {{ user.username }}">
        {% else %}
            <img src="{% static 'images/default-avatar.png' %}" alt="Avatar par défaut">
        {% endif %}

        <h2>{{ user.username }}</h2>
        <p>{{ user.bio|default(value="Aucune biographie") }}</p>
    </div>

    <div class="profile-stats">
        <p>Inscrit depuis : {{ user.created_at|date(format="%d/%m/%Y") }}</p>
        <p>Articles publiés : {{ user.posts|length }}</p>
    </div>

    {% if user.id == current_user.id %}
        <a href="{% link 'profile_edit' %}" class="btn">Modifier mon profil</a>
    {% endif %}
</div>
{% endblock %}
```

### Exemple 3 : Liste de posts avec pagination

```html
{% extends "base.html" %}

{% block title %}Articles - Page {{ page }}{% endblock %}

{% block content %}
<h2>Articles récents</h2>

{% for post in posts %}
<article class="post">
    <h3>
        <a href="{% link 'post_detail' id=post.id %}">{{ post.title }}</a>
    </h3>

    <div class="post-meta">
        Par {{ post.author.username }} le {{ post.created_at|date(format="%d/%m/%Y") }}
    </div>

    <p>{{ post.content|truncate(length=300) }}</p>

    <a href="{% link 'post_detail' id=post.id %}" class="read-more">
        Lire la suite →
    </a>
</article>
{% endfor %}

<!-- Pagination -->
<div class="pagination">
    {% if has_previous %}
        <a href="{% link 'post_list' %}?page={{ page - 1 }}">← Précédent</a>
    {% endif %}

    <span>Page {{ page }} sur {{ total_pages }}</span>

    {% if has_next %}
        <a href="{% link 'post_list' %}?page={{ page + 1 }}">Suivant →</a>
    {% endif %}
</div>
{% endblock %}
```

### Exemple 4 : Formulaire auto-généré

```html
{% extends "base.html" %}

{% block title %}Créer un article{% endblock %}

{% block content %}
<h2>Nouvel article</h2>

<form method="post">
    {% csrf %}

    <!-- Rendu automatique du formulaire -->
    {{ form }}

    <!-- Ou rendu manuel champ par champ -->
    <!--
    <div class="form-group">
        <label>{{ form.title.label }}</label>
        {{ form.title }}
        {% if form.title.errors %}
            <div class="errors">
                {% for error in form.title.errors %}
                    <span class="error">{{ error }}</span>
                {% endfor %}
            </div>
        {% endif %}
    </div>
    -->

    <button type="submit">Publier</button>
</form>
{% endblock %}
```

### Exemple 5 : Scripts inline avec CSP nonce

```html
{% extends "base.html" %}

{% block title %}Dashboard{% endblock %}

{% block content %}
<h2>Tableau de bord</h2>

<div id="chart"></div>

{% endblock %}

{% block extra_js %}
<!-- Script externe (pas besoin de nonce) -->
<script src="{% static 'js/chart.min.js' %}"></script>

<!-- Script inline (nécessite nonce si CSP strict) -->
<script nonce="{{ csp }}">
    // Code JavaScript inline
    const data = {{ chart_data|json_encode|safe }};

    new Chart(document.getElementById('chart'), {
        type: 'bar',
        data: data,
    });
</script>
{% endblock %}
```

---

## Filtres personnalisés Rusti

En plus des filtres Tera standards, Rusti peut ajouter des filtres personnalisés :

### Filtre `json_encode`

```html
<script nonce="{{ csp }}">
    const config = {{ config|json_encode|safe }};
</script>
```

### Filtre `slugify`

```html
<a href="/posts/{{ post.title|slugify }}/">{{ post.title }}</a>
```

### Filtre `markdown`

```html
<div class="content">
    {{ post.content|markdown|safe }}
</div>
```

---

## Bonnes pratiques

### 1. Utilisez l'héritage de templates

```html
<!-- Bon -->
{% extends "base.html" %}
{% block content %}...{% endblock %}

<!-- Mauvais : duplication de code -->
<!DOCTYPE html>
<html>
<head>...</head>
<body>...</body>
</html>
```

### 2. Nommez vos routes pour le reverse routing

```rust
// Bon
path!("posts/<id>/", detail_post, "post_detail")

// Moins bon (pas de nom)
path!("posts/<id>/", detail_post)
```

### 3. Échappez les variables utilisateur

```html
<!-- Bon (échappement automatique) -->
<p>{{ user.bio }}</p>

<!-- Dangereux (désactive l'échappement) -->
<p>{{ user.bio|safe }}</p>
```

### 4. Organisez vos templates

```
templates/
├── base.html
├── components/
│   ├── header.html
│   ├── footer.html
│   └── pagination.html
├── posts/
│   ├── list.html
│   ├── detail.html
│   └── create.html
└── users/
    ├── profile.html
    └── settings.html
```

### 5. Utilisez des includes pour les composants réutilisables

```html
<!-- templates/components/pagination.html -->
<div class="pagination">
    {% if has_previous %}
        <a href="?page={{ page - 1 }}">← Précédent</a>
    {% endif %}
    <span>Page {{ page }}</span>
    {% if has_next %}
        <a href="?page={{ page + 1 }}">Suivant →</a>
    {% endif %}
</div>

<!-- Utilisation -->
{% include "components/pagination.html" %}
```

---

## Voir aussi

- [Guide de démarrage](GETTING_STARTED.md)
- [Sécurité (CSP)](SECURITY.md)
- [Formulaires](FORMULAIRE.md)
- [Documentation Tera](https://keats.github.io/tera/)

Créez des templates puissants et sécurisés avec Rusti !

---

**Version:** 1.0 (Corrigée - 2 Janvier 2026)
**Licence:** MIT
