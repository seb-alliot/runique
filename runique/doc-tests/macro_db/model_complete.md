# Modèle complet avec Objects manager

```rust,ignore
use runique::prelude::*;
use sea_orm::entity::prelude::*;
use sea_orm::Set;

// Définition de l'entité SeaORM
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "articles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub published: bool,
    pub views: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Activation du manager Objects (style Django)
impl_objects!(Entity);

// ──────────────────────────────────────────────────────────────
// Utilisation dans un handler
// ──────────────────────────────────────────────────────────────

async fn articles_handler(ctx: Request) -> Response {
    let db = ctx.db();

    // Tous les articles
    let all = Entity::objects.all().all(db).await.unwrap();

    // Filtre : articles publiés
    let published = Entity::objects
        .filter(Column::Published.eq(true))
        .order_by_desc(Column::Views)
        .limit(10)
        .all(db)
        .await
        .unwrap();

    // Exclusion : articles masqués
    let visible = Entity::objects
        .exclude(Column::Published.eq(false))
        .all(db)
        .await
        .unwrap();

    // Comptage
    let total = Entity::objects.count(db).await.unwrap();

    // Récupération par ID (retourne Err si non trouvé)
    let article = Entity::objects.get(db, 1).await.unwrap();

    // Récupération par ID (retourne None si non trouvé)
    let maybe = Entity::objects.get_optional(db, 99).await.unwrap();

    // Récupération ou 404 automatique
    let article_or_404 = Entity::objects
        .get_or_404(db, 1, &ctx, "Article introuvable")
        .await
        .unwrap();

    let mut context = ctx.context.clone();
    context.insert("articles", &published);
    context.insert("total", &total);
    ctx.render("articles.html", &context)
}
```
