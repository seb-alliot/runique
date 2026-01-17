# Guide des Formulaires Runique

Documentation complète du système de formulaires inspiré de Django pour Runique.

## Table des matières

1. [Introduction](#introduction)
2. [Création manuelle de formulaires](#création-manuelle-de-formulaires)
3. [Génération automatique avec macros](#génération-automatique-avec-macros)
4. [Types de champs disponibles](#types-de-champs-disponibles)
5. [Validation](#validation)
6. [Affichage des erreurs](#affichage-des-erreurs)
7. [Sauvegarde en base de données](#sauvegarde-en-base-de-données)
8. [Protection CSRF](#protection-csrf)
9. [Exemples complets](#exemples-complets)
10. [Bonnes pratiques](#bonnes-pratiques)

---

## Introduction

Runique propose un système de formulaires inspiré de Django qui permet de :
- ✅ Créer des formulaires typés et validés
- ✅ Gérer automatiquement les erreurs
- ✅ Intégrer facilement avec SeaORM
- ✅ Générer automatiquement depuis les models via macros
- ✅ Générer du HTML via templates Tera
- ✅ Protection CSRF intégrée

---

## Création manuelle de formulaires

### Structure de base

Chaque formulaire Runique implémente le trait `RuniqueForm` :

```rust
use runique::prelude::*;
use runique::serde::{Serialize, Serializer};

#[derive(Serialize)]
#[serde(transparent)]
pub struct ContactForm {
    pub form: Forms,
}

impl RuniqueForm for ContactForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &GenericField::text("name")
                .placeholder("Votre nom")
                .required("Le nom est requis"),
        );
        
        form.field(
            &GenericField::email("email")
                .placeholder("votre@email.com")
                .required("L'email est requis"),
        );
        
        form.field(
            &GenericField::textarea("message")
                .placeholder("Votre message...")
                .required("Le message est requis"),
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

### Utilisation dans un handler

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
    ExtractForm(mut contact_form): ExtractForm<ContactForm>,
) -> Result<Redirect, Html<String>> {
    if !contact_form.is_valid().await {
        let html = state.tera
            .render("contact.html", &tera::context! { form => contact_form })
            .unwrap();
        return Err(Html(html));
    }
    
    // Traiter le message
    // ...
    
    Ok(Redirect::to("/success"))
}
```

**Note importante** : Utilisez `ExtractForm<T>` au lieu de `Form<HashMap<String, String>>` pour extraire directement le formulaire validé. L'extracteur `ExtractForm` :
- Récupère automatiquement les données du formulaire
- Reconstruit le formulaire avec les données
- Valide le token CSRF automatiquement
- Vous évite de manipuler `HashMap` manuellement

---

## Génération automatique avec macros

Runique propose **deux macros** pour générer automatiquement des formulaires :

### 1. Macro `#[runique_form]`

Pour créer des formulaires personnalisés avec gestion automatique de `Deref/DerefMut` :

```rust
use runique::prelude::*;

#[runique_form]
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginForm {
    pub form: Forms,
}
```

**Ce qui est généré automatiquement :**

```rust
// ✅ Implémentation de Deref
impl std::ops::Deref for LoginForm {
    type Target = Forms;
    fn deref(&self) -> &Self::Target { &self.form }
}

// ✅ Implémentation de DerefMut
impl std::ops::DerefMut for LoginForm {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.form }
}
```

### 2. Macro `#[derive(DeriveModelForm)]`

Pour générer automatiquement un formulaire complet depuis un model SeaORM :

```rust
use sea_orm::entity::prelude::*;

// Votre model SeaORM
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

// Génération automatique du formulaire
#[derive(DeriveModelForm)]
pub struct User;
```

**Ce qui est généré automatiquement :**

#### a) Structure du formulaire

```rust
#[derive(Serialize, Debug, Clone)]
pub struct UserForm {
    #[serde(flatten, skip_deserializing)]
    pub form: Forms,
}
```

#### b) Implémentation de `RuniqueForm`

```rust
impl RuniqueForm for UserForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &GenericField::text("username")
                .label("Username")
                .required("Ce champ est obligatoire")
        );
        
        form.field(
            &GenericField::email("email")
                .label("Email")
                .required("Ce champ est obligatoire")
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

**Logique de détection automatique des champs** :

La macro `DeriveModelForm` analyse les champs de votre model et détermine automatiquement le type de champ approprié :

1. **Par nom du champ** (priorité haute) :
   - Contient `email` → `GenericField::email()`
   - Contient `password` ou `pwd` → `GenericField::password()`
   - Contient `url`, `link` ou `website` → `GenericField::url()`

2. **Par type Rust + nom** (pour String) :
   - `String` avec nom contenant `description`, `bio`, `content` ou `message` → `GenericField::textarea()`
   - `String` par défaut → `GenericField::text()`

3. **Par type Rust** :
   - `i32`, `i64`, `u32` → `GenericField::int()`

4. **Champs exclus automatiquement** :
   - `id` (ou tout champ avec `#[sea_orm(primary_key)]`)
   - `csrf_token`, `_csrf_token`, `form`
   - `created_at`, `updated_at` (gérés automatiquement)
   - `is_active`, `deleted_at`

#### c) Méthode `to_active_model()`

Conversion automatique vers SeaORM `ActiveModel` :

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

**Conversions automatiques par type** :

La macro génère le code de conversion approprié selon le type du champ :

- **String** : `Set(self.form.get_value("field").unwrap_or_default())`
- **Option<String>** : `Set(self.form.get_value("field"))`
- **i32, i64, u32** : Parse automatique avec fallback à 0
- **f32, f64** : Parse automatique avec fallback à 0.0
- **bool** : Accepte `"true"`, `"1"`, `"on"` comme valeurs vraies
- **created_at, updated_at** : Automatiquement définis avec `chrono::Utc::now().naive_utc()`

#### d) Méthode `save()`

Sauvegarde directe en base de données :

```rust
impl UserForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<Model, DbErr> {
        use sea_orm::EntityTrait;
        self.to_active_model().insert(db).await
    }
}
```

### Comparaison des macros

| Élément | `#[runique_form]` | `#[derive(DeriveModelForm)]` |
|---------|-------------------|------------------------------|
| Struct formulaire | ❌ Manuel | ✅ Généré |
| `impl RuniqueForm` | ❌ Manuel | ✅ Généré |
| `impl Deref/DerefMut` | ✅ Généré | ✅ Généré |
| `to_active_model()` | ❌ Manuel | ✅ Généré |
| `save()` | ❌ Manuel | ✅ Généré |
| **Usage** | Formulaires personnalisés | Formulaires liés aux models |

---

## Types de champs disponibles

### Champs textuels

```rust
// Champ texte simple
GenericField::text("username")
    .placeholder("Nom d'utilisateur...")
    .required("Le nom est requis")
    .min_length(3, "Au moins 3 caractères")
    .max_length(20, "Maximum 20 caractères")

// Zone de texte
GenericField::textarea("description")
    .placeholder("Description longue...")
    .required("La description est requise")
    .max_length(500, "Maximum 500 caractères")

// Texte enrichi (WYSIWYG)
GenericField::richtext("content")
    .required("Le contenu est requis")
```

### Champs avec validation spéciale

```rust
// Email (validation automatique du format)
GenericField::email("email")
    .placeholder("exemple@domaine.com")
    .required("Email requis")

// URL (validation automatique du format)
GenericField::url("website")
    .placeholder("https://exemple.com")

// Mot de passe (masqué dans le HTML)
GenericField::password("password")
    .required("Mot de passe requis")
    .min_length(8, "Minimum 8 caractères")
```

### Champs numériques

```rust
// Nombre entier
GenericField::int("age")
    .required("L'âge est requis")

// Nombre décimal (float)
GenericField::float("price")
    .required("Le prix est requis")

// Booléen
GenericField::boolean("accept_terms")
    .required("Vous devez accepter les conditions")
```

### Méthodes de configuration (Builder Pattern)

Toutes les méthodes peuvent être chaînées :

| Méthode | Description | Exemple |
|---------|-------------|---------|
| `.placeholder("texte")` | Texte d'aide | `.placeholder("Entrez votre nom")` |
| `.label("Label")` | Label du champ | `.label("Nom complet")` |
| `.required("message")` | Champ obligatoire | `.required("Ce champ est requis")` |
| `.min_length(n, "msg")` | Longueur minimale | `.min_length(3, "Min 3 caractères")` |
| `.max_length(n, "msg")` | Longueur maximale | `.max_length(50, "Max 50 caractères")` |
| `.readonly(bool, "msg")` | Lecture seule | `.readonly(true, Some("Non modifiable"))` |
| `.disabled(bool, "msg")` | Désactivé | `.disabled(true, Some("Champ désactivé"))` |
| `.html_attribute("key", "value")` | Attribut HTML personnalisé | `.html_attribute("data-custom", "value")` |

### Utilitaires pour les mots de passe

Le champ `password` propose des méthodes de hachage intégrées :

```rust
// Créer un champ password
let password_field = GenericField::password("password")
    .required("Mot de passe requis")
    .min_length(8, "Minimum 8 caractères");

// Hacher un mot de passe (utilise bcrypt)
let hashed = password_field.hash_password()?;

// Vérifier un mot de passe
let is_valid = GenericField::verify_password("plain_password", &hashed);
```

---

## Validation

### Validation automatique

Appelez `is_valid()` pour valider tous les champs :

```rust
pub async fn create_post(
    State(state): State<AppState>,
    ExtractForm(mut post_form): ExtractForm<PostForm>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Validation automatique de tous les champs
    if !post_form.is_valid().await {
        // Le formulaire contient des erreurs
        return Err((
            StatusCode::BAD_REQUEST,
            Html(state.tera.render("post_form.html", &context! { form => post_form }).unwrap())
        ));
    }
    
    // Le formulaire est valide
    Ok(Redirect::to("/success"))
}
```

### Validations effectuées automatiquement

1. **Champs requis** : Vérifie que les champs obligatoires ne sont pas vides
2. **Longueur min/max** : Respecte les contraintes de longueur
3. **Format email** : Valide le format email avec une regex robuste
4. **Format URL** : Valide le format URL (http/https)
5. **Types numériques** : Vérifie la conversion en nombre

### Validation métier personnalisée

Implémentez la méthode `clean()` pour des validations métier complexes :

```rust
impl RuniqueForm for RegisterForm {
    // ... autres méthodes ...
    
    fn clean(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), HashMap<String, String>>> + Send + '_>> {
        Box::pin(async move {
            let mut errors = HashMap::new();
            
            // Vérifier que les mots de passe correspondent
            let password = self.form.get_value("password").unwrap_or_default();
            let password_confirm = self.form.get_value("password_confirm").unwrap_or_default();
            
            if password != password_confirm {
                errors.insert(
                    "password_confirm".to_string(),
                    "Les mots de passe ne correspondent pas".to_string()
                );
            }
            
            // Vérifier la force du mot de passe
            if password.len() >= 8 && !password.chars().any(|c| c.is_numeric()) {
                errors.insert(
                    "password".to_string(),
                    "Le mot de passe doit contenir au moins un chiffre".to_string()
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

### Validation asynchrone (avec base de données)

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

// Dans le handler
pub async fn register(
    Form(mut form): Form<RegisterForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    if !form.is_valid().await {
        return template.render("register.html", context! { form });
    }
    
    // Validation asynchrone personnalisée
    if !form.validate_unique_email(&*db).await {
        form.get_form_mut()
            .fields
            .get_mut("email")
            .unwrap()
            .set_error("Cet email est déjà utilisé".to_string());
        
        return template.render("register.html", context! { form });
    }
    
    // Sauvegarder...
}
```

---

## Affichage des erreurs

### Syntaxe Django-like dans les templates

Runique supporte une syntaxe similaire à Django pour afficher les formulaires dans Tera. Le framework transforme automatiquement ces balises via des filtres personnalisés.

#### Rendu automatique complet

```html
<form method="post">
    {% csrf %}
    {% form.register_form %}  <!-- Génère tous les champs automatiquement -->
    <button type="submit">Valider</button>
</form>
```

**Ce qui est généré** : Le formulaire complet avec tous les champs, leurs labels, erreurs et attributs HTML.

#### Rendu d'un champ spécifique

```html
<form method="post">
    {% csrf %}
    
    <div class="form-group">
        <label>Nom d'utilisateur</label>
        {% form.register_form.username %}
        <!-- Génère l'input avec tous ses attributs et erreurs -->
    </div>
    
    <div class="form-group">
        <label>Email</label>
        {% form.register_form.email %}
    </div>
    
    <button type="submit">S'inscrire</button>
</form>
```

#### Syntaxe alternative (filtre explicite)

Si vous préférez utiliser la syntaxe Tera standard :

```html
<!-- Formulaire complet -->
{{ register_form | form | safe }}

<!-- Champ spécifique -->
{% set field = register_form | form(field="username") %}
{% set input_type = field.field_type %}
{% include "base_string" %}
```

### Rendu manuel avec contrôle total

Pour un contrôle total sur le HTML :

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
    
    <button type="submit" class="btn btn-primary">Valider</button>
</form>
```

### Comment ça fonctionne ?

Runique transforme automatiquement la syntaxe Django-like en appels de filtres Tera :

1. **`{% form.register_form %}`** → `{{ register_form | form | safe }}`
2. **`{% form.register_form.username %}`** → `{{ register_form | form(field="username") }}`

Le filtre `form` :
- **Sans argument** : retourne le HTML complet du formulaire
- **Avec `field="nom"`** : retourne le HTML du champ spécifique

Cette transformation se fait au moment du chargement des templates (dans `RuniqueApp::new()`)

### Autres balises Django-like

Runique supporte plusieurs balises Django-like qui sont automatiquement transformées :

```html
{# Token CSRF #}
{% csrf %}  → {% include "csrf" %}

{# Messages flash #}
{% messages %}  → {% include "message" %}

{# En-têtes CSP #}
{{ csp }}  → {% include "csp" %}

{# Fichiers statiques #}
{% static "css/main.css" %}  → {{ "css/main.css" | static }}
{% static "js/app.js" %}     → {{ "js/app.js" | static }}

{# Fichiers media #}
{% media "images/logo.png" %}  → {{ "images/logo.png" | media }}

{# URLs nommées (reverse) #}
{% link "home" %}                      → {{ link(link='home') }}
{% link "user-detail", id=user.id %}   → {{ link(link='user-detail', id=user.id) }}
```

**Ces transformations sont appliquées automatiquement** par Runique lors du chargement des templates via des regex dans `app.rs`.

### Récupérer les erreurs en Rust

```rust
// Vérifier si le formulaire a des erreurs
if post_form.form.has_errors() {
    // Récupérer toutes les erreurs
    let all_errors = post_form.form.errors();
    
    for (field_name, error_msg) in all_errors {
        println!("Erreur sur {}: {}", field_name, error_msg);
    }
}

// Récupérer l'erreur d'un champ spécifique
if let Some(field) = post_form.form.fields.get("email") {
    if let Some(error) = field.error() {
        println!("Erreur email: {}", error);
    }
}

// Ajouter une erreur globale manuellement
post_form.form.global_errors.push("Erreur serveur temporaire".to_string());
```

### Types d'erreurs

1. **Erreurs de champ** : Associées à un champ spécifique (affichées sous le champ)
2. **Erreurs globales** : Erreurs non liées à un champ particulier (affichées en haut du formulaire)

---

## Sauvegarde en base de données

### Avec `DeriveModelForm` (méthode recommandée)

La macro génère automatiquement la méthode `save()` :

```rust
pub async fn create_article(
    Form(form): Form<ArticleForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    if !form.is_valid().await {
        return template.render("article_form.html", context! { form });
    }
    
    // ✅ Sauvegarde en une ligne
    match form.save(&*db).await {
        Ok(article) => redirect(&format!("/article/{}", article.id)),
        Err(e) => {
            error!(message, "Erreur lors de la création");
            template.render("article_form.html", context! { form })
        }
    }
}
```

### Avec formulaire manuel

Ajoutez une méthode `save()` à votre formulaire :

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

### Gérer les erreurs de base de données

Utilisez `database_error()` pour parser automatiquement les erreurs DB :

```rust
match register_form.save(&state.db).await {
    Ok(user) => {
        success!(message, "Utilisateur créé avec succès !");
        Ok(Redirect::to(&format!("/user/{}", user.id)))
    }
    Err(db_err) => {
        // Parser l'erreur et l'assigner au bon champ automatiquement
        register_form.database_error(&db_err);
        
        Err((
            StatusCode::BAD_REQUEST,
            Html(state.tera.render("register.html", &context! { form => register_form }).unwrap())
        ))
    }
}
```

La méthode `database_error()` détecte automatiquement :
- ✅ Les violations de contraintes UNIQUE
- ✅ Les erreurs de clés étrangères
- ✅ Et les assigne au champ concerné avec un message localisé

---

## Protection CSRF

### Activation automatique

La protection CSRF est **automatiquement activée** quand vous ajoutez le middleware :

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    RuniqueApp::new(settings).await?
        .routes(routes())
        .with_default_middleware()  // ✅ CSRF activé
        .run()
        .await?;

    Ok(())
}
```

### Utilisation dans les templates

```html
<form method="post">
    <!-- Token CSRF automatique -->
    {% csrf %}

    {% form.nom_form %}

    <button type="submit">Envoyer</button>
</form>
```

### Validation automatique

La validation CSRF est **automatique** via l'extracteur `Form<T>` :

```rust
pub async fn submit_form(
    Form(form): Form<ContactForm>,  // ✅ CSRF validé automatiquement
    template: Template,
) -> Response {
    // Si le token CSRF est invalide, une erreur 403 est retournée
    // AVANT que ce code soit exécuté
    
    if !form.is_valid() {
        return template.render("contact.html", context! { form });
    }

    // Form est valide ET le token CSRF a été vérifié
}
```

**Note :** Si le token CSRF est invalide, une erreur `403 Forbidden` est retournée automatiquement **avant** l'appel du handler.

---

## Exemples complets

### Exemple 1 : Formulaire de contact simple

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
                .placeholder("Votre nom")
                .required("Le nom est requis")
                .min_length(2, "Au moins 2 caractères"),
        );
        
        form.field(
            &GenericField::email("email")
                .placeholder("votre@email.com")
                .required("L'email est requis"),
        );
        
        form.field(
            &GenericField::text("subject")
                .placeholder("Sujet du message")
                .required("Le sujet est requis"),
        );
        
        form.field(
            &GenericField::textarea("message")
                .placeholder("Votre message...")
                .required("Le message est requis")
                .max_length(1000, "Maximum 1000 caractères"),
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
    ExtractForm(mut form): ExtractForm<ContactForm>,
    mut message: Message,
) -> Response {
    if !form.is_valid().await {
        return Html(state.tera.render("contact.html", &tera::context! { form }).unwrap())
            .into_response();
    }
    
    // Traiter le message (envoyer email, sauvegarder, etc.)
    // ...
    
    success!(message, "Message envoyé avec succès !");
    Redirect::to("/").into_response()
}
```

### Exemple 2 : Formulaire lié à un model avec `DeriveModelForm`

```rust
use runique::prelude::*;
use sea_orm::entity::prelude::*;

// Model SeaORM
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

// Génération automatique du formulaire
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
    ExtractForm(mut form): ExtractForm<ArticleForm>,
    State(state): State<AppState>,
    mut message: Message,
) -> Response {
    if !form.is_valid().await {
        error!(message, "Le formulaire contient des erreurs");
        return Html(state.tera.render("article_form.html", &tera::context! { form }).unwrap())
            .into_response();
    }
    
    // ✅ Sauvegarde automatique avec .save()
    match form.save(&*state.db).await {
        Ok(article) => {
            success!(message, "Article créé avec succès !");
            Redirect::to(&format!("/article/{}", article.id)).into_response()
        }
        Err(e) => {
            form.database_error(&e);
            error!(message, "Erreur lors de la création");
            Html(state.tera.render("article_form.html", &tera::context! { form }).unwrap())
                .into_response()
        }
    }
}
```

### Exemple 3 : Formulaire d'inscription avec validation personnalisée

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
                .placeholder("Nom d'utilisateur")
                .required("Le nom d'utilisateur est requis")
                .min_length(3, "Au moins 3 caractères")
                .max_length(20, "Maximum 20 caractères"),
        );
        
        form.field(
            &GenericField::email("email")
                .placeholder("votre@email.com")
                .required("L'email est requis"),
        );
        
        form.field(
            &GenericField::password("password")
                .required("Le mot de passe est requis")
                .min_length(8, "Minimum 8 caractères"),
        );
        
        form.field(
            &GenericField::password("password_confirm")
                .required("Confirmez votre mot de passe"),
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
    
    // Validation personnalisée
    fn clean(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), HashMap<String, String>>> + Send + '_>> {
        Box::pin(async move {
            let mut errors = HashMap::new();
            
            let password = self.form.get_value("password").unwrap_or_default();
            let password_confirm = self.form.get_value("password_confirm").unwrap_or_default();
            
            // Vérifier que les mots de passe correspondent
            if password != password_confirm {
                errors.insert(
                    "password_confirm".to_string(),
                    "Les mots de passe ne correspondent pas".to_string()
                );
            }
            
            // Vérifier la complexité du mot de passe
            if !password.chars().any(|c| c.is_numeric()) {
                errors.insert(
                    "password".to_string(),
                    "Le mot de passe doit contenir au moins un chiffre".to_string()
                );
            }
            
            if !password.chars().any(|c| c.is_uppercase()) {
                errors.insert(
                    "password".to_string(),
                    "Le mot de passe doit contenir au moins une majuscule".to_string()
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

### Exemple 4 : Édition d'un article existant

```rust
pub async fn edit_article_view(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Response {
    // Récupérer l'article existant
    let article = match Article::find_by_id(id).one(&*state.db).await {
        Ok(Some(a)) => a,
        _ => return (StatusCode::NOT_FOUND, "Article non trouvé").into_response(),
    };
    
    // Créer le formulaire et le pré-remplir
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
    ExtractForm(mut form): ExtractForm<ArticleForm>,
    State(state): State<AppState>,
    mut message: Message,
) -> Response {
    if !form.is_valid().await {
        error!(message, "Le formulaire contient des erreurs");
        return Html(state.tera.render("article_form.html", &tera::context! {
            form,
            article_id: id,
            is_edit: true,
        }).unwrap()).into_response();
    }
    
    // Récupérer l'article existant
    let existing = match Article::find_by_id(id).one(&*state.db).await {
        Ok(Some(a)) => a,
        _ => return (StatusCode::NOT_FOUND, "Article non trouvé").into_response(),
    };
    
    // Créer ActiveModel pour la mise à jour
    let mut active_model: ActiveModel = existing.into();
    active_model.title = Set(form.form.get_value("title").unwrap_or_default());
    active_model.slug = Set(form.form.get_value("slug").unwrap_or_default());
    active_model.content = Set(form.form.get_value("content").unwrap_or_default());
    
    match active_model.update(&*state.db).await {
        Ok(updated) => {
            success!(message, "Article mis à jour avec succès !");
            Redirect::to(&format!("/article/{}", updated.id)).into_response()
        }
        Err(e) => {
            form.database_error(&e);
            error!(message, "Erreur lors de la mise à jour");
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

## Bonnes pratiques

### 1. Toujours valider les formulaires

```rust
// ✅ CORRECT
pub async fn submit(
    ExtractForm(mut form): ExtractForm<MyForm>,
    State(state): State<AppState>,
) -> Response {
    if !form.is_valid().await {
        return template.render("form.html", context! { form });
    }
    
    // Traiter les données validées
}

// ❌ INCORRECT
pub async fn submit(
    ExtractForm(form): ExtractForm<MyForm>,
    State(state): State<AppState>,
) -> Response {
    // Pas de validation !
    form.save(&db).await?; // Risque de données invalides
}
```

### 2. Utiliser les méthodes auto-générées

```rust
// ✅ OPTIMAL (avec DeriveModelForm)
match form.save(&*db).await {
    Ok(article) => redirect("/success"),
    Err(e) => handle_error(e),
}

// ⚠️ ACCEPTABLE mais plus verbeux
let active_model = form.to_active_model();
match active_model.insert(&*db).await {
    Ok(article) => redirect("/success"),
    Err(e) => handle_error(e),
}
```

### 3. Gérer les erreurs proprement

```rust
pub async fn create_user(
    ExtractForm(mut form): ExtractForm<UserForm>,
    State(state): State<AppState>,
    mut message: Message,
    template: Template,
) -> Response {
    if !form.is_valid().await {
        error!(message, "Le formulaire contient des erreurs");
        return template.render("form.html", context! { form });
    }
    
    match form.save(&*state.db).await {
        Ok(user) => {
            success!(message, "Utilisateur créé !");
            Redirect::to(&format!("/user/{}", user.id)).into_response()
        }
        Err(DbErr::RecordNotFound(_)) => {
            error!(message, "Enregistrement non trouvé");
            template.render("form.html", context! { form })
        }
        Err(DbErr::Exec(_)) => {
            // Contrainte de base de données violée
            form.database_error(&DbErr::Exec("".into()));
            error!(message, "Contrainte de base de données (doublon ?)");
            template.render("form.html", context! { form })
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            error!(message, "Erreur interne");
            template.render("form.html", context! { form })
        }
    }
}
```

### 4. Utiliser les transactions pour opérations complexes

```rust
pub async fn create_user_with_profile(
    ExtractForm(user_form): ExtractForm<UserForm>,
    ExtractForm(profile_form): ExtractForm<ProfileForm>,
    State(state): State<AppState>,
    mut message: Message,
    template: Template,
) -> Response {
    if !user_form.is_valid().await || !profile_form.is_valid().await {
        error!(message, "Le formulaire contient des erreurs");
        return template.render("form.html", context! {
            user_form,
            profile_form,
        });
    }
    
    // Transaction pour garantir la cohérence
    let result = state.db.transaction::<_, (), DbErr>(|txn| {
        Box::pin(async move {
            // Créer l'utilisateur
            let user = user_form.to_active_model().insert(txn).await?;
            
            // Créer le profil lié
            let mut profile = profile_form.to_active_model();
            profile.user_id = Set(user.id);
            profile.insert(txn).await?;
            
            Ok(())
        })
    }).await;
    
    match result {
        Ok(_) => {
            success!(message, "Compte créé avec succès !");
            Redirect::to("/success").into_response()
        }
        Err(e) => {
            error!(message, "Erreur lors de la création du compte");
            template.render("form.html", context! {
                user_form,
                profile_form,
            })
        }
    }
}
```

### 5. Utiliser les macros de messages

```rust
use runique::prelude::*;

pub async fn submit(
    ExtractForm(mut form): ExtractForm<ArticleForm>,
    State(state): State<AppState>,
    mut message: Message,
    template: Template,
) -> Response {
    if !form.is_valid().await {
        error!(message, "Le formulaire contient des erreurs");
        return template.render("form.html", context! { form });
    }
    
    match form.save(&*state.db).await {
        Ok(article) => {
            success!(message, "Article créé avec succès !");
            
            if article.published {
                info!(message, "Votre article est maintenant visible publiquement");
            } else {
                warning!(message, "Votre article est en brouillon");
            }
            
            Redirect::to(&format!("/article/{}", article.id)).into_response()
        }
        Err(e) => {
            error!(message, "Erreur lors de la création");
            template.render("form.html", context! { form })
        }
    }
}
```

---

## Récapitulatif de l'API

### Cycle de vie d'un formulaire

1. **Création** : `MonFormulaire::build(tera)` ou `build_with_data(&data, tera)`
2. **Validation** : `form.is_valid().await`
3. **Récupération des données** : `form.data()` ou `form.get_value("champ")`
4. **Sauvegarde** : `form.save(&db).await`
5. **Gestion d'erreurs** : `form.database_error(&err)`

### API principale

```rust
// Création de formulaires
MonFormulaire::build(tera)                        // Formulaire vide
MonFormulaire::build_with_data(&data, tera)       // Formulaire pré-rempli

// Validation
form.is_valid().await                             // Valide tous les champs
form.has_errors()                                 // Vérifie la présence d'erreurs

// Récupération des données
form.data()                                       // HashMap<String, Value>
form.get_value("nom")                             // Option<String>
form.get_value_or_default("nom")                  // String

// Gestion des erreurs
form.errors()                                     // HashMap<String, String>
form.global_errors                                // Vec<String>
form.database_error(&db_err)                      // Parse les erreurs DB

// Accès aux champs
field.error()                                     // Option<&String>
field.value()                                     // &str
field.validate()                                  // bool
field.set_value("valeur")                         // Modifie la valeur
field.set_error("message")                        // Ajoute une erreur
```

---

### A voir aussi 

- [Guide de démarrage](informations/documentation_french/GETTING_STARTED.md)
- [Templates](informations/documentation_french/TEMPLATES.md)
- [Sécurité](informations/documentation_french/CSP.md)
- [Base de données](informations/documentation_french/DATABASE.md)

Créez des formulaires robustes avec Runique!

---

**Version:** 1.0.87 (Mise à jour - 17 Janvier 2026)
**Dernière mise à jour:** Janvier 2026  
**Licence:** MIT

*Documentation created with ❤️ by Claude for Itsuki*
