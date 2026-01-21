# 📚 Exemples Pratiques

## 1️⃣ Blog CRUD Complet

### Modèle

```rust
// demo-app/src/models/blog.rs
use sea_orm::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub content: String,
    pub author_id: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Title,
    Content,
    AuthorId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::AuthorId",
        to = "super::users::Column::Id",
        ondelete = "Cascade"
    )]
    Users,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationTwoMany {
        Relation::Users.def()
    }
}
```

### Formulaire

```rust
// demo-app/src/forms.rs
use runique::derive_form::RuniqueForm;
use serde::{Deserialize, Serialize};

#[derive(RuniqueForm, Debug, Clone, Serialize, Deserialize)]
pub struct BlogForm {
    #[field(label = "Titre", required, min_length = 5, max_length = 200)]
    pub title: String,

    #[field(label = "Contenu", required, input_type = "textarea", min_length = 20)]
    pub content: String,
}
```

### Routes

```rust
// demo-app/src/main.rs
use axum::routing::{get, post, put, delete};

fn routes() -> Router {
    Router::new()
        .route("/blogs", get(list_blogs).post(create_blog))
        .route("/blogs/:id", get(detail_blog).put(update_blog).delete(delete_blog))
        .route("/blogs/new", get(blog_form))
}

// Handlers
async fn list_blogs(
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    let db = ctx.engine.db.clone();
    
    let blogs = Blog::Entity::find()
        .order_by_desc(Blog::Column::CreatedAt)
        .all(&*db)
        .await
        .unwrap_or_default();

    template.render("blog/list.html", &context! {
        "blogs" => blogs
    })
}

async fn blog_form(template: TemplateContext) -> Response {
    template.render("blog/form.html", &context! {
        "form" => BlogForm::new()
    })
}

async fn create_blog(
    mut ctx: RuniqueContext,
    template: TemplateContext,
    ExtractForm(mut form): ExtractForm<BlogForm>,
) -> Response {
    if !form.is_valid().await {
        return template.render("blog/form.html", &context! {
            "form" => form
        });
    }

    let db = ctx.engine.db.clone();
    let user_id = ctx.session.get::<i32>("user_id")
        .unwrap_or_default()
        .unwrap_or(1); // Demo

    let blog = blog::ActiveModel {
        title: Set(form.title.clone()),
        content: Set(form.content.clone()),
        author_id: Set(user_id),
        ..Default::default()
    };

    match blog.insert(&*db).await {
        Ok(blog) => {
            success!(ctx.flash => "Article créé!");
            Redirect::to(&format!("/blogs/{}", blog.id)).into_response()
        }
        Err(e) => {
            error!(ctx.flash => format!("Erreur: {}", e));
            template.render("blog/form.html", &context! {
                "form" => form
            })
        }
    }
}

async fn detail_blog(
    Path(id): Path<i32>,
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    let db = ctx.engine.db.clone();

    match blog::Entity::find_by_id(id)
        .one(&*db)
        .await
    {
        Ok(Some(blog)) => {
            template.render("blog/detail.html", &context! {
                "blog" => blog
            })
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn update_blog(
    Path(id): Path<i32>,
    mut ctx: RuniqueContext,
    template: TemplateContext,
    ExtractForm(mut form): ExtractForm<BlogForm>,
) -> Response {
    let db = ctx.engine.db.clone();

    let blog = match blog::Entity::find_by_id(id)
        .one(&*db)
        .await
    {
        Ok(Some(b)) => b,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    let mut active = blog.into_active_model();
    active.title = Set(form.title.clone());
    active.content = Set(form.content.clone());

    match active.update(&*db).await {
        Ok(_) => {
            success!(ctx.flash => "Article mis à jour!");
            Redirect::to(&format!("/blogs/{}", id)).into_response()
        }
        Err(e) => {
            error!(ctx.flash => format!("Erreur: {}", e));
            template.render("blog/form.html", &context! {
                "form" => form
            })
        }
    }
}

async fn delete_blog(
    Path(id): Path<i32>,
    mut ctx: RuniqueContext,
) -> Response {
    let db = ctx.engine.db.clone();

    match blog::Entity::delete_by_id(id).exec(&*db).await {
        Ok(_) => {
            success!(ctx.flash => "Article supprimé");
            Redirect::to("/blogs").into_response()
        }
        Err(_) => {
            error!(ctx.flash => "Erreur lors de la suppression");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

---

## 2️⃣ Authentification

### Login Handler

```rust
async fn login_form(template: TemplateContext) -> Response {
    template.render("auth/login.html", &context! {
        "form" => LoginForm::new()
    })
}

async fn login_submit(
    mut ctx: RuniqueContext,
    template: TemplateContext,
    ExtractForm(mut form): ExtractForm<LoginForm>,
) -> Response {
    if !form.is_valid().await {
        return template.render("auth/login.html", &context! {
            "form" => form
        });
    }

    let db = ctx.engine.db.clone();

    // Trouver l'utilisateur
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(&form.email))
        .one(&*db)
        .await
        .unwrap_or(None);

    if let Some(user) = user {
        // Vérifier le mot de passe
        if verify_password(&form.password, &user.password_hash) {
            // Créer la session
            ctx.session.insert("user_id", user.id).unwrap();
            ctx.session.insert("username", &user.username).unwrap();

            success!(ctx.flash => "Bienvenue!");
            return Redirect::to("/dashboard").into_response();
        }
    }

    error!(ctx.flash => "Email ou mot de passe incorrect");
    template.render("auth/login.html", &context! {
        "form" => form
    })
}

