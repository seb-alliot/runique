# Guide des formulaires - Rusti Framework

Rusti propose un système de formulaires inspiré de Django avec génération automatique via macros procédurales.

## Table des matières

1. [Formulaires manuels](#formulaires-manuels)
2. [Formulaires auto-générés](#formulaires-auto-générés)
3. [Validation](#validation)
4. [Rendu HTML](#rendu-html)
5. [CSRF Protection](#csrf-protection)
6. [Exemples complets](#exemples-complets)

---

## Formulaires manuels

### Création basique

```rust
use rusti::forms::{RustiForm, Field, fields::{CharField, EmailField}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginForm {
    pub username: CharField,
    pub password: CharField,
}

impl RustiForm for LoginForm {
    fn new() -> Self {
        Self {
            username: CharField::new()
                .max_length(150)
                .required(true)
                .label("Nom d'utilisateur"),
            password: CharField::new()
                .max_length(128)
                .required(true)
                .widget("password")
                .label("Mot de passe"),
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

### Utilisation dans un handler

```rust
use rusti::prelude::*;

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

    // Traitement du formulaire valide
    let username = form.username.value();
    let password = form.password.value();

    // Authentification...
    redirect("/dashboard")
}
```

---

## Formulaires auto-générés

Rusti propose **deux macros** pour générer automatiquement des formulaires depuis vos modèles SeaORM :

1. **`#[rusti_form]`** - Pour créer des formulaires personnalisés
2. **`#[derive(DeriveModelForm)]`** - Pour générer des formulaires liés aux modèles

### Macro `#[rusti_form]`

Cette macro génère automatiquement l'implémentation du trait `RustiForm`.

```rust
use rusti::forms::prelude::*;

#[rusti_form]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactForm {
    #[field(max_length = 100, required = true)]
    pub name: CharField,

    #[field(required = true)]
    pub email: EmailField,

    #[field(max_length = 50, required = true)]
    pub subject: CharField,

    #[field(widget = "textarea", required = true)]
    pub message: CharField,
}
```

**Ce qui est généré automatiquement :**

```rust
impl RustiForm for ContactForm {
    fn new() -> Self { /* ... */ }
    fn is_valid(&self) -> bool { /* ... */ }
    fn errors(&self) -> Vec<String> { /* ... */ }
}
```

### Macro `#[derive(DeriveModelForm)]`

Cette macro est plus puissante et génère **plusieurs éléments** :

#### 1. Structure du formulaire

```rust
use rusti::forms::prelude::*;
use sea_orm::entity::prelude::*;

// Votre modèle SeaORM
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
}

// Génération du formulaire
#[derive(DeriveModelForm, Debug, Clone, Serialize, Deserialize)]
#[sea_orm(model = "Model", entity = "Entity")]
pub struct UserForm {
    #[field(max_length = 150, required = true)]
    pub username: CharField,

    #[field(required = true)]
    pub email: EmailField,

    #[field(widget = "textarea")]
    pub bio: CharField,
}
```

#### 2. Méthodes auto-générées

La macro `DeriveModelForm` génère **automatiquement** les éléments suivants :

##### a) Implémentation de `Deref` et `DerefMut`

```rust
// ✅ Généré automatiquement
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

##### b) Implémentation du trait `RustiForm`

```rust
// ✅ Généré automatiquement
impl RustiForm for UserForm {
    fn new() -> Self { /* ... */ }
    fn is_valid(&self) -> bool { /* ... */ }
    fn errors(&self) -> Vec<String> { /* ... */ }
}
```

##### c) Méthode `to_active_model()`

**Conversion automatique vers `ActiveModel` SeaORM :**

```rust
// ✅ Généré automatiquement
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

**Utilisation :**

```rust
pub async fn create_user(
    Form(form): Form<UserForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    if !form.is_valid() {
        return template.render("form.html", context! { form });
    }

    // ✅ Conversion automatique en ActiveModel
    let active_model = form.to_active_model();

    // Insertion en base
    match active_model.insert(&*db).await {
        Ok(user) => redirect(&format!("/user/{}", user.id)),
        Err(e) => template.render("form.html", context! {
            form,
            error: "Erreur lors de la création"
        }),
    }
}
```

##### d) Méthode `save()`

**Sauvegarde directe en base de données :**

```rust
// ✅ Généré automatiquement
impl UserForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<Model, DbErr> {
        let active_model = self.to_active_model();
        active_model.insert(db).await
    }
}
```

**Utilisation simplifiée :**

```rust
pub async fn create_user_simple(
    Form(form): Form<UserForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    if !form.is_valid() {
        return template.render("form.html", context! { form });
    }

    // ✅ Sauvegarde directe en une ligne
    match form.save(&*db).await {
        Ok(user) => redirect(&format!("/user/{}", user.id)),
        Err(e) => template.render("form.html", context! {
            form,
            error: "Erreur lors de la sauvegarde"
        }),
    }
}
```

#### 3. Récapitulatif des éléments générés

| Élément | Généré par `#[rusti_form]` | Généré par `#[derive(DeriveModelForm)]` |
|---------|---------------------------|----------------------------------------|
| Struct du formulaire | ✅ (manuel) | ✅ (manuel) |
| `impl RustiForm` | ✅ | ✅ |
| `impl Deref/DerefMut` | ❌ | ✅ |
| Méthode `to_active_model()` | ❌ | ✅ |
| Méthode `save()` | ❌ | ✅ |

---

## Validation

### Types de champs disponibles

```rust
use rusti::forms::fields::*;

// Texte simple
pub name: CharField,

// Email avec validation
pub email: EmailField,

// Nombre entier
pub age: IntegerField,

// Booléen
pub is_active: BooleanField,

// Date
pub birth_date: DateField,

// Choix (select)
pub role: ChoiceField,
```

### Attributs de validation

```rust
#[rusti_form]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterForm {
    // Longueur max
    #[field(max_length = 150, required = true)]
    pub username: CharField,

    // Validation email automatique
    #[field(required = true)]
    pub email: EmailField,

    // Longueur min et max
    #[field(min_length = 8, max_length = 128, required = true, widget = "password")]
    pub password: CharField,

    // Valeur par défaut
    #[field(default = "user")]
    pub role: CharField,

    // Optionnel (non requis)
    #[field(required = false, widget = "textarea")]
    pub bio: CharField,
}
```

### Validation personnalisée

```rust
impl RegisterForm {
    pub fn validate_passwords_match(&self, password_confirm: &str) -> bool {
        self.password.value() == password_confirm
    }

    pub fn validate_username_unique(&self, db: &DatabaseConnection) -> bool {
        // Vérifier en base de données
        // ...
        true
    }
}
```

---

## Rendu HTML

### Rendu automatique dans les templates

```html
<!-- templates/form.html -->
<form method="post">
    {{ csrf_input() }}

    <!-- Rendu automatique de tous les champs -->
    {{ form }}

    <button type="submit">Envoyer</button>
</form>
```

### Rendu champ par champ

```html
<form method="post">
    {{ csrf_input() }}

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

    <button type="submit">Créer</button>
</form>
```

### Widgets personnalisés

```rust
#[rusti_form]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleForm {
    #[field(max_length = 200, required = true)]
    pub title: CharField,

    // Textarea pour contenu long
    #[field(widget = "textarea", required = true)]
    pub content: CharField,

    // Input password
    #[field(widget = "password", max_length = 128)]
    pub password: CharField,

    // Select dropdown
    #[field(widget = "select")]
    pub category: ChoiceField,
}
```

---

## CSRF Protection

### Activation automatique

La protection CSRF est **automatiquement activée** dans Rusti lorsque le middleware `CsrfMiddleware` est ajouté.

```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    RustiApp::new(settings).await?
        .middleware(CsrfMiddleware::new())  // ✅ CSRF activé
        .routes(routes())
        .run()
        .await?;

    Ok(())
}
```

### Utilisation dans les templates

```html
<form method="post">
    <!-- Token CSRF automatique -->
    {{ csrf_input() }}

    {{ form }}

    <button type="submit">Envoyer</button>
</form>
```

### Validation dans les handlers

La validation CSRF est **automatique** via l'extracteur `Form<T>` :

```rust
pub async fn submit_form(
    Form(form): Form<ContactForm>,  // ✅ CSRF validé automatiquement
    template: Template,
) -> Response {
    if !form.is_valid() {
        return template.render("contact.html", context! { form });
    }

    // Le formulaire est valide ET le token CSRF a été vérifié
    // ...
}
```

**Note :** Si le token CSRF est invalide, une erreur `403 Forbidden` est automatiquement retournée **avant** que le handler ne soit appelé.

---

## Exemples complets

### Exemple 1 : Formulaire de contact simple

```rust
use rusti::prelude::*;
use rusti::forms::prelude::*;

#[rusti_form]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactForm {
    #[field(max_length = 100, required = true)]
    pub name: CharField,

    #[field(required = true)]
    pub email: EmailField,

    #[field(max_length = 50, required = true)]
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

    // Traiter le message (envoyer email, etc.)
    let _ = message.success("Message envoyé avec succès !").await;

    redirect("/")
}
```

### Exemple 2 : Formulaire lié au modèle avec sauvegarde

```rust
use rusti::prelude::*;
use rusti::forms::prelude::*;
use sea_orm::entity::prelude::*;

// Modèle SeaORM
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

// Formulaire auto-généré
#[derive(DeriveModelForm, Debug, Clone, Serialize, Deserialize)]
#[sea_orm(model = "Model", entity = "Entity")]
pub struct ArticleForm {
    #[field(max_length = 200, required = true)]
    pub title: CharField,

    #[field(max_length = 200, required = true)]
    pub slug: CharField,

    #[field(widget = "textarea", required = true)]
    pub content: CharField,

    #[field(default = "false")]
    pub published: BooleanField,
}

// Handler de création
pub async fn create_article(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    let form = ArticleForm::new();
    template.render("article_form.html", context! { form })
}

// Handler de soumission
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

    // ✅ Méthode 1 : Sauvegarde directe avec .save()
    match form.save(&*db).await {
        Ok(article) => {
            success!(message, "Article créé avec succès !");
            redirect(&format!("/article/{}", article.id))
        }
        Err(e) => {
            error!(message, "Erreur lors de la création");
            template.render("article_form.html", context! { form })
        }
    }

    // ✅ Méthode 2 : Conversion manuelle avec .to_active_model()
    // let active_model = form.to_active_model();
    // match active_model.insert(&*db).await {
    //     Ok(article) => { /* ... */ }
    //     Err(e) => { /* ... */ }
    // }
}
```

### Exemple 3 : Formulaire avec édition

```rust
pub async fn edit_article(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    let article = match Article::objects.get(&*db, id).await {
        Ok(a) => a,
        Err(_) => return (StatusCode::NOT_FOUND, "Article introuvable").into_response(),
    };

    // Pré-remplir le formulaire avec les données existantes
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

    // Récupérer l'article existant
    let existing = match Article::objects.get(&*db, id).await {
        Ok(a) => a,
        Err(_) => return (StatusCode::NOT_FOUND, "Article introuvable").into_response(),
    };

    // Créer un ActiveModel pour la mise à jour
    let mut active_model: ActiveModel = existing.into();
    active_model.title = Set(form.title.value().clone());
    active_model.slug = Set(form.slug.value().clone());
    active_model.content = Set(form.content.value().clone());
    active_model.published = Set(form.published.value().parse().unwrap_or(false));

    match active_model.update(&*db).await {
        Ok(updated) => {
            success!(message, "Article modifié avec succès !");
            redirect(&format!("/article/{}", updated.id))
        }
        Err(e) => {
            error!(message, "Erreur lors de la modification");
            template.render("article_form.html", context! {
                form,
                article_id: id,
                is_edit: true,
            })
        }
    }
}
```

### Exemple 4 : Validation personnalisée

```rust
#[derive(DeriveModelForm, Debug, Clone, Serialize, Deserialize)]
#[sea_orm(model = "Model", entity = "Entity")]
pub struct UserForm {
    #[field(max_length = 150, required = true)]
    pub username: CharField,

    #[field(required = true)]
    pub email: EmailField,

    #[field(min_length = 8, max_length = 128, required = true, widget = "password")]
    pub password: CharField,
}

impl UserForm {
    /// Validation personnalisée : vérifier que le username est unique
    pub async fn validate_unique_username(&self, db: &DatabaseConnection) -> bool {
        let existing = User::objects
            .filter(users::Column::Username.eq(self.username.value()))
            .first(db)
            .await;

        existing.is_err() // True si aucun utilisateur trouvé (username disponible)
    }

    /// Validation personnalisée : vérifier que l'email est unique
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
    // Validation standard
    if !form.is_valid() {
        return template.render("register.html", context! {
            form,
            errors: form.errors(),
        });
    }

    // Validations personnalisées
    if !form.validate_unique_username(&*db).await {
        error!(message, "Ce nom d'utilisateur est déjà pris");
        return template.render("register.html", context! { form });
    }

    if !form.validate_unique_email(&*db).await {
        error!(message, "Cet email est déjà utilisé");
        return template.render("register.html", context! { form });
    }

    // Sauvegarder l'utilisateur
    match form.save(&*db).await {
        Ok(user) => {
            success!(message, "Compte créé avec succès !");
            redirect(&format!("/user/{}", user.id))
        }
        Err(e) => {
            error!(message, "Erreur lors de la création du compte");
            template.render("register.html", context! { form })
        }
    }
}
```

---

## Bonnes pratiques

### 1. Toujours valider les formulaires

```rust
// ✅ Bon
pub async fn submit(Form(form): Form<MyForm>) -> Response {
    if !form.is_valid() {
        return template.render("form.html", context! { form });
    }
    // Traiter...
}

// ❌ Mauvais
pub async fn submit(Form(form): Form<MyForm>) -> Response {
    // Pas de validation !
    form.save(&db).await?; // Risque de données invalides
}
```

### 2. Utiliser les méthodes générées automatiquement

```rust
// ✅ Bon (utilise .save() généré automatiquement)
match form.save(&*db).await {
    Ok(user) => redirect("/success"),
    Err(e) => handle_error(e),
}

// ⚠️ Acceptable mais plus verbeux
let active_model = form.to_active_model();
match active_model.insert(&*db).await {
    Ok(user) => redirect("/success"),
    Err(e) => handle_error(e),
}
```

### 3. Gérer les erreurs proprement

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
            let _ = message.success("Utilisateur créé !").await;
            redirect(&format!("/user/{}", user.id))
        }
        Err(DbErr::RecordNotFound(_)) => {
            let _ = message.error("Enregistrement introuvable").await;
            template.render("form.html", context! { form })
        }
        Err(DbErr::Exec(_)) => {
            let _ = message.error("Erreur de contrainte (doublon ?)").await;
            template.render("form.html", context! { form })
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            let _ = message.error("Erreur interne").await;
            template.render("form.html", context! { form })
        }
    }
}
```

### 4. Utiliser des transactions pour les opérations complexes

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

    // Transaction pour garantir la cohérence
    let result = db.transaction::<_, (), DbErr>(|txn| {
        Box::pin(async move {
            // Créer l'utilisateur
            let user = user_form.to_active_model().insert(txn).await?;

            // Créer le profil
            let mut profile = profile_form.to_active_model();
            profile.user_id = Set(user.id);
            profile.insert(txn).await?;

            Ok(())
        })
    }).await;

    match result {
        Ok(_) => redirect("/success"),
        Err(e) => {
            let _ = message.error("Erreur lors de la création").await;
            template.render("form.html", context! {
                user_form,
                profile_form,
            })
        }
    }
}
```

### Exemple 5 : Utilisation des macros de messages

Les macros `success!()`, `error!()`, `info!()` et `warning!()` simplifient l'envoi de messages :

```rust
use rusti::prelude::*;

pub async fn create_article(
    Form(form): Form<ArticleForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
) -> Response {
    // Validation standard
    if !form.is_valid() {
        error!(message, "Le formulaire contient des erreurs");
        return template.render("article_form.html", context! { form });
    }

    // Validation personnalisée
    if form.title.value().len() < 10 {
        error!(message, "Le titre doit contenir au moins 10 caractères");
        return template.render("article_form.html", context! { form });
    }

    // Vérifier si le slug est unique
    if Article::slug_exists(&form.slug.value(), &db).await {
        error!(message, "Ce slug est déjà utilisé");
        warning!(message, "Essayez d'ajouter un numéro ou une date");
        return template.render("article_form.html", context! { form });
    }

    // Sauvegarder
    match form.save(&*db).await {
        Ok(article) => {
            success!(message, "Article créé avec succès !");
            
            if article.published {
                info!(message, "Votre article est maintenant visible par tous");
            } else {
                info!(message, "Votre article est en brouillon");
            }
            
            redirect(&format!("/articles/{}", article.id))
        }
        Err(e) => {
            error!(message, "Erreur lors de la création");
            template.render("article_form.html", context! { form })
        }
    }
}
```

**Comparaison syntaxe :**

```rust
let _ = message.success("Article créé !").await;
let _ = message.error("Erreur").await;

// Nouvelle syntaxe (avec macros)
success!(message, "Article créé !");
error!(message, "Erreur");
```

---

## Voir aussi

- [Guide de démarrage](GETTING_STARTED.md)
- [Templates](TEMPLATES.md)
- [Sécurité](SECURITY.md)
- [Base de données](DATABASE.md)

Créez des formulaires robustes avec Rusti !

---

**Version:** 1.0 (Corrigée - 2 Janvier 2026)
**Licence:** MIT