# Application CRUD complète avec Rusti

## Introduction

Ce guide vous montre comment créer une application CRUD (Create, Read, Update, Delete) complète avec Rusti, incluant :
- Base de données PostgreSQL
- Templates HTML
- Formulaires de validation
- Flash messages
- Protection CSRF

**Temps estimé :** 30-45 minutes

---

## Prérequis

- Rust 1.70 ou supérieur
- PostgreSQL installé et en cours d'exécution
- Connaissances de base en SQL

**Vérifier PostgreSQL :**
```bash
psql --version
```

---

## Ce que nous allons construire

Une application de gestion de tâches (Todo App) avec :
- Liste des tâches
- Création de tâche
- Modification de tâche
- Suppression de tâche
- Marquer comme complétée

**Fonctionnalités :**
- ORM Django-like pour les requêtes
- Templates avec héritage
- Validation de formulaires
- Messages flash
- Interface responsive

---

## Étape 1 : Créer le projet
```bash
cargo new todo-app
cd todo-app
```

---

## Étape 2 : Configuration

### Cargo.toml
```toml
[package]
name = "todo-app"
version = "1.0.0"
edition = "2021"

[dependencies]
rusti = { version = "1.0.0", features = ["postgres"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sea-orm = { version = "1", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Fichier .env

Créez `.env` à la racine :
```env
# Serveur
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=change-this-secret-key-in-production

# Base de données
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=postgres
DB_HOST=localhost
DB_PORT=5432
DB_NAME=todo_app
```

---

## Étape 3 : Créer la base de données
```bash
# Se connecter à PostgreSQL
psql -U postgres

# Créer la base
CREATE DATABASE todo_app;

# Se connecter à la base
\c todo_app

