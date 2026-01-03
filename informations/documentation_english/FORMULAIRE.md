# Forms Guide - Runique Framework

Runique provides a Django-inspired form system with automatic generation via procedural macros.

## Table of Contents

1. [Manual Forms](#manual-forms)
2. [Auto-generated Forms](#auto-generated-forms)
3. [Validation](#validation)
4. [HTML Rendering](#html-rendering)
5. [CSRF Protection](#csrf-protection)
6. [Complete Examples](#complete-examples)

---

## Manual Forms

### Basic Creation

```rust
use runique::forms::{RuniqueForm, Field, fields::{CharField, EmailField}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginForm {
    pub username: CharField,
    pub password: CharField,
}

impl RuniqueForm for LoginForm {
    fn new() -> Self {
        Self {
            username: CharField::new()
                .required(true)
                .label("Username"),
            password: CharField::new()
                .required(true)
                .widget("password")
                .label("Password"),
        }
    }

    fn is_valid(&self) -> bool {
        self.username.is_valid() && self.password.is_valid()
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        errors.extend(self.username.errors());
        errors.extend(self.password.errors());
        errors
    }
}
```

### Usage in a Handler

```rust
use runique::prelude::*;

pub async fn login(
    Form(form): Form<LoginForm>,
    template: Template,
) -> Response {
    if !form.is_valid() {
        return template.render("login.html", context! {
            form: form,
            errors: form.errors(),
        });
    }

    // Process valid form
    let username = form.username.value();
    let password = form.password.value();

    // Authentication...
    redirect("/dashboard")
}
```

---

## Auto-generated Forms

Runique provides **two macros** to automatically generate forms from your SeaORM models:

1. **`#[runique_form]`** - For creating custom forms
2. **`#[derive(DeriveModelForm)]`** - For generating model-linked forms

### `#[runique_form]` Macro

This macro automatically generates the implementation of the `RuniqueForm` trait.

```rust
use runique::forms::prelude::*;

#[runique_form]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactForm {
    #[field(required = true)]
    pub name: CharField,

    #[field(required = true)]
    pub email: EmailField,

    #[field(required = true)]
    pub subject: CharField,

    #[field(widget = "textarea", required = true)]
    pub message: CharField,
}
```

**What is automatically generated:**

```rust
impl RuniqueForm for ContactForm {
    fn new() -> Self { /* ... */ }
    fn is_valid(&self) -> bool { /* ... */ }
    fn errors(&self) -> Vec<String> { /* ... */ }
}
```

### `#[derive(DeriveModelForm)]` Macro

This macro is more powerful and generates **multiple elements**:

#### 1. Form Structure

```rust
use runique::forms::prelude::*;
use sea_orm::entity::prelude::*;

// Your SeaORM model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
}

// Form generation
#[derive(DeriveModelForm, Debug, Clone, Serialize, Deserialize)]
#[sea_orm(model = "Model", entity = "Entity")]
pub struct UserForm {
    #[field(required = true)]
    pub username: CharField,

    #[field(required = true)]
    pub email: EmailField,

    #[field(widget = "textarea")]
    pub bio: CharField,
}
```

#### 2. Auto-generated Methods

The `DeriveModelForm` macro **automatically generates** the following elements:

##### a) Implementation of `Deref` and `DerefMut`

```rust
// ✅ Generated automatically
impl std::ops::Deref for UserForm {
    type Target = UserFormInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for UserForm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
```

##### b) Implementation of `RuniqueForm` Trait

```rust
// ✅ Generated automatically
impl RuniqueForm for UserForm {
    fn new() -> Self { /* ... */ }
    fn is_valid(&self) -> bool { /* ... */ }
    fn errors(&self) -> Vec<String> { /* ... */ }
}
```

##### c) `to_active_model()` Method

**Automatic conversion to SeaORM `ActiveModel`:**

```rust
// ✅ Generated automatically
impl UserForm {
    pub fn to_active_model(&self) -> ActiveModel {
        ActiveModel {
            username: Set(self.username.value().clone()),
            email: Set(self.email.value().clone()),
            bio: Set(self.bio.value().clone().into()),
            ..Default::default()
        }
    }
}
```

**Usage:**

```rust
pub async fn create_user(
    Form(form): Form<UserForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    if !form.is_valid() {
        return template.render("form.html", context! { form });
    }

    // ✅ Automatic conversion to ActiveModel
    let active_model = form.to_active_model();

    // Insert into database
    match active_model.insert(&*db).await {
        Ok(user) => redirect(&format!("/user/{}", user.id)),
        Err(e) => template.render("form.html", context! {
            form,
            error: "Error creating user"
        }),
    }
}
```

##### d) `save()` Method

**Direct database save:**

```rust
// ✅ Generated automatically
impl UserForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<Model, DbErr> {
        let active_model = self.to_active_model();
        active_model.insert(db).await
    }
}
```

**Simplified usage:**

```rust
pub async fn create_user_simple(
    Form(form): Form<UserForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    if !form.is_valid() {
        return template.render("form.html", context! { form });
    }

    // ✅ Direct save in one line
    match form.save(&*db).await {
        Ok(user) => redirect(&format!("/user/{}", user.id)),
        Err(e) => template.render("form.html", context! {
            form,
            error: "Error saving"
        }),
    }
}
```

#### 3. Summary of Generated Elements

| Element | Generated by `#[runique_form]` | Generated by `#[derive(DeriveModelForm)]` |
|---------|------------------------------|------------------------------------------|
| Form struct | ✅ (manual) | ✅ (manual) |
| `impl RuniqueForm` | ✅ | ✅ |
| `impl Deref/DerefMut` | ❌ | ✅ |
| `to_active_model()` method | ❌ | ✅ |
| `save()` method | ❌ | ✅ |

---

## Validation

### Available Field Types

```rust
use runique::forms::fields::*;

// Simple text
pub name: CharField,

// Email with validation
pub email: EmailField,

// Integer
pub age: IntegerField,

// Boolean
pub is_active: BooleanField,

// Date
pub birth_date: DateField,

// Choice (select)
pub role: ChoiceField,
```

### Validation Attributes

```rust
#[runique_form]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterForm {
    // Max length
    #[field(required = true)]
    pub username: CharField,

    // Automatic email validation
    #[field(required = true)]
    pub email: EmailField,

    // Min and max length
    #[field(required = true, widget = "password")]
    pub password: CharField,

    // Default value
    #[field(default = "user")]
    pub role: CharField,

    // Optional (not required)
    #[field(required = false, widget = "textarea")]
    pub bio: CharField,
}
```

### Custom Validation

```rust
impl RegisterForm {
    pub fn validate_passwords_match(&self, password_confirm: &str) -> bool {
        self.password.value() == password_confirm
    }

    pub fn validate_username_unique(&self, db: &DatabaseConnection) -> bool {
        // Check database
        // ...
        true
    }
}
```

---

## HTML Rendering

### Automatic Rendering in Templates

```html
<!-- templates/form.html -->
<form method="post">
    {% csrf %}

    <!-- Automatic rendering of all fields -->
    {{ form }}

    <button type="submit">Submit</button>
</form>
```

### Field-by-field Rendering

```html
<form method="post">
    {% csrf %}

    <div class="form-group">
        <label>{{ form.username.label }}</label>
        {{ form.username }}
        {% if form.username.errors %}
            <div class="errors">
                {% for error in form.username.errors %}
                    <span class="error">{{ error }}</span>
                {% endfor %}
            </div>
        {% endif %}
    </div>

    <div class="form-group">
        <label>{{ form.email.label }}</label>
        {{ form.email }}
        {% if form.email.errors %}
            <div class="errors">
                {% for error in form.email.errors %}
                    <span class="error">{{ error }}</span>
                {% endfor %}
            </div>
        {% endif %}
    </div>

    <button type="submit">Create</button>
</form>
```

### Custom Widgets

```rust
#[runique_form]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleForm {
    #[field(required = true)]
    pub title: CharField,

    // Textarea for long content
    #[field(widget = "textarea", required = true)]
    pub content: CharField,

    // Password input
    #[field(widget = "password")]
    pub password: CharField,

    // Select dropdown
    #[field(widget = "select")]
    pub category: ChoiceField,
}
```

---

## CSRF Protection

### Automatic Activation

CSRF protection is **automatically activated** in Runique when the `CsrfMiddleware` is added.

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    RuniqueApp::new(settings).await?
        .middleware(CsrfMiddleware::new())  // ✅ CSRF enabled
        .routes(routes())
        .run()
        .await?;

    Ok(())
}
```

### Usage in Templates

```html
<form method="post">
    <!-- Automatic CSRF token -->
    {% csrf %}

    {{ form }}

    <button type="submit">Submit</button>
</form>
```

### Validation in Handlers

CSRF validation is **automatic** via the `Form<T>` extractor:

```rust
pub async fn submit_form(
    Form(form): Form<ContactForm>,  // ✅ CSRF validated automatically
    template: Template,
) -> Response {
    if !form.is_valid() {
        return template.render("contact.html", context! { form });
    }

    // Form is valid AND CSRF token has been verified
    // ...
}
```

**Note:** If the CSRF token is invalid, a `403 Forbidden` error is automatically returned **before** the handler is called.

---

## Complete Examples

### Example 1: Simple Contact Form

```rust
use runique::prelude::*;
use runique::forms::prelude::*;

#[runique_form]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactForm {
    #[field(required = true)]
    pub name: CharField,

    #[field(required = true)]
    pub email: EmailField,

    #[field(required = true)]
    pub subject: CharField,

    #[field(widget = "textarea", required = true)]
    pub message: CharField,
}

pub async fn contact_view(template: Template) -> Response {
    let form = ContactForm::new();
    template.render("contact.html", context! { form })
}

pub async fn contact_submit(
    Form(form): Form<ContactForm>,
    template: Template,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        return template.render("contact.html", context! {
            form,
            errors: form.errors(),
        });
    }

    // Process message (send email, etc.)
    let _ = message.success("Message sent successfully!").await;

    redirect("/")
}
```

### Example 2: Model-linked Form with Save

```rust
use runique::prelude::*;
use runique::forms::prelude::*;
use sea_orm::entity::prelude::*;

// SeaORM Model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "articles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub published: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Auto-generated form
#[derive(DeriveModelForm, Debug, Clone, Serialize, Deserialize)]
#[sea_orm(model = "Model", entity = "Entity")]
pub struct ArticleForm {
    #[field(required = true)]
    pub title: CharField,

    #[field(required = true)]
    pub slug: CharField,

    #[field(widget = "textarea", required = true)]
    pub content: CharField,

    #[field(default = "false")]
    pub published: BooleanField,
}

// Create handler
pub async fn create_article(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    let form = ArticleForm::new();
    template.render("article_form.html", context! { form })
}

// Submit handler
pub async fn store_article(
    Form(form): Form<ArticleForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        return template.render("article_form.html", context! {
            form,
            errors: form.errors(),
        });
    }

    // ✅ Method 1: Direct save with .save()
    match form.save(&*db).await {
        Ok(article) => {
            success!(message, "Article created successfully!");
            redirect(&format!("/article/{}", article.id))
        }
        Err(e) => {
            error!(message, "Error creating article");
            template.render("article_form.html", context! { form })
        }
    }

    // ✅ Method 2: Manual conversion with .to_active_model()
    // let active_model = form.to_active_model();
    // match active_model.insert(&*db).await {
    //     Ok(article) => { /* ... */ }
    //     Err(e) => { /* ... */ }
    // }
}
```

### Example 3: Form with Editing

```rust
pub async fn edit_article(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    let article = match Article::objects.get(&*db, id).await {
        Ok(a) => a,
        Err(_) => return (StatusCode::NOT_FOUND, "Article not found").into_response(),
    };

    // Pre-fill form with existing data
    let mut form = ArticleForm::new();
    form.title.set_value(article.title);
    form.slug.set_value(article.slug);
    form.content.set_value(article.content);
    form.published.set_value(article.published.to_string());

    template.render("article_form.html", context! {
        form,
        article_id: id,
        is_edit: true,
    })
}

pub async fn update_article(
    Path(id): Path<i32>,
    Form(form): Form<ArticleForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        return template.render("article_form.html", context! {
            form,
            article_id: id,
            is_edit: true,
        });
    }

    // Get existing article
    let existing = match Article::objects.get(&*db, id).await {
        Ok(a) => a,
        Err(_) => return (StatusCode::NOT_FOUND, "Article not found").into_response(),
    };

    // Create ActiveModel for update
    let mut active_model: ActiveModel = existing.into();
    active_model.title = Set(form.title.value().clone());
    active_model.slug = Set(form.slug.value().clone());
    active_model.content = Set(form.content.value().clone());
    active_model.published = Set(form.published.value().parse().unwrap_or(false));

    match active_model.update(&*db).await {
        Ok(updated) => {
            success!(message, "Article updated successfully!");
            redirect(&format!("/article/{}", updated.id))
        }
        Err(e) => {
            error!(message, "Error updating article");
            template.render("article_form.html", context! {
                form,
                article_id: id,
                is_edit: true,
            })
        }
    }
}
```

### Example 4: Custom Validation

```rust
#[derive(DeriveModelForm, Debug, Clone, Serialize, Deserialize)]
#[sea_orm(model = "Model", entity = "Entity")]
pub struct UserForm {
    #[field(required = true)]
    pub username: CharField,

    #[field(required = true)]
    pub email: EmailField,

    #[field(required = true, widget = "password")]
    pub password: CharField,
}

impl UserForm {
    /// Custom validation: check username is unique
    pub async fn validate_unique_username(&self, db: &DatabaseConnection) -> bool {
        let existing = User::objects
            .filter(users::Column::Username.eq(self.username.value()))
            .first(db)
            .await;

        existing.is_err() // True if no user found (username available)
    }

    /// Custom validation: check email is unique
    pub async fn validate_unique_email(&self, db: &DatabaseConnection) -> bool {
        let existing = User::objects
            .filter(users::Column::Email.eq(self.email.value()))
            .first(db)
            .await;

        existing.is_err()
    }
}

pub async fn register_user(
    Form(mut form): Form<UserForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
) -> Response {
    // Standard validation
    if !form.is_valid() {
        return template.render("register.html", context! {
            form,
            errors: form.errors(),
        });
    }

    // Custom validations
    if !form.validate_unique_username(&*db).await {
        error!(message, "This username is already taken");
        return template.render("register.html", context! { form });
    }

    if !form.validate_unique_email(&*db).await {
        error!(message, "This email is already in use");
        return template.render("register.html", context! { form });
    }

    // Save user
    match form.save(&*db).await {
        Ok(user) => {
            success!(message, "Account created successfully!");
            redirect(&format!("/user/{}", user.id))
        }
        Err(e) => {
            error!(message, "Error creating account");
            template.render("register.html", context! { form })
        }
    }
}
```

---

## Best Practices

### 1. Always Validate Forms

```rust
// ✅ Good
pub async fn submit(Form(form): Form<MyForm>) -> Response {
    if !form.is_valid() {
        return template.render("form.html", context! { form });
    }
    // Process...
}

// ❌ Bad
pub async fn submit(Form(form): Form<MyForm>) -> Response {
    // No validation!
    form.save(&db).await?; // Risk of invalid data
}
```

### 2. Use Auto-generated Methods

```rust
// ✅ Good (uses automatically generated .save())
match form.save(&*db).await {
    Ok(user) => redirect("/success"),
    Err(e) => handle_error(e),
}

// ⚠️ Acceptable but more verbose
let active_model = form.to_active_model();
match active_model.insert(&*db).await {
    Ok(user) => redirect("/success"),
    Err(e) => handle_error(e),
}
```

### 3. Handle Errors Properly

```rust
pub async fn submit(
    Form(form): Form<UserForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        return template.render("form.html", context! {
            form,
            errors: form.errors(),
        });
    }

    match form.save(&*db).await {
        Ok(user) => {
            let _ = message.success("User created!").await;
            redirect(&format!("/user/{}", user.id))
        }
        Err(DbErr::RecordNotFound(_)) => {
            let _ = message.error("Record not found").await;
            template.render("form.html", context! { form })
        }
        Err(DbErr::Exec(_)) => {
            let _ = message.error("Constraint error (duplicate?)").await;
            template.render("form.html", context! { form })
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            let _ = message.error("Internal error").await;
            template.render("form.html", context! { form })
        }
    }
}
```

### 4. Use Transactions for Complex Operations

```rust
pub async fn create_user_with_profile(
    Form(user_form): Form<UserForm>,
    Form(profile_form): Form<ProfileForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    if !user_form.is_valid() || !profile_form.is_valid() {
        return template.render("form.html", context! {
            user_form,
            profile_form,
        });
    }

    // Transaction to ensure consistency
    let result = db.transaction::<_, (), DbErr>(|txn| {
        Box::pin(async move {
            // Create user
            let user = user_form.to_active_model().insert(txn).await?;

            // Create profile
            let mut profile = profile_form.to_active_model();
            profile.user_id = Set(user.id);
            profile.insert(txn).await?;

            Ok(())
        })
    }).await;

    match result {
        Ok(_) => redirect("/success"),
        Err(e) => {
            let _ = message.error("Error creating account").await;
            template.render("form.html", context! {
                user_form,
                profile_form,
            })
        }
    }
}
```

### Example 5: Using Message Macros

The `success!()`, `error!()`, `info!()` and `warning!()` macros simplify message sending:

```rust
use runique::prelude::*;

