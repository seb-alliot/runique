# Autres exemples

## Flash messages — tous les types

Les macros `success!`, `info!`, `warning!`, `error!` stockent le message en session : il s'affiche à la page **suivante**, donc après une redirection.

```rust
pub async fn demo_messages(request: Request) -> AppResult<Response> {
    success!(request.notices => "Ceci est un message de succès.");
    info!(request.notices => "Ceci est un message d'information.");
    warning!(request.notices => "Ceci est un message d'avertissement.");
    error!(request.notices => "Ceci est un message d'erreur.");
    Ok(Redirect::to("/").into_response())
}
```

Pour un message sur la page rendue par la même requête (formulaire ré-affiché après une erreur), utiliser `flash_now!` :

```rust
context_update!(request => {
    "messages" => flash_now!(warning => "Vérifiez les champs."),
});
request.render("page.html")
```

`{% messages %}` affiche la clé `messages` du contexte : les messages de session arrivés avec la requête, ou ceux de `flash_now!` si vous la remplacez. Remplacée, elle perd les messages de session de cette requête.

---

## API REST

### Routes API

```rust
pub fn routes() -> Routes {
    urlpatterns! {
        "/api/articles" => view!{ api_list_articles }, name = "api_articles",
    }
}
```

### Handler API JSON

```rust
use runique::prelude::*;
use runique::prelude::serde_json::json;

pub async fn api_list_articles(request: Request) -> AppResult<Response> {
    let articles = article::Entity::find()
        .all(&*request.engine.db)
        .await?;

    Ok(Json(json!({
        "count": articles.len(),
        "data": articles
    })).into_response())
}
```

`data` sérialise chaque colonne du modèle. Ne faites jamais ça sur une table qui contient un secret (les comptes : hash du mot de passe) : sélectionnez les colonnes à exposer, ou passez par une struct dédiée.

Un POST JSON est refusé en 403 par le middleware CSRF sans en-tête `X-CSRF-Token` valide. Un POST de formulaire HTML que la vue traite sans `request.form` (déconnexion, action) doit vérifier `request.prisme.csrf_valid` lui-même (voir [Formulaires](/docs/fr/exemple/formulaires)).

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
            <a href='{% link "signup" %}'>Inscription</a>
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
| `ValidationForm::try_form(form, &request).await` | Valider un formulaire (seulement s'il a été envoyé) |
| `validated.save(&db).await` | Sauvegarder en base (pas pour un compte : voir [Formulaires](/docs/fr/exemple/formulaires)) |
| `form.forms_mut().database_error(&err)` | Afficher une erreur DB dans le formulaire |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application minimale](/docs/fr/exemple/minimal) | Point de départ simple |
| [Formulaires](/docs/fr/exemple/formulaires) | CRUD avec formulaires |
| [Upload](/docs/fr/exemple/upload) | Upload de fichier |

## Retour au sommaire

- [Exemples](/docs/fr/exemple)
