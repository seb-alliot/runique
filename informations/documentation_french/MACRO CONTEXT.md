# Guide d'utilisation de la macro context!

**Version:** 1.0
**Framework:** Rusti
**Langue:** Français

---

## Introduction

La macro `context!` simplifie la création de contextes Tera pour vos templates, de manière similaire à Django en Python.

---

## Installation

```rust
use rusti::context;
```

---

## Deux méthodes d'utilisation

### Méthode 1 : Macro avec point-virgule (Recommandée)

**Syntaxe :**
```rust
let ctx = context! {
    "clé", valeur ;
    "clé2", valeur2
};
```

**Caractéristiques :**
- Concis et lisible
- Syntaxe proche de Python/Django
- Toutes les clés visibles d'un coup
- Parfait pour les contextes simples

**Exemples :**

```rust
// Contexte simple
let ctx = context! {
    "title", "Bienvenue"
};

// Contexte multiple
let ctx = context! {
    "title", "Mon Profil" ;
    "username", "Alice" ;
    "age", 25
};

// Avec variables
let form = UserForm::new();
let error_msg = "Erreur !";

let ctx = context! {
    "form", &form ;
    "error", error_msg ;
    "show_help", true
};

// Point-virgule final optionnel
let ctx = context! {
    "title", "Mon titre" ;
    "count", 42
};
```

---

### Méthode 2 : Chaînage avec .add()

**Syntaxe :**
```rust
let ctx = context!()
    .add("clé", valeur)
    .add("clé2", valeur2);
```

**Caractéristiques :**
- Flexible et extensible
- Permet la construction progressive
- Idéal pour les contextes conditionnels

**Exemples :**

```rust
// Contexte simple
let ctx = context!()
    .add("title", "Bienvenue");

// Contexte multiple
let ctx = context!()
    .add("title", "Mon Profil")
    .add("username", "Alice")
    .add("age", 25);

// Avec variables
let form = UserForm::new();
let error_msg = "Erreur !";

let ctx = context!()
    .add("form", &form)
    .add("error", error_msg)
    .add("show_help", true);
```

---

## Utilisation dans les handlers

### Exemple complet avec formulaire

```rust
use rusti::{context, Template, Response, ExtractForm};

pub async fn user_profile_submit(
    template: Template,
    ExtractForm(form): ExtractForm<UserForm>,
) -> Response {
    if !form.is_valid() {
        // Créer un contexte avec le formulaire et les erreurs
        let ctx = context! {
            "form", &form ;
            "title", "Erreur de validation"
        };

        return template.render("profile.html", &ctx);
    }

    // Traiter le formulaire valide...
    let ctx = context! {
        "success", true ;
        "message", "Profil mis à jour !"
    };

    template.render("success.html", &ctx)
}
```

### Exemple avec gestion d'erreurs DB

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
                message.success("Utilisateur créé !").await.ok();
                return Redirect::to("/success").into_response();
            }
            Err(e) => {
                message.error("Erreur lors de la création").await.ok();

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
        "title", "Créer un utilisateur"
    };

    template.render("form.html", &ctx)
}
```

---

## Erreurs courantes

### Erreur 1 : Utiliser deux-points au lieu de virgule

```rust
// FAUX
let ctx = context! {
    "title": "Titre",
};

// CORRECT
let ctx = context! {
    "title", "Titre"
};
```

### Erreur 2 : Oublier le & pour render

```rust
// FAUX
template.render("page.html", ctx)

// CORRECT
template.render("page.html", &ctx)
```

### Erreur 3 : Mauvais nom de variable

```rust
// FAUX
let contexte = context! {
    "title", "Titre"
};

template.render("page.html", &context)  // contexte != context

// CORRECT
let ctx = context! {
    "title", "Titre"
};

template.render("page.html", &ctx)
```

---

## Tableau de comparaison

| Critère | Macro ; | Chaînage .add() |
|---------|---------|-----------------|
| Flexibilité | Moyenne | Haute |
| Lisibilité | Excellente | Bonne |
| Simplicité | Très simple | Simple |
| Construction progressive | Non | Oui |
| Cas d'usage | Contextes simples | Contextes conditionnels |

---

## Recommandations

**Pour débuter :** Utilisez la macro avec point-virgule

```rust
let ctx = context! {
    "key", "value"
};
```

**Pour les contextes conditionnels :** Utilisez le chaînage

```rust
let mut ctx = context!()
    .add("key", "value");

if condition {
    ctx = ctx.add("extra", "data");
}
```

---

## Comparaison avec Django

### Django (Python)
```python
context = {
    'form': form,
    'title': 'Mon titre',
    'count': 42,
}
return render(request, 'page.html', context)
```

### Rusti (Rust) - Méthode 1
```rust
let ctx = context! {
    "form", &form ;
    "title", "Mon titre" ;
    "count", 42
};

template.render("page.html", &ctx)
```

### Rusti (Rust) - Méthode 2
```rust
let ctx = context!()
    .add("form", &form)
    .add("title", "Mon titre")
    .add("count", 42);

template.render("page.html", &ctx)
```

---

## Aide-mémoire rapide

```rust
// Créer un contexte vide
let ctx = context!();

// Une clé (macro)
let ctx = context! {
    "key", "value"
};

// Plusieurs clés (macro)
let ctx = context! {
    "key1", "value1" ;
    "key2", "value2"
};

// Une clé (chaînage)
let ctx = context!()
    .add("key", "value");

// Plusieurs clés (chaînage)
let ctx = context!()
    .add("key1", "value1")
    .add("key2", "value2");

// Utilisation
template.render("page.html", &ctx)
```

---

## Exemples avancés

### Construction conditionnelle

```rust
pub async fn show_profile(
    template: Template,
    user: User,
    is_admin: bool,
) -> Response {
    let mut ctx = context!()
        .add("user", &user)
        .add("title", "Profile");

    // Ajout conditionnel
    if is_admin {
        ctx = ctx.add("admin_panel", true)
                 .add("permissions", &get_permissions());
    }

    template.render("profile.html", &ctx)
}
```

### Avec update()

```rust
use serde_json::json;

let ctx = context!()
    .add("title", "Dashboard");

// Ajouter plusieurs clés d'un coup
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

## Notes techniques

### Type système

```rust
// ContextHelper implémente Deref vers Context

let ctx = context!()  // Type: ContextHelper
    .add("key", "value");

template.render("page.html", &ctx)  // &ContextHelper -> &Context (via Deref)
```

### Sérialisation

```rust
// Toute valeur qui implémente Serialize peut être ajoutée

struct User {
    name: String,
    age: u32,
}

let user = User { name: "Alice".into(), age: 25 };

let ctx = context! {
    "user", &user  // Fonctionne si User: Serialize
};
```

---

## Ressources

- **Documentation Rusti:** rusti.dev (à venir)
- **Exemples:** rusti/examples/demo-app
- **Issues:** GitHub Issues

---

**Version:** 1.0
**Dernière mise à jour:** Décembre 2025
**Licence:** MIT
