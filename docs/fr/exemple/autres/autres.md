# Autres exemples

## Flash messages — tous les types

```rust
pub async fn demo_messages(mut request: Request) -> AppResult<Response> {
    success!(request.notices => "Ceci est un message de succès.");
    info!(request.notices => "Ceci est un message d'information.");
    warning!(request.notices => "Ceci est un message d'avertissement.");
    error!(request.notices => "Ceci est un message d'erreur.");

    context_update!(request => {
        "title" => "Démo messages",
    });
    request.render("demo.html")
}
```

```html
{% extends "base.html" %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}
    <p>Les messages ci-dessus viennent de la session flash.</p>
{% endblock %}
```

---

## API REST

### Routes API

```rust
pub fn routes() -> Router {
    urlpatterns! {
        "/api/users" => view!{ api_list_users }, name = "api_users",
    }
}
```

### Handler API JSON

```rust
use axum::Json;
use serde_json::json;

pub async fn api_list_users(request: Request) -> AppResult<Response> {
    let users = users::Entity::find()
        .all(&*request.engine.db)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "count": users.len(),
        "data": users
    })).into_response())
}
```

---

## Template de base complet

```html
<!-- templates/base.html -->
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Mon App{% endblock %}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
    {% block extra_css %}{% endblock %}
</head>
<body>
    <header>
        <nav>
            <a href='{% link "index" %}'>Accueil</a>
            <a href='{% link "about" %}'>À propos</a>
            <a href='{% link "inscription" %}'>Inscription</a>
        </nav>
    </header>

    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>
        <p>&copy; 2026 — Propulsé par Runique</p>
    </footer>

    {% block extra_js %}{% endblock %}
</body>
</html>
```

---

## Résumé des patterns

| Pattern | Quand l'utiliser |
|---------|-----------------|
| `request.render("template.html")` | Rendu HTML standard |
| `Redirect::to("/").into_response()` | Après une action réussie (POST) |
| `context_update!(request => {...})` | Injecter des variables dans le template |
| `success!(request.notices => "...")` | Message flash avant redirect |
| `flash_now!(error => "...")` | Message immédiat (pas de redirect) |
| `form.is_valid().await` | Valider un formulaire |
| `form.save(&db).await` | Sauvegarder en base de données |
| `form.get_form_mut().database_error(&err)` | Afficher une erreur DB dans le formulaire |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application minimale](/docs/fr/exemple/minimal) | Point de départ simple |
| [Formulaires](/docs/fr/exemple/formulaires) | CRUD avec formulaires |
| [Upload](/docs/fr/exemple/upload) | Upload de fichier |

## Retour au sommaire

- [Exemples](/docs/fr/exemple)