# Créer la table
CREATE TABLE tasks (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    completed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

# Quitter
\q
```

---

## Étape 4 : Structure du projet
```bash
mkdir -p src templates static/css
touch src/{main.rs,models.rs,urls.rs,views.rs,forms.rs}
touch templates/{base.html,index.html,create.html,edit.html}
touch static/css/style.css
```

**Structure finale :**
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

## Étape 5 : Modèle de données

### src/models.rs
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use rusti::impl_objects;

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
use rusti::prelude::*;
use rusti::formulaire::formsrusti::{Forms, FormulaireTrait};
use rusti::formulaire::field::{CharField, TextField, BooleanField};
use std::collections::HashMap;

#[rusti_form]
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

## Étape 7 : Handlers (Views)

### src/views.rs
```rust
use rusti::prelude::*;
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

---

## Étape 8 : Routes

### src/urls.rs
```rust
use rusti::{Router, urlpatterns};
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
use rusti::prelude::*;
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

    println!("🦀 Rusti Todo App starting...");
    println!("📊 Database connected");
    println!("🌐 Server running on http://{}:{}",
        settings.server.ip_server,
        settings.server.port
    );

    // Lancer l'application
    RustiApp::new(settings).await?
        .with_database(db)
        .routes(urls::routes())
        .with_static_files()?
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

---

## Étape 10 : Templates

### templates/base.html
```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Todo App{% endblock %} - Rusti</title>
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
            <p>Développé avec Rusti Framework</p>
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

---

## Étape 11 : CSS

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

## Étape 12 : Lancer l'application
```bash
cargo run
```

**Sortie attendue :**
```
🦀 Rusti Todo App starting...
📊 Database connected
🌐 Server running on http://127.0.0.1:3000
```

**Ouvrez :** http://127.0.0.1:3000

---

## Fonctionnalités disponibles

### 1. Créer une tâche
- Cliquez sur "Nouvelle tâche"
- Remplissez le titre (obligatoire)
- Ajoutez une description (optionnelle)
- Cliquez sur "Créer"

### 2. Lister les tâches
- Page d'accueil affiche toutes les tâches
- Triées par date de création (plus récentes en premier)
- Les tâches terminées sont grisées

### 3. Modifier une tâche
- Cliquez sur "Modifier"
- Changez le titre, description ou statut
- Cliquez sur "Modifier"

### 4. Terminer une tâche
- Cliquez sur "Terminer"
- La tâche est marquée comme complétée
- Cliquez sur "Réactiver" pour la rouvrir

### 5. Supprimer une tâche
- Cliquez sur "Supprimer"
- Confirmez la suppression
- La tâche est définitivement supprimée

---

## Architecture de l'application

### Structure MVC
```
┌─────────────────────────────────────┐
│          Navigateur                 │
│  (Envoie requête HTTP)              │
└────────────────┬────────────────────┘
                 │
                 v
┌─────────────────────────────────────┐
│          Router (urls.rs)           │
│  Route vers le bon handler          │
└────────────────┬────────────────────┘
                 │
                 v
┌─────────────────────────────────────┐
│        Handler (views.rs)           │
│  - Récupère les données             │
│  - Valide les formulaires           │
│  - Traite la logique métier         │
└────────────────┬────────────────────┘
                 │
                 v
┌─────────────────────────────────────┐
│         Modèle (models.rs)          │
│  Interaction avec PostgreSQL        │
│  via SeaORM                         │
└────────────────┬────────────────────┘
                 │
                 v
┌─────────────────────────────────────┐
│       Template (templates/)         │
│  Génère le HTML avec Tera           │
└────────────────┬────────────────────┘
                 │
                 v
┌─────────────────────────────────────┐
│          Response HTTP              │
│  Envoyée au navigateur              │
└─────────────────────────────────────┘
```

### Flux de données

**Création d'une tâche :**
1. Utilisateur remplit le formulaire
2. Navigateur envoie POST à `/create`
3. Router appelle `create_submit`
4. Handler valide le formulaire
5. Si valide : insertion en base de données
6. Redirection vers `/` avec flash message
7. Liste des tâches rafraîchie

**Affichage de la liste :**
1. Navigateur demande GET `/`
2. Router appelle `index`
3. Handler récupère toutes les tâches
4. Template génère le HTML
5. HTML envoyé au navigateur

---

## Points clés de l'implémentation

### 1. ORM Django-like
```rust
// Simple et expressif
let tasks = Task::objects
    .order_by_desc(tasks::Column::CreatedAt)
    .all(&*db)
    .await?;
```

Au lieu de SQL brut :
```sql
SELECT * FROM tasks ORDER BY created_at DESC;
```

### 2. Validation de formulaires
```rust
// Validation automatique
self.require("title", &CharField { allow_blank: false }, raw_data);

// Validation personnalisée
if title.len() < 3 {
    self.errors.insert("title".to_string(), "Trop court".to_string());
}
```

### 3. Flash messages
```rust
let _ = message.success("Tâche créée avec succès !").await;
```

Affichés automatiquement dans le template avec `{% messages %}`.

### 4. Protection CSRF
```html
<form method="post">
    {% csrf %}
    <!-- ... -->
</form>
```

Token validé automatiquement par le middleware.

### 5. Templates avec héritage
```html
{% extends "base.html" %}

{% block content %}
    <!-- Contenu spécifique -->
{% endblock %}
```

---

## Tests de l'application

### Test manuel

**Créer 5 tâches :**
1. "Apprendre Rusti" (description: "Framework web Rust")
2. "Créer un projet" (description: "Application CRUD complète")
3. "Déployer en production" (pas de description)
4. "Écrire la documentation" (description: "Guide complet")
5. "Partager sur GitHub" (pas de description)

**Tester les fonctionnalités :**
- ✅ Créer une tâche
- ✅ Lister les tâches
- ✅ Modifier une tâche
- ✅ Terminer une tâche
- ✅ Réactiver une tâche
- ✅ Supprimer une tâche
- ✅ Validation des formulaires (titre vide, titre < 3 caractères)
- ✅ Flash messages
- ✅ Responsive design

### Test de validation

**Essayez de créer une tâche avec :**
- Titre vide → Erreur "Requis"
- Titre "AB" → Erreur "au moins 3 caractères"
- Titre de 300 caractères → Erreur "ne peut pas dépasser 255 caractères"
- Titre "ABC" → Succès

---

## Évolutions possibles

### 1. Pagination
```rust
let tasks = Task::objects
    .order_by_desc(tasks::Column::CreatedAt)
    .limit(10)
    .offset(page * 10)
    .all(&*db)
    .await?;
```

### 2. Recherche
```rust
let tasks = Task::objects
    .filter(tasks::Column::Title.like(&format!("%{}%", query)))
    .all(&*db)
    .await?;
```

### 3. Catégories

Ajouter une colonne `category` :
```sql
ALTER TABLE tasks ADD COLUMN category VARCHAR(50);
```

### 4. Priorités

Ajouter une colonne `priority` :
```sql
ALTER TABLE tasks ADD COLUMN priority INTEGER DEFAULT 0;
```

### 5. Utilisateurs

Associer les tâches à des utilisateurs :
```sql
ALTER TABLE tasks ADD COLUMN user_id INTEGER REFERENCES users(id);
```

### 6. API REST

Ajouter des routes JSON :
```rust
"/api/tasks" => get(api_list_tasks),
"/api/tasks/{id}" => get(api_get_task),
"/api/tasks" => post(api_create_task),
"/api/tasks/{id}" => put(api_update_task),
"/api/tasks/{id}" => delete(api_delete_task),
```

---

## Déploiement

### 1. Build de production
```bash
cargo build --release
```

### 2. Configuration production

Modifier `.env` :
```env
IP_SERVER=0.0.0.0
PORT=8080
SECRET_KEY=production-secret-key-very-long-and-secure
DB_HOST=production-host
DB_PASSWORD=production-password
```

### 3. Docker

Créer `Dockerfile` :
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/todo-app .
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/static ./static
CMD ["./todo-app"]
```
```bash
docker build -t todo-app .
docker run -p 8080:8080 --env-file .env todo-app
```

---

## Comparaison avec Django

### Django
```python
# models.py
class Task(models.Model):
    title = models.CharField(max_length=255)
    description = models.TextField(blank=True)
    completed = models.BooleanField(default=False)
    created_at = models.DateTimeField(auto_now_add=True)

# forms.py
class TaskForm(forms.ModelForm):
    class Meta:
        model = Task
        fields = ['title', 'description', 'completed']

# views.py
def index(request):
    tasks = Task.objects.all().order_by('-created_at')
    return render(request, 'index.html', {'tasks': tasks})

def create(request):
    if request.method == 'POST':
        form = TaskForm(request.POST)
        if form.is_valid():
            form.save()
            messages.success(request, 'Tâche créée !')
            return redirect('index')
    else:
        form = TaskForm()
    return render(request, 'create.html', {'form': form})
```

### Rusti
```rust
// models.rs
#[derive(DeriveEntityModel)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: DateTime,
}

