# Modèle complet avec Objects manager

```rust,ignore
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
        author_id:    int      [required],
        is_published: bool     [default: false],
        views:        bigint   [default: 0],
        created_at:   datetime [auto_now],
        updated_at:   datetime [auto_now_update],
    },
    relations: {
        belongs_to: eihwaz_users via author_id [cascade],
    }
}

// ── Activation du manager Objects (style Django) ───────────────
impl_objects!(Entity);

// ── Utilisation dans un handler ────────────────────────────────

async fn posts_handler(ctx: Request) -> Response {
    let db = ctx.db();

    // Tous les posts
    let all = Entity::objects.all().all(db).await.unwrap();

    // Filtre : posts publiés, triés par vues décroissant
    let published = Entity::objects
        .filter(Column::IsPublished.eq(true))
        .order_by_desc(Column::Views)
        .limit(10)
        .all(db)
        .await
        .unwrap();

    // Exclure les brouillons
    let visible = Entity::objects
        .exclude(Column::IsPublished.eq(false))
        .all(db)
        .await
        .unwrap();

    // Comptage total
    let total = Entity::objects.count(db).await.unwrap();

    // Récupération par ID — retourne Err si non trouvé
    let post = Entity::objects.get(db, 1).await.unwrap();

    // Récupération par ID — retourne None si non trouvé
    let maybe = Entity::objects.get_optional(db, 99).await.unwrap();

    // Récupération ou 404 automatique
    let post_or_404 = Entity::objects
        .get_or_404(db, 1, &ctx, "Article introuvable")
        .await
        .unwrap();

    ctx.render("posts/list.html", context! { posts: published, total })
}
```
