# context! Macro Usage Guide

**Version:** 1.0
**Framework:** Rusti
**Language:** English

---

## Introduction

The `context!` macro simplifies the creation of Tera contexts for your templates, similar to Django in Python.

---

## Installation

```rust
use rusti::context;
```

---

## Two Usage Methods

### Method 1: Macro with Semicolon (Recommended)

**Syntax:**
```rust
let ctx = context! {
    "key", value ;
    "key2", value2
};
```

**Features:**
- Concise and readable
- Syntax similar to Python/Django
- All keys visible at once
- Perfect for simple contexts

**Examples:**

```rust
// Simple context
let ctx = context! {
    "title", "Welcome"
};

// Multiple context
let ctx = context! {
    "title", "My Profile" ;
    "username", "Alice" ;
    "age", 25
};

// With variables
let form = UserForm::new();
let error_msg = "Error!";

let ctx = context! {
    "form", &form ;
    "error", error_msg ;
    "show_help", true
};

// Trailing semicolon is optional
let ctx = context! {
    "title", "My title" ;
    "count", 42
};
```

---

### Method 2: Chaining with .add()

**Syntax:**
```rust
let ctx = context!()
    .add("key", value)
    .add("key2", value2);
```

**Features:**
- Flexible and extensible
- Allows progressive construction
- Ideal for conditional contexts

**Examples:**

```rust
// Simple context
let ctx = context!()
    .add("title", "Welcome");

// Multiple context
let ctx = context!()
    .add("title", "My Profile")
    .add("username", "Alice")
    .add("age", 25);

// With variables
let form = UserForm::new();
let error_msg = "Error!";

let ctx = context!()
    .add("form", &form)
    .add("error", error_msg)
    .add("show_help", true);
```

---

## Usage in Handlers

### Complete example with form

```rust
use rusti::{context, Template, Response, ExtractForm};

pub async fn user_profile_submit(
    template: Template,
    ExtractForm(form): ExtractForm<UserForm>,
) -> Response {
    if !form.is_valid() {
        // Create context with form and errors
        let ctx = context! {
            "form", &form ;
            "title", "Validation Error"
        };

        return template.render("profile.html", &ctx);
    }

    // Process valid form...
    let ctx = context! {
        "success", true ;
        "message", "Profile updated!"
    };

    template.render("success.html", &ctx)
}
```

### Example with database error handling

```rust
use rusti::{context, Template, Message, DatabaseConnection};
use rusti::axum::Extension;
use std::sync::Arc;

pub async fn create_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(form): ExtractForm<UserForm>,
) -> Response {
    if form.is_valid() {
        match form.save(&*db).await {
            Ok(user) => {
                message.success("User created!").await.ok();
                return Redirect::to("/success").into_response();
            }
            Err(e) => {
                message.error("Error during creation").await.ok();

                let ctx = context! {
                    "form", &form ;
                    "db_error", e.to_string()
                };

                return template.render("form.html", &ctx);
            }
        }
    }

    let ctx = context! {
        "form", &form ;
        "title", "Create User"
    };

    template.render("form.html", &ctx)
}
```

---

## Common Errors

### Error 1: Using colon instead of comma

```rust
// WRONG
let ctx = context! {
    "title": "Title",
};

// CORRECT
let ctx = context! {
    "title", "Title"
};
```

### Error 2: Forgetting & for render

```rust
// WRONG
template.render("page.html", ctx)

// CORRECT
template.render("page.html", &ctx)
```

### Error 3: Wrong variable name

```rust
// WRONG
let contexte = context! {
    "title", "Title"
};

template.render("page.html", &context)  // contexte != context

// CORRECT
let ctx = context! {
    "title", "Title"
};

template.render("page.html", &ctx)
```

---

## Comparison Table

| Criteria | Macro ; | Chaining .add() |
|----------|---------|-----------------|
| Flexibility | Medium | High |
| Readability | Excellent | Good |
| Simplicity | Very simple | Simple |
| Progressive construction | No | Yes |
| Use case | Simple contexts | Conditional contexts |

---

## Recommendations

**For beginners:** Use the macro with semicolon

```rust
let ctx = context! {
    "key", "value"
};
```

**For conditional contexts:** Use chaining

```rust
let mut ctx = context!()
    .add("key", "value");

if condition {
    ctx = ctx.add("extra", "data");
}
```

---

## Comparison with Django

### Django (Python)
```python
context = {
    'form': form,
    'title': 'My title',
    'count': 42,
}
return render(request, 'page.html', context)
```

### Rusti (Rust) - Method 1
```rust
let ctx = context! {
    "form", &form ;
    "title", "My title" ;
    "count", 42
};

template.render("page.html", &ctx)
```

### Rusti (Rust) - Method 2
```rust
let ctx = context!()
    .add("form", &form)
    .add("title", "My title")
    .add("count", 42);

template.render("page.html", &ctx)
```

---

## Quick Reference

```rust
// Create empty context
let ctx = context!();

// One key (macro)
let ctx = context! {
    "key", "value"
};

// Multiple keys (macro)
let ctx = context! {
    "key1", "value1" ;
    "key2", "value2"
};

// One key (chaining)
let ctx = context!()
    .add("key", "value");

// Multiple keys (chaining)
let ctx = context!()
    .add("key1", "value1")
    .add("key2", "value2");

// Usage
template.render("page.html", &ctx)
```

---

## Advanced Examples

### Conditional Construction

```rust
pub async fn show_profile(
    template: Template,
    user: User,
    is_admin: bool,
) -> Response {
    let mut ctx = context!()
        .add("user", &user)
        .add("title", "Profile");

    // Conditional addition
    if is_admin {
        ctx = ctx.add("admin_panel", true)
                 .add("permissions", &get_permissions());
    }

    template.render("profile.html", &ctx)
}
```

### With update()

```rust
use serde_json::json;

let ctx = context!()
    .add("title", "Dashboard");

// Add multiple keys at once
let ctx = ctx.update(json!({
    "stats": {
        "users": 100,
        "posts": 500,
    },
    "recent_activity": activity_list,
}));

template.render("dashboard.html", &ctx)
```

---

## Technical Notes

### Type System

```rust
// ContextHelper implements Deref to Context

let ctx = context!()  // Type: ContextHelper
    .add("key", "value");

template.render("page.html", &ctx)  // &ContextHelper -> &Context (via Deref)
```

### Serialization

```rust
// Any value implementing Serialize can be added

struct User {
    name: String,
    age: u32,
}

let user = User { name: "Alice".into(), age: 25 };

let ctx = context! {
    "user", &user  // Works if User: Serialize
};
```

---

## Resources

- **Rusti Documentation:** rusti.dev (coming soon)
- **Examples:** rusti/examples/demo-app
- **Issues:** GitHub Issues

---

**Version:** 1.0
**Last updated:** December 2025
**License:** MIT
