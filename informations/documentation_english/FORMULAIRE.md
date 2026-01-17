# Runique Forms Guide

Complete documentation of the Django-inspired forms system for Runique.

## Table of Contents

1. [Introduction](#introduction)
2. [Manual Form Creation](#manual-form-creation)
3. [Automatic Generation with Macros](#automatic-generation-with-macros)
4. [Available Field Types](#available-field-types)
5. [Validation](#validation)
6. [Error Display](#error-display)
7. [Database Saving](#database-saving)
8. [CSRF Protection](#csrf-protection)
9. [Complete Examples](#complete-examples)
10. [Best Practices](#best-practices)

---

## Introduction

Runique provides a Django-inspired forms system that allows you to:
- ✅ Create typed and validated forms
- ✅ Automatically manage errors
- ✅ Easily integrate with SeaORM
- ✅ Automatically generate from models via macros
- ✅ Generate HTML via Tera templates
- ✅ Built-in CSRF protection

---

## Manual Form Creation

### Basic Structure

Each Runique form implements the `RuniqueForm` trait:

```rust
use runique::prelude::*;
use runique::serde::{Serialize, Serializer};

#[derive(Serialize)]
#[serde(transparent)]
pub struct ContactForm {
    pub form: Forms,
}

impl Serialize for ContactForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.form.serialize(serializer)
    }
}

impl RuniqueForm for ContactForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &GenericField::text("name")
                .placeholder("Your name")
                .required("Name is required"),
        );
        
        form.field(
            &GenericField::email("email")
                .placeholder("your@email.com")
                .required("Email is required"),
        );
        
        form.field(
            &GenericField::textarea("message")
                .placeholder("Your message...")
                .required("Message is required"),
        );
    }
    
    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    
    fn get_form(&self) -> &Forms {
        &self.form
    }
    
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}
```

### Using in a Handler

```rust
use axum::{extract::{Form, State}, response::{Html, Redirect}};
use std::collections::HashMap;

pub async fn show_contact(State(state): State<AppState>) -> Html<String> {
    let contact_form = ContactForm::build(state.tera.clone());
    
    let html = state.tera
        .render("contact.html", &tera::context! { form => contact_form })
        .unwrap();
    
    Html(html)
}

pub async fn submit_contact(
    State(state): State<AppState>,
    Form(raw_data): Form<HashMap<String, String>>,
) -> Result<Redirect, Html<String>> {
    let mut contact_form = ContactForm::build_with_data(&raw_data, state.tera.clone());
    
    if !contact_form.is_valid().await {
        let html = state.tera
            .render("contact.html", &tera::context! { form => contact_form })
            .unwrap();
        return Err(Html(html));
    }
    
    // Process the message
    // ...
    
    Ok(Redirect::to("/success"))
}
```

---

## Automatic Generation with Macros

Runique provides **two macros** to automatically generate forms:

### 1. `#[runique_form]` Macro

For creating custom forms with automatic `Deref/DerefMut` management:

```rust
use runique::prelude::*;

#[runique_form]
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginForm {
    pub form: Forms,
}
```

**What is automatically generated:**

```rust
// ✅ Deref implementation
impl std::ops::Deref for LoginForm {
    type Target = Forms;
    fn deref(&self) -> &Self::Target { &self.form }
}

// ✅ DerefMut implementation
impl std::ops::DerefMut for LoginForm {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.form }
}
```

### 2. `#[derive(DeriveModelForm)]` Macro

To automatically generate a complete form from a SeaORM model:

```rust
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
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Automatic form generation
#[derive(DeriveModelForm)]
pub struct User;
```

**What is automatically generated:**

#### a) Form Structure

```rust
#[derive(Serialize, Debug, Clone)]
pub struct UserForm {
    #[serde(flatten, skip_deserializing)]
    pub form: Forms,
}
```

#### b) `RuniqueForm` Implementation

```rust
impl RuniqueForm for UserForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &GenericField::text("username")
                .label("Username")
                .required("This field is required")
        );
        
        form.field(
            &GenericField::email("email")
                .label("Email")
                .required("This field is required")
        );
        
        form.field(
            &GenericField::textarea("bio")
                .label("Bio")
        );
    }
    
    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    
    fn get_form(&self) -> &Forms {
        &self.form
    }
    
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}
```

#### c) `to_active_model()` Method

Automatic conversion to SeaORM `ActiveModel`:

```rust
impl UserForm {
    pub fn to_active_model(&self) -> ActiveModel {
        use sea_orm::ActiveValue::Set;
        
        ActiveModel {
            username: Set(self.form.get_value("username").unwrap_or_default()),
            email: Set(self.form.get_value("email").unwrap_or_default()),
            bio: Set(self.form.get_value("bio")),
            ..Default::default()
        }
    }
}
```

#### d) `save()` Method

Direct database saving:

```rust
impl UserForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<Model, DbErr> {
        use sea_orm::EntityTrait;
        self.to_active_model().insert(db).await
    }
}
```

### Macro Comparison

| Element | `#[runique_form]` | `#[derive(DeriveModelForm)]` |
|---------|-------------------|------------------------------|
| Form struct | ❌ Manual | ✅ Generated |
| `impl RuniqueForm` | ❌ Manual | ✅ Generated |
| `impl Deref/DerefMut` | ✅ Generated | ✅ Generated |
| `to_active_model()` | ❌ Manual | ✅ Generated |
| `save()` | ❌ Manual | ✅ Generated |
| **Usage** | Custom forms | Model-linked forms |

---

## Available Field Types

### Text Fields

```rust
// Simple text field
GenericField::text("username")
    .placeholder("Username...")
    .required("Username is required")
    .min_length(3, "At least 3 characters")
    .max_length(20, "Maximum 20 characters")

// Textarea
GenericField::textarea("description")
    .placeholder("Long description...")
    .required("Description is required")
    .max_length(500, "Maximum 500 characters")

// Rich text (WYSIWYG)
GenericField::richtext("content")
    .required("Content is required")
```

### Fields with Special Validation

```rust
// Email (automatic format validation)
GenericField::email("email")
    .placeholder("example@domain.com")
    .required("Email required")

// URL (automatic format validation)
GenericField::url("website")
    .placeholder("https://example.com")

// Password (masked in HTML)
GenericField::password("password")
    .required("Password required")
    .min_length(8, "Minimum 8 characters")
```

### Numeric Fields

```rust
// Integer
GenericField::int("age")
    .required("Age is required")
```

### Configuration Methods (Builder Pattern)

All methods can be chained:

| Method | Description | Example |
|--------|-------------|---------|
| `.placeholder("text")` | Help text | `.placeholder("Enter your name")` |
| `.label("Label")` | Field label | `.label("Full Name")` |
| `.required("message")` | Required field | `.required("This field is required")` |
| `.min_length(n, "msg")` | Minimum length | `.min_length(3, "Min 3 characters")` |
| `.max_length(n, "msg")` | Maximum length | `.max_length(50, "Max 50 characters")` |

---

## Validation

### Automatic Validation

Call `is_valid()` to validate all fields:

```rust
pub async fn create_post(
    State(state): State<AppState>,
    Form(raw_data): Form<HashMap<String, String>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut post_form = PostForm::build_with_data(&raw_data, state.tera.clone());
    
    // Automatic validation of all fields
    if !post_form.is_valid().await {
        // Form contains errors
        return Err((
            StatusCode::BAD_REQUEST,
            Html(state.tera.render("post_form.html", &context! { form => post_form }).unwrap())
        ));
    }
    
    // Form is valid
    Ok(Redirect::to("/success"))
}
```

### Automatic Validations Performed

1. **Required fields**: Checks that required fields are not empty
2. **Min/max length**: Respects length constraints
3. **Email format**: Validates email format with robust regex
4. **URL format**: Validates URL format (http/https)
5. **Numeric types**: Checks conversion to number

### Custom Business Validation

Implement the `clean()` method for complex business validations:

```rust
impl RuniqueForm for RegisterForm {
    // ... other methods ...
    
    fn clean(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), HashMap<String, String>>> + Send + '_>> {
        Box::pin(async move {
            let mut errors = HashMap::new();
            
            // Check that passwords match
            let password = self.form.get_value("password").unwrap_or_default();
            let password_confirm = self.form.get_value("password_confirm").unwrap_or_default();
            
            if password != password_confirm {
                errors.insert(
                    "password_confirm".to_string(),
                    "Passwords do not match".to_string()
                );
            }
            
            // Check password strength
            if password.len() >= 8 && !password.chars().any(|c| c.is_numeric()) {
                errors.insert(
                    "password".to_string(),
                    "Password must contain at least one digit".to_string()
                );
            }
            
            if !errors.is_empty() {
                return Err(errors);
            }
            
            Ok(())
        })
    }
}
```

### Asynchronous Validation (with database)

```rust
impl RegisterForm {
    pub async fn validate_unique_email(&self, db: &DatabaseConnection) -> bool {
        use crate::models::users;
        use sea_orm::EntityTrait;
        
        let email = self.form.get_value("email").unwrap_or_default();
        
        let existing = users::Entity::find()
            .filter(users::Column::Email.eq(&email))
            .one(db)
            .await;
        
        existing.is_err() || existing.unwrap().is_none()
    }
}

// In handler
pub async fn register(
    Form(mut form): Form<RegisterForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    if !form.is_valid().await {
        return template.render("register.html", context! { form });
    }
    
    // Custom async validation
    if !form.validate_unique_email(&*db).await {
        form.get_form_mut()
            .fields
            .get_mut("email")
            .unwrap()
            .set_error("This email is already in use".to_string());
        
        return template.render("register.html", context! { form });
    }
    
    // Save...
}
```

---

## Error Display

### Django-like Syntax in Templates

Runique supports Django-like syntax for displaying forms in Tera templates. The framework automatically transforms these tags via custom filters.

#### Complete Automatic Rendering

```html
<form method="post">
    {% csrf %}
    {% form.register_form %}  <!-- Generates all fields automatically -->
    <button type="submit">Submit</button>
</form>
```

**What is generated**: The complete form with all fields, labels, errors, and HTML attributes.

#### Rendering a Specific Field

```html
<form method="post">
    {% csrf %}
    
    <div class="form-group">
        <label>Username</label>
        {% form.register_form.username %}
        <!-- Generates the input with all attributes and errors -->
    </div>
    
    <div class="form-group">
        <label>Email</label>
        {% form.register_form.email %}
    </div>
    
    <button type="submit">Register</button>
</form>
```

#### Alternative Syntax (explicit filter)

If you prefer using standard Tera syntax:

```html
<!-- Complete form -->
{{ register_form | form | safe }}

<!-- Specific field -->
{% set field = register_form | form(field="username") %}
{% set input_type = field.field_type %}
{% include "base_string" %}
```

### Manual Rendering with Full Control

For complete control over HTML:

```html
<form method="post">
    {% for field_name, field in form.fields %}
    <div class="form-group">
        <label for="{{ field.name }}">{{ field.label or field.name }}</label>
        <input 
            type="{{ field.field_type }}" 
            id="{{ field.name }}"
            name="{{ field.name }}"
            value="{{ field.value }}"
            placeholder="{{ field.placeholder }}"
            class="{% if field.error %}is-invalid{% endif %}"
            {% if field.is_required.choice %}required{% endif %}
        />
        
        {% if field.error %}
        <div class="invalid-feedback">{{ field.error }}</div>
        {% endif %}
    </div>
    {% endfor %}
    
    {% if form.global_errors %}
    <div class="alert alert-danger">
        <ul class="mb-0">
            {% for error in form.global_errors %}
            <li>{{ error }}</li>
            {% endfor %}
        </ul>
    </div>
    {% endif %}
    
    <button type="submit" class="btn btn-primary">Submit</button>
</form>
```

### How It Works

Runique automatically transforms Django-like syntax into Tera filter calls:

1. **`{% form.register_form %}`** → `{{ register_form | form | safe }}`
2. **`{% form.register_form.username %}`** → `{{ register_form | form(field="username") }}`

The `form` filter:
- **Without arguments**: returns the complete form HTML
- **With `field="name"`**: returns the HTML for the specific field

This transformation happens when templates are loaded (in `RuniqueApp::new()`)

### Other Django-like Tags

Runique supports several Django-like tags that are automatically transformed:

```html
{# CSRF Token #}
{% csrf %}  → {% include "csrf" %}

{# Flash messages #}
{% messages %}  → {% include "message" %}

{# CSP headers #}
{{ csp }}  → {% include "csp" %}

{# Static files #}
{% static "css/main.css" %}  → {{ "css/main.css" | static }}
{% static "js/app.js" %}     → {{ "js/app.js" | static }}

{# Media files #}
{% media "images/logo.png" %}  → {{ "images/logo.png" | media }}

{# Named URLs (reverse) #}
{% link "home" %}                      → {{ link(link='home') }}
{% link "user-detail", id=user.id %}   → {{ link(link='user-detail', id=user.id) }}
```

**These transformations are applied automatically** by Runique when loading templates via regex in `app.rs`.

### Retrieving Errors in Rust

```rust
// Check if form has errors
if post_form.form.has_errors() {
    // Get all errors
    let all_errors = post_form.form.errors();
    
    for (field_name, error_msg) in all_errors {
        println!("Error on {}: {}", field_name, error_msg);
    }
}

// Get error for specific field
if let Some(field) = post_form.form.fields.get("email") {
    if let Some(error) = field.error() {
        println!("Email error: {}", error);
    }
}

// Add global error manually
post_form.form.global_errors.push("Temporary server error".to_string());
```

### Error Types

1. **Field errors**: Associated with a specific field (displayed under the field)
2. **Global errors**: Errors not related to a particular field (displayed at top of form)

---

## Database Saving

### With `DeriveModelForm` (recommended method)

The macro automatically generates the `save()` method:

```rust
pub async fn create_article(
    Form(form): Form<ArticleForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    if !form.is_valid().await {
        return template.render("article_form.html", context! { form });
    }
    
    // ✅ Save in one line
    match form.save(&*db).await {
        Ok(article) => redirect(&format!("/article/{}", article.id)),
        Err(e) => {
            error!(message, "Error during creation");
            template.render("article_form.html", context! { form })
        }
    }
}
```

### With Manual Form

Add a `save()` method to your form:

```rust
impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        use sea_orm::{ActiveModelTrait, Set};
        use crate::models::users;
        
        let username = self.form.get_value("username").unwrap_or_default();
        let email = self.form.get_value("email").unwrap_or_default();
        let password = self.form.get_value("password").unwrap_or_default();
        
        let new_user = users::ActiveModel {
            username: Set(username),
            email: Set(email),
            password: Set(password),
            ..Default::default()
        };
        
        new_user.insert(db).await
    }
}
```

### Handling Database Errors

Use `database_error()` to automatically parse DB errors:

```rust
match register_form.save(&state.db).await {
    Ok(user) => {
        success!(message, "User created successfully!");
        Ok(Redirect::to(&format!("/user/{}", user.id)))
    }
    Err(db_err) => {
        // Parse error and assign to correct field automatically
        register_form.database_error(&db_err);
        
        Err((
            StatusCode::BAD_REQUEST,
            Html(state.tera.render("register.html", &context! { form => register_form }).unwrap())
        ))
    }
}
```

The `database_error()` method automatically detects:
- ✅ UNIQUE constraint violations
- ✅ Foreign key errors
- ✅ And assigns them to the concerned field with a localized message

---

## CSRF Protection

### Automatic Activation

CSRF protection is **automatically activated** when you add the middleware:

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

### Using in Templates

```html
<form method="post">
    <!-- Automatic CSRF token -->
    {% csrf %}

    {{ form }}

    <button type="submit">Submit</button>
</form>
```

### Automatic Validation

CSRF validation is **automatic** via the `Form<T>` extractor:

```rust
pub async fn submit_form(
    Form(form): Form<ContactForm>,  // ✅ CSRF validated automatically
    template: Template,
) -> Response {
    // If CSRF token is invalid, a 403 error is returned
    // BEFORE this code is executed
    
    if !form.is_valid() {
        return template.render("contact.html", context! { form });
    }

    // Form is valid AND CSRF token has been verified
}
```

**Note:** If the CSRF token is invalid, a `403 Forbidden` error is automatically returned **before** the handler is called.

---

## Complete Examples

### Example 1: Simple Contact Form

```rust
use runique::prelude::*;

#[derive(Serialize)]
#[serde(transparent)]
pub struct ContactForm {
    pub form: Forms,
}

impl Serialize for ContactForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.form.serialize(serializer)
    }
}

impl RuniqueForm for ContactForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &GenericField::text("name")
                .placeholder("Your name")
                .required("Name is required")
                .min_length(2, "At least 2 characters"),
        );
        
        form.field(
            &GenericField::email("email")
                .placeholder("your@email.com")
                .required("Email is required"),
        );
        
        form.field(
            &GenericField::text("subject")
                .placeholder("Message subject")
                .required("Subject is required"),
        );
        
        form.field(
            &GenericField::textarea("message")
                .placeholder("Your message...")
                .required("Message is required")
                .max_length(1000, "Maximum 1000 characters"),
        );
    }
    
    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    
    fn get_form(&self) -> &Forms {
        &self.form
    }
    
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}

// Handlers
pub async fn contact_view(
    State(state): State<AppState>,
) -> Html<String> {
    let form = ContactForm::build(state.tera.clone());
    
    Html(state.tera.render("contact.html", &tera::context! { form }).unwrap())
}

pub async fn contact_submit(
    State(state): State<AppState>,
    Form(raw_data): Form<HashMap<String, String>>,
    mut message: Message,
) -> Response {
    let mut form = ContactForm::build_with_data(&raw_data, state.tera.clone());
    
    if !form.is_valid().await {
        return Html(state.tera.render("contact.html", &tera::context! { form }).unwrap())
            .into_response();
    }
    
    // Process message (send email, save, etc.)
    // ...
    
    success!(message, "Message sent successfully!");
    Redirect::to("/").into_response()
}
```

### Example 2: Model-linked Form with `DeriveModelForm`

```rust
use runique::prelude::*;
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
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Automatic form generation
#[derive(DeriveModelForm)]
pub struct Article;

// Handlers
pub async fn create_article_view(
    State(state): State<AppState>,
) -> Html<String> {
    let form = ArticleForm::build(state.tera.clone());
    Html(state.tera.render("article_form.html", &tera::context! { form }).unwrap())
}

pub async fn store_article(
    Form(raw_data): Form<HashMap<String, String>>,
    State(state): State<AppState>,
    mut message: Message,
) -> Response {
    let mut form = ArticleForm::build_with_data(&raw_data, state.tera.clone());
    
    if !form.is_valid().await {
        error!(message, "Form contains errors");
        return Html(state.tera.render("article_form.html", &tera::context! { form }).unwrap())
            .into_response();
    }
    
    // ✅ Automatic save with .save()
    match form.save(&*state.db).await {
        Ok(article) => {
            success!(message, "Article created successfully!");
            Redirect::to(&format!("/article/{}", article.id)).into_response()
        }
        Err(e) => {
            form.database_error(&e);
            error!(message, "Error during creation");
            Html(state.tera.render("article_form.html", &tera::context! { form }).unwrap())
                .into_response()
        }
    }
}
```

### Example 3: Registration Form with Custom Validation

```rust
use runique::prelude::*;

#[derive(Serialize)]
#[serde(transparent)]
pub struct RegisterForm {
    pub form: Forms,
}

impl Serialize for RegisterForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.form.serialize(serializer)
    }
}

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &GenericField::text("username")
                .placeholder("Username")
                .required("Username is required")
                .min_length(3, "At least 3 characters")
                .max_length(20, "Maximum 20 characters"),
        );
        
        form.field(
            &GenericField::email("email")
                .placeholder("your@email.com")
                .required("Email is required"),
        );
        
        form.field(
            &GenericField::password("password")
                .required("Password is required")
                .min_length(8, "Minimum 8 characters"),
        );
        
        form.field(
            &GenericField::password("password_confirm")
                .required("Confirm your password"),
        );
    }
    
    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    
    fn get_form(&self) -> &Forms {
        &self.form
    }
    
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
    
    // Custom validation
    fn clean(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), HashMap<String, String>>> + Send + '_>> {
        Box::pin(async move {
            let mut errors = HashMap::new();
            
            let password = self.form.get_value("password").unwrap_or_default();
            let password_confirm = self.form.get_value("password_confirm").unwrap_or_default();
            
            // Check passwords match
            if password != password_confirm {
                errors.insert(
                    "password_confirm".to_string(),
                    "Passwords do not match".to_string()
                );
            }
            
            // Check password complexity
            if !password.chars().any(|c| c.is_numeric()) {
                errors.insert(
                    "password".to_string(),
                    "Password must contain at least one digit".to_string()
                );
            }
            
            if !password.chars().any(|c| c.is_uppercase()) {
                errors.insert(
                    "password".to_string(),
                    "Password must contain at least one uppercase letter".to_string()
                );
            }
            
            if !errors.is_empty() {
                return Err(errors);
            }
            
            Ok(())
        })
    }
}

impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        use sea_orm::{ActiveModelTrait, Set};
        use crate::models::users;
        
        let new_user = users::ActiveModel {
            username: Set(self.form.get_value("username").unwrap_or_default()),
            email: Set(self.form.get_value("email").unwrap_or_default()),
            password: Set(self.form.get_value("password").unwrap_or_default()),
            ..Default::default()
        };
        
        new_user.insert(db).await
    }
}
```

### Example 4: Editing an Existing Article

```rust
pub async fn edit_article_view(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Response {
    // Get existing article
    let article = match Article::find_by_id(id).one(&*state.db).await {
        Ok(Some(a)) => a,
        _ => return (StatusCode::NOT_FOUND, "Article not found").into_response(),
    };
    
    // Create form and pre-fill it
    let mut form = ArticleForm::build(state.tera.clone());
    
    if let Some(title_field) = form.form.fields.get_mut("title") {
        title_field.set_value(&article.title);
    }
    if let Some(slug_field) = form.form.fields.get_mut("slug") {
        slug_field.set_value(&article.slug);
    }
    if let Some(content_field) = form.form.fields.get_mut("content") {
        content_field.set_value(&article.content);
    }
    
    Html(state.tera.render("article_form.html", &tera::context! {
        form,
        article_id: id,
        is_edit: true,
    }).unwrap()).into_response()
}

pub async fn update_article(
    Path(id): Path<i32>,
    Form(raw_data): Form<HashMap<String, String>>,
    State(state): State<AppState>,
    mut message: Message,
) -> Response {
    let mut form = ArticleForm::build_with_data(&raw_data, state.tera.clone());
    
    if !form.is_valid().await {
        error!(message, "Form contains errors");
        return Html(state.tera.render("article_form.html", &tera::context! {
            form,
            article_id: id,
            is_edit: true,
        }).unwrap()).into_response();
    }
    
    // Get existing article
    let existing = match Article::find_by_id(id).one(&*state.db).await {
        Ok(Some(a)) => a,
        _ => return (StatusCode::NOT_FOUND, "Article not found").into_response(),
    };
    
    // Create ActiveModel for update
    let mut active_model: ActiveModel = existing.into();
    active_model.title = Set(form.form.get_value("title").unwrap_or_default());
    active_model.slug = Set(form.form.get_value("slug").unwrap_or_default());
    active_model.content = Set(form.form.get_value("content").unwrap_or_default());
    
    match active_model.update(&*state.db).await {
        Ok(updated) => {
            success!(message, "Article updated successfully!");
            Redirect::to(&format!("/article/{}", updated.id)).into_response()
        }
        Err(e) => {
            form.database_error(&e);
            error!(message, "Error during update");
            Html(state.tera.render("article_form.html", &tera::context! {
                form,
                article_id: id,
                is_edit: true,
            }).unwrap()).into_response()
        }
    }
}
```

---

## Best Practices

### 1. Always Validate Forms

```rust
// ✅ CORRECT
pub async fn submit(Form(raw_data): Form<HashMap<String, String>>) -> Response {
    let mut form = MyForm::build_with_data(&raw_data, tera);
    
    if !form.is_valid().await {
        return template.render("form.html", context! { form });
    }
    
    // Process validated data
}

// ❌ INCORRECT
pub async fn submit(Form(raw_data): Form<HashMap<String, String>>) -> Response {
    let form = MyForm::build_with_data(&raw_data, tera);
    
    // No validation!
    form.save(&db).await?; // Risk of invalid data
}
```

### 2. Use Auto-generated Methods

```rust
// ✅ OPTIMAL (with DeriveModelForm)
match form.save(&*db).await {
    Ok(article) => redirect("/success"),
    Err(e) => handle_error(e),
}

// ⚠️ ACCEPTABLE but more verbose
let active_model = form.to_active_model();
match active_model.insert(&*db).await {
    Ok(article) => redirect("/success"),
    Err(e) => handle_error(e),
}
```

### 3. Handle Errors Properly

```rust
pub async fn create_user(
    Form(raw_data): Form<HashMap<String, String>>,
    State(state): State<AppState>,
    mut message: Message,
) -> Response {
    let mut form = UserForm::build_with_data(&raw_data, state.tera.clone());
    
    if !form.is_valid().await {
        error!(message, "Form contains errors");
        return template.render("form.html", context! { form });
    }
    
    match form.save(&*state.db).await {
        Ok(user) => {
            success!(message, "User created!");
            Redirect::to(&format!("/user/{}", user.id)).into_response()
        }
        Err(DbErr::RecordNotFound(_)) => {
            error!(message, "Record not found");
            template.render("form.html", context! { form })
        }
        Err(DbErr::Exec(_)) => {
            // Database constraint violated
            form.database_error(&DbErr::Exec("".into()));
            error!(message, "Database constraint (duplicate?)");
            template.render("form.html", context! { form })
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            error!(message, "Internal error");
            template.render("form.html", context! { form })
        }
    }
}
```

### 4. Use Transactions for Complex Operations

```rust
pub async fn create_user_with_profile(
    Form(user_data): Form<HashMap<String, String>>,
    Form(profile_data): Form<HashMap<String, String>>,
    State(state): State<AppState>,
    mut message: Message,
) -> Response {
    let mut user_form = UserForm::build_with_data(&user_data, state.tera.clone());
    let mut profile_form = ProfileForm::build_with_data(&profile_data, state.tera.clone());
    
    if !user_form.is_valid().await || !profile_form.is_valid().await {
        error!(message, "Form contains errors");
        return template.render("form.html", context! {
            user_form,
            profile_form,
        });
    }
    
    // Transaction to ensure consistency
    let result = state.db.transaction::<_, (), DbErr>(|txn| {
        Box::pin(async move {
            // Create user
            let user = user_form.to_active_model().insert(txn).await?;
            
            // Create linked profile
            let mut profile = profile_form.to_active_model();
            profile.user_id = Set(user.id);
            profile.insert(txn).await?;
            
            Ok(())
        })
    }).await;
    
    match result {
        Ok(_) => {
            success!(message, "Account created successfully!");
            Redirect::to("/success").into_response()
        }
        Err(e) => {
            error!(message, "Error creating account");
            template.render("form.html", context! {
                user_form,
                profile_form,
            })
        }
    }
}
```

### 5. Use Message Macros

```rust
use runique::prelude::*;

pub async fn submit(
    Form(raw_data): Form<HashMap<String, String>>,
    State(state): State<AppState>,
    mut message: Message,
) -> Response {
    let mut form = ArticleForm::build_with_data(&raw_data, state.tera.clone());
    
    if !form.is_valid().await {
        error!(message, "Form contains errors");
        return template.render("form.html", context! { form });
    }
    
    match form.save(&*state.db).await {
        Ok(article) => {
            success!(message, "Article created successfully!");
            
            if article.published {
                info!(message, "Your article is now publicly visible");
            } else {
                warning!(message, "Your article is a draft");
            }
            
            Redirect::to(&format!("/article/{}", article.id)).into_response()
        }
        Err(e) => {
            error!(message, "Error during creation");
            template.render("form.html", context! { form })
        }
    }
}
```

---

## API Summary

### Form Lifecycle

1. **Creation**: `MyForm::build(tera)` or `build_with_data(&data, tera)`
2. **Validation**: `form.is_valid().await`
3. **Data Retrieval**: `form.data()` or `form.get_value("field")`
4. **Saving**: `form.save(&db).await`
5. **Error Handling**: `form.database_error(&err)`

### Main API

```rust
// Form Creation
MyForm::build(tera)                               // Empty form
MyForm::build_with_data(&data, tera)              // Pre-filled form

// Validation
form.is_valid().await                             // Validate all fields
form.has_errors()                                 // Check for errors

// Data Retrieval
form.data()                                       // HashMap<String, Value>
form.get_value("name")                            // Option<String>
form.get_value_or_default("name")                 // String

// Error Management
form.errors()                                     // HashMap<String, String>
form.global_errors                                // Vec<String>
form.database_error(&db_err)                      // Parse DB errors

// Field Access
field.error()                                     // Option<&String>
field.value()                                     // &str
field.validate()                                  // bool
field.set_value("value")                          // Modify value
field.set_error("message")                        // Add error
```

---

## See Also


- [Getting Started](informations/documentation_english/GETTING_STARTED.md)
- [Templates](informations/documentation_english/TEMPLATES.md)
- [Security](informations/documentation_english/CSP.md)
- [Database](informations/documentation_english/DATABASE.md)

Create robust forms with Runique!

---

**Version:** 1.0.86 (Corrected - January 2, 2026)
**Last Updated:** January 2026  
**License:** MIT

*Documentation created with ❤️ by Claude for Itsuki*
