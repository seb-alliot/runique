# Formulaires, contexte & pièges

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
pub async fn inscription(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form();

    let mut validated = match ValidationForm::try_new(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => {
                "title" => "Inscription",
                "inscription_form" => &form,
                "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
            });
            return request.render("inscription.html");
        }
    };

    let user = validated.save(&request.engine.db).await.map_err(|err| {
        validated.database_error(&err);
        AppError::from(err)
    })?;
    success!(request.notices => format!("Bienvenue {} !", user.username));
    Ok(Redirect::to("/").into_response())
}
```

```html
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