// forms.rs
#[rusti_form]
pub struct TaskForm {
    pub form: Forms,
}

impl FormulaireTrait for TaskForm {
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.require("title", &CharField { allow_blank: false }, raw_data);
        self.optional("description", &TextField, raw_data);
        self.is_valid()
    }
}

// views.rs
pub async fn index(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    let tasks = Task::objects
        .order_by_desc(tasks::Column::CreatedAt)
        .all(&*db)
        .await
        .unwrap_or_default();

    let ctx = context! {
        "tasks", tasks
    };

    template.render("index.html", &ctx)
}

pub async fn create_submit(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    ExtractForm(form): ExtractForm<TaskForm>,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        let ctx = context! { "form", &form };
        return template.render("create.html", &ctx);
    }

    let task = tasks::ActiveModel {
        title: Set(form.get_value("title").unwrap()),
        description: Set(form.get_value("description")),
        completed: Set(false),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    match task.insert(&*db).await {
        Ok(_) => {
            success!(message, "Tâche créée avec succès !");
            redirect("/")
        }
        Err(e) => {
            let _ = message.error(&format!("Erreur : {}", e)).await;
            // ou 
            // let error = format!("Erreur : {}", e)
            // error!(message, error)
            let ctx = context! { "form", &form };
            template.render("create.html", &ctx)
        }
    }
}
```

**Similitudes :**
- Structure MVC identique
- ORM similaire (filter, all, order_by)
- Validation de formulaires automatique
- Flash messages
- Templates avec héritage

**Avantages Rusti :**
- Type-safety (erreurs à la compilation)
- Performances (async natif)
- Sécurité mémoire garantie
- Pas de runtime overhead

---

## Ressources

### Documentation
- [Guide de démarrage](../documentation%20french/GETTING_STARTED.md)
- [Guide de la base de données](../documentation%20french/DATABASE.md)
- [Guide des formulaires](../documentation%20french/FORMULAIRE.md)
- [Guide des templates](../documentation%20french/TEMPLATES.md)

### Code source
- [Tests d'intégration](../tests/)
- [Documentation complète](../documentation%20french/)

---

## Récapitulatif

**Vous avez appris à :**
- Configurer PostgreSQL avec Rusti
- Créer un modèle avec SeaORM
- Utiliser l'ORM Django-like
- Créer des formulaires avec validation
- Gérer le CRUD complet
- Utiliser les templates avec héritage
- Afficher des flash messages
- Protéger les formulaires avec CSRF
- Styliser l'application avec CSS

**Une application complète en environ 500 lignes de code !**

---

**Félicitations ! Vous maîtrisez maintenant Rusti.**

**Développé avec passion en Rust**