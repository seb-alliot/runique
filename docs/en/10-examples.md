# 📚 Practical Examples

## 1️⃣ Complete Blog CRUD

### Model

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

### Form

```rust
// demo-app/src/forms.rs
use runique::derive_form::RuniqueForm;
use serde::{Deserialize, Serialize};

#[derive(RuniqueForm, Debug, Clone, Serialize, Deserialize)]
pub struct BlogForm {
    #[field(label = "Title", required, min_length = 5, max_length = 200)]
    pub title: String,

    #[field(label = "Content", required, input_type = "textarea", min_length = 20)]
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
    mut template: TemplateContext,
) -> Response {
    let db = /* access db from app state */;

    let blogs = Blog::Entity::find()
        .order_by_desc(Blog::Column::CreatedAt)
        .all(&*db)
        .await
        .unwrap_or_default();

    template.context.insert("blogs", blogs);
    template.render("blog/list.html")
}

async fn blog_form(mut template: TemplateContext) -> Response {
    template.context.insert("form", BlogForm::new());
    template.render("blog/form.html")
}

async fn create_blog(
    mut template: TemplateContext,
    Message(mut messages): Message,
    Prisme(mut form): Prisme<BlogForm>,
) -> Response {
    if !form.is_valid().await {
        template.context.insert("form", form);
        return template.render("blog/form.html");
    }

    let db = /* access db from app state */;
    let user_id = 1; // Demo

    let blog = blog::ActiveModel {
        title: Set(form.title.clone()),
        content: Set(form.content.clone()),
        author_id: Set(user_id),
        ..Default::default()
    };

    match blog.insert(&*db).await {
        Ok(blog) => {
            messages.success("Blog created!");
            Redirect::to(&format!("/blogs/{}", blog.id)).into_response()
        }
        Err(e) => {
            messages.error(format!("Error: {}", e));
            template.context.insert("form", form);
            template.render("blog/form.html")
        }
    }
}

async fn detail_blog(
    Path(id): Path<i32>,
    mut template: TemplateContext,
) -> Response {
    let db = /* access db from app state */;

    match blog::Entity::find_by_id(id)
        .one(&*db)
        .await
    {
        Ok(Some(blog)) => {
            template.context.insert("blog", blog);
            template.render("blog/detail.html")
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn update_blog(
    Path(id): Path<i32>,
    mut template: TemplateContext,
    Message(mut messages): Message,
    Prisme(mut form): Prisme<BlogForm>,
) -> Response {
    let db = /* access db from app state */;

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
            messages.success("Blog updated!");
            Redirect::to(&format!("/blogs/{}", id)).into_response()
        }
        Err(e) => {
            messages.error(format!("Error: {}", e));
            template.context.insert("form", form);
            template.render("blog/form.html")
        }
    }
}

async fn delete_blog(
    Path(id): Path<i32>,
    Message(mut messages): Message,
) -> Response {
    let db = /* access db from app state */;

    match blog::Entity::delete_by_id(id).exec(&*db).await {
        Ok(_) => {
            messages.success("Blog deleted");
            Redirect::to("/blogs").into_response()
        }
        Err(_) => {
            messages.error("Error deleting blog");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

---

## 2️⃣ Authentication

### Login Handler

```rust
async fn login_form(mut template: TemplateContext) -> Response {
    template.context.insert("form", LoginForm::new());
    template.render("auth/login.html")
}

async fn login_submit(
    mut template: TemplateContext,
    Message(mut messages): Message,
    Prisme(mut form): Prisme<LoginForm>,
) -> Response {
    if !form.is_valid().await {
        template.context.insert("form", form);
        return template.render("auth/login.html");
    }

    let db = /* access db from app state */;

    // Find user
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(&form.email))
        .one(&*db)
        .await
        .unwrap_or(None);

    if let Some(user) = user {
        // Verify password
        if verify_password(&form.password, &user.password_hash) {
            // Create session - TODO: add session layer extraction
            // session.insert("user_id", user.id);
            // session.insert("username", &user.username);

            messages.success("Welcome!");
            return Redirect::to("/dashboard").into_response();
        }
    }

    messages.error("Email or password incorrect");
    template.context.insert("form", form);
    template.render("auth/login.html")
}

async fn logout(
    Message(mut messages): Message,
) -> Response {
    // TODO: flush session from session layer
    messages.success("Logout successful");
    Redirect::to("/").into_response()
}
```

---

## 3️⃣ REST API + AJAX

### JSON Endpoint

```rust
#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
}

async fn api_users() -> Json<Vec<UserResponse>> {
    let db = /* access db from app state */;

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
    Json(payload): Json<CreateUserRequest>,
) -> (StatusCode, Json<UserResponse>) {
    let db = /* access db from app state */;

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
            console.log('User created:', user);
            location.reload();
        }
    } catch (e) {
        console.error('Error:', e);
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
    mut template: TemplateContext,
) -> Response {
    let db = /* access db from app state */;
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

    template.context.insert("users", users);
    template.context.insert("page", page);
    template.context.insert("total_pages", total_pages);
    template.context.insert("has_next", page < total_pages);
    template.context.insert("has_prev", page > 1);
    template.render("users/list.html")
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
        <a href="?page={{ page - 1 }}">← Previous</a>
    {% endif %}

    <span>Page {{ page }}/{{ total_pages }}</span>

    {% if has_next %}
        <a href="?page={{ page + 1 }}">Next →</a>
    {% endif %}
</nav>
```

---

## 5️⃣ Advanced Validation

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

        // Length validation
        if self.username.len() < 3 || self.username.len() > 50 {
            self.add_error("username", "3-50 characters");
            valid = false;
        }

        // Email format check
        if !self.email.contains('@') {
            self.add_error("email", "Invalid email");
            valid = false;
        }

        // Password strength check
        if self.password.len() < 8 {
            self.add_error("password", "Min 8 characters");
            valid = false;
        }

        if !self.password.chars().any(|c| c.is_uppercase()) {
            self.add_error("password", "Need uppercase");
            valid = false;
        }

        // Password confirmation
        if self.password != self.confirm_password {
            self.add_error("confirm_password", "Passwords don't match");
            valid = false;
        }

        // Check uniqueness
        let db = get_db_connection();
        if let Ok(Some(_)) = users::Entity::find()
            .filter(users::Column::Email.eq(&self.email))
            .one(&*db)
            .await
        {
            self.add_error("email", "Email already used");
            valid = false;
        }

        valid
    }
}
```

---

## More Resources

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md)
- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md)
- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md)
- [Routing](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md)
- [Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md)
- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/06-templates.md)
- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md)
- [Middleware](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md)
- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/en/09-flash-messages.md)

← [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md) | [**Back to README**](https://github.com/seb-alliot/runique/blob/main/README.md) →
