Understood. Here is your guide rewritten entirely in English (no side-by-side translation, English only).

---

# Full CRUD Application with Runique

## Introduction

This guide walks you through building a complete CRUD (Create, Read, Update, Delete) application with Runique, including:

* PostgreSQL database
* HTML templates
* Form validation
* Flash messages
* CSRF protection

**Estimated time:** 30–45 minutes

---

## Prerequisites

* Rust 1.75 or higher
* PostgreSQL installed and running
* Basic SQL knowledge

**Verify PostgreSQL:**

```bash
psql --version
```

---

## What We Will Build

A Todo management application with:

* Task listing
* Task creation
* Task editing
* Task deletion
* Mark as completed

**Features:**

* Django-like ORM API
* Template inheritance
* Form validation
* Flash messages
* Responsive UI

---

## Step 1: Create the project

```bash
cargo new todo-app
cd todo-app
```

---

## Step 2: Configuration

### Cargo.toml

```toml
[package]
name = "todo-app"
version = "1.0"
edition = "2021"

[dependencies]
runique = { version = "1.0.86", features = ["orm", "sqlite"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sea-orm = { version = "1", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }
chrono = { version = "0.4", features = ["serde"] }
```

### .env file

Create `.env` at the project root:

```env
# Server
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=change-this-secret-key-in-production

# Database
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=postgres
DB_HOST=localhost
DB_PORT=5432
DB_NAME=todo_app
```

---

## Step 3: Create the database

```bash
psql -U postgres
```

Then inside PostgreSQL:

```sql
CREATE DATABASE todo_app;
\c todo_app

CREATE TABLE tasks (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    completed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

\q
```

---

## Step 4: Project structure

```bash
mkdir -p src templates static/css
touch src/{main.rs,models.rs,urls.rs,views.rs,forms.rs}
touch templates/{base.html,index.html,create.html,edit.html}
touch static/css/style.css
```

Final structure:

```
todo-app/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs
│   ├── models.rs
│   ├── urls.rs
│   ├── views.rs
│   └── forms.rs
├── templates/
│   ├── base.html
│   ├── index.html
│   ├── create.html
│   └── edit.html
└── static/
    └── css/
        └── style.css
```

---

## Step 5: Data model

### src/models.rs
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use runique::impl_objects;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Active l'API Django-like
impl_objects!(Entity);
```

---


## Étape 6 : Formulaires

### src/forms.rs
```rust
use runique::prelude::*;
use runique::formulaire::formsrunique::{Forms, FormulaireTrait};
use runique::formulaire::field::{CharField, TextField, BooleanField};
use std::collections::HashMap;

#[runique_form]
pub struct TaskForm {
    pub form: Forms,
}

impl FormulaireTrait for TaskForm {
    fn new() -> Self {
        Self { form: Forms::new() }
    }

    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        // Titre requis
        self.require("title", &CharField { allow_blank: false }, raw_data);

        // Description optionnelle
        self.optional("description", &TextField, raw_data);

        // Completed optionnel (checkbox)
        self.optional("completed", &BooleanField, raw_data);

        self.is_valid()
    }
}

impl TaskForm {
    // Validation personnalisée : titre entre 3 et 255 caractères
    pub fn clean(&mut self, raw_data: &HashMap<String, String>) {
        if self.is_not_valid() {
            return;
        }

        if let Some(title) = self.get_value::<String>("title") {
            if title.len() < 3 {
                self.errors.insert(
                    "title".to_string(),
                    "Le titre doit contenir au moins 3 caractères".to_string()
                );
            }
            if title.len() > 255 {
                self.errors.insert(
                    "title".to_string(),
                    "Le titre ne peut pas dépasser 255 caractères".to_string()
                );
            }
        }
    }
}
```

---
## Step 7: Views / Handlers
### src/views.rs
```rust
use runique::prelude::*;
use crate::models::{tasks, Entity as Task};
use crate::forms::TaskForm;
use sea_orm::ActiveValue::Set;

// Liste des tâches
pub async fn index(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    // Récupérer toutes les tâches, triées par date de création
    let tasks = Task::objects
        .order_by_desc(tasks::Column::CreatedAt)
        .all(&*db)
        .await
        .unwrap_or_default();

    let ctx = context! {
        "title", "Liste des tâches" ;
        "tasks", tasks
    };

    template.render("index.html", &ctx)
}

