# Avancé — Transactions, Relations & Pattern complet

## Transactions

Une transaction regroupe plusieurs écritures : elles deviennent toutes réelles au `commit()`, ou aucune. Sans `commit()`, la transaction est annulée (rollback) d'elle-même à la sortie du handler — un `return`, une erreur remontée par `?` —, sans `rollback()` à écrire.

### Dans un handler — `request.db_atomic()`

`request.db_atomic()` n'est qu'un raccourci d'écriture : il fait exactement `request.engine.db.begin()`, la transaction de SeaORM, sans importer `TransactionTrait`. C'est la forme à utiliser dans un handler.

`ValidationForm::save` et `delete` acceptent la connexion (`&request.engine.db`) comme la transaction (`&txn`). Un formulaire correspond à un modèle et à un `save` ; la transaction sert à lier ce `save` à vos autres écritures :

```rust
pub async fn publier(mut request: Request) -> AppResult<Response> {
    let template = "article/nouveau.html";
    let form: ArticleForm = request.form("article");
    let form = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "article_form" => &form });
            return request.render(template); // rien n'a été ouvert
        }
    };

    let txn = request.db_atomic().await?;

    if let Err(form) = form.save(&txn).await {
        context_update!(request => { "article_form" => &form });
        return request.render(template); // sortie sans commit : rollback
    }
    journal::ActiveModel {
        action: Set("article publié".to_string()),
        ..Default::default()
    }
    .insert(&txn)
    .await?; // erreur : rollback, l'article est annulé aussi

    txn.commit().await?; // l'article et le journal, ou aucun des deux
    Ok(Redirect::to("/articles").into_response())
}
```

À l'intérieur, `save` ouvre sa propre transaction ; sur une transaction déjà ouverte, SeaORM en fait un savepoint. Un `save` qui échoue n'annule que sa propre écriture et rend le formulaire avec ses erreurs ; c'est la sortie sans `commit()` qui annule le reste.

**Fichiers.** `save` déplace le fichier entre son écriture et la fin de son savepoint. Si le handler sort ensuite sans `commit()`, la ligne est annulée mais le fichier déplacé reste sur le disque, orphelin — jamais une ligne qui pointe vers un fichier absent. Voir [Champs](/docs/fr/formulaire/champs).

**Routes `admin`.** Sur une route déclarée `admin` dans `urlpatterns!`, `save` écrit par la porte de l'admin, qui lit et écrit par la connexion : un formulaire de ces routes refuse une transaction (message de refus habituel). Passez-lui `&request.engine.db`.

### Hors requête — CLI, tâches

Sans `Request`, on ouvre la transaction directement sur la connexion, avec `TransactionTrait` importé :

```rust
use sea_orm::TransactionTrait;

let txn = engine.db.begin().await?;

let user = users::ActiveModel {
    email: Set("test@example.com".to_string()),
    ..Default::default()
}.insert(&txn).await?;

profiles::ActiveModel {
    user_id: Set(user.id),
    ..Default::default()
}.insert(&txn).await?;

txn.commit().await?;
```

C'est adapté à une commande qui crée un compte et ses lignes liées. Les migrations, elles, ne passent pas par là : `runique migration up` laisse sea-orm-migration décider, et il n'enveloppe chaque migration dans une transaction que sous Postgres. Voir [Migrations](/docs/fr/installation/migrations).

---

## Relations

### One-to-Many

```rust
let user = users::Entity::objects.get_optional(&db, 1).await?;

if let Some(user) = user {
    let posts = user.find_related(posts::Entity)
        .all(&db)
        .await?;
}
```

### Many-to-Many

```rust
let user = users::Entity::objects.get_optional(&db, 1).await?;

if let Some(user) = user {
    let roles = user.find_related(roles::Entity)
        .all(&db)
        .await?;
}
```

---

## Pattern CRUD complet

```rust
use demo_app::entities::users;
use runique::prelude::*;
use axum::extract::Path;

async fn list_users(request: Request) -> AppResult<Response> {
    let users = users::Entity::find()
        .all(request.db())
        .await?;

    Ok(Json(json!({ "users": users })).into_response())
}

async fn get_user(
    Path(id): Path<i32>,
    request: Request,
) -> AppResult<Response> {
    match users::Entity::find_by_id(id).one(request.db()).await? {
        Some(user) => Ok(Json(json!({ "user": user })).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

async fn create_user(
    Json(payload): Json<CreateUserRequest>,
    mut request: Request,  // Request doit être en dernier
) -> AppResult<Response> {
    let user = users::ActiveModel {
        email: Set(payload.email),
        username: Set(payload.username),
        ..Default::default()
    }.insert(request.db()).await?;

    success!(request.notices => "Utilisateur créé!");
    Ok((StatusCode::CREATED, Json(json!({ "user": user }))).into_response())
}

async fn delete_user(
    Path(id): Path<i32>,
    mut request: Request,
) -> AppResult<Response> {
    let result = users::Entity::delete_by_id(id)
        .exec(request.db())
        .await?;

    if result.rows_affected > 0 {
        success!(request.notices => "Utilisateur supprimé");
        Ok(Redirect::to("/users").into_response())
    } else {
        Ok(StatusCode::NOT_FOUND.into_response())
    }
}
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Manager & helpers](/docs/fr/orm/manager) | `impl_objects!`, helpers |
| [Requêtes CRUD](/docs/fr/orm/requetes) | SELECT, INSERT, UPDATE, DELETE |

## Retour au sommaire

- [ORM](/docs/fr/orm)
