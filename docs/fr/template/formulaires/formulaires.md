# Formulaires, contexte & pièges

## Gestion des erreurs de formulaire

### Affichage automatique (via {% form.xxx %})

Quand vous utilisez `{% form.signup_form %}`, les erreurs de validation sont **rendues automatiquement** sous chaque champ concerné.

### Affichage manuel des erreurs globales

```html
{% if signup_form.errors %}
    <div class="alert alert-warning">
        <ul>
            {% for field_name, error_msg in signup_form.errors %}
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
pub async fn signup(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form("signup");

    let validated = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "title" => "Inscription", "signup_form" => &form });
            // Premier affichage (GET) : pas soumis, pas de message d'erreur
            if form.is_submitted() {
                context_update!(request => {
                    "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
                });
            }
            return request.render("signup.html");
        }
    };

    match validated.save(&request.engine.db).await {
        Ok(_) => {
            success!(request.notices => "Bienvenue !");
            Ok(Redirect::to("/").into_response())
        }
        Err(form) => {
            context_update!(request => {
                "title" => "Inscription",
                "signup_form" => &form,
                "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
            });
            request.render("signup.html")
        }
    }
}
```

```html
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}
    <form method="post" action="/inscription">
        {% form.signup_form %}
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
| `dev` | `bool` | `true` avec `RUNIQUE_ENV=dev` |
| `current_user` | objet | Compte connecté (`username`, `id`, rang…), absent pour un visiteur anonyme |
| `current_path` | `String` | Chemin de l'URL courante, sans query string |

---

## Piège courant : collision de noms de variables

Quand vous utilisez `{% form.user %}`, la variable `user` **doit être un formulaire** :

```rust
// ❌ ERREUR : "user" est un Model SeaORM, pas un formulaire
context_update!(request => {
    "user" => &db_user,  // le filtre form va crasher !
});

// ✅ CORRECT : séparer le formulaire et l'entité DB
context_update!(request => {
    "user" => &form,           // formulaire → {% form.user %} fonctionne
    "found_user" => &db_user,  // Model → {{ found_user.email }}
});
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Tags Django-like](/docs/fr/template/tags) | `{% form.xxx %}`, `{% csrf %}` |
| [Filtres & fonctions](/docs/fr/template/filtres) | Filtres bas niveau |

## Retour au sommaire

- [Templates](/docs/fr/template)