async fn logout(
    mut ctx: RuniqueContext,
) -> Response {
    ctx.session.flush().await.ok();
    success!(ctx.flash => "Déconnexion réussie");
    Redirect::to("/").into_response()
}
```

---

## 3️⃣ API REST + AJAX

### Endpoint JSON

```rust
#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
}

async fn api_users(
    ctx: RuniqueContext,
) -> Json<Vec<UserResponse>> {
    let db = ctx.engine.db.clone();

    let users: Vec<UserResponse> = users::Entity::find()
        .all(&*db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|u| UserResponse {
            id: u.id,
            username: u.username,
            email: u.email,
        })
        .collect();

    Json(users)
}

async fn api_create_user(
    mut ctx: RuniqueContext,
    Json(payload): Json<CreateUserRequest>,
) -> (StatusCode, Json<UserResponse>) {
    let db = ctx.engine.db.clone();

    let user = users::ActiveModel {
        username: Set(payload.username),
        email: Set(payload.email),
        ..Default::default()
    };

    match user.insert(&*db).await {
        Ok(user) => (
            StatusCode::CREATED,
            Json(UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
            })
        ),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(UserResponse {
                id: 0,
                username: String::new(),
                email: String::new(),
            })
        )
    }
}
```

### Frontend AJAX

```html
<script>
async function createUser() {
    const data = {
        username: document.getElementById('username').value,
        email: document.getElementById('email').value
    };

    try {
        const response = await fetch('/api/users', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-CSRF-Token': document.querySelector('[name="csrf_token"]').value
            },
            body: JSON.stringify(data)
        });

        if (response.ok) {
            const user = await response.json();
            console.log('Utilisateur créé:', user);
            location.reload();
        }
    } catch (e) {
        console.error('Erreur:', e);
    }
}
</script>
```

---

## 4️⃣ Pagination

```rust
#[derive(Deserialize)]
pub struct PaginationQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_with_pagination(
    Query(query): Query<PaginationQuery>,
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    let db = ctx.engine.db.clone();
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).min(100);
    let offset = (page - 1) * limit;

    let total = users::Entity::find()
        .count(&*db)
        .await
        .unwrap_or(0) as u32;

    let users = users::Entity::find()
        .limit(limit as u64)
        .offset(offset as u64)
        .all(&*db)
        .await
        .unwrap_or_default();

    let total_pages = (total + limit - 1) / limit;

    template.render("users/list.html", &context! {
        "users" => users,
        "page" => page,
        "total_pages" => total_pages,
        "has_next" => page < total_pages,
        "has_prev" => page > 1
    })
}
```

Template:
```html
{% for user in users %}
    <tr>
        <td>{{ user.username }}</td>
        <td>{{ user.email }}</td>
    </tr>
{% endfor %}

<nav>
    {% if has_prev %}
        <a href="?page={{ page - 1 }}">← Précédent</a>
    {% endif %}
    
    <span>Page {{ page }}/{{ total_pages }}</span>
    
    {% if has_next %}
        <a href="?page={{ page + 1 }}">Suivant →</a>
    {% endif %}
</nav>
```

---

## 5️⃣ Validation Avancée

```rust
#[derive(RuniqueForm)]
pub struct RegistrationForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

impl RegistrationForm {
    pub async fn is_valid(&mut self) -> bool {
        let mut valid = true;

        // Validation de longueur
        if self.username.len() < 3 || self.username.len() > 50 {
            self.add_error("username", "3-50 caractères");
            valid = false;
        }

        // Vérifier format email
        if !self.email.contains('@') {
            self.add_error("email", "Email invalide");
            valid = false;
        }

        // Vérifier force mot de passe
        if self.password.len() < 8 {
            self.add_error("password", "Min 8 caractères");
            valid = false;
        }

        if !self.password.chars().any(|c| c.is_uppercase()) {
            self.add_error("password", "Besoin d'une majuscule");
            valid = false;
        }

        // Confirmation mot de passe
        if self.password != self.confirm_password {
            self.add_error("confirm_password", "Les mots de passe ne correspondent pas");
            valid = false;
        }

        // Vérifier unicité
        let db = get_db_connection();
        if let Ok(Some(_)) = users::Entity::find()
            .filter(users::Column::Email.eq(&self.email))
            .one(&*db)
            .await
        {
            self.add_error("email", "Email déjà utilisé");
            valid = false;
        }

        valid
    }
}
```

---

## Conclusion

Pour plus de détails:
- [Installation](./01-installation.md)
- [Architecture](./02-architecture.md)
- [Configuration](./03-configuration.md)
- [Routage](./04-routing.md)
- [Formulaires](./05-forms.md)
- [Templates](./06-templates.md)
- [ORM](./07-orm.md)
- [Middleware](./08-middleware.md)
- [Flash Messages](./09-flash-messages.md)

← [**Flash Messages**](./09-flash-messages.md) | [**Retour au README**](../../README.md) →