pub async fn create_article(
    Form(form): Form<ArticleForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
) -> Response {
    // Standard validation
    if !form.is_valid() {
        error!(message, "The form contains errors");
        return template.render("article_form.html", context! { form });
    }

    // Custom validation
    if form.title.value().len() < 10 {
        error!(message, "Title must be at least 10 characters long");
        return template.render("article_form.html", context! { form });
    }

    // Check if slug is unique
    if Article::slug_exists(&form.slug.value(), &db).await {
        error!(message, "This slug is already in use");
        warning!(message, "Try adding a number or date");
        return template.render("article_form.html", context! { form });
    }

    // Save
    match form.save(&*db).await {
        Ok(article) => {
            success!(message, "Article created successfully!");

            if article.published {
                info!(message, "Your article is now visible to everyone");
            } else {
                info!(message, "Your article is a draft");
            }

            redirect(&format!("/articles/{}", article.id))
        }
        Err(e) => {
            error!(message, "Error during creation");
            template.render("article_form.html", context! { form })
        }
    }
}
```

**Syntax comparison:**

```rust
// ❌ Old syntax (verbose)
let _ = message.success("Article created!").await;
let _ = message.error("Error").await;

// ✅ New syntax (with macros)
success!(message, "Article created!");
error!(message, "Error");
```

---

## See Also

- [Getting Started](informations/documentation_english/GETTING_STARTED.md)
- [Templates](informations/documentation_english/TEMPLATES.md)
- [Security](informations/documentation_english/CSP.md)
- [Database](informations/documentation_english/DATABASE.md)

Create robust forms with Runique!

---

**Version:** 1.0 (Corrected - January 2, 2026)
**License:** MIT

*Documentation created with ❤️ by Claude for Itsuki*
