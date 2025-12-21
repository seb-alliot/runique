# Balises de Template Rusti

Rusti supporte des balises Django-like qui sont transformées en syntaxe Tera native au chargement des templates.

## Balises disponibles

### {% static "path" %}

Charge un fichier statique (CSS, JS, images du dossier static).

**Syntaxe :**
```html
{% static "chemin/vers/fichier" %}
```

**Exemples :**
```html
<!-- CSS -->
<link rel="stylesheet" href='{% static "css/main.css" %}'>

<!-- JavaScript -->
<script src='{% static "js/app.js" %}'></script>

<!-- Images statiques -->
<img src='{% static "images/logo.png" %}' alt="Logo">

<!-- Fonts -->
<link rel="preload" href='{% static "fonts/roboto.woff2" %}' as="font">
```

**Transformation :**
```
{% static "css/main.css" %}  →  {{ "css/main.css" | static }}
```

---

### {% media "path" %}

Charge un fichier média uploadé par les utilisateurs.

**Syntaxe :**
```html
{% media "chemin/vers/fichier" %}
```

**Exemples :**
```html
<!-- Avatar utilisateur -->
<img src='{% media "avatars/user123.jpg" %}' alt="Avatar">

<!-- Document uploadé -->
<a href='{% media "documents/report.pdf" %}'>Télécharger le rapport</a>

<!-- Vidéo -->
<video src='{% media "videos/demo.mp4" %}' controls></video>
```

**Transformation :**
```
{% media "avatars/user.jpg" %}  →  {{ "avatars/user.jpg" | media }}
```

---

### {% csrf %}

Insère le token CSRF dans un formulaire pour la protection contre les attaques CSRF.

**Syntaxe :**
```html
{% csrf %}
```

**Exemples :**
```html
<!-- Formulaire POST -->
<form method="post" action="/submit">
    {% csrf %}
    <input type="text" name="message">
    <button type="submit">Envoyer</button>
</form>

<!-- Formulaire de connexion -->
<form method="post" action="/login">
    {% csrf %}
    <input type="email" name="email" required>
    <input type="password" name="password" required>
    <button type="submit">Se connecter</button>
</form>
```

**Transformation :**
```
{% csrf %}  →  {% include "csrf" %}
```

**Note :** Le middleware CSRF doit être activé dans votre application :
```rust
app.with_csrf_tokens()
```

---

### {% messages %}

Affiche les messages flash (success, error, info).

**Syntaxe :**
```html
{% messages %}
```

**Exemples :**
```html
<!-- Dans le header -->
<header>
    {% messages %}
    <h1>Mon Application</h1>
</header>

<!-- Zone dédiée -->
<div class="alerts-container">
    {% messages %}
</div>

<!-- Dans un bloc -->
{% block notifications %}
    {% messages %}
{% endblock %}
```

**Transformation :**
```
{% messages %}  →  {% include "message" %}
```

**Note :** Le middleware flash doit être activé :
```rust
app.with_flash_messages()
```

**Utilisation dans les handlers :**
```rust
async fn my_handler(mut messages: Message) -> Response {
    let _ = messages.success("Opération réussie !").await;
    let _ = messages.error("Une erreur est survenue").await;
    let _ = messages.info("Information importante").await;
    // ...
}
```

---

### {% link "route_name", params %}

Génère une URL via reverse routing (résolution inverse des routes).

**Syntaxe :**
```html
<!-- Route simple -->
{% link "nom_route" %}

<!-- Route avec paramètres -->
{% link "nom_route", param1=value1, param2=value2 %}
```

**Exemples :**

```html
<!-- Navigation simple -->
<nav>
    <a href='{% link "home" %}'>Accueil</a>
    <a href='{% link "about" %}'>À propos</a>
    <a href='{% link "contact" %}'>Contact</a>
</nav>

<!-- Lien avec paramètres -->
<a href='{% link "user_profile", id=42 %}'>Voir le profil</a>

<!-- Lien avec plusieurs paramètres -->
<a href='{% link "post_detail", id=post.id, slug=post.slug %}'>
    Lire l'article
</a>

<!-- Dans un formulaire -->
<form method="post" action='{% link "submit_form" %}'>
    {% csrf %}
    <button type="submit">Envoyer</button>
</form>

<!-- Bouton de suppression -->
<a href='{% link "delete_item", id=item.id %}' 
   class="btn-danger"
   onclick="return confirm('Êtes-vous sûr ?')">
    Supprimer
</a>
```

**Transformation :**
```
{% link "home" %}  →  {{ link(link='home') }}

{% link "user_profile", id=42 %}  →  {{ link(link='user_profile', id=42) }}
```

**Définition des routes :**
```rust
use rusti::urlpatterns;

let routes = urlpatterns![
    "/" => get(home), name = "home",
    "/about" => get(about), name = "about",
    "/user/{id}" => get(user_profile), name = "user_profile",
    "/post/{id}/{slug}" => get(post_detail), name = "post_detail",
];
```

---

## Comment ça marche ?

### Preprocessing des templates

Les balises personnalisées sont transformées **AVANT** le parsing par Tera, lors du chargement des templates :

1. **Lecture des fichiers** `.html` dans `templates/`
2. **Transformation regex** des balises personnalisées
3. **Ajout à Tera** avec la syntaxe native

```rust
// Exemple de transformation interne
{% static "file.css" %}     →  {{ "file.css" | static }}
{% csrf %}                   →  {% include "csrf" %}
{% link "home" %}            →  {{ link(link='home') }}
```

