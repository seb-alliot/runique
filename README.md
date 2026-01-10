# Runique

**A Django-inspired Rust web framework**

Runique is a modern web framework that combines Rust's safety and performance with Django's ergonomics. It offers a familiar development experience for Django developers while leveraging the power of Rust's type system.

[![Version](https://img.shields.io/badge/version-1.0.86-blue.svg)](https://crates.io/crates/runique)
[![docs.rs](https://img.shields.io/docsrs/runique)](https://docs.rs/runique)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

---

## 🤔 Why Runique?

- **For Django developers**: Familiar API and patterns with Rust's performance and safety
- **For Rust developers**: Django-inspired ergonomics without sacrificing type safety
- **For everyone**: Security built-in from day one, not bolted-on as an afterthought

---

## 🚀 Main Features

### Django-like Architecture
- **Declarative routing** with `urlpatterns!` macro
- **Intuitive ORM** based on SeaORM with Django-style API (`User::objects.filter(...)`)
- **Template system** Tera with custom preprocessing and Django-like tags
- **Automatic form generation** via procedural macros (`#[derive(DeriveModelForm)]`)
- **Flash messages** between requests with type safety
- **Static and media file management**

### Built-in Security
- ✅ **CSRF Protection** (HMAC-SHA256 with token masking against BREACH attacks)
- ✅ **Content Security Policy** (CSP) with automatic nonce generation
- ✅ **XSS Sanitization** with automatic input sanitization
- ✅ **Automatic Security Headers** (HSTS, X-Frame-Options, etc.)
- ✅ **ALLOWED_HOSTS Validation** with wildcard subdomain support
- ✅ **Integrated Argon2id Hashing** for passwords
- ✅ **Login Required Middleware** for authentication protection

### Advanced Form System
- **Automatic HTML generation** from models
- **Built-in validation** with custom rules
- **Field types**: CharField, EmailField, PasswordField, IntegerField, DateField, URLField, SlugField, FileField, SelectField, and more
- **SeaORM integration** with automatic model conversion
- **Error handling** with database constraint detection
- **CSRF protection** built into forms

### Multi-database Support
- PostgreSQL
- MySQL / MariaDB
- SQLite
- Connection pooling and timeout configuration
- Automatic driver detection from URL
- Easy database switching via environment variables

### Modern Development
- **Native Async/await** with Tokio
- **Type-safe** thanks to Rust's type system
- **Zero-cost abstractions**
- **CLI tool** for project scaffolding
- **Hot reload** in development
- **Complete documentation** with examples

---

## 📦 Installation

### Prerequisites

- Rust 1.75+ ([install Rust](https://www.rust-lang.org/tools/install))
- Cargo

### Add Runique to Your Project

```toml
# Cargo.toml

# Minimal configuration (SQLite)
[dependencies]
runique = { version = "1.0.86", features = ["sqlite"] }

# With PostgreSQL
[dependencies]
runique = { version = "1.0.86", features = ["postgres"] }

# With MySQL
[dependencies]
runique = { version = "1.0.86", features = ["mysql"] }

# With MariaDB
[dependencies]
runique = { version = "1.0.86", features = ["mariadb"] }

# With multiple databases (PostgreSQL + SQLite)
[dependencies]
runique = { version = "1.0.86", features = ["postgres", "sqlite"] }

# With all databases
[dependencies]
runique = { version = "1.0.86", features = ["all-databases"] }
```

### Available Cargo Features

| Feature | Description | Default |
|---------|-------------|---------|
| `orm` | Enables SeaORM | ✅ |
| `sqlite` | SQLite driver | ❌ (must be explicitly enabled) |
| `postgres` | PostgreSQL driver | ❌ (must be explicitly enabled) |
| `mysql` | MySQL driver | ❌ (must be explicitly enabled) |
| `mariadb` | MariaDB driver (uses MySQL driver) | ❌ (must be explicitly enabled) |
| `all-databases` | Enables all drivers simultaneously | ❌ (must be explicitly enabled) |

**Note:** You must explicitly specify at least one database driver feature.

**Configuration examples:**

```toml
# SQLite only
[dependencies]
runique = { version = "1.0.86", features = ["sqlite"] }

# PostgreSQL only
[dependencies]
runique = { version = "1.0.86", features = ["postgres"] }

# PostgreSQL + MySQL
[dependencies]
runique = { version = "1.0.86", features = ["postgres", "mysql"] }

# All databases
[dependencies]
runique = { version = "1.0.86", features = ["all-databases"] }

# Without ORM (minimal framework)
[dependencies]
runique = { version = "1.0.86", default-features = false }
```

### Create a New Project with CLI

```bash
# Install Runique CLI
cargo install runique

# Create a new project (generates complete structure)
runique new my_app
cd my_app

# Run the project
cargo run
```

The CLI generates a complete project structure with:
- Pre-configured `Cargo.toml`
- User model with authentication
- Registration and login forms
- Static files (CSS with dark theme)
- Templates with responsive design
- Database migrations ready
- Environment configuration

---

## 🛠️ CLI Tool

Runique provides a powerful CLI tool to scaffold new projects with a complete, production-ready structure.

### Creating a New Project

```bash
# Install the CLI (if not already installed)
cargo install runique

# Create a new project
runique new my_app

# Navigate to the project
cd my_app

# Run the application
cargo run
```

### Generated Project Structure

```
my_app/
├── Cargo.toml (pre-configured with Runique)
├── .env (database configuration)
├── .gitignore
├── README.md
├── src/
│   ├── main.rs (application entry point)
│   ├── forms.rs (form definitions)
│   ├── url.rs (URL patterns)
│   ├── views.rs (view handlers)
│   ├── models/
│   │   ├── mod.rs
│   │   └── users.rs (example User model)
│   ├── static/
│   │   ├── css/ (responsive dark theme included)
│   │   │   ├── main.css
│   │   │   ├── variables.css
│   │   │   ├── about.css
│   │   │   ├── register-form.css
│   │   │   └── search-user.css
│   │   ├── js/
│   │   └── images/
│   └── media/
│       ├── favicon/
│       │   └── favicon.ico
│       └── toshiro.jpg (example image)
└── templates/
    ├── index.html
    ├── about/
    │   └── about.html
    └── profile/
        ├── register_user.html
        └── view_user.html
```

The generated project includes:
- ✅ Complete CRUD example with User model
- ✅ Form validation and error handling
- ✅ Responsive CSS with dark theme
- ✅ CSRF protection enabled
- ✅ Flash messages configured
- ✅ Database migrations ready
- ✅ Authentication middleware examples

---

## 🏁 Quick Start

### Minimal Application

```rust
// src/main.rs
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RuniqueApp::new(settings).await?
        .routes(routes())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}

fn routes() -> Router<Arc<Tera>> {
    urlpatterns![
        "/" => get(index), name = "index",
        "/hello/{name}" => get(hello), name = "hello"
    ]
}

async fn index(template: Template) -> Response {
    let ctx = context!();
    template.render("index.html", &ctx)
}

async fn hello(
    Path(name): Path<String>,
    template: Template
) -> Response {
    let ctx = context! {
        "name", name
    };
    template.render("hello.html", &ctx)
}
```

### Configuration (.env)

```env
# Server Configuration
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=your-secret-key-here-change-in-production
ALLOWED_HOSTS=localhost,127.0.0.1

# Debug Mode (disable in production)
DEBUG=true

# Database Configuration (PostgreSQL example)
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb

# Or SQLite (default)
DB_ENGINE=sqlite
DB_NAME=app.db
```

### Launch

```bash
cargo run
```

Open [http://localhost:3000](http://localhost:3000)

---

## 📚 Documentation

- [🚀 Getting Started](informations/documentation_english/GETTING_STARTED.md)
- [⚙️ Configuration](informations/documentation_english/CONFIGURATION.md)
- [🗄️ Database](informations/documentation_english/DATABASE.md)
- [📝 Forms](informations/documentation_english/FORMULAIRE.md)
- [🎨 Templates](informations/documentation_english/TEMPLATES.md)
- [🔒 Security](informations/documentation_english/CSP.md)
- [🛣️ Macros](informations/documentation_english/MACRO_CONTEXT.md)
- [🔧 Changelog](informations/documentation_english/CHANGELOG.md)
- [🚀 Contributing](informations/documentation_english/CONTRIBUTING.md)
- [🆕 New Project](informations/documentation_english/NEW_PROJECT.md)
- [📖 API Documentation](https://docs.rs/runique)

---

## 🎯 Complete Example

### Project Structure
**Generated automatically with `runique new project_name`**

```
my_app/
├── Cargo.toml
├── .env
├── .gitignore
├── README.md
├── src/
│   ├── main.rs
│   ├── forms.rs
│   ├── url.rs
│   ├── views.rs
│   ├── models/
│   │   ├── mod.rs
│   │   └── users.rs
│   ├── static/
│   │   ├── css/
│   │   │   ├── main.css
│   │   │   ├── variables.css
│   │   │   ├── register-form.css
│   │   │   ├── search-user.css
│   │   │   └── about.css
│   │   ├── js/
│   │   └── images/
│   └── media/
│       └── favicon/
│           └── favicon.ico
└── templates/
    ├── index.html
    ├── about/
    │   └── about.html
    └── profile/
        ├── register_user.html
        └── view_user.html
```

### Model Definition with SeaORM

```rust
// src/models/users.rs
use sea_orm::entity::prelude::*;
use runique::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    
    #[sea_orm(unique)]
    pub username: String,
    
    #[sea_orm(unique)]
    pub email: String,
    
    pub password: String,
    pub age: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Add Django-style ORM methods
impl_objects!(Entity);
```

### Automatic Form Generation

```rust
// src/forms.rs
use runique::prelude::*;
use crate::models::users;

// Generate form automatically from model
#[derive(DeriveModelForm)]
#[model_form(model = "users::Model")]
pub struct UserForm;

// Form will include:
// - username (CharField)
// - email (EmailField - auto-detected)
// - password (PasswordField - auto-hashed with Argon2)
// - age (IntegerField)
// - CSRF token protection
// - Automatic validation
// - Error handling
```

### Advanced Handler with Form

```rust
// src/views.rs
use runique::prelude::*;
use crate::forms::UserForm;
use crate::models::{users, Entity as User};

// Display form (GET)
pub async fn register_form(template: Template) -> Response {
    let form = UserForm::build(template.tera.clone());
    
    let ctx = context! {
        "title", "User Registration";
        "form", form
    };
    
    template.render("profile/register_user.html", &ctx)
}

// Handle form submission (POST)
pub async fn register(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(form): ExtractForm<UserForm>,
) -> Response {
    // Automatic validation
    if form.is_valid() {
        match form.save(&db).await {
            Ok(user) => {
                success!(message, "Registration successful! Welcome!");
                
                let url = reverse_with_parameters(
                    "user_profile",
                    &[("id", &user.id.to_string())]
                ).unwrap();
                
                return Redirect::to(&url).into_response();
            }
            Err(err) => {
                // Automatic detection of database errors
                let mut form = form;
                form.get_form_mut().handle_database_error(&err);
                
                let ctx = context! {
                    "title", "Registration Error";
                    "form", form;
                    "messages", flash_now!(error, "An error occurred")
                };
                
                return template.render("profile/register_user.html", &ctx);
            }
        }
    }
    
    // Validation errors
    let ctx = context! {
        "title", "Validation Error";
        "form", form;
        "messages", flash_now!(error, "Please correct the errors")
    };
    
    template.render("profile/register_user.html", &ctx)
}

// Display user profile
pub async fn user_profile(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    // Django-style query with error handling
    match User::objects.get_or_404(&db, id, &template, "User not found").await {
        Ok(user) => {
            let ctx = context! {
                "title", "User Profile";
                "user", user
            };
            template.render("profile/view_user.html", &ctx)
        }
        Err(response) => response
    }
}

// List users with filtering
pub async fn user_list(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    // Django-style ORM query
    let users = User::objects
        .filter(users::Column::Age.gte(18))
        .order_by_desc(users::Column::CreatedAt)
        .limit(20)
        .all(&db)
        .await
        .unwrap_or_default();
    
    let ctx = context! {
        "title", "User List";
        "users", users
    };
    
    template.render("profile/user_list.html", &ctx)
}
```

### Templates with Django-like Syntax

```html
<!-- templates/profile/register_user.html -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{{ title }}</title>
    <link rel="stylesheet" href="{% static 'css/main.css' %}">
    <link rel="stylesheet" href="{% static 'css/register-form.css' %}">
</head>
<body>
    <div class="container">
        <h1>{{ title }}</h1>
        
        <!-- Flash messages -->
        {% messages %}
        
        <!-- Form with automatic CSRF protection -->
        <form method="post" action="{% link 'register' %}">
            {% csrf %}
            
            <!-- Automatic form rendering -->
            {% form.register_form %}
            
            <!-- Or render specific fields -->
            {% form.register_form.username %}
            {% form.register_form.email %}
            {% form.register_form.password %}
            {% form.register_form.age %}
            
            <button type="submit">Register</button>
        </form>
        
        <p>
            Already have an account? 
            <a href="{% link 'login' %}">Login</a>
        </p>
    </div>
    
    <!-- CSP-compliant JavaScript -->
    <script {{ csp }}>
        console.log('Registration form loaded');
    </script>
</body>
</html>
```

### Routing Configuration

```rust
// src/url.rs
use runique::prelude::*;
use crate::views;

pub fn routes() -> Router<Arc<Tera>> {
    urlpatterns![
        // Public routes
        "/" => get(views::index), name = "index",
        "/about" => get(views::about), name = "about",
        
        // Authentication
        "/register" => get(views::register_form)
                      .post(views::register), 
                      name = "register",
        
        "/login" => get(views::login_form)
                   .post(views::login), 
                   name = "login",
        
        "/logout" => post(views::logout), name = "logout",
        
        // Protected routes (with login_required middleware)
        "/profile/{id}" => get(views::user_profile)
                          .layer(middleware::from_fn(login_required)), 
                          name = "user_profile",
        
        "/users" => get(views::user_list)
                   .layer(middleware::from_fn(login_required)), 
                   name = "user_list"
    ]
}
```

### Main Application Setup

```rust
// src/main.rs
use runique::prelude::*;

mod models;
mod forms;
mod views;
mod url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load settings
    let settings = Settings::builder()
        .debug(true)
        .server("127.0.0.1", 3000, "secret-key")
        .sanitize_inputs(true)
        .build();
    
    // Database connection
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;
    
    // Build and run application
    RuniqueApp::new(settings).await?
        .with_database(db)
        .with_static_files()?
        .with_security_headers(CspConfig::strict())
        .with_default_middleware()
        .routes(url::routes())
        .run()
        .await?;
    
    Ok(())
}
```

---

## 🗄️ Database

### Configuration with Builder Pattern

```rust
use runique::prelude::*;

// From environment variables
let db_config = DatabaseConfig::from_env()?.build();
let db = db_config.connect().await?;

// Or with custom configuration
let db_config = DatabaseConfig::from_url("sqlite://app.db")?
    .max_connections(50)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(10))
    .logging(true)
    .build();
```

### Django-like ORM API

```rust
use crate::models::{users, Entity as User};

// All records
let all_users = User::objects.all().all(&db).await?;

// Get by ID
let user = User::objects.get(&db, 1).await?;

// Get by ID (returns Option)
let user: Option<Model> = User::objects.get_optional(&db, 1).await?;

// Get or 404 (automatic error response)
let user = User::objects.get_or_404(
    &db, 
    1, 
    &template, 
    "User not found"
).await?;

// Filtering
let active_users = User::objects
    .filter(users::Column::IsActive.eq(true))
    .filter(users::Column::Age.gte(18))
    .all(&db)
    .await?;

// Exclusion
let non_admin_users = User::objects
    .exclude(users::Column::Role.eq("admin"))
    .all(&db)
    .await?;

// Ordering
let recent_users = User::objects
    .order_by_desc(users::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;

// Pagination
let page_2 = User::objects
    .order_by_asc(users::Column::Username)
    .limit(20)
    .offset(20)
    .all(&db)
    .await?;

// Count
let total = User::objects.count(&db).await?;

// Get first result
let first_user = User::objects
    .order_by_asc(users::Column::CreatedAt)
    .first(&db)
    .await?;

// Query Builder with get_or_404
let user = User::objects
    .filter(users::Column::Username.eq("admin"))
    .get_or_404(&db, &template, "Admin user not found")
    .await?;

// Complex queries
let filtered = User::objects
    .filter(users::Column::Age.gte(18))
    .exclude(users::Column::Status.eq("banned"))
    .order_by_desc(users::Column::CreatedAt)
    .limit(50)
    .all(&db)
    .await?;
```

### Advanced ORM Methods

```rust
// RuniqueQueryBuilder methods
let query = User::objects
    .filter(users::Column::Age.gte(18))
    .order_by_desc(users::Column::CreatedAt);

// Get all results
let users: Vec<Model> = query.clone().all(&db).await?;

// Get first result
let first: Option<Model> = query.clone().first(&db).await?;

// Count results
let count: u64 = query.clone().count(&db).await?;

// Get first or 404
let user: Model = query
    .get_or_404(&db, &template, "No matching user found")
    .await?;
```

### Migrations with SeaORM CLI

```bash
# Install CLI
cargo install sea-orm-cli

# Initialize migrations
sea-orm-cli migrate init

# Create migration
sea-orm-cli migrate generate create_users_table

# Apply migrations
sea-orm-cli migrate up

# Rollback
sea-orm-cli migrate down

# Check status
sea-orm-cli migrate status

# Generate entities from existing database
sea-orm-cli generate entity \
    --database-url "sqlite://app.db" \
    --output-dir src/models
    
# For PostgreSQL
sea-orm-cli generate entity \
    --database-url "postgres://user:password@localhost/mydb" \
    --output-dir src/models

# For MySQL
sea-orm-cli generate entity \
    --database-url "mysql://user:password@localhost/mydb" \
    --output-dir src/models
```

**After generating entities, don't forget to:**

1. Add the `impl_objects!` macro to enable Django-style ORM:
```rust
// In your generated entity file (e.g., src/models/users.rs)
use runique::prelude::*;

// After the Entity definition, add:
impl_objects!(Entity);
```

2. Generate forms automatically from your models:
```rust
// In src/forms.rs
use runique::prelude::*;

#[derive(DeriveModelForm)]
#[model_form(model = "users::Model")]
pub struct UserForm;

// The form is now ready with:
// - Automatic field detection
// - Built-in validation
// - CSRF protection
// - Error handling
// - Database integration
```

### Automatic Form Generation

Runique provides a powerful form generation system that automatically creates forms from your SeaORM models.

#### Basic Usage

```rust
use runique::prelude::*;

// Your SeaORM model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub age: i32,
}

// Generate form automatically
#[derive(DeriveModelForm)]
#[model_form(model = "users::Model")]
pub struct UserForm;

// That's it! The form now includes:
// ✅ Automatic field type detection (CharField, EmailField, PasswordField, IntegerField)
// ✅ HTML generation for each field
// ✅ Built-in validation
// ✅ CSRF protection
// ✅ Error handling with user-friendly messages
// ✅ SeaORM integration (save directly to database)
```

#### Field Type Detection

The form generator automatically detects field types based on:

1. **Field names** (intelligent detection):
   - `email` → EmailField (with email validation)
   - `password`, `pwd` → PasswordField (automatically hashed with Argon2)
   - `url`, `link`, `website` → URLField
   - `slug` → SlugField
   - `description`, `bio`, `content`, `text` → TextField (textarea)

2. **Rust types**:
   - `String` → CharField
   - `i32`, `i64` → IntegerField
   - `f32`, `f64` → FloatField
   - `bool` → BooleanField (checkbox)
   - `NaiveDate` → DateField
   - `NaiveDateTime`, `DateTime` → DateTimeField
   - `IpAddr` → IPAddressField
   - `Value`, `Json` → JSONField

3. **Optional fields** (`Option<T>`):
   - Automatically detected as optional
   - No validation error if left empty

#### Using the Form

```rust
// Display form (GET request)
pub async fn register_form(template: Template) -> Response {
    let form = UserForm::build(template.tera.clone());
    
    let ctx = context! {
        "form", form
    };
    
    template.render("register.html", &ctx)
}

// Handle submission (POST request)
pub async fn register(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    ExtractForm(form): ExtractForm<UserForm>,
) -> Response {
    if form.is_valid() {
        // Save directly to database
        match form.save(&db).await {
            Ok(user) => {
                // User created successfully
                Redirect::to("/success").into_response()
            }
            Err(err) => {
                // Handle database errors
                let mut form = form;
                form.get_form_mut().handle_database_error(&err);
                // Re-render with errors
            }
        }
    }
    
    // Re-render with validation errors
}
```

#### Template Rendering

```html
<!-- Render entire form -->
{% form.user_form %}

<!-- Or render specific fields -->
<div class="form-group">
    {% form.user_form.username %}
</div>
<div class="form-group">
    {% form.user_form.email %}
</div>
<div class="form-group">
    {% form.user_form.password %}
</div>
```

#### Custom Validation

```rust
// Add custom validation logic
impl UserForm {
    pub fn validate_custom(&mut self) -> bool {
        let form = self.get_form_mut();
        
        // Access field values
        if let Some(age) = form.get_value::<i64>("age") {
            if age < 18 {
                form.add_error("age", "Must be 18 or older");
                return false;
            }
        }
        
        self.is_valid()
    }
}
```

#### Database Error Handling

The form system automatically detects common database errors:

```rust
// Automatically handles:
// ✅ Unique constraint violations
// ✅ Field-specific errors (username, email, etc.)
// ✅ User-friendly error messages

match form.save(&db).await {
    Ok(user) => { /* Success */ }
    Err(err) => {
        form.get_form_mut().handle_database_error(&err);
        // Error like "This username is already in use" 
        // automatically added to form.errors
    }
}
```

#### Advanced Features

```rust
// Manual field access
let username: Option<String> = form.get_value("username");
let age: Option<i64> = form.get_value("age");

// Check specific field errors
if let Some(error) = form.get_errors().get("email") {
    println!("Email error: {}", error);
}

// Add manual errors
form.get_form_mut().add_error("custom_field", "Custom error message");

// Convert to ActiveModel for advanced operations
let active_model = form.to_active_model();
```

---

## 🎨 Templates

### Django-like Template Tags

```html
<!-- Static files -->
<link rel="stylesheet" href="{% static 'css/style.css' %}">
<script src="{% static 'js/main.js' %}"></script>

<!-- Media files (user uploads) -->
<img src="{% media 'avatars/user.jpg' %}" alt="Avatar">

<!-- Runique internal assets -->
<link rel="stylesheet" href="{% runique_static 'css/error.css' %}">

<!-- CSRF token (automatic protection) -->
<form method="post">
    {% csrf %}
    <!-- form fields -->
</form>

<!-- Flash messages -->
{% messages %}

<!-- URL reversing -->
<a href="{% link 'home' %}">Home</a>
<a href="{% link 'user_profile' id=user.id %}">Profile</a>
<a href="{% link 'post_detail' slug=post.slug id=post.id %}">Read more</a>

<!-- CSP nonce for inline scripts -->
<script {{ csp }}>
    // This script is CSP-compliant
    console.log('Safe JavaScript');
</script>

<!-- Forms (automatic rendering) -->
{% form.user_form %}

<!-- Or render specific fields -->
{% form.user_form.username %}
{% form.user_form.email %}
```

### Template Context from Handler

```rust
use runique::prelude::*;

async fn my_handler(template: Template) -> Response {
    let ctx = context! {
        "title", "My Page";
        "user", user;
        "count", 42;
        "items", vec!["a", "b", "c"]
    };
    
    template.render("my_template.html", &ctx)
}
```

### Template Processor (Extractor)

The `Template` extractor automatically injects common variables into your templates:

```rust
use runique::prelude::*;

async fn handler(template: Template) -> Response {
    // Already available in templates without manual insertion:
    // - csrf_token (CSRF protection)
    // - messages (flash messages)
    // - debug (debug mode flag)
    // - csp_nonce (CSP nonce for inline scripts)
    // - static_runique (Runique's internal static URL)
    
    let ctx = context! { "user", user };
    template.render("profile.html", &ctx)
}

// Custom status codes
async fn not_found(template: Template) -> Response {
    let ctx = context! { "reason", "Page not found" };
    template.render_with_status("404.html", &ctx, StatusCode::NOT_FOUND)
}

// Helper methods
async fn error_handler(template: Template) -> Response {
    template.render_404("This resource does not exist")
    // or
    template.render_500("An error occurred")
}
```

### Message Extractor (Flash Messages)

The `Message` extractor provides a convenient API for flash messages:

```rust
use runique::prelude::*;

async fn create_user(mut message: Message) -> Response {
    // Send success message
    message.success("User created successfully").await?;
    
    // Or send multiple messages
    message.success("User created").await?;
    message.info("Verification email sent").await?;
    
    Redirect::to("/users").into_response()
}

async fn handle_form(mut message: Message, form: ExtractForm<UserForm>) -> Response {
    if form.is_valid() {
        message.success("Form saved!").await?;
    } else {
        message.error("Invalid form data").await?;
        message.warning("Please check your input").await?;
    }
    
    Redirect::to("/form").into_response()
}
```

### Tera Filters and Functions

Runique provides custom Tera filters and functions:

```html
<!-- Filters -->
{{ "style.css" | static }}           <!-- /static/style.css -->
{{ "avatar.jpg" | media }}           <!-- /media/avatar.jpg -->
{{ "error.css" | runique_static }}   <!-- /runique/static/error.css -->

<!-- Form rendering -->
{{ user_form | form }}               <!-- Render entire form -->
{{ user_form | form(field='email') }}  <!-- Render single field -->

<!-- URL reversing with parameters -->
{{ link(link='user_detail', id=123) }}
{{ link(link='post_detail', slug='my-post', id=456) }}

<!-- CSP nonce for inline scripts -->
<script {{ csp }}>
    console.log('CSP-compliant script');
</script>
```

---

## 📦 Utility Macros

### Flash Messages

```rust
use runique::prelude::*;

async fn my_handler(mut message: Message) -> Response {
    // Simple messages
    success!(message, "Operation successful!");
    error!(message, "An error occurred");
    info!(message, "Important information");
    warning!(message, "Warning");
    
    // Multiple messages
    success!(
        message,
        "User created",
        "Email sent",
        "Welcome!"
    );
    
    Redirect::to("/").into_response()
}

// Or use flash_now! for immediate display
async fn show_error(template: Template) -> Response {
    let ctx = context! {
        "messages", flash_now!(error, "Invalid credentials")
    };
    template.render("login.html", &ctx)
}
```

### Context Macro

```rust
// Simple key-value pairs
let ctx = context! {
    "name", "John";
    "age", 30;
    "active", true
};

// Works with any Serialize type
let ctx = context! {
    "user", user_model;
    "posts", posts_vec;
    "metadata", json!({"key": "value"})
};

// Empty context
let ctx = context!();
```

### URL Reversing

```rust
// Simple URL
let url = reverse("home").unwrap();

// URL with parameters
let url = reverse_with_parameters(
    "user_profile",
    &[("id", "123")]
).unwrap();

// Multiple parameters
let url = reverse_with_parameters(
    "post_detail",
    &[
        ("slug", "my-post"),
        ("id", "456")
    ]
).unwrap();
```

---

## 🔒 Security

### Built-in Security Features

Runique includes comprehensive security features enabled by default:

#### CSRF Protection

```rust
// Automatically enabled with default middleware
RuniqueApp::new(settings).await?
    .with_default_middleware()
    .run()
    .await?;

// Manual configuration
RuniqueApp::new(settings).await?
    .with_csrf_tokens()
    .run()
    .await?;
```

Templates automatically include CSRF tokens:
```html
<form method="post">
    {% csrf %}  <!-- Automatic CSRF token -->
    <!-- form fields -->
</form>
```

#### Content Security Policy

```rust
use runique::prelude::*;

// Strict CSP (recommended for production)
RuniqueApp::new(settings).await?
    .with_security_headers(CspConfig::strict())
    .run()
    .await?;

// Permissive CSP (for development)
RuniqueApp::new(settings).await?
    .with_csp(CspConfig::permissive())
    .run()
    .await?;

// Custom CSP
let csp = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
    img_src: vec!["'self'".to_string(), "https:".to_string()],
    use_nonce: true,
    ..Default::default()
};

RuniqueApp::new(settings).await?
    .with_security_headers(csp)
    .run()
    .await?;
```

#### ALLOWED_HOSTS Validation

```rust
// From .env
// ALLOWED_HOSTS=example.com,www.example.com,.api.example.com

let settings = Settings::from_env();

RuniqueApp::new(settings).await?
    .with_allowed_hosts(None)  // Uses .env
    .run()
    .await?;

// Or programmatically
RuniqueApp::new(settings).await?
    .with_allowed_hosts(Some(vec![
        "example.com".to_string(),
        ".api.example.com".to_string()  // Matches all subdomains
    ]))
    .run()
    .await?;
```

#### Input Sanitization

```rust
// Enable automatic sanitization
RuniqueApp::new(settings).await?
    .with_sanitize_text_inputs(true)
    .run()
    .await?;
```

Automatically sanitizes:
- XSS attacks (`<script>` tags)
- JavaScript event handlers (`onclick=`, etc.)
- `javascript:` protocol
- HTML tags in text inputs
- Preserves formatting (line breaks, spaces)

#### Authentication Middleware

```rust
use runique::prelude::*;

// Protect routes
let protected_routes = Router::new()
    .route("/dashboard", get(dashboard))
    .route("/profile", get(profile))
    .layer(middleware::from_fn(login_required));

// Redirect authenticated users
let public_routes = Router::new()
    .route("/login", get(login_form).post(login))
    .layer(middleware::from_fn(redirect_if_authenticated));
```

#### Password Hashing

```rust
// Automatic with PasswordField
use runique::formulaire::field::PasswordField;

let field = PasswordField;
let hashed = field.process("user_password").unwrap();
// Returns Argon2id hash

// Manual hashing
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng};

let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let hash = argon2.hash_password(b"password", &salt)
    .unwrap()
    .to_string();
```

### Security Headers

All security headers enabled with `.with_security_headers()`:

- ✅ Content-Security-Policy
- ✅ X-Content-Type-Options: nosniff
- ✅ X-Frame-Options: DENY
- ✅ X-XSS-Protection: 1; mode=block
- ✅ Referrer-Policy: strict-origin-when-cross-origin
- ✅ Permissions-Policy
- ✅ Cross-Origin-Embedder-Policy
- ✅ Cross-Origin-Opener-Policy
- ✅ Cross-Origin-Resource-Policy

---

## 🔐 Authentication & Authorization

### Authentication Middleware

Runique provides built-in middleware for protecting routes:

```rust
use runique::prelude::*;

// Protected routes (require authentication)
let protected_routes = Router::new()
    .route("/dashboard", get(dashboard))
    .route("/profile", get(profile))
    .layer(middleware::from_fn(login_required));

// Public routes (redirect authenticated users)
let public_routes = Router::new()
    .route("/login", get(login_page))
    .route("/register", get(register_page))
    .layer(middleware::from_fn(redirect_if_authenticated));
```

### Session Management

```rust
use runique::prelude::*;
use runique::middleware::login_requiert::*;

// Login a user
async fn login(session: Session, form: ExtractForm<LoginForm>) -> Response {
    if let Some(user) = authenticate_user(&form).await {
        login_user(&session, user.id, &user.username).await?;
        Redirect::to("/dashboard").into_response()
    } else {
        // Handle error
    }
}

// Logout a user
async fn logout(session: Session) -> Response {
    logout(&session).await?;
    Redirect::to("/").into_response()
}

// Check if authenticated
async fn check_auth(session: Session) -> Response {
    if is_authenticated(&session).await {
        // User is logged in
    }
}

// Get user info
async fn get_info(session: Session) -> Response {
    if let Some(user_id) = get_user_id(&session).await {
        if let Some(username) = get_username(&session).await {
            // Use user info
        }
    }
}
```

### CurrentUser Extractor

Use the `load_user_middleware` to automatically inject user information:

```rust
use runique::prelude::*;
use runique::middleware::login_requiert::{load_user_middleware, CurrentUser};

// Configure middleware
let app = Router::new()
    .route("/dashboard", get(dashboard))
    .layer(middleware::from_fn(load_user_middleware));

// Access current user in handlers
async fn dashboard(Extension(user): Extension<CurrentUser>) -> Response {
    // user.id and user.username are available
    let ctx = context! {
        "user_id", user.id;
        "username", &user.username
    };
    
    template.render("dashboard.html", &ctx)
}
```

### Permission Checking (Stub)

```rust
use runique::middleware::login_requiert::has_permission;

async fn admin_page(session: Session) -> Response {
    if has_permission(&session, "admin").await {
        // User has admin permission
    } else {
        // Access denied
    }
}
```

**Note**: `has_permission` is currently a stub. You'll need to implement the full permission logic with your database.

---

## 🛡️ Advanced Middleware

### Available Middleware

Runique provides several middleware components:

```rust
use runique::prelude::*;
use runique::middleware::*;

let app = RuniqueApp::new(settings).await?
    .routes(routes)
    // Error handling with custom 404/500 pages
    .layer(middleware::from_fn(error_handler_middleware))
    
    // Flash messages support
    .layer(middleware::from_fn(flash_middleware))
    
    // CSRF protection
    .layer(middleware::from_fn(csrf_middleware))
    
    // Input sanitization (if enabled in settings)
    .layer(middleware::from_fn_with_state(
        settings.clone(),
        sanitize_middleware
    ))
    
    // ALLOWED_HOSTS validation
    .layer(middleware::from_fn(allowed_hosts_middleware))
    
    // Security headers (CSP, HSTS, etc.)
    .layer(middleware::from_fn_with_state(
        CspConfig::strict(),
        security_headers_middleware
    ))
    
    // Authentication
    .layer(middleware::from_fn(login_required))
    
    // Auto-inject CurrentUser
    .layer(middleware::from_fn(load_user_middleware))
    
    .run()
    .await?;
```

### Error Handler Middleware

Automatically intercepts 404 and 500 errors:

```rust
// Configured automatically with .with_default_middleware()
// Or manually:
.layer(middleware::from_fn(error_handler_middleware))

// In debug mode: shows detailed error pages
// In production: shows custom 404.html and 500.html templates
```

### Sanitization Middleware

Automatically sanitizes form inputs to prevent XSS:

```rust
let settings = Settings::builder()
    .sanitize_inputs(true)  // Enable auto-sanitization
    .build();

// Middleware automatically sanitizes:
// - application/x-www-form-urlencoded (forms)
// - application/json (APIs)
// - Skips sensitive fields (password, token, secret, key)
```

### CSRF Token Functions

Advanced CSRF token management:

```rust
use runique::utils::*;

// Generate masked token (protection against BREACH attack)
let masked_token = mask_csrf_token(&raw_token);

// Unmask token for validation
let raw_token = unmask_csrf_token(&masked_token)?;

// Generate user-specific token
let user_token = generate_user_token(&secret_key, &user_id.to_string());
```

---

## 🚀 Performance

Runique leverages Rust and Tokio for exceptional performance:

- **Zero-cost abstractions**: No runtime overhead
- **Native async/await**: Efficient concurrency with Tokio
- **Connection pooling**: Optimized database connection management
- **Optimized compilation**: Highly optimized binary with LTO
- **Memory safety**: No garbage collector, predictable performance

### Benchmark (indicative)

```
Setup: AMD Ryzen 7 5800X, 32GB RAM
Requests/sec: ~60,000
Latency p50: ~0.8ms
Latency p99: ~3ms
Memory: ~15MB (idle)
```

*Note: Actual performance depends on your hardware and application complexity.*

---

## 🛠️ Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with specific features
cargo test --features sqlite
cargo test --features postgres

# Run integration tests
cargo test --test integration

# Run doc tests
cargo test --doc

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Linting
cargo clippy

# Formatting
cargo fmt

# Check without building
cargo check

# Security audit
cargo audit
```

### Documentation

```bash
# Generate and open documentation
cargo doc --open --no-deps

# Test documentation examples
cargo test --doc

# Check documentation coverage
cargo doc --document-private-items
```

### Benchmarking

```bash
# Run benchmarks (requires nightly)
cargo +nightly bench

# With specific features
cargo +nightly bench --features all-databases
```

---

## 🤝 Contributing

Contributions are welcome! Here's how to contribute:

1. **Fork the project**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Commit your changes**
   ```bash
   git commit -m 'Add amazing feature'
   ```
4. **Push to the branch**
   ```bash
   git push origin feature/amazing-feature
   ```
5. **Open a Pull Request**

### Guidelines

- ✅ Write tests for new features
- ✅ Follow Rust conventions (`cargo fmt`)
- ✅ Document public APIs with examples
- ✅ Update CHANGELOG.md
- ✅ Add examples if relevant
- ✅ Ensure all tests pass
- ✅ Run `cargo clippy` before submitting

### Development Setup

```bash
# Clone the repository
git clone https://github.com/seb-alliot/runique.git
cd runique

# Install development dependencies
cargo install cargo-watch
cargo install cargo-edit
cargo install sea-orm-cli

# Run tests in watch mode
cargo watch -x test

# Run with hot reload
cargo watch -x run
```

See [CONTRIBUTING.md](informations/documentation_english/CONTRIBUTING.md) for more details.

---

## 📝 Roadmap

### Version 1.1 (Current)
- [x] CLI tool for project generation
- [x] Complete form system with validation
- [x] CSRF protection with token masking
- [x] CSP with nonce generation
- [x] Automatic input sanitization
- [x] Login/logout middleware
- [ ] Session management improvements
- [ ] Rate limiting middleware

### Version 1.2
- [ ] Admin panel generator
- [ ] WebSocket support
- [ ] Background jobs with Tokio
- [ ] Cache layer (Redis)
- [ ] File upload handling
- [ ] Email integration

### Version 2.0
- [ ] GraphQL support
- [ ] Plugin system
- [ ] Multi-tenancy
- [ ] Internationalization (i18n)
- [ ] Advanced ORM features (relationships, aggregation)
- [ ] Real-time features
- [ ] Microservices support

---

## 📚 API Reference

### Utility Macros

```rust
// get_or_return! - Unwrap or early return
let value = get_or_return!(some_result);
// Equivalent to:
let value = match some_result {
    Ok(v) => v,
    Err(e) => return e,
};

// view! - Combined GET/POST routing
let route = view!(
    GET => get_handler,
    POST => post_handler
);
```

### Form Utilities

```rust
use runique::formulaire::*;

// Check if a value contains dangerous content
if is_dangerous("<script>alert('xss')</script>") {
    // Handle dangerous input
}

// Check if a field is sensitive (password, token, secret, key)
if is_sensitive_field("password") {
    // Don't sanitize this field
}

// Manual sanitization
let clean = auto_sanitize("<script>alert('xss')</script>");
// Result: "alert('xss')"
```

### Response Helpers

```rust
use runique::response::*;

// JSON response
let response = json_response(
    StatusCode::OK,
    json!({ "status": "success", "data": data })
);

// HTML response
let response = html_response(
    StatusCode::OK,
    "<h1>Hello World</h1>"
);

// Redirect
let response = redirect("/dashboard");

// Fallback 404 page (when template not found)
let response = fallback_404_html();
```

### Settings Configuration

All available settings fields:

```rust
let settings = Settings::builder()
    // Server
    .server("0.0.0.0", 8000, "secret-key")
    
    // Security
    .debug(false)
    .allowed_hosts(vec!["example.com".to_string()])
    .sanitize_inputs(true)
    .strict_csp(true)
    .rate_limiting(true)
    .enforce_https(true)
    
    // Paths
    .templates_dir(vec!["templates".to_string()])
    .staticfiles_dirs("static")
    .media_root("media")
    .static_url("/static")
    .media_url("/media")
    
    // Runique internal paths (usually don't need to change)
    .static_runique_path("path/to/runique/static")
    .static_runique_url("/runique/static")
    .media_runique_path("path/to/runique/media")
    .media_runique_url("/runique/media")
    .templates_runique("path/to/runique/templates")
    
    .build();

// Additional fields available in Settings struct:
// - installed_apps: Vec<String>
// - middleware: Vec<String>
// - root_urlconf: String
// - staticfiles_storage: String
// - language_code: String (default: "en-us")
// - time_zone: String (default: "UTC")
// - use_i18n: bool
// - use_tz: bool
// - auth_password_validators: Vec<String>
// - password_hashers: Vec<String>
// - default_auto_field: String
// - logging_config: String
```

### Error Context

```rust
use runique::error::*;

// Create from Tera error
let ctx = ErrorContext::from_tera_error(&error, "template.html", &tera);

// Create from anyhow error
let ctx = ErrorContext::from_anyhow(&error);

// Create 404 error
let ctx = ErrorContext::not_found("/missing-page");

// Create generic error
let ctx = ErrorContext::generic(StatusCode::BAD_REQUEST, "Invalid input")
    .with_request(&request)
    .with_details("Expected JSON, got XML");

// Available fields in ErrorContext:
// - status_code: u16
// - error_type: ErrorType (Template, NotFound, Internal, Database, Validation)
// - timestamp: String (ISO 8601)
// - title: String
// - message: String
// - details: Option<String>
// - template_info: Option<TemplateInfo>
// - request_info: Option<RequestInfo>
// - stack_trace: Vec<StackFrame>
// - environment: EnvironmentInfo
```

### Session Traits

```rust
use tower_sessions::Session;
use runique::middleware::csrf::CsrfSession;
use runique::middleware::flash_message::FlashMessageSession;

// CSRF token management
let token = session.get_csrf_token().await?;

// Flash messages
session.insert_message(FlashMessage::success("Done!")).await?;
session.insert_message(FlashMessage::error("Failed!")).await?;
session.insert_message(FlashMessage::info("Note")).await?;
session.insert_message(FlashMessage::warning("Be careful")).await?;
```

### CSRF Token Management

```rust
use runique::utils::*;

// Generate secure token
let token = generate_token("secret_key", "session_id");

// Generate user-specific token
let user_token = generate_user_token("secret_key", &user_id.to_string());

// Mask token (protection against BREACH attack)
let masked = mask_csrf_token(&token);

// Unmask token for validation
let original = unmask_csrf_token(&masked)?;
```

---

## 📄 License

This project is licensed under the MIT License. See the [LICENSE-MIT](LICENSE-MIT) file for details.

---

## 🙏 Acknowledgments

Runique builds upon excellent libraries from the Rust ecosystem:

- [Axum](https://github.com/tokio-rs/axum) - Web framework foundation
- [Tokio](https://tokio.rs/) - Async runtime
- [SeaORM](https://www.sea-ql.org/SeaORM/) - ORM with great developer experience
- [Tera](https://keats.github.io/tera/) - Django-inspired template engine
- [Tower](https://github.com/tower-rs/tower) - Middleware and service abstractions
- [Argon2](https://github.com/RustCrypto/password-hashes) - Secure password hashing
- [Serde](https://serde.rs/) - Serialization framework

Special thanks to:
- The Django project for inspiration
- The Rust community for incredible tools
- All contributors who help improve Runique

---

## 📧 Contact

- **GitHub**: [seb-alliot/runique](https://github.com/seb-alliot/runique)
- **Issues**: [Report bugs or request features](https://github.com/seb-alliot/runique/issues)
- **Discord**: [Join our community](https://discord.gg/Y5zW7rbt)
- **Email**: alliotsebastien04@gmail.com
- **Crates.io**: [runique](https://crates.io/crates/runique)
- **Docs.rs**: [API Documentation](https://docs.rs/runique)

---

## ⭐ Support the Project

If Runique helps you build better web applications, consider:

- ⭐ [Star the project on GitHub](https://github.com/seb-alliot/runique)
- 🐛 [Report bugs and issues](https://github.com/seb-alliot/runique/issues)
- 💡 [Suggest new features](https://github.com/seb-alliot/runique/issues/new)
- 📖 [Improve documentation](https://github.com/seb-alliot/runique/tree/main/informations/documentation_english)
- 🤝 [Contribute code](https://github.com/seb-alliot/runique/pulls)
- 💬 [Join our Discord community](https://discord.gg/Y5zW7rbt)
- 📢 Share Runique with others

---

## 🌟 Featured Projects

Projects built with Runique:

- **Coming soon!** Be the first to showcase your project

Want to add your project? [Contact us](mailto:alliotsebastien04@gmail.com) or submit a PR!

---

**Build secure and performant web applications with Runique!** 🦀

---

**Current Version:** 1.0.86  
**License:** MIT  
**Status:** Active Development  
**Rust Version:** 1.75+  

*Made with ❤️ and 🦀 by the Runique community*