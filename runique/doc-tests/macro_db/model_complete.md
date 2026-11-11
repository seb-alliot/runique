# Modèle complet avec le manager `objects`

`model!{}` génère l'entité SeaORM, son formulaire et le manager `objects`, à la manière de Django
(pas besoin d'appeler `impl_objects!` : la macro le fait).

```rust,no_run
use runique::prelude::*;

// ── Déclaration du modèle ──────────────────────────────────────
model! {
    Post,
    table: "posts",
    pk: id => i32,
    {
        title:        text     [required, max_length: 255],
        slug:         text     [required, unique, max_length: 255],
        content:      textarea [required],
        excerpt:      text     [nullable, max_length: 500],
        author_id:    Pk       [required],
        is_published: bool     [default: false],
        views:        bigint   [default: 0],
        created_at:   datetime [auto_now],
        updated_at:   datetime [auto_now_update],
    },
    relations: {
        belongs_to: eihwaz_users via author_id [cascade],
    }
}

// ── Utilisation dans un handler ────────────────────────────────
async fn posts(mut request: Request) -> AppResult<Response> {
    let db = request.engine.db.clone();

    // Tous les posts
    let all = Entity::objects.all().all(&db).await.unwrap_or_default();

    // Posts publiés, triés par vues décroissantes, 10 au plus
    let published = Entity::objects
        .filter(Column::IsPublished.eq(true))
        .order_by_desc(Column::Views)
        .limit(10)
        .all(&db)
        .await
        .unwrap_or_default();

    // Sans les brouillons
    let visible = Entity::objects
        .exclude(Column::IsPublished.eq(false))
        .all(&db)
        .await
        .unwrap_or_default();

    // Comptage
    let total = Entity::objects.count(&db).await.unwrap_or(0);

    // Par clé : `get` renvoie une erreur si la ligne n'existe pas,
    // `get_optional` renvoie `None`
    let maybe = Entity::objects.get_optional(&db, 99).await.ok().flatten();

    // Par clé, ou la page 404 (rendue par le middleware d'erreurs)
    let post = match Entity::objects.get_or_404(&db, 1, &request, "Article introuvable").await {
        Ok(post) => post,
        Err(response) => return Ok(response),
    };

    context_update!(request => {
        "posts"   => &published,
        "total"   => &total,
        "post"    => &post,
    });
    let _ = (all, visible, maybe);
    request.render("posts/list.html")
}

fn main() {}
```
