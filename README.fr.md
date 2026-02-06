# 🚀 Runique - Framework Web Rust inspiré de Django

> **⚠️ Note** : Cette documentation a été générée avec l'assistance de l'IA. Bien que des efforts aient été faits pour assurer l'exactitude, certains liens ou détails peuvent contenir des erreurs. Veuillez signaler les problèmes sur [GitHub](https://github.com/seb-alliot/runique/issues).

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)]()
[![Tests](https://img.shields.io/badge/tests-36%2F36%20passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-green)]()
[![Version](https://img.shields.io/badge/version-1.1.1-blue)]()
[![Crates.io](https://img.shields.io/crates/v/runique)]()

Un framework web Rust moderne et complet, inspiré par Django, pour construire des applications web robustes et performantes.

🌍 **Langues** : [English](README.md) | [🇫🇷 Français](#-installation)

## 📚 Table des matières

- 🚀 [Installation](#-installation)
- 🏗️ [Architecture](#️-architecture)
- ⚙️ [Configuration](#️-configuration)
- 🛣️ [Routage](#️-routage)
- 📝 [Formulaires](#-formulaires)
- 🎨 [Templates](#-templates)
- 🗄️ [ORM](#️-orm)
- 🔒 [Middlewares](#-middlewares)
- 💬 [Flash Messages](#-flash-messages)
- 🎓 [Exemples](#-exemples)

---

## 🚀 Installation

**Documentation complète** : [Guide d'installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/01-installation.md)

Démarrage rapide :

```bash
git clone https://github.com/seb-alliot/runique
cd runique
cargo build
cargo test --all
```

👉 **Lire** : [docs/fr/01-installation.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/01-installation.md) pour les détails complets

---

## 🏗️ Architecture

**Documentation complète** : [Guide d'architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/02-architecture.md)

Vue d'ensemble de l'architecture Runique :

```
Runique Framework
├── Forms System      # Formulaires type-safe
├── Routing Engine    # Routage URL patterns
├── Template Engine   # Templates Tera
├── Middleware Stack  # Sécurité & en-têtes
├── ORM Layer         # Intégration SeaORM
└── Utils             # Utilitaires et helpers
```

👉 **Lire** : [docs/fr/02-architecture.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/02-architecture.md) pour la structure interne

---

## ⚙️ Configuration

**Documentation complète** : [Guide de configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/03-configuration.md)

Configurer votre serveur et application :

```rust
let settings = Settings {
    server: ServerConfig { ... },
    database: DatabaseConfig { ... },
    security: SecurityConfig { ... },
};
```

👉 **Lire** : [docs/fr/03-configuration.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/03-configuration.md) pour toutes les options

---

## 🛣️ Routage

**Documentation complète** : [Guide de routage](https://github.com/seb-alliot/runique/blob/main/docs/fr/04-routing.md)

Définir vos routes avec la macro `urlpatterns!` :

```rust
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view}; // Macros explicites

pub fn routes() -> Router {
    let router = urlpatterns! {
        "/" => view!{ GET => views::index }, name = "index",

        "/about" => view! { GET => views::about }, name = "about",
        "/inscription" => view! { GET => views::inscription, POST => views::soumission_inscription }, name = "inscription",
    };
    router
}


pub async fn inscription(mut template: TemplateContext) -> AppResult<Response> {
    let form = template.form::<RegisterForm>();
    context_update!(template => {
        "title" => "Inscription user",
        "inscription_form" => &form,
    });

    template.render("inscription_form.html")
}

// Handle form submission
async fn soumission_inscription(
    Prisme(mut form): Prisme<UserForm>,
    mut template: TemplateContext,
) -> AppResult<Response> {
    if form.is_valid().await {
    }
    context_update!(template => {
        "form" => form,
    });
    template.render("register.html")
}
```

👉 **Lire** : [docs/fr/04-routing.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/04-routing.md) pour les patterns et options

---

## 📝 Formulaires

**Documentation complète** : [Guide des formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/05-forms.md)

Créer des formulaires facilement :

```rust
let mut form = Forms::new("csrf_token");

form.field(&TextField::text("username")
    .label("Nom d'utilisateur")
    .required());

form.field(&TextField::email("email")
    .label("Email"));
```

👉 **Lire** : [docs/fr/05-forms.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/05-forms.md) pour tous les types de champs

---

## 🎨 Templates

**Documentation complète** : [Guide des templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/06-templates.md)

Utiliser les templates Tera :

```html
<h1>{{ title }}</h1>
{% for item in items %}
  <p>{{ item }}</p>
{% endfor %}
```

👉 **Lire** : [docs/fr/06-templates.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/06-templates.md) pour la syntaxe complète

---

## 🗄️ ORM

**Documentation complète** : [Guide ORM](https://github.com/seb-alliot/runique/blob/main/docs/fr/07-orm.md)

Utiliser SeaORM avec pattern Django-like :

```rust
impl_objects!(User);

let users = User::objects
    .filter(active.eq(true))
    .all(&db)
    .await?;
```

👉 **Lire** : [docs/fr/07-orm.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/07-orm.md) pour les queries avancées

---

## 🔒 Middlewares

**Documentation complète** : [Guide des middlewares](https://github.com/seb-alliot/runique/blob/main/docs/fr/08-middleware.md)

Middlewares de sécurité intégrés :

- Protection CSRF
- Content-Security-Policy (CSP)
- Allowed Hosts
- En-têtes de sécurité
- Sanitizer XSS

👉 **Lire** : [docs/fr/08-middleware.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/08-middleware.md) pour la configuration

---

## 💬 Flash Messages

**Documentation complète** : [Guide Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/fr/09-flash-messages.md)

Messages temporaires pour l'utilisateur :

```rust
success!("Opération réussie !");
error!("Une erreur s'est produite");
warning!("Attention !");
```

👉 **Lire** : [docs/fr/09-flash-messages.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/09-flash-messages.md) pour les détails

---

## 🎓 Exemples

**Documentation complète** : [Guide des exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/10-examples.md)

Exemples complets d'utilisation :

- Application blog complète
- Authentification utilisateur
- Upload de fichiers
- API REST

👉 **Lire** : [docs/fr/10-examples.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/10-examples.md) pour les exemples complets

---

## 🧪 Tests

```bash
# Tests unitaires
cargo test --lib

# Tests d'intégration
cargo test --test integration_tests

# Tous les tests
cargo test --all
```

Résultats : **36/36 tests passent** ✅

---

## 📖 Documentation complète

### English (EN)
- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md)
- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md)
- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md)
- [Routage](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md)
- [Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md)
- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/06-templates.md)
- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md)
- [Middlewares](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md)
- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/en/09-flash-messages.md)
- [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md)

### Français (FR)
- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/01-installation.md)
- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/02-architecture.md)
- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/03-configuration.md)
- [Routage](https://github.com/seb-alliot/runique/blob/main/docs/fr/04-routing.md)
- [Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/05-forms.md)
- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/06-templates.md)
- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/fr/07-orm.md)
- [Middlewares](https://github.com/seb-alliot/runique/blob/main/docs/fr/08-middleware.md)
- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/fr/09-flash-messages.md)
- [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/10-examples.md)

---

## 🎯 Démarrage rapide

1. **Lire** [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/01-installation.md)
2. **Comprendre** [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/02-architecture.md)
3. **Consulter** [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/10-examples.md)
4. **Coder** votre application

---

## 📊 État du projet

- ✅ **Compilation** : Sans erreurs
- ✅ **Tests** : 36/36 passent (100%)
- ✅ **Documentation** : Complète (EN & FR)
- ✅ **Production** : Prêt

Voir [PROJECT_STATUS.md](PROJECT_STATUS.md) pour plus de détails.

---

## 🔗 Ressources

- 📁 [Structure du projet](INDEX.md)
- 📊 [État complet](PROJECT_STATUS.md)
- 🧪 [Rapports de tests](TEST_REPORT.md)
- 📋 [Changelog](CHANGELOG.md)
- 📖 [Guide de documentation](README.md)

---

## 📝 Licence

MIT License - voir [SECURITY.md](SECURITY.md)

---

## 🚀 Prêt pour la production

Le framework Runique est **stable, testé et documenté**, prêt pour une utilisation en production.


**Démarrer maintenant** → [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/01-installation.md)

---

🌍 **Disponible en** : [English](README.md) | [🇫🇷 Français](#)