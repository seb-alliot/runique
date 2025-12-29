# Documentation Rusti Forms

**Système de formulaires Django-like pour Rust**

Version 2.0 - Documentation complète - Décembre 2025

---

## Table des matières

1. [Introduction](#1-introduction)
2. [Démarrage rapide](#2-démarrage-rapide)
3. [Types de champs disponibles](#3-types-de-champs-disponibles)
4. [Exemples pratiques](#4-exemples-pratiques)
5. [Comparaison Django vers Rusti](#5-comparaison-django-vers-rusti)
6. [Référence API](#6-référence-api)
7. [Erreurs courantes](#7-erreurs-courantes)
8. [Sécurité intégrée](#8-sécurité-intégrée)
9. [Templates HTML](#9-templates-html)
10. [FAQ](#10-faq)
11. [Index des méthodes](#11-index-des-méthodes)

---

## 1. Introduction

Rusti Forms est un système de validation de formulaires pour le framework web Rusti, inspiré du système de formulaires de Django. Il combine la facilité d'utilisation de Django avec la sécurité et les performances de Rust.

### Différence majeure avec Django

**Approche Django :** Vous devez manuellement créer un fichier séparé `forms.py` et définir chaque classe de formulaire avec tous ses champs :

```python
# forms.py - OBLIGATOIRE dans Django
from django import forms
from .models import User

class UserForm(forms.ModelForm):
    class Meta:
        model = User
        fields = ['username', 'email', 'bio']
```

**Approche Rusti :** Avec `DeriveModelForm`, les formulaires sont **générés automatiquement** directement depuis vos modèles - aucun fichier forms séparé nécessaire !

```rust
// models.rs - C'est tout ce dont vous avez besoin !
#[derive(DeriveModelForm, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
}
// UserForm est auto-généré avec les méthodes validate() et save() !
```

C'est un **gain de productivité majeur** - un fichier en moins à maintenir, mises à jour instantanées quand les modèles changent, et zéro code répétitif !

### Caractéristiques principales

- Validation automatique avec types de champs prédéfinis
- Sanitisation XSS intégrée
- Trim automatique des espaces blancs
- Hachage sécurisé des mots de passe (Argon2id)
- API familière Django-like
- Type-safe grâce à Rust
- **Formulaires auto-générés depuis les modèles** (pas besoin de forms.py !)

### Installation

Ajoutez Rusti à votre `Cargo.toml` :

```toml
[dependencies]
rusti = "0.1"
```

---

## 2. Démarrage rapide

### Installation et premier formulaire en 30 secondes

```rust
// Cargo.toml
[dependencies]
rusti = "0.1"

// main.rs
use rusti::rusti_form;
use rusti::formulaire::formsrusti::{Forms, FormulaireTrait};
use rusti::formulaire::field::{CharField, EmailField};
use std::collections::HashMap;

#[rusti_form]
pub struct ContactForm {
    pub form: Forms,
}

impl FormulaireTrait for ContactForm {
    fn new() -> Self {
        Self { form: Forms::new() }
    }

    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.require("name", &CharField { allow_blank: false }, raw_data);
        self.require("email", &EmailField, raw_data);
        self.is_valid()
    }
}

// Utilisation
let mut form = ContactForm::new();
if form.validate(&raw_data) {
    let name: String = form.get_value("name").unwrap();
    // Traiter les données...
}
```

---

## 3. Types de champs disponibles

| Type | Description | Validation |
|------|-------------|------------|
| **CharField** | Champ texte court | Sanitisation XSS, trim |
| **TextField** | Champ texte long | Sanitisation XSS, trim |
| **EmailField** | Adresse email | Format RFC 5322 |
| **PasswordField** | Mot de passe | Hash Argon2id |
| **IntegerField** | Nombre entier | Parse vers i64 |
| **FloatField** | Nombre décimal | Parse vers f64 |
| **BooleanField** | Booléen | true/false, 1/0, on/off |
| **DateField** | Date | Format YYYY-MM-DD |
| **SlugField** | Slug URL-friendly | Lettres, chiffres, tirets |
| **URLField** | URL | Format URL valide |
| **IPAddressField** | Adresse IP | IPv4 ou IPv6 |
| **JSONField** | Données JSON | Parse JSON valide |

---

## 4. Exemples pratiques

### 4.1 Formulaire simple

Formulaire de contact basique :

```rust
use rusti::rusti_form;
use rusti::formulaire::formsrusti::{Forms, FormulaireTrait};
use rusti::formulaire::field::{CharField, EmailField, TextField};
use std::collections::HashMap;

#[rusti_form]
pub struct ContactForm {
    pub form: Forms,
}

impl FormulaireTrait for ContactForm {
    fn new() -> Self {
        Self { form: Forms::new() }
    }

    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.require("name", &CharField { allow_blank: false }, raw_data);
        self.require("email", &EmailField, raw_data);
        self.require("message", &TextField, raw_data);

        self.is_valid()
    }
}
```

### Utilisation dans un handler Axum

```rust
use rusti::prelude::*;

pub async fn contact_submit(
    template: Template,
    ExtractForm(form): ExtractForm<ContactForm>,
) -> Response {
    if form.is_valid() {
        let name: String = form.get_value("name").unwrap();
        let email: String = form.get_value("email").unwrap();
        let message: String = form.get_value("message").unwrap();

        // Traiter les données (envoyer email, sauvegarder en BDD, etc.)
        // ...

        let ctx = context! {
            "success", true ;
            "message", "Merci de nous avoir contactés !"
        };

        template.render("contact_success.html", &ctx)
    } else {
        // Afficher les erreurs
        let ctx = context! {
            "form", &form
        };

        template.render("contact.html", &ctx)
    }
}
```

### 4.2 Validation personnalisée

Formulaire d'inscription avec validation du mot de passe :

```rust
use rusti::rusti_form;
use rusti::formulaire::formsrusti::{Forms, FormulaireTrait};
use rusti::formulaire::field::{CharField, EmailField, PasswordField};
use std::collections::HashMap;
use fancy_regex::Regex;

#[rusti_form]
pub struct RegisterForm {
    pub form: Forms,
}

impl FormulaireTrait for RegisterForm {
    fn new() -> Self {
        Self { form: Forms::new() }
    }

    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.require("username", &CharField { allow_blank: false }, raw_data);
        self.require("email", &EmailField, raw_data);

        if let Some(password) = raw_data.get("password") {
            self.validate_password(password);
        } else {
            self.errors.insert("password".to_string(), "Requis".to_string());
        }

        self.is_valid()
    }
}

impl RegisterForm {
    fn validate_password(&mut self, raw_value: &str) -> Option<String> {
        let regex = Regex::new(
            r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[A-Za-z\d@$!%*?&]{8,}$"
        ).unwrap();

        if raw_value.len() < 8 {
            self.errors.insert(
                "password".to_string(),
                "Le mot de passe doit contenir au moins 8 caractères".to_string()
            );
            return None;
        }

        if !regex.is_match(raw_value).unwrap_or(false) {
            self.errors.insert(
                "password".to_string(),
                "Doit contenir majuscule, minuscule et chiffre".to_string()
            );
            return None;
        }

        self.field("password", &PasswordField, raw_value)
    }
}
```

### 4.3 Validation inter-champs

Vérification de cohérence entre plusieurs champs (style `clean()` de Django) :

```rust
impl RegisterForm {
    fn clean(&mut self, raw_data: &HashMap<String, String>) {
        // Ignorer si déjà des erreurs
        if self.is_not_valid() {
            return;
        }

        let username: Option<String> = self.get_value("username");
        let password = raw_data.get("password");

        if let (Some(user), Some(pass)) = (username, password) {
            if pass.to_lowercase().contains(&user.to_lowercase()) {
                self.errors.insert(
                    "password".to_string(),
                    "Le mot de passe ne peut pas contenir le nom d'utilisateur".to_string()
                );
            }
        }

        // Vérifier la confirmation du mot de passe
        let password_confirm = raw_data.get("password_confirm");

        if let (Some(pass1), Some(pass2)) = (password, password_confirm) {
            if pass1 != pass2 {
                self.errors.insert(
                    "password_confirm".to_string(),
                    "Les mots de passe ne correspondent pas".to_string()
                );
            }
        }
    }
}

// Utilisation dans validate()
impl FormulaireTrait for RegisterForm {
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        // 1. Validation individuelle des champs
        self.require("username", &CharField { allow_blank: false }, raw_data);
        self.require("email", &EmailField, raw_data);

        // 2. Validation personnalisée
        if let Some(password) = raw_data.get("password") {
            self.validate_password(password);
        }

        // 3. Validation inter-champs
        self.clean(raw_data);

        // 4. Retourner le résultat
        self.is_valid()
    }
}
```


### 4.4 Model Forms - Générés depuis les modèles de base de données

Tout comme les `ModelForm` de Django, Rusti peut automatiquement générer des formulaires depuis vos modèles de base de données avec la macro `DeriveModelForm`.

#### Formulaire de modèle basique

```rust
use sea_orm::entity::prelude::*;
use rusti::DeriveModelForm;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveModelForm)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,                    // Automatiquement exclu
    pub username: String,           // CharField
    pub email: String,              // EmailField (détecté par le nom)
    pub bio: Option<String>,        // CharField optionnel
    pub age: i32,                   // IntegerField
    pub is_active: bool,            // BooleanField
    pub created_at: DateTime,       // Automatiquement exclu
}

// Cela génère automatiquement :
// - La struct UserForm
// - L'implémentation FormulaireTrait
// - La méthode to_active_model()
// - La méthode save()
```

#### Détection automatique des champs

La macro infère automatiquement les types de champs :

| Type du champ modèle | Détecté comme | Notes |
|----------------------|---------------|-------|
| `String` | `CharField` | Champ texte par défaut |
| Champ nommé `*email*` | `EmailField` | Détection par nom |
| Champ nommé `*password*` | `PasswordField` | Détection par nom |
| Champ nommé `*slug*` | `SlugField` | Détection par nom |
| `i32`, `i64` | `IntegerField` | Nombres entiers |
| `f32`, `f64` | `FloatField` | Nombres décimaux |
| `bool` | `BooleanField` | Booléen |
| `NaiveDate` | `DateField` | Date seulement |
| `DateTime` | `DateTimeField` | Date et heure |
| `Option<T>` | `optional()` | Champs nullables |

#### Exclusions automatiques

Ces champs sont **automatiquement exclus** du formulaire :

- Champ `id`
- Champ `created_at`
- Champ `updated_at`
- Champs avec `#[sea_orm(primary_key)]`

#### Utilisation dans un handler

```rust
use rusti::prelude::*;

pub async fn create_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    ExtractForm(form): ExtractForm<UserForm>,  // Auto-généré
    mut message: Message,
) -> Response {
    // La validation se fait automatiquement
    if !form.is_valid() {
        let ctx = context! {
            "form", &form
        };
        return template.render("users/create.html", &ctx);
    }

    // Méthode 1 : Sauvegarde directe
    match form.save(&db).await {
        Ok(user) => {
            message.success(&format!("Utilisateur {} créé !", user.username)).await.ok();
            redirect("/users")
        }
        Err(e) => {
            message.error("Erreur lors de la création").await.ok();
            let ctx = context! {
                "form", &form ;
                "error", e.to_string()
            };
            template.render("users/create.html", &ctx)
        }
    }
}
```

#### Avancé : to_active_model()

Pour plus de contrôle, utilisez `to_active_model()` :

```rust
pub async fn create_user_advanced(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    ExtractForm(form): ExtractForm<UserForm>,
) -> Response {
    if form.is_valid() {
        use sea_orm::ActiveValue::Set;

        // Obtenir l'ActiveModel
        let mut user = form.to_active_model();

        // Ajouter des champs personnalisés
        user.created_at = Set(chrono::Utc::now());
        user.updated_at = Set(chrono::Utc::now());

        // Insérer manuellement
        match user.insert(&*db).await {
            Ok(inserted) => {
                // Traiter...
            }
            Err(e) => {
                // Gérer l'erreur...
            }
        }
    }
}
```

#### Comparaison : Django ModelForm vs Rusti DeriveModelForm

**Django - Approche manuelle (2 fichiers requis) :**

```python
# models.py
class User(models.Model):
    username = models.CharField(max_length=100)
    email = models.EmailField()
    bio = models.TextField(blank=True)
    age = models.IntegerField()

# forms.py - FICHIER SÉPARÉ OBLIGATOIRE
from django import forms
from .models import User

class UserForm(forms.ModelForm):
    class Meta:
        model = User
        fields = ['username', 'email', 'bio', 'age']
        exclude = ['id', 'created_at']

# views.py
form = UserForm(request.POST)
if form.is_valid():
    user = form.save()
```

**Rusti - Approche automatique (1 seul fichier) :**

```rust
// models.rs - SEUL FICHIER NÉCESSAIRE !
#[derive(DeriveModelForm, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    pub id: i32,          // Auto-exclu
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
    pub age: i32,
}
// UserForm est automatiquement généré !

// handlers.rs
let mut form = UserForm::new();
if form.validate(&raw_data) {
    let user = form.save(&db).await?;
}
```

**Avantages clés :**

1. **Pas de fichier forms.py** - Un fichier en moins à maintenir
2. **Pas de duplication** - Le modèle est la source unique de vérité
3. **Synchro auto** - Le formulaire se met à jour automatiquement quand le modèle change
4. **Zéro code répétitif** - Pas de classe Meta, pas de liste de champs
5. **Sécurité à la compilation** - Rust détecte les erreurs avant l'exécution

#### Validation personnalisée avec Model Forms

Vous pouvez ajouter une validation personnalisée aux formulaires de modèle :

```rust
impl UserForm {
    fn clean(&mut self, raw_data: &HashMap<String, String>) {
        if self.is_not_valid() {
            return;
        }

        // Validation personnalisée
        let username: Option<String> = self.get_value("username");
        if let Some(user) = username {
            if user.len() < 3 {
                self.errors.insert(
                    "username".to_string(),
                    "Le nom d'utilisateur doit contenir au moins 3 caractères".to_string()
                );
            }
        }
    }
}

// Utiliser dans validate()
pub async fn register(
    ExtractForm(mut form): ExtractForm<UserForm>,
) -> Response {
    // Validation standard
    form.validate(&raw_data);

    // Ajouter validation personnalisée
    form.clean(&raw_data);

    if form.is_valid() {
        // Traiter...
    }
}
```

#### Avantages de DeriveModelForm

- **Zéro code répétitif** - Détection automatique des champs
- **Pas de fichier forms séparé** - Contrairement à Django, pas besoin de forms.py
- **Type-safe** - Validation à la compilation
- **Principe DRY** - Source unique de vérité (le modèle)
- **Django-like** - Méthode `.save()` familière
- **Défauts intelligents** - Exclut id, timestamps automatiquement
- **Extensible** - Ajout facile de validation personnalisée
- **Synchro auto** - Les formulaires se mettent à jour quand les modèles changent (pas de synchro manuelle)

---

## 5. Comparaison Django vers Rusti

Pour faciliter la transition depuis Django, voici les équivalences entre les deux frameworks :

| Django | Rusti | Notes |
|--------|-------|-------|
| `forms.CharField(max_length=100)` | `CharField { allow_blank: false }` | Pas de max_length en Rust |
| `forms.EmailField()` | `EmailField` | Validation RFC 5322 |
| `forms.ModelForm` | `#[derive(DeriveModelForm)]` | Auto-génère le formulaire depuis le modèle |
| `form.is_valid()` | `self.is_valid()` | Via Deref |
| `form.cleaned_data["email"]` | `self.get_value::<String>("email")` | Type-safe |
| `form.errors["email"]` | `self.errors.get("email")` | HashMap standard |
| `def clean(self):` | `fn clean(&mut self, data: &HashMap)` | Validation inter-champs |
| `raise ValidationError("msg")` | `self.errors.insert(field, msg)` | Ajout manuel d'erreur |
| `form.save()` | `form.save(&db).await` | Sauvegarde async en base de données |

---

## 6. Référence API

### Méthodes principales

| Méthode | Signature | Description |
|---------|-----------|-------------|
| **require** | `(&mut self, name: &str, field: &impl FieldTrait, data: &HashMap)` | Valide un champ requis |
| **optional** | `(&mut self, name: &str, field: &impl FieldTrait, data: &HashMap)` | Valide un champ optionnel |
| **field** | `(&mut self, name: &str, field: &impl FieldTrait, value: &str) -> Option<String>` | Valide une valeur brute directement |
| **is_valid** | `(&self) -> bool` | Retourne true si aucune erreur |
| **is_not_valid** | `(&self) -> bool` | Retourne true s'il y a des erreurs |
| **get_value** | `<T>(&self, name: &str) -> Option<T>` | Récupère une valeur validée typée |
| **clear** | `(&mut self)` | Réinitialise errors et cleaned_data |

### Gestion des erreurs

| Méthode | Description |
|---------|-------------|
| **errors.insert(name, msg)** | Ajoute une erreur personnalisée |
| **errors.get(name)** | Récupère l'erreur d'un champ |
| **errors.is_empty()** | Vérifie s'il n'y a pas d'erreurs |

### Accès aux données

| Méthode | Description |
|--------|-------------|
| **cleaned_data.get(name)** | Récupère la valeur brute validée |
| **cleaned_data.contains_key(name)** | Vérifie si le champ a été validé |

---

## 7. Erreurs courantes

### 7.1 Utilisation redondante de self.form

**À ne pas faire :**

```rust
// INCORRECT - Redondant avec Deref
self.form.require("email", &EmailField, raw_data);
self.form.is_valid()
```

**À faire :**

```rust
// CORRECT - Direct grâce à la macro #[rusti_form]
self.require("email", &EmailField, raw_data);
self.is_valid()
```

### 7.2 Mauvais type sur get_value

**À ne pas faire :**

```rust
// INCORRECT - Incompatibilité de type
let age: String = self.get_value("age").unwrap();  // age est un IntegerField !
```

**À faire :**

```rust
// CORRECT - Type correspondant au champ
let age: i64 = self.get_value("age").unwrap();     // IntegerField -> i64
let email: String = self.get_value("email").unwrap();  // EmailField -> String
```

### 7.3 Oublier de vérifier is_valid() avant get_value()

**À ne pas faire :**

```rust
// INCORRECT - Peut paniquer si la validation a échoué
let email: String = form.get_value("email").unwrap();  // unwrap() peut échouer !
```

**À faire :**

```rust
// CORRECT - Toujours vérifier d'abord
if form.is_valid() {
    let email: String = form.get_value("email").unwrap();  // Sûr ici
} else {
    // Gérer les erreurs
}
```

### 7.4 Oublier la macro #[rusti_form]

**À ne pas faire :**

```rust
// INCORRECT - Pas de macro
#[derive(Serialize, Deserialize, Debug)]
pub struct UserForm {
    #[serde(flatten)]  // Doit être ajouté manuellement
    pub form: Forms,
}

// Et vous devez implémenter Deref manuellement...
```

**À faire :**

```rust
// CORRECT - La macro fait tout automatiquement
#[rusti_form]
pub struct UserForm {
    pub form: Forms,
}

// Deref + DerefMut + #[serde(flatten)] ajoutés automatiquement !
```

---

## 8. Sécurité intégrée

| Fonctionnalité | Description | Champs affectés |
|----------------|-------------|-----------------|
| **Protection XSS** | Suppression automatique des balises `<script>`, `<iframe>`, `<object>` et attributs JavaScript (`onclick`, `onerror`, etc.) | CharField, TextField |
| **Hash Argon2id** | Hash sécurisé avec sel unique, résistant aux attaques GPU/ASIC. Format standard PHC. | PasswordField |
| **Trim automatique** | Suppression des espaces blancs en début et fin | Tous les champs texte |
| **Validation de format** | Vérification stricte du format (RFC 5322 email, URL, IP, JSON) | EmailField, URLField, IPAddressField, JSONField |

### Bonnes pratiques de sécurité

```rust
// Bon - Mot de passe haché automatiquement
self.require("password", &PasswordField, raw_data);
let hashed: String = self.get_value("password").unwrap();
// hashed est déjà au format Argon2id

// Bon - Protection XSS automatique
self.require("comment", &TextField, raw_data);
let safe_comment: String = self.get_value("comment").unwrap();
// Les balises <script> ont été supprimées

// Mauvais - Ne pas hacher les mots de passe manuellement
let raw_password = raw_data.get("password").unwrap();
// Ne jamais stocker les mots de passe en clair !
```

---

## 9. Templates HTML

### Affichage des erreurs dans les templates Tera

```html
<!-- Formulaire avec erreurs -->
<form method="post" action='{% link "contact_submit" %}'>
    {% csrf %}

    <div class="form-group">
        <label for="name">Nom :</label>
        <input type="text" name="name" id="name"
               value="{{ form.cleaned_data.name | default(value='') }}"
               class="{% if form.errors.name %}error{% endif %}">

        {% if form.errors.name %}
            <span class="error-message">{{ form.errors.name }}</span>
        {% endif %}
    </div>

    <div class="form-group">
        <label for="email">Email :</label>
        <input type="email" name="email" id="email"
               value="{{ form.cleaned_data.email | default(value='') }}"
               class="{% if form.errors.email %}error{% endif %}">

        {% if form.errors.email %}
            <span class="error-message">{{ form.errors.email }}</span>
        {% endif %}
    </div>

    <div class="form-group">
        <label for="message">Message :</label>
        <textarea name="message" id="message"
                  class="{% if form.errors.message %}error{% endif %}">{{ form.cleaned_data.message | default(value='') }}</textarea>

        {% if form.errors.message %}
            <span class="error-message">{{ form.errors.message }}</span>
        {% endif %}
    </div>

    <!-- Afficher toutes les erreurs en haut -->
    {% if form.errors %}
        <div class="alert alert-danger">
            <h4>Veuillez corriger les erreurs suivantes :</h4>
            <ul>
            {% for field, message in form.errors %}
                <li><strong>{{ field }}</strong> : {{ message }}</li>
            {% endfor %}
            </ul>
        </div>
    {% endif %}

    <button type="submit" class="btn btn-primary">Envoyer</button>
</form>
```

### Style CSS

```css
.form-group {
    margin-bottom: 1.5rem;
}

.form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
}

.form-group input,
.form-group textarea {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
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

.alert {
    padding: 1rem;
    margin-bottom: 1rem;
    border-radius: 4px;
}

.alert-danger {
    background-color: #f8d7da;
    border: 1px solid #f5c6cb;
    color: #721c24;
}
```


---

## 10. FAQ (Foire Aux Questions)

### Q1 : Comment valider plusieurs champs ensemble ?

Utilisez une méthode `clean()` comme dans Django :

```rust
impl MyForm {
    fn clean(&mut self, raw_data: &HashMap<String, String>) {
        if self.is_not_valid() {
            return;  // Ignorer si déjà des erreurs
        }

        let password: Option<String> = self.get_value("password");
        let confirm: Option<String> = self.get_value("password_confirm");

        if password != confirm {
            self.errors.insert(
                "password_confirm".to_string(),
                "Les mots de passe ne correspondent pas".to_string()
            );
        }
    }
}
```

### Q2 : Différence entre require() et optional() ?

**require()** : Le champ DOIT être présent et non vide. Ajoute une erreur "Requis" si absent.

**optional()** : Le champ peut être absent ou vide. Pas d'erreur si absent, mais validation appliquée si présent.

```rust
// require() - Champ requis
self.require("email", &EmailField, raw_data);
// Si absent ou vide -> erreur "Requis"

// optional() - Champ optionnel
self.optional("phone", &CharField { allow_blank: false }, raw_data);
// Si absent -> Pas d'erreur
// Si présent -> Validation appliquée
```

### Q3 : Comment réutiliser des validateurs personnalisés ?

Créez une fonction helper réutilisable :

```rust
fn validate_strong_password(form: &mut Forms, field_name: &str, value: &str) {
    let regex = Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$").unwrap();

    if !regex.is_match(value).unwrap_or(false) {
        form.errors.insert(
            field_name.to_string(),
            "Mot de passe faible : doit contenir majuscule, minuscule et chiffre".to_string()
        );
    }
}

// Utilisation dans plusieurs formulaires
impl RegisterForm {
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        // ...
        if let Some(pwd) = raw_data.get("password") {
            validate_strong_password(self, "password", pwd);
        }
        self.is_valid()
    }
}
```

### Q4 : Que fait exactement la macro #[rusti_form] ?

La macro génère automatiquement :

1. **#[derive(Serialize, Deserialize, Debug)]** si pas déjà présent
2. **#[serde(flatten)]** sur le champ Forms
3. **impl Deref** pour accéder directement aux méthodes Forms
4. **impl DerefMut** pour les méthodes mutables

Cela élimine le code répétitif et facilite l'utilisation des formulaires !

### Q5 : Peut-on avoir plusieurs champs Forms dans une struct ?

**Non.** La macro #[rusti_form] nécessite exactement UN champ Forms. Si vous avez besoin de formulaires imbriqués, créez des structs séparées et combinez-les manuellement :

```rust
#[rusti_form]
pub struct AddressForm {
    pub form: Forms,
}

#[rusti_form]
pub struct UserForm {
    pub form: Forms,
}

// Combiner manuellement dans votre handler
pub async fn register(
    ExtractForm(user_form): ExtractForm<UserForm>,
    ExtractForm(address_form): ExtractForm<AddressForm>,
) -> Response {
    if user_form.is_valid() && address_form.is_valid() {
        // Traiter les deux formulaires
    }
}
```

### Q6 : Comment gérer les uploads de fichiers ?

Pour les uploads de fichiers, utilisez le support multipart d'Axum avec les formulaires :

```rust
use axum::extract::Multipart;

pub async fn upload_profile(
    mut multipart: Multipart,
    ExtractForm(form): ExtractForm<ProfileForm>,
) -> Response {
    if !form.is_valid() {
        // Gérer les erreurs du formulaire
        return template.render("profile.html", &context! { "form", &form });
    }

    // Traiter l'upload de fichier
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        if name == "avatar" {
            // Sauvegarder le fichier avatar
        }
    }

    // Continuer le traitement...
}
```

### Q7 : Quand utiliser #[rusti_form] vs #[derive(DeriveModelForm)] ?

**Utilisez `#[rusti_form]`** quand :
- Vous avez besoin d'un formulaire personnalisé non lié à un modèle
- Vous voulez un contrôle total sur les types de champs
- Le formulaire ne correspond pas exactement à la structure de la base de données
- Exemple : Formulaire de contact, recherche, connexion

```rust
#[rusti_form]
pub struct ContactForm {
    pub form: Forms,
}

impl FormulaireTrait for ContactForm {
    fn validate(&mut self, raw_data: &HashMap) -> bool {
        self.require("name", &CharField { allow_blank: false }, raw_data);
        self.require("email", &EmailField, raw_data);
        self.is_valid()
    }
}
```

**Utilisez `#[derive(DeriveModelForm)]`** quand :
- Le formulaire correspond directement à un modèle de base de données
- Vous voulez la détection automatique des champs
- Vous avez besoin de la fonctionnalité `.save()`
- Vous suivez le principe DRY (modèle comme source unique de vérité)
- Exemple : Inscription utilisateur, création produit, édition profil

```rust
#[derive(DeriveModelForm, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    pub id: i32,
    pub username: String,
    pub email: String,
}

// C'est tout ! UserForm est auto-généré avec validate() et save()
```

**Comparaison :**

| Fonctionnalité | #[rusti_form] | #[derive(DeriveModelForm)] |
|----------------|---------------|----------------------------|
| Définition manuelle des champs | Oui | Non - Auto-détecté |
| Validation personnalisée | Oui | Oui - Via clean() |
| Méthode .save() | Non - Manuel | Oui - Automatique |
| Lié au modèle | Non | Oui |
| Meilleur pour | Formulaires personnalisés | CRUD basé sur modèle |

---

## 11. Index des méthodes alphabétique

| Méthode | Page | Catégorie |
|---------|------|----------|
| **clean()** | 4.3, 10 | Validation inter-champs |
| **clear()** | 6 | Réinitialisation |
| **field()** | 6 | Validation directe |
| **get_value<T>()** | 4, 6 | Récupération des données |
| **is_not_valid()** | 6 | Vérification |
| **is_valid()** | 4, 6 | Vérification |
| **new()** | 4 | Construction |
| **optional()** | 6, 10 | Validation de champ |
| **require()** | 4, 6, 10 | Validation de champ |
| **validate()** | 4 | Trait FormulaireTrait |

---

## Conclusion

Rusti Forms offre un système de validation puissant et sécurisé, combinant la familiarité de Django avec la robustesse de Rust. La macro **#[rusti_form]** réduit considérablement le code répétitif, permettant aux développeurs de se concentrer sur la logique métier. La sécurité est intégrée par défaut avec la sanitisation XSS, le hachage Argon2 et la validation stricte des formats.

### Points clés à retenir

- **Inspiré de Django** - API familière pour une transition facile
- **Type-safe** - Le système de types de Rust prévient les erreurs d'exécution
- **Sécurisé par défaut** - Protection XSS, hachage des mots de passe inclus
- **Zéro code répétitif** - La macro `#[rusti_form]` fait le gros du travail
- **Prêt pour la production** - Utilisé dans de vraies applications Rusti

### Prochaines étapes

- Lire la [Documentation Rusti Framework](../README.md)
- Consulter le [Guide de démarrage](GETTING_STARTED.md)
- Explorer la [Documentation des templates](TEMPLATES.md)
- Apprendre l'[Intégration base de données](DATABASE.md)

---

**Rusti Forms v2.0**
Sécurisé - Type-safe - Performant

Documentation - Décembre 2025
Développé avec passion en Rust par Itsuki