### Avantages de cette approche

✅ **Compatibilité** : Utilise les capacités natives de Tera  
✅ **Performance** : Transformation une seule fois au chargement  
✅ **Maintenabilité** : Pas de custom parser compliqué  
✅ **Familiarité** : Syntaxe proche de Django  
✅ **Pas de runtime overhead** : Tout est fait au démarrage  

---

## Balises Tera natives toujours disponibles

Toutes les fonctionnalités Tera restent disponibles :

```html
<!-- Variables -->
{{ user.name }}
{{ product.price }}

<!-- Filtres -->
{{ title | upper }}
{{ content | safe }}
{{ date | date(format="%Y-%m-%d") }}

<!-- Conditions -->
{% if user.is_admin %}
    <button>Admin Panel</button>
{% endif %}

<!-- Boucles -->
{% for item in items %}
    <li>{{ item.name }}</li>
{% endfor %}

<!-- Héritage -->
{% extends "base.html" %}
{% block content %}
    <!-- contenu -->
{% endblock %}

<!-- Includes -->
{% include "header.html" %}
```

---

## Exemple complet

### Template `base.html`

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <title>{% block title %}Mon Site{% endblock %}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
    {% block extra_css %}{% endblock %}
</head>
<body>
    {% if debug %}
    <div class="debug-banner">
        🔧 Mode Debug Activé
    </div>
    {% endif %}

    <header>
        <img src='{% static "images/logo.png" %}' alt="Logo">
        <nav>
            <a href='{% link "home" %}'>Accueil</a>
            <a href='{% link "about" %}'>À propos</a>
            <a href='{% link "contact" %}'>Contact</a>
        </nav>
    </header>

    <main>
        {% messages %}
        
        {% block content %}
        <!-- Contenu par défaut -->
        {% endblock %}
    </main>

    <footer>
        <p>&copy; 2025 Mon Application</p>
    </footer>

    <script src='{% static "js/main.js" %}'></script>
    {% block extra_js %}{% endblock %}
</body>
</html>
```

### Template `user_profile.html`

```html
{% extends "base.html" %}

{% block title %}Profil de {{ user.name }}{% endblock %}

{% block content %}
<div class="profile">
    <img src='{% media user.avatar %}' alt="Avatar" class="avatar">
    
    <h1>{{ user.name }}</h1>
    <p>{{ user.bio }}</p>
    
    <form method="post" action='{% link "update_profile", id=user.id %}'>
        {% csrf %}
        
        <input type="text" name="name" value="{{ user.name }}">
        <textarea name="bio">{{ user.bio }}</textarea>
        
        <button type="submit">Mettre à jour</button>
    </form>
    
    <a href='{% link "user_list" %}'>← Retour à la liste</a>
</div>
{% endblock %}
```

---

## Configuration

### Structure des dossiers

```
mon-projet/
├── src/
│   ├── templates/        # Templates utilisateur
│   │   ├── base.html
│   │   ├── index.html
│   │   └── ...
│   ├── static/           # Fichiers statiques
│   │   ├── css/
│   │   ├── js/
│   │   └── images/
│   └── media/            # Fichiers uploadés
│       └── ...
```

### Settings

```rust
use rusti::Settings;

let settings = Settings::builder()
    .templates_dir(vec!["src/templates".to_string()])
    .staticfiles_dirs("src/static")
    .media_root("src/media")
    .static_url("/static")
    .media_url("/media")
    .build();
```

### Application complète

```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();
    
    let routes = urlpatterns![
        "/" => get(index), name = "home",
        "/about" => get(about), name = "about",
        "/user/{id}" => get(user_profile), name = "user_profile",
    ];
    
    RustiApp::new(settings).await?
        .routes(routes)
        .with_static_files()?
        .with_default_middleware()
        .with_flash_messages()    // Active {% messages %}
        .with_csrf_tokens()       // Active {% csrf %}
        .run()
        .await?;
    
    Ok(())
}
```

---

## Limitations connues

### Guillemets

Utilisez des guillemets simples `'` ou doubles `"` de manière cohérente :

```html
✅ {% static "file.css" %}
✅ {% static 'file.css' %}
❌ {% static file.css %}     <!-- Sans guillemets -->
```

### Espaces

Des espaces supplémentaires sont tolérés :

```html
✅ {% static "file.css" %}
✅ {%  static  "file.css"  %}
```

### Variables dans les balises

Les variables ne sont pas supportées dans les balises personnalisées, utilisez la syntaxe Tera native :

```html
❌ {% static my_var %}
✅ {{ my_var | static }}

❌ {% link route_name %}
✅ {{ link(link=route_name) }}
```

---

## Dépannage

### "Template not found"

Vérifiez que votre template est dans le dossier configuré :
- Chemin configuré dans `Settings::templates_dir`
- Fichier avec extension `.html`

### "Route not found" avec {% link %}

Assurez-vous que la route est enregistrée avec un nom :

```rust
urlpatterns![
    "/user/{id}" => get(handler), name = "user_profile",
    //                            ^^^^^^^^^^^^^^^^^^^^^^
];
```

### Token CSRF manquant

Activez le middleware CSRF :

```rust
app.with_csrf_tokens()
```

### Messages flash ne s'affichent pas

Activez le middleware flash :

```rust
app.with_flash_messages()
```

---

## Voir aussi

- [Documentation Tera](https://keats.github.io/tera/)
- [Guide du routing Rusti](./ROUTING.md)
- [Middleware Rusti](./MIDDLEWARE.md)
- [Exemples complets](../examples/)
