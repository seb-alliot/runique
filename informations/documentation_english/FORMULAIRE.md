# Rusti Forms Documentation

**Django-like Form System for Rust**

Version 2.0 - Complete Documentation - December 2025

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Quick Start](#2-quick-start)
3. [Available Field Types](#3-available-field-types)
4. [Practical Examples](#4-practical-examples)
5. [Django to Rusti Comparison](#5-django-to-rusti-comparison)
6. [API Reference](#6-api-reference)
7. [Common Pitfalls](#7-common-pitfalls)
8. [Built-in Security](#8-built-in-security)
9. [HTML Templates](#9-html-templates)
10. [FAQ](#10-faq)
11. [Method Index](#11-method-index)

---

## 1. Introduction

Rusti Forms is a form validation system for the Rusti web framework, inspired by Django's forms system. It combines Django's ease of use with Rust's security and performance.

### Key Difference from Django

**Django approach:** You must manually create a separate `forms.py` file and define every form class with all its fields:

```python
# forms.py - REQUIRED in Django
from django import forms
from .models import User

class UserForm(forms.ModelForm):
    class Meta:
        model = User
        fields = ['username', 'email', 'bio']
```

**Rusti approach:** With `DeriveModelForm`, forms are **automatically generated** directly from your models - no separate forms file needed!

```rust
// models.rs - That's all you need!
#[derive(DeriveModelForm, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
}
// UserForm is auto-generated with validate() and save() methods!
```

This is a **major productivity gain** - one less file to maintain, instant updates when models change, and zero boilerplate!

### Key Features

- Automatic validation with predefined field types
- Built-in XSS sanitization
- Automatic whitespace trimming
- Secure password hashing (Argon2id)
- Familiar Django-like API
- Type-safe thanks to Rust
- **Auto-generated forms from models** (no separate forms.py needed!)

### Installation

Add Rusti to your `Cargo.toml`:

```toml
[dependencies]
rusti = "0.1"
```

---

## 2. Quick Start

### Installation and first form in 30 seconds

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

// Usage
let mut form = ContactForm::new();
if form.validate(&raw_data) {
    let name: String = form.get_value("name").unwrap();
    // Process data...
}
```

---

## 3. Available Field Types

| Type | Description | Validation |
|------|-------------|------------|
| **CharField** | Short text field | XSS sanitization, trim |
| **TextField** | Long text field | XSS sanitization, trim |
| **EmailField** | Email address | RFC 5322 format |
| **PasswordField** | Password | Argon2id hash |
| **IntegerField** | Integer number | Parse to i64 |
| **FloatField** | Decimal number | Parse to f64 |
| **BooleanField** | Boolean | true/false, 1/0, on/off |
| **DateField** | Date | YYYY-MM-DD format |
| **SlugField** | URL-friendly slug | Letters, numbers, hyphens |
| **URLField** | URL | Valid URL format |
| **IPAddressField** | IP Address | IPv4 or IPv6 |
| **JSONField** | JSON data | Valid JSON parse |

---

## 4. Practical Examples

### 4.1 Simple Form

Basic contact form:

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

### Usage in an Axum handler

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

        // Process data (send email, save to DB, etc.)
        // ...

        let ctx = context! {
            "success", true ;
            "message", "Thank you for contacting us!"
        };

        template.render("contact_success.html", &ctx)
    } else {
        // Display errors
        let ctx = context! {
            "form", &form
        };

        template.render("contact.html", &ctx)
    }
}
```

### 4.2 Custom Validation

Registration form with password validation:

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
            self.errors.insert("password".to_string(), "Required".to_string());
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
                "Password must be at least 8 characters".to_string()
            );
            return None;
        }

        if !regex.is_match(raw_value).unwrap_or(false) {
            self.errors.insert(
                "password".to_string(),
                "Must contain uppercase, lowercase and digit".to_string()
            );
            return None;
        }

        self.field("password", &PasswordField, raw_value)
    }
}
```

### 4.3 Cross-field Validation

Consistency check between multiple fields (Django-style `clean()`):

```rust
impl RegisterForm {
    fn clean(&mut self, raw_data: &HashMap<String, String>) {
        // Skip if already errors
        if self.is_not_valid() {
            return;
        }

        let username: Option<String> = self.get_value("username");
        let password = raw_data.get("password");

        if let (Some(user), Some(pass)) = (username, password) {
            if pass.to_lowercase().contains(&user.to_lowercase()) {
                self.errors.insert(
                    "password".to_string(),
                    "Password cannot contain username".to_string()
                );
            }
        }

        // Check password confirmation
        let password_confirm = raw_data.get("password_confirm");

        if let (Some(pass1), Some(pass2)) = (password, password_confirm) {
            if pass1 != pass2 {
                self.errors.insert(
                    "password_confirm".to_string(),
                    "Passwords do not match".to_string()
                );
            }
        }
    }
}

// Usage in validate()
impl FormulaireTrait for RegisterForm {
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        // 1. Individual field validation
        self.require("username", &CharField { allow_blank: false }, raw_data);
        self.require("email", &EmailField, raw_data);

        // 2. Custom validation
        if let Some(password) = raw_data.get("password") {
            self.validate_password(password);
        }

        // 3. Cross-field validation
        self.clean(raw_data);

        // 4. Return result
        self.is_valid()
    }
}
```

### 4.4 Model Forms - Generated from Database Models

Just like Django's `ModelForm`, Rusti can automatically generate forms from your database models using the `DeriveModelForm` macro.

#### Basic Model Form

```rust
use sea_orm::entity::prelude::*;
use rusti::DeriveModelForm;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveModelForm)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,                    // Automatically excluded
    pub username: String,           // CharField
    pub email: String,              // EmailField (detected by name)
    pub bio: Option<String>,        // Optional CharField
    pub age: i32,                   // IntegerField
    pub is_active: bool,            // BooleanField
    pub created_at: DateTime,       // Automatically excluded
}

// This generates automatically:
// - UserForm struct
// - FormulaireTrait implementation
// - to_active_model() method
// - save() method
```

#### Automatic Field Detection

The macro automatically infers field types:

| Model Field Type | Detected As | Notes |
|------------------|-------------|-------|
| `String` | `CharField` | Default text field |
| Field named `*email*` | `EmailField` | By name detection |
| Field named `*password*` | `PasswordField` | By name detection |
| Field named `*slug*` | `SlugField` | By name detection |
| `i32`, `i64` | `IntegerField` | Integer numbers |
| `f32`, `f64` | `FloatField` | Decimal numbers |
| `bool` | `BooleanField` | Boolean |
| `NaiveDate` | `DateField` | Date only |
| `DateTime` | `DateTimeField` | Date and time |
| `Option<T>` | `optional()` | Nullable fields |

#### Automatic Exclusions

These fields are **automatically excluded** from the form:

- `id` field
- `created_at` field
- `updated_at` field
- Fields with `#[sea_orm(primary_key)]`

#### Usage in Handler

```rust
use rusti::prelude::*;

pub async fn create_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    ExtractForm(form): ExtractForm<UserForm>,  // Auto-generated
    mut message: Message,
) -> Response {
    // Validation happens automatically
    if !form.is_valid() {
        let ctx = context! {
            "form", &form
        };
        return template.render("users/create.html", &ctx);
    }

    // Method 1: Direct save
    match form.save(&db).await {
        Ok(user) => {
            message.success(&format!("User {} created!", user.username)).await.ok();
            redirect("/users")
        }
        Err(e) => {
            message.error("Error creating user").await.ok();
            let ctx = context! {
                "form", &form ;
                "error", e.to_string()
            };
            template.render("users/create.html", &ctx)
        }
    }
}
```

#### Advanced: to_active_model()

For more control, use `to_active_model()`:

```rust
pub async fn create_user_advanced(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    ExtractForm(form): ExtractForm<UserForm>,
) -> Response {
    if form.is_valid() {
        use sea_orm::ActiveValue::Set;

        // Get the ActiveModel
        let mut user = form.to_active_model();

        // Add custom fields
        user.created_at = Set(chrono::Utc::now());
        user.updated_at = Set(chrono::Utc::now());

        // Insert manually
        match user.insert(&*db).await {
            Ok(inserted) => {
                // Process...
            }
            Err(e) => {
                // Handle error...
            }
        }
    }
}
```

#### Comparison: Django ModelForm vs Rusti DeriveModelForm

**Django - Manual approach (2 files required):**

```python
# models.py
class User(models.Model):
    username = models.CharField(max_length=100)
    email = models.EmailField()
    bio = models.TextField(blank=True)
    age = models.IntegerField()

# forms.py - SEPARATE FILE REQUIRED
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

**Rusti - Automatic approach (1 file only):**

```rust
// models.rs - ONLY FILE NEEDED!
#[derive(DeriveModelForm, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    pub id: i32,          // Auto-excluded
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
    pub age: i32,
}
// UserForm is automatically generated!

// handlers.rs
let mut form = UserForm::new();
if form.validate(&raw_data) {
    let user = form.save(&db).await?;
}
```

**Key Advantages:**

1. **No forms.py file** - One less file to maintain
2. **No duplication** - Model is the single source of truth
3. **Auto-sync** - Form updates automatically when model changes
4. **Zero boilerplate** - No Meta class, no field list
5. **Compile-time safety** - Rust catches errors before runtime

#### Custom Validation with Model Forms

You can add custom validation to model forms:

```rust
impl UserForm {
    fn clean(&mut self, raw_data: &HashMap<String, String>) {
        if self.is_not_valid() {
            return;
        }

        // Custom validation
        let username: Option<String> = self.get_value("username");
        if let Some(user) = username {
            if user.len() < 3 {
                self.errors.insert(
                    "username".to_string(),
                    "Username must be at least 3 characters".to_string()
                );
            }
        }
    }
}

// Use it in validate()
pub async fn register(
    ExtractForm(mut form): ExtractForm<UserForm>,
) -> Response {
    // Standard validation
    form.validate(&raw_data);

    // Add custom validation
    form.clean(&raw_data);

    if form.is_valid() {
        // Process...
    }
}
```

#### Benefits of DeriveModelForm

- **Zero boilerplate** - Automatic field detection
- **No separate forms file** - Unlike Django, no forms.py needed
- **Type-safe** - Compile-time validation
- **DRY principle** - Single source of truth (the model)
- **Django-like** - Familiar `.save()` method
- **Smart defaults** - Excludes id, timestamps automatically
- **Extensible** - Add custom validation easily
- **Auto-sync** - Forms update when models change (no manual sync needed)

---

## 5. Django to Rusti Comparison

To facilitate the transition from Django, here are the equivalences between the two frameworks:

| Django | Rusti | Notes |
|--------|-------|-------|
| `forms.CharField(max_length=100)` | `CharField { allow_blank: false }` | No max_length in Rust |
| `forms.EmailField()` | `EmailField` | RFC 5322 validation |
| `forms.ModelForm` | `#[derive(DeriveModelForm)]` | Auto-generates form from model |
| `form.is_valid()` | `self.is_valid()` | Via Deref |
| `form.cleaned_data["email"]` | `self.get_value::<String>("email")` | Type-safe |
| `form.errors["email"]` | `self.errors.get("email")` | Standard HashMap |
| `def clean(self):` | `fn clean(&mut self, data: &HashMap)` | Cross-field validation |
| `raise ValidationError("msg")` | `self.errors.insert(field, msg)` | Manual error addition |
| `form.save()` | `form.save(&db).await` | Async save to database |

---

## 6. API Reference

### Core Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| **require** | `(&mut self, name: &str, field: &impl FieldTrait, data: &HashMap)` | Validates a required field |
| **optional** | `(&mut self, name: &str, field: &impl FieldTrait, data: &HashMap)` | Validates an optional field |
| **field** | `(&mut self, name: &str, field: &impl FieldTrait, value: &str) -> Option<String>` | Validates a raw value directly |
| **is_valid** | `(&self) -> bool` | Returns true if no errors |
| **is_not_valid** | `(&self) -> bool` | Returns true if there are errors |
| **get_value** | `<T>(&self, name: &str) -> Option<T>` | Retrieves a typed validated value |
| **clear** | `(&mut self)` | Resets errors and cleaned_data |

### Error Management

| Method | Description |
|--------|-------------|
| **errors.insert(name, msg)** | Adds a custom error |
| **errors.get(name)** | Retrieves a field's error |
| **errors.is_empty()** | Check if there are no errors |

### Data Access

| Method | Description |
|--------|-------------|
| **cleaned_data.get(name)** | Retrieves the validated raw value |
| **cleaned_data.contains_key(name)** | Check if field was validated |

---

## 7. Common Pitfalls

### 7.1 Redundant use of self.form

**Don't do:**

```rust
// INCORRECT - Redundant with Deref
self.form.require("email", &EmailField, raw_data);
self.form.is_valid()
```

**Do:**

```rust
// CORRECT - Direct thanks to #[rusti_form] macro
self.require("email", &EmailField, raw_data);
self.is_valid()
```

### 7.2 Type mismatch on get_value

**Don't do:**

```rust
// INCORRECT - Type mismatch
let age: String = self.get_value("age").unwrap();  // age is an IntegerField!
```

**Do:**

```rust
// CORRECT - Type matching the field
let age: i64 = self.get_value("age").unwrap();     // IntegerField -> i64
let email: String = self.get_value("email").unwrap();  // EmailField -> String
```

### 7.3 Forgetting to check is_valid() before get_value()

**Don't do:**

```rust
// INCORRECT - Can panic if validation failed
let email: String = form.get_value("email").unwrap();  // unwrap() can fail!
```

**Do:**

```rust
// CORRECT - Always check first
if form.is_valid() {
    let email: String = form.get_value("email").unwrap();  // Safe here
} else {
    // Handle errors
}
```

### 7.4 Forgetting the #[rusti_form] macro

**Don't do:**

```rust
// INCORRECT - No macro
#[derive(Serialize, Deserialize, Debug)]
pub struct UserForm {
    #[serde(flatten)]  // Must be added manually
    pub form: Forms,
}

// And you need to implement Deref manually...
```

**Do:**

```rust
// CORRECT - Macro does everything automatically
#[rusti_form]
pub struct UserForm {
    pub form: Forms,
}

// Deref + DerefMut + #[serde(flatten)] added automatically!
```

---

## 8. Built-in Security

| Feature | Description | Affected Fields |
|---------|-------------|-----------------|
| **XSS Protection** | Automatic removal of `<script>`, `<iframe>`, `<object>` tags and JavaScript attributes (`onclick`, `onerror`, etc.) | CharField, TextField |
| **Argon2id Hash** | Secure hash with unique salt, resistant to GPU/ASIC attacks. PHC standard format. | PasswordField |
| **Automatic Trim** | Removal of leading and trailing whitespace | All text fields |
| **Format Validation** | Strict format verification (RFC 5322 email, URL, IP, JSON) | EmailField, URLField, IPAddressField, JSONField |

### Security Best Practices

```rust
// Good - Password hashed automatically
self.require("password", &PasswordField, raw_data);
let hashed: String = self.get_value("password").unwrap();
// hashed is already in Argon2id format

// Good - XSS protection automatic
self.require("comment", &TextField, raw_data);
let safe_comment: String = self.get_value("comment").unwrap();
// <script> tags have been removed

// Bad - Don't hash passwords manually
let raw_password = raw_data.get("password").unwrap();
// Never store raw passwords!
```

---

## 9. HTML Templates

### Displaying errors in Tera templates

```html
<!-- Form with errors -->
<form method="post" action='{% link "contact_submit" %}'>
    {% csrf %}

    <div class="form-group">
        <label for="name">Name:</label>
        <input type="text" name="name" id="name"
               value="{{ form.cleaned_data.name | default(value='') }}"
               class="{% if form.errors.name %}error{% endif %}">

        {% if form.errors.name %}
            <span class="error-message">{{ form.errors.name }}</span>
        {% endif %}
    </div>

    <div class="form-group">
        <label for="email">Email:</label>
        <input type="email" name="email" id="email"
               value="{{ form.cleaned_data.email | default(value='') }}"
               class="{% if form.errors.email %}error{% endif %}">

        {% if form.errors.email %}
            <span class="error-message">{{ form.errors.email }}</span>
        {% endif %}
    </div>

    <div class="form-group">
        <label for="message">Message:</label>
        <textarea name="message" id="message"
                  class="{% if form.errors.message %}error{% endif %}">{{ form.cleaned_data.message | default(value='') }}</textarea>

        {% if form.errors.message %}
            <span class="error-message">{{ form.errors.message }}</span>
        {% endif %}
    </div>

    <!-- Display all errors at the top -->
    {% if form.errors %}
        <div class="alert alert-danger">
            <h4>Please correct the following errors:</h4>
            <ul>
            {% for field, message in form.errors %}
                <li><strong>{{ field }}</strong>: {{ message }}</li>
            {% endfor %}
            </ul>
        </div>
    {% endif %}

    <button type="submit" class="btn btn-primary">Submit</button>
</form>
```

### CSS Styling

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

## 10. FAQ (Frequently Asked Questions)

### Q1: How to validate multiple fields together?

Use a `clean()` method like in Django:

```rust
impl MyForm {
    fn clean(&mut self, raw_data: &HashMap<String, String>) {
        if self.is_not_valid() {
            return;  // Skip if already errors
        }

        let password: Option<String> = self.get_value("password");
        let confirm: Option<String> = self.get_value("password_confirm");

        if password != confirm {
            self.errors.insert(
                "password_confirm".to_string(),
                "Passwords do not match".to_string()
            );
        }
    }
}
```

### Q2: Difference between require() and optional()?

**require()**: Field MUST be present and non-empty. Adds a "Required" error if absent.

**optional()**: Field can be absent or empty. No error if absent, but validation applied if present.

```rust
// require() - Required field
self.require("email", &EmailField, raw_data);
// If absent or empty -> "Required" error

// optional() - Optional field
self.optional("phone", &CharField { allow_blank: false }, raw_data);
// If absent -> No error
// If present -> Validation applied
```

### Q3: How to reuse custom validators?

Create a reusable helper function:

```rust
fn validate_strong_password(form: &mut Forms, field_name: &str, value: &str) {
    let regex = Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$").unwrap();

    if !regex.is_match(value).unwrap_or(false) {
        form.errors.insert(
            field_name.to_string(),
            "Weak password: must contain uppercase, lowercase and digit".to_string()
        );
    }
}

// Usage in multiple forms
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

### Q4: What exactly does the #[rusti_form] macro do?

The macro automatically generates:

1. **#[derive(Serialize, Deserialize, Debug)]** if not already present
2. **#[serde(flatten)]** on the Forms field
3. **impl Deref** to directly access Forms methods
4. **impl DerefMut** for mutable methods

This eliminates boilerplate and makes forms easier to use!

### Q5: Can you have multiple Forms fields in a struct?

**No.** The #[rusti_form] macro requires exactly ONE Forms field. If you need nested forms, create separate structs and combine them manually:

```rust
#[rusti_form]
pub struct AddressForm {
    pub form: Forms,
}

#[rusti_form]
pub struct UserForm {
    pub form: Forms,
}

// Combine manually in your handler
pub async fn register(
    ExtractForm(user_form): ExtractForm<UserForm>,
    ExtractForm(address_form): ExtractForm<AddressForm>,
) -> Response {
    if user_form.is_valid() && address_form.is_valid() {
        // Process both forms
    }
}
```

### Q6: How to handle file uploads?

For file uploads, use Axum's multipart support alongside forms:

```rust
use axum::extract::Multipart;

pub async fn upload_profile(
    mut multipart: Multipart,
    ExtractForm(form): ExtractForm<ProfileForm>,
) -> Response {
    if !form.is_valid() {
        // Handle form errors
        return template.render("profile.html", &context! { "form", &form });
    }

    // Process file upload
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        if name == "avatar" {
            // Save avatar file
        }
    }

    // Continue processing...
}
```

### Q7: When to use #[rusti_form] vs #[derive(DeriveModelForm)]?

**Use `#[rusti_form]`** when:
- You need a custom form not tied to a model
- You want full control over field types
- Form doesn't match database structure exactly
- Example: Contact form, search form, login form

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

**Use `#[derive(DeriveModelForm)]`** when:
- Form directly maps to a database model
- You want automatic field detection
- You need `.save()` functionality
- Following DRY principle (model as single source of truth)
- Example: User registration, product creation, profile edit

```rust
#[derive(DeriveModelForm, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    pub id: i32,
    pub username: String,
    pub email: String,
}

// That's it! UserForm is auto-generated with validate() and save()
```

**Comparison:**

| Feature | #[rusti_form] | #[derive(DeriveModelForm)] |
|---------|---------------|----------------------------|
| Manual field definition | Yes | No - Auto-detected |
| Custom validation | Yes | Yes - Via clean() |
| .save() method | No - Manual | Yes - Automatic |
| Tied to model | No | Yes |
| Best for | Custom forms | Model-based CRUD |

---

## 11. Alphabetical Method Index

| Method | Page | Category |
|--------|------|----------|
| **clean()** | 4.3, 10 | Cross-field validation |
| **clear()** | 6 | Reset |
| **field()** | 6 | Direct validation |
| **get_value<T>()** | 4, 6 | Data retrieval |
| **is_not_valid()** | 6 | Verification |
| **is_valid()** | 4, 6 | Verification |
| **new()** | 4 | Construction |
| **optional()** | 6, 10 | Field validation |
| **require()** | 4, 6, 10 | Field validation |
| **validate()** | 4 | FormulaireTrait trait |

---

## Conclusion

Rusti Forms offers a powerful and secure validation system, combining Django's familiarity with Rust's robustness. The **#[rusti_form]** macro significantly reduces boilerplate code, allowing developers to focus on business logic. Security is built-in by default with XSS sanitization, Argon2 hashing, and strict format validation.

### Key Takeaways

- **Django-inspired** - Familiar API for easy transition
- **Type-safe** - Rust's type system prevents runtime errors
- **Secure by default** - XSS protection, password hashing included
- **Zero boilerplate** - `#[rusti_form]` macro does the heavy lifting
- **Production-ready** - Used in real-world Rusti applications

### Next Steps

- Read the [Rusti Framework Documentation](../README.md)
- Check out the [Getting Started Guide](GETTING_STARTED.md)
- Explore [Template Documentation](TEMPLATES.md)
- Learn about [Database Integration](DATABASE.md)

---

**Rusti Forms v2.0**
Secure · Type-safe · Performant

Documentation - December 2025
Developed with love in Rust by Itsuki
