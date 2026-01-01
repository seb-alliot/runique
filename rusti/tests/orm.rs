use sea_orm::{entity::prelude::*, DatabaseConnection, Database, DbErr, Set};
use rusti::impl_objects;

// ========================================
// Mock Entity pour les tests
// ========================================

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub age: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// ✨ Ajouter objects à Entity
impl_objects!(Entity);

// ========================================
// Tests de compilation (sans DB)
// ========================================

#[test]
fn test_objects_without_parentheses() {
    let _builder = Entity::objects;
    println!("Entity::objects fonctionne (sans parenthèses)");
}

#[test]
fn test_filter_without_parentheses() {
    let _query = Entity::objects
        .filter(Column::Age.gte(18));

    println!("Entity::objects.filter() fonctionne (sans parenthèses)");
}

#[test]
fn test_chaining() {
    let _query = Entity::objects
        .filter(Column::Age.gte(18))
        .exclude(Column::Username.eq("banned"))
        .order_by_desc(Column::Age)
        .limit(10);

    println!("Chaînage complet fonctionne");
}

// ========================================
// Tests avec base de données en mémoire
// ========================================

async fn setup_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;

    use sea_orm::Schema;
    let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);
    let stmt = schema.create_table_from_entity(Entity);

    // Correction : execute le Statement directement
    db.execute(&stmt).await?;

    Ok(db)
}

#[tokio::test]
async fn test_objects_all() -> Result<(), DbErr> {
    let db = setup_db().await?;

    let user1 = ActiveModel {
        username: Set("alice".to_string()),
        age: Set(25),
        ..Default::default()
    };
    let user2 = ActiveModel {
        username: Set("bob".to_string()),
        age: Set(30),
        ..Default::default()
    };

    user1.insert(&db).await?;
    user2.insert(&db).await?;

    let users = Entity::objects.all().all(&db).await?;
    assert_eq!(users.len(), 2);
    println!("objects.all().all() fonctionne : {} utilisateurs", users.len());

    Ok(())
}

#[tokio::test]
async fn test_objects_filter() -> Result<(), DbErr> {
    let db = setup_db().await?;

    let young = ActiveModel {
        username: Set("young".to_string()),
        age: Set(16),
        ..Default::default()
    };
    let adult = ActiveModel {
        username: Set("adult".to_string()),
        age: Set(25),
        ..Default::default()
    };

    young.insert(&db).await?;
    adult.insert(&db).await?;

    let adults = Entity::objects
        .filter(Column::Age.gte(18))
        .all(&db)
        .await?;

    assert_eq!(adults.len(), 1);
    assert_eq!(adults[0].username, "adult");
    println!("objects.filter() fonctionne : {} adulte trouvé", adults.len());

    Ok(())
}

#[tokio::test]
async fn test_objects_exclude() -> Result<(), DbErr> {
    let db = setup_db().await?;

    let alice = ActiveModel {
        username: Set("alice".to_string()),
        age: Set(25),
        ..Default::default()
    };
    let banned = ActiveModel {
        username: Set("banned".to_string()),
        age: Set(30),
        ..Default::default()
    };

    alice.insert(&db).await?;
    banned.insert(&db).await?;

    let active_users = Entity::objects
        .exclude(Column::Username.eq("banned"))
        .all(&db)
        .await?;

    assert_eq!(active_users.len(), 1);
    assert_eq!(active_users[0].username, "alice");
    println!("objects.exclude() fonctionne : {} utilisateur actif", active_users.len());

    Ok(())
}

#[tokio::test]
async fn test_objects_get() -> Result<(), DbErr> {
    let db = setup_db().await?;

    let user = ActiveModel {
        username: Set("test".to_string()),
        age: Set(25),
        ..Default::default()
    };

    let inserted = user.insert(&db).await?;

    let found = Entity::objects.get(&db, inserted.id).await?;
    assert_eq!(found.username, "test");
    println!("objects.get() fonctionne : utilisateur '{}'", found.username);

    Ok(())
}

#[tokio::test]
async fn test_objects_count() -> Result<(), DbErr> {
    let db = setup_db().await?;

    for i in 1..=3 {
        let user = ActiveModel {
            username: Set(format!("user{}", i)),
            age: Set(20 + i),
            ..Default::default()
        };
        user.insert(&db).await?;
    }

    let count = Entity::objects.count(&db).await?;
    assert_eq!(count, 3);
    println!("objects.count() fonctionne : {} utilisateurs", count);

    Ok(())
}

#[tokio::test]
async fn test_complex_query() -> Result<(), DbErr> {
    let db = setup_db().await?;

    for i in 1..=10 {
        let user = ActiveModel {
            username: Set(format!("user{}", i)),
            age: Set(15 + i as i32),
            ..Default::default()
        };
        user.insert(&db).await?;
    }

    let results = Entity::objects
        .filter(Column::Age.gte(20))
        .exclude(Column::Username.eq("user5"))
        .order_by_desc(Column::Age)
        .limit(3)
        .all(&db)
        .await?;

    assert!(results.len() <= 3);
    println!("Query complexe fonctionne : {} résultats", results.len());

    Ok(())
}