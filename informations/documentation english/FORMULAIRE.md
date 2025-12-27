# Rusti Forms Documentation

**Django-like Form System for Rust**

Version 1.0 - Complete Documentation

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

### Key Features

- Automatic validation with predefined field types
- Built-in XSS sanitization
- Automatic whitespace trimming
- Secure password hashing (Argon2)
- Familiar Django-like API
- Type-safe thanks to Rust

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
| CharField | Short text field | XSS sanitization, trim |
| TextField | Long text field | XSS sanitization, trim |
| EmailField | Email | RFC 5322 format |
| PasswordField | Password | Argon2id hash |
| IntegerField | Integer | Parse to i64 |
| FloatField | Decimal number | Parse to f64 |
| BooleanField | Boolean | true/false, 1/0, on/off |
| DateField | Date | YYYY-MM-DD format |
| SlugField | URL-friendly slug | Letters, numbers, hyphens |
| URLField | URL | Valid URL format |
| IPAddressField | IP Address | IPv4 or IPv6 |
| JSONField | JSON data | Valid JSON parse |

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
pub async fn contact_submit(
    RustiForm(form): ExtractForm<ContactForm>,
) -> Response {
    if form.is_valid() {
        let name: String = form.get_value("name").unwrap();
        let email: String = form.get_value("email").unwrap();
        // Process data...
    } else {
        // Display errors
        let mut context = Context::new();
        context.insert("form", &form);
        template.render("contact.html", &context)
    }
}
```

### 4.2 Custom Validation

Registration form with password validation:

```rust
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
        use fancy_regex::Regex;

        let regex = Regex::new(
            r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[A-Za-z\d]{8,}$"
        ).unwrap();

        if raw_value.len() < 8 {
            self.errors.insert(
                "password".to_string(),
                "Minimum 8 characters".to_string()
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

Consistency check between multiple fields (Django-style clean()):

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

---

## 5. Django to Rusti Comparison

To facilitate the transition from Django, here are the equivalences between the two frameworks:

| Django | Rusti | Notes |
|--------|-------|-------|
| forms.CharField(max_length=100) | CharField { allow_blank: false } | No max_length in Rust |
| forms.EmailField() | EmailField | RFC 5322 validation |
| form.is_valid() | self.is_valid() | Via Deref |
| form.cleaned_data["email"] | self.get_value::<String>("email") | Type-safe |
| form.errors["email"] | self.errors.get("email") | Standard HashMap |
| def clean(self): | fn clean(&mut self, data: &HashMap) | Cross-field validation |
| raise ValidationError("msg") | self.errors.insert(field, msg) | Manual error addition |

---

## 6. API Reference

| Method | Description |
|--------|-------------|
| require(name, field, data) | Validates a required field |
| optional(name, field, data) | Validates an optional field |
| field(name, field, value) | Validates a raw value directly |
| is_valid() | Returns true if no errors |
| is_not_valid() | Returns true if there are errors |
| get_value<T>(name) | Retrieves a typed validated value |
| errors.insert(name, msg) | Adds a custom error |
| errors.get(name) | Retrieves a field's error |
| cleaned_data.get(name) | Retrieves the validated raw value |
| clear() | Resets errors and cleaned_data |

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
| XSS Protection | Automatic removal of script, iframe, object tags and JavaScript attributes (onclick, onerror, etc.) | CharField, TextField |
| Argon2id Hash | Secure hash with unique salt, resistant to GPU/ASIC attacks. PHC standard format. | PasswordField |
| Automatic Trim | Removal of leading and trailing whitespace | All text fields |
| Format Validation | Strict format verification (RFC 5322 email, URL, IP, JSON) | EmailField, URLField, IPAddressField, JSONField |

---

## 9. HTML Templates

### Displaying errors in Tera templates

```html
<!-- Form with errors -->
<form method="post">
    <div class="form-group">
        <label for="email">Email:</label>
        <input type="email" name="email" id="email"
               value="{{ form.cleaned_data.email | default(value="") }}">

        {% if form.errors.email %}
            <span class="error">{{ form.errors.email }}</span>
        {% endif %}
    </div>

    <div class="form-group">
        <label for="password">Password:</label>
        <input type="password" name="password" id="password">

        {% if form.errors.password %}
            <span class="error">{{ form.errors.password }}</span>
        {% endif %}
    </div>

    <!-- Display all errors at the top -->
    {% if form.errors %}
        <div class="alert alert-danger">
            <ul>
            {% for field, message in form.errors %}
                <li><strong>{{ field }}</strong>: {{ message }}</li>
            {% endfor %}
            </ul>
        </div>
    {% endif %}

    <button type="submit">Submit</button>
</form>
```

---

## 10. FAQ (Frequently Asked Questions)

### Q1: How to validate multiple fields together?

Use a clean() method like in Django:

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
self.optional("phone", &CharField::new(), raw_data);
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
            "Weak password".to_string()
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

### Q5: Can you have multiple Forms fields in a struct?

**No.** The #[rusti_form] macro requires exactly ONE Forms field. If you need nested forms, create separate structs and combine them manually.

---

## 11. Alphabetical Method Index

| Method | Page | Category |
|--------|------|----------|
| clean() | 5, 11 | Cross-field validation |
| clear() | 8 | Reset |
| field() | 8 | Direct validation |
| get_value<T>() | 4, 8 | Data retrieval |
| is_not_valid() | 8 | Verification |
| is_valid() | 4, 8 | Verification |
| new() | 4 | Construction |
| optional() | 8, 11 | Field validation |
| require() | 4, 8, 11 | Field validation |
| validate() | 4 | FormulaireTrait trait |

---

## Conclusion

Rusti Forms offers a powerful and secure validation system, combining Django's familiarity with Rust's robustness. The **#[rusti_form]** macro significantly reduces boilerplate code, allowing developers to focus on business logic. Security is built-in by default with XSS sanitization, Argon2 hashing, and strict format validation.

For more information, consult the complete Rusti framework documentation and examples in the GitHub repository.

---

**Rusti Forms v2.0**
Secure - Type-safe - Performant

Documentation - December 2025