// Formulaire de création
pub async fn create_form(template: Template) -> Response {
    let form = TaskForm::new();

    let ctx = context! {
        "title", "Nouvelle tâche" ;
        "form", &form
    };

    template.render("create.html", &ctx)
}

// Traitement de la création
pub async fn create_submit(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    ExtractForm(mut form): ExtractForm<TaskForm>,
    mut message: Message,
) -> Response {
    // Validation supplémentaire
    form.clean(&form.form.cleaned_data.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect());

    if !form.is_valid() {
        let ctx = context! {
            "title", "Nouvelle tâche" ;
            "form", &form
        };
        return template.render("create.html", &ctx);
    }

    // Créer la tâche
    let task = tasks::ActiveModel {
        title: Set(form.get_value("title").unwrap()),
        description: Set(form.get_value("description")),
        completed: Set(form.get_value("completed").unwrap_or(false)),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    match task.insert(&*db).await {
        Ok(_) => {
            let _ = message.success("Tâche créée avec succès !").await;
            redirect("/")
        }
        Err(e) => {
            let _ = message.error(&format!("Erreur : {}", e)).await;
            let ctx = context! {
                "title", "Nouvelle tâche" ;
                "form", &form
            };
            template.render("create.html", &ctx)
        }
    }
}

// Formulaire d'édition
pub async fn edit_form(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
) -> Response {
    // Récupérer la tâche
    let task = match Task::objects.get(&*db, id).await {
        Ok(task) => task,
        Err(_) => {
            let _ = message.error("Tâche introuvable").await;
            return redirect("/");
        }
    };

    let ctx = context! {
        "title", "Modifier la tâche" ;
        "task", task
    };

    template.render("edit.html", &ctx)
}

// Traitement de la modification
pub async fn edit_submit(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    ExtractForm(mut form): ExtractForm<TaskForm>,
    mut message: Message,
) -> Response {
    // Validation
    form.clean(&form.form.cleaned_data.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect());

    if !form.is_valid() {
        // Récupérer la tâche pour l'afficher à nouveau
        let task = Task::objects.get(&*db, id).await.unwrap();
        let ctx = context! {
            "title", "Modifier la tâche" ;
            "form", &form ;
            "task", task
        };
        return template.render("edit.html", &ctx);
    }

    // Récupérer et mettre à jour la tâche
    let mut task: tasks::ActiveModel = Task::objects
        .get(&*db, id)
        .await
        .unwrap()
        .into();

    task.title = Set(form.get_value("title").unwrap());
    task.description = Set(form.get_value("description"));
    task.completed = Set(form.get_value("completed").unwrap_or(false));
    task.updated_at = Set(chrono::Utc::now());

    match task.update(&*db).await {
        Ok(_) => {
            let _ = message.success("Tâche modifiée avec succès !").await;
            redirect("/")
        }
        Err(e) => {
            let _ = message.error(&format!("Erreur : {}", e)).await;
            redirect(&format!("/edit/{}", id))
        }
    }
}

// Suppression
pub async fn delete(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
) -> Response {
    match Task::objects.get(&*db, id).await {
        Ok(task) => {
            let active_model: tasks::ActiveModel = task.into();
            match active_model.delete(&*db).await {
                Ok(_) => {
                    let _ = message.success("Tâche supprimée avec succès !").await;
                }
                Err(e) => {
                    let _ = message.error(&format!("Erreur : {}", e)).await;
                }
            }
        }
        Err(_) => {
            let _ = message.error("Tâche introuvable").await;
        }
    }

    redirect("/")
}

// Toggle completed
pub async fn toggle_completed(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
) -> Response {
    match Task::objects.get(&*db, id).await {
        Ok(task) => {
            let mut active_task: tasks::ActiveModel = task.into();
            active_task.completed = Set(!active_task.completed.unwrap());
            active_task.updated_at = Set(chrono::Utc::now());

            match active_task.update(&*db).await {
                Ok(_) => {
                    let _ = message.success("Statut mis à jour !").await;
                }
                Err(e) => {
                    let _ = message.error(&format!("Erreur : {}", e)).await;
                }
            }
        }
        Err(_) => {
            let _ = message.error("Tâche introuvable").await;
        }
    }

    redirect("/")
}
```


## Step 8: Routes
### src/urls.rs
```rust
use runique::{Router, urlpatterns};
use crate::views;

pub fn routes() -> Router {
    urlpatterns! {
        "/" => get(views::index), name = "index",
        "/create" => get(views::create_form), name = "create_form",
        "/create" => post(views::create_submit), name = "create_submit",
        "/edit/{id}" => get(views::edit_form), name = "edit_form",
        "/edit/{id}" => post(views::edit_submit), name = "edit_submit",
        "/delete/{id}" => post(views::delete), name = "delete",
        "/toggle/{id}" => post(views::toggle_completed), name = "toggle_completed",
    }
}
```

## Step 9: Main application

```rust
use runique::{Router, urlpatterns};
use crate::views;

pub fn routes() -> Router {
    urlpatterns! {
        "/" => get(views::index), name = "index",
        "/create" => get(views::create_form), name = "create_form",
        "/create" => post(views::create_submit), name = "create_submit",
        "/edit/{id}" => get(views::edit_form), name = "edit_form",
        "/edit/{id}" => post(views::edit_submit), name = "edit_submit",
        "/delete/{id}" => post(views::delete), name = "delete",
        "/toggle/{id}" => post(views::toggle_completed), name = "toggle_completed",
    }
}
```

---

## Étape 9 : Application principale

### src/main.rs
```rust
use runique::prelude::*;
use std::sync::Arc;

mod models;
mod forms;
mod urls;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Charger .env
    dotenvy::dotenv().ok();

    // Configuration
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .staticfiles_dirs("static")
        .server(
            &std::env::var("IP_SERVER").unwrap_or_else(|_| "127.0.0.1".to_string()),
            std::env::var("PORT")?.parse()?,
            &std::env::var("SECRET_KEY")?,
        )
        .build();

    // Base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    println!("🦀 Runique Todo App starting...");
    println!("📊 Database connected");
    println!("🌐 Server running on http://{}:{}",
        settings.server.ip_server,
        settings.server.port
    );

    // Lancer l'application
    RuniqueApp::new(settings).await?
        .with_database(db)
        .routes(urls::routes())
        .with_static_files()?
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

## Step 10: Templates
### templates/base.html
```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Todo App{% endblock %} - Runique</title>
    <link rel="stylesheet" href='{% static "css/style.css" %}'>
</head>
<body>
    <nav class="navbar">
        <div class="container">
            <a href='{% link "index" %}' class="logo">📝 Todo App</a>
            <div class="nav-links">
                <a href='{% link "index" %}'>Tâches</a>
                <a href='{% link "create_form" %}' class="btn-primary">Nouvelle tâche</a>
            </div>
        </div>
    </nav>

    <main class="container">
        {% messages %}

        {% block content %}
        {% endblock %}
    </main>

    <footer class="footer">
        <div class="container">
            <p>Développé avec Runique Framework</p>
        </div>
    </footer>
</body>
</html>
```

### templates/index.html
```html
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
<div class="page-header">
    <h1>{{ title }}</h1>
    <a href='{% link "create_form" %}' class="btn btn-primary">Nouvelle tâche</a>
</div>

{% if tasks %}
<div class="tasks-list">
    {% for task in tasks %}
    <div class="task-card {% if task.completed %}completed{% endif %}">
        <div class="task-header">
            <h3>{{ task.title }}</h3>
            <span class="task-date">{{ task.created_at | date(format="%d/%m/%Y %H:%M") }}</span>
        </div>

        {% if task.description %}
        <p class="task-description">{{ task.description }}</p>
        {% endif %}

        <div class="task-actions">
            <form method="post" action='{% link "toggle_completed", id=task.id %}' style="display: inline;">
                {% csrf %}
                <button type="submit" class="btn btn-small {% if task.completed %}btn-warning{% else %}btn-success{% endif %}">
                    {% if task.completed %}Réactiver{% else %}Terminer{% endif %}
                </button>
            </form>

            <a href='{% link "edit_form", id=task.id %}' class="btn btn-small btn-secondary">Modifier</a>

            <form method="post" action='{% link "delete", id=task.id %}' style="display: inline;" onsubmit="return confirm('Êtes-vous sûr ?');">
                {% csrf %}
                <button type="submit" class="btn btn-small btn-danger">Supprimer</button>
            </form>
        </div>
    </div>
    {% endfor %}
</div>
{% else %}
<div class="empty-state">
    <p>Aucune tâche pour le moment.</p>
    <a href='{% link "create_form" %}' class="btn btn-primary">Créer votre première tâche</a>
</div>
{% endif %}
{% endblock %}
```

### templates/create.html
```html
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
<div class="page-header">
    <h1>{{ title }}</h1>
</div>

<div class="form-container">
    <form method="post" action='{% link "create_submit" %}'>
        {% csrf %}

        <div class="form-group">
            <label for="title">Titre *</label>
            <input
                type="text"
                name="title"
                id="title"
                value="{{ form.cleaned_data.title | default(value='') }}"
                class="{% if form.errors.title %}error{% endif %}"
                required
            >
            {% if form.errors.title %}
                <span class="error-message">{{ form.errors.title }}</span>
            {% endif %}
        </div>

        <div class="form-group">
            <label for="description">Description</label>
            <textarea
                name="description"
                id="description"
                rows="5"
                class="{% if form.errors.description %}error{% endif %}"
            >{{ form.cleaned_data.description | default(value='') }}</textarea>
            {% if form.errors.description %}
                <span class="error-message">{{ form.errors.description }}</span>
            {% endif %}
        </div>

        <div class="form-actions">
            <button type="submit" class="btn btn-primary">Créer</button>
            <a href='{% link "index" %}' class="btn btn-secondary">Annuler</a>
        </div>
    </form>
</div>
{% endblock %}
```

### templates/edit.html
```html
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
<div class="page-header">
    <h1>{{ title }}</h1>
</div>

<div class="form-container">
    <form method="post" action='{% link "edit_submit", id=task.id %}'>
        {% csrf %}

        <div class="form-group">
            <label for="title">Titre *</label>
            <input
                type="text"
                name="title"
                id="title"
                value="{{ form.cleaned_data.title | default(value=task.title) }}"
                class="{% if form.errors.title %}error{% endif %}"
                required
            >
            {% if form.errors.title %}
                <span class="error-message">{{ form.errors.title }}</span>
            {% endif %}
        </div>

        <div class="form-group">
            <label for="description">Description</label>
            <textarea
                name="description"
                id="description"
                rows="5"
                class="{% if form.errors.description %}error{% endif %}"
            >{{ form.cleaned_data.description | default(value=task.description) }}</textarea>
            {% if form.errors.description %}
                <span class="error-message">{{ form.errors.description }}</span>
            {% endif %}
        </div>

        <div class="form-group">
            <label>
                <input
                    type="checkbox"
                    name="completed"
                    value="true"
                    {% if task.completed %}checked{% endif %}
                >
                Tâche terminée
            </label>
        </div>

        <div class="form-actions">
            <button type="submit" class="btn btn-primary">Modifier</button>
            <a href='{% link "index" %}' class="btn btn-secondary">Annuler</a>
        </div>
    </form>
</div>
{% endblock %}
```

## Step 11: CSS
### static/css/style.css
```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    line-height: 1.6;
    color: #333;
    background: #f5f5f5;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
}

/* Navbar */
.navbar {
    background: #667eea;
    color: white;
    padding: 1rem 0;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.navbar .container {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.logo {
    color: white;
    text-decoration: none;
    font-size: 1.5rem;
    font-weight: bold;
}

.nav-links {
    display: flex;
    gap: 1rem;
    align-items: center;
}

.nav-links a {
    color: white;
    text-decoration: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    transition: background 0.3s;
}

.nav-links a:hover {
    background: rgba(255,255,255,0.1);
}

/* Main */
main {
    padding: 2rem 0;
    min-height: calc(100vh - 200px);
}

/* Page header */
.page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
}

.page-header h1 {
    color: #333;
    font-size: 2rem;
}

/* Buttons */
.btn {
    display: inline-block;
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    text-decoration: none;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s;
}

.btn-primary {
    background: #667eea;
    color: white;
}

.btn-primary:hover {
    background: #5568d3;
}

.btn-secondary {
    background: #6c757d;
    color: white;
}

.btn-secondary:hover {
    background: #5a6268;
}

.btn-danger {
    background: #dc3545;
    color: white;
}

.btn-danger:hover {
    background: #c82333;
}

.btn-success {
    background: #28a745;
    color: white;
}

.btn-success:hover {
    background: #218838;
}

.btn-warning {
    background: #ffc107;
    color: #333;
}

.btn-warning:hover {
    background: #e0a800;
}

.btn-small {
    padding: 0.4rem 0.8rem;
    font-size: 0.875rem;
}

/* Tasks list */
.tasks-list {
    display: grid;
    gap: 1rem;
}

.task-card {
    background: white;
    padding: 1.5rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    transition: transform 0.2s;
}

.task-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0,0,0,0.15);
}

.task-card.completed {
    opacity: 0.7;
    background: #f8f9fa;
}

.task-card.completed h3 {
    text-decoration: line-through;
    color: #6c757d;
}

.task-header {
    display: flex;
    justify-content: space-between;
    align-items: start;
    margin-bottom: 0.5rem;
}

.task-header h3 {
    color: #333;
    font-size: 1.25rem;
    margin: 0;
}

.task-date {
    color: #6c757d;
    font-size: 0.875rem;
}

.task-description {
    color: #666;
    margin: 0.5rem 0 1rem 0;
}

.task-actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
}

/* Empty state */
.empty-state {
    text-align: center;
    padding: 4rem 2rem;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.empty-state p {
    color: #6c757d;
    font-size: 1.125rem;
    margin-bottom: 1.5rem;
}

/* Form */
.form-container {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    max-width: 600px;
}

.form-group {
    margin-bottom: 1.5rem;
}

.form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: #333;
}

.form-group input[type="text"],
.form-group textarea {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
    font-family: inherit;
}

.form-group input[type="text"]:focus,
.form-group textarea:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102,126,234,0.1);
}

.form-group input.error,
.form-group textarea.error {
    border-color: #dc3545;
}

.error-message {
    color: #dc3545;
    font-size: 0.875rem;
    margin-top: 0.25rem;
    display: block;
}

.form-group input[type="checkbox"] {
    margin-right: 0.5rem;
}

.form-actions {
    display: flex;
    gap: 1rem;
    margin-top: 2rem;
}

/* Flash messages */
.flash-messages {
    margin-bottom: 2rem;
}

.message {
    padding: 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    border-left: 4px solid;
}

.message-success {
    background: #d4edda;
    color: #155724;
    border-left-color: #28a745;
}

.message-error {
    background: #f8d7da;
    color: #ffa200ff;
    border-left-color: #dc3545;
}

.message-info {
    background: #d1ecf1;
    color: #0c5460;
    border-left-color: #17a2b8;
}
.message-warning {
    background: #eb9d00ff;
    color: #ffbb00ff;
    border-left-color: #fbbc00ff;
}
/* Footer */
.footer {
    background: #f8f9fa;
    padding: 2rem 0;
    text-align: center;
    color: #6c757d;
    margin-top: 4rem;
}

/* Responsive */
@media (max-width: 768px) {
    .page-header {
        flex-direction: column;
        align-items: start;
        gap: 1rem;
    }

    .navbar .container {
        flex-direction: column;
        gap: 1rem;
    }

    .task-header {
        flex-direction: column;
    }

    .form-actions {
        flex-direction: column;
    }

    .form-actions .btn {
        width: 100%;
    }
}
```

---
Understood.
Here is the **entire missing section rewritten fully in English**.
No French remains. No side-by-side translation. English only.

---

# Step 12: Run the Application

```bash
cargo run
```

**Expected output:**

```
🦀 Runique Todo App starting...
📊 Database connected
🌐 Server running on http://127.0.0.1:3000
```

**Open in browser:**
[http://127.0.0.1:3000](http://127.0.0.1:3000)

---

# Available Features

## 1. Create a Task

* Click “New Task”
* Enter a title (required)
* Optionally enter a description
* Click “Create”

## 2. List Tasks

* The home page shows all tasks
* Sorted by newest first
* Completed tasks appear faded/greyed out

## 3. Edit a Task

* Click “Edit”
* Update title, description, or status
* Click “Save”

## 4. Complete a Task

* Click “Complete”
* Task becomes marked as completed
* Click “Reopen” to activate it again

## 5. Delete a Task

* Click “Delete”
* Confirm deletion
* Task is permanently removed

---

# Application Architecture

## MVC Structure

```
┌─────────────────────────────────────┐
│               Browser               │
│        (Sends HTTP Request)         │
└────────────────┬────────────────────┘
                 │
                 v
┌─────────────────────────────────────┐
│          Router (urls.rs)           │
│     Dispatches to correct handler   │
└────────────────┬────────────────────┘
                 │
                 v
┌─────────────────────────────────────┐
│          Handler (views.rs)         │
│ - Reads request data                │
│ - Validates forms                   │
│ - Executes business logic           │
└────────────────┬────────────────────┘
                 │
                 v
┌─────────────────────────────────────┐
│          Model (models.rs)          │
│ Database access via SeaORM          │
└────────────────┬────────────────────┘
                 │
                 v
┌─────────────────────────────────────┐
│       Templates (templates/)        │
│  HTML rendering via Tera            │
└────────────────┬────────────────────┘
                 │
                 v
┌─────────────────────────────────────┐
│           HTTP Response             │
│   Returned to the browser           │
└─────────────────────────────────────┘
```

---

# Data Flow

## Creating a Task

1. User submits the form
2. Browser sends POST `/create`
3. Router dispatches to `create_submit`
4. Handler validates the form
5. If valid → insert into database
6. Redirect to `/` with flash message
7. Task list is refreshed

## Rendering the Task List

1. Browser requests GET `/`
2. Router calls `index`
3. Handler fetches tasks
4. Template renders HTML
5. Browser displays final page

---

# Key Implementation Concepts

## 1. Django-like ORM API

```rust
let tasks = Task::objects
    .order_by_desc(tasks::Column::CreatedAt)
    .all(&*db)
    .await?;
```

Instead of raw SQL:

```sql
SELECT * FROM tasks ORDER BY created_at DESC;
```

---

## 2. Form Validation

```rust
self.require("title", &CharField { allow_blank: false }, raw_data);
```

Custom validation:

```rust
if title.len() < 3 {
    self.errors.insert("title".to_string(), "Too short".to_string());
}
```

---

## 3. Flash Messages

```rust
let _ = message.success("Task created successfully!").await;
```

Automatically rendered in templates using
`{% messages %}`

---

## 4. CSRF Protection

```html
<form method="post">
    {% csrf %}
</form>
```

Token validation is automatic.

---

## 5. Template Inheritance

```html
{% extends "base.html" %}

{% block content %}
...
{% endblock %}
```

Keeps layout DRY and consistent.

---

# Manual Testing

## Create These 5 Tasks

1. “Learn Runique” — “Rust web framework”
2. “Create a project” — “Full CRUD app”
3. “Deploy to production” — no description
4. “Write documentation” — “Complete guide”
5. “Share on GitHub” — no description

Verify:

* Create
* List
* Edit
* Complete
* Reopen
* Delete
* Validation errors
* Flash messages
* Mobile layout

---

## Validation Testing

Test creating a task with:

| Input               | Expected Result  |
| ------------------- | ---------------- |
| Empty title         | Validation error |
| 2-character title   | Validation error |
| 300-character title | Validation error |
| “ABC”               | Success          |

---

# Possible Enhancements

## 1. Pagination

```rust
.limit(10)
.offset(page * 10)
```

## 2. Search

```rust
.filter(tasks::Column::Title.like("%query%"))
```

## 3. Categories

Add a column:

```sql
ALTER TABLE tasks ADD COLUMN category VARCHAR(50);
```

## 4. Priorities

```sql
ALTER TABLE tasks ADD COLUMN priority INTEGER DEFAULT 0;
```

## 5. Users

```sql
ALTER TABLE tasks ADD COLUMN user_id INTEGER REFERENCES users(id);
```

## 6. REST API

Add JSON endpoints.

---

# Deployment

## Build

```bash
cargo build --release
```

## Production Env

```env
IP_SERVER=0.0.0.0
PORT=8080
SECRET_KEY=your-secure-key
```

## Docker

Dockerfile already prepared.

---

# Comparison with Django

You get:

* MVC
* ORM
* Templates
* Validation
* Flash messages

But with:

* Compile-time safety
* High performance
* Memory safety

---

# Summary

You have learned how to:

✔ Configure PostgreSQL
✔ Define ORM models
✔ Build forms with validation
✔ Implement full CRUD
✔ Render templates with inheritance
✔ Use flash messages
✔ Apply CSRF protection
✔ Style the UI

All in **~500 lines of Rust**.

---

# Congratulations

You now understand how to build a complete web application using **Runique**.

Built with Rust.

*Documentation created with ❤️ by Claude for Itsuki*
