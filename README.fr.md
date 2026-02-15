# 🚀 Runique - Framework Web Rust inspiré de Django

> **⚠️ Note** :
Cette documentation a été générée avec l'assistance de l'IA.
Bien que des efforts aient été faits pour assurer l'exactitude, certains liens ou détails peuvent contenir des erreurs.
Veuillez signaler les problèmes sur [GitHub](https://github.com/seb-alliot/runique/issues).

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)]()
[![Tests](https://img.shields.io/badge/tests-36%2F36%20passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-green)]()
[![Version](https://img.shields.io/badge/version-1.1.25-blue)]()
[![Crates.io](https://img.shields.io/crates/v/runique)]()

Un framework web Rust moderne et complet, inspiré par Django, pour construire des applications web robustes et performantes.

🌍 **Langues** : [English](README.md) | [🇫🇷 Français](https://github.com/seb-alliot/runique/blob/main/README.fr.md)

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
- 🧭 [Admin-beta](#-admin-beta)

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
use runique::prelude::*;
use runique::{urlpatterns, view}; // <= Macros explicites

pub fn routes() -> Router {
    let router = urlpatterns! {
        "/" => view!{ views::index }, name = "index",

        "/inscription" => view! { views::inscription }, name = "inscription",
    };
    router
}

pub async fn inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_get() {
        context_update!(request => {
            "title" => "Inscription utilisateur",
            "inscription_form" => &form,
        });
        return request.render("inscription_form.html");
    }

    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;
            println!("Nouvel utilisateur créé donné dans views.rs : {:?}", user);

            success!(request.notices => format!("Bienvenue {}, votre compte est créé !", user.username));
            return Ok(Redirect::to("/").into_response());
        }

        // Validation échouée
        context_update!(request => {
            "title" => "Erreur de validation",
            "inscription_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render("inscription_form.html");
    }

    // Cas fallback
    request.render("inscription_form.html")
}
```

👉 **Lire** : [docs/fr/04-routing.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/04-routing.md) pour les patterns et options

---

## 📝 Formulaires

**Documentation complète** : [Guide des formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/05-forms.md)

Créer des formulaires facilement :

```rust
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct RegisterForm {
    pub form: Forms,
}

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Entrez votre nom d'utilisateur")
                .required(),
        );

        form.field(
            &TextField::email("email")
                .label("Entrez votre email")
                .required(),
        );

        form.field(
            &TextField::password("password")
                .label("Entrez un mot de passe")
                .required(),
        );
    }

    impl_form_access!();
}
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

## 🧭 Admin-beta
---

Runique intègre une **vue d’administration en version bêta**, basée sur une macro déclarative `admin!` et un système de génération automatique.

Les ressources administrables sont déclarées dans `src/admin.rs`.
À partir de cette déclaration, Runique génère automatiquement une interface CRUD complète (routes, handlers, formulaires) sous forme de **code Rust standard**, lisible et auditable.

Cette approche met l’accent sur :

* la **sécurité de typage** (vérification à la compilation des modèles et formulaires)
* la **transparence** (pas de logique cachée, pas de macro procédurale)
* le **contrôle développeur** sur le code généré

Le daemon (`runique start`) permet une régénération automatique, tandis qu’un workflow `cargo run` peut être utilisé lorsque des modifications manuelles sont nécessaires.

> ⚠️ La vue admin est actuellement en **bêta** et pose volontairement des bases simples, déclaratives et sûres. Des évolutions sont prévues (permissions plus fines, meilleur feedback, protections supplémentaires).

---
**Full Documentation** : [Examples Guide](https://github.com/seb-alliot/runique/blob/main/docs/fr/11-Admin.md)

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
- [Admin-beta](https://github.com/seb-alliot/runique/blob/main/docs/en/11-Admin.md)

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
- [Admin-beta](https://github.com/seb-alliot/runique/blob/main/docs/fr/11-Admin.md)
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

Voir [PROJECT_STATUS.md](https://github.com/seb-alliot/runique/blob/main/PROJECT_STATUS.md) pour plus de détails.

---

## 🔗 Ressources

- 📁 [Project Structure](https://github.com/seb-alliot/runique/blob/main/INDEX.md)
- 📊 [Full Status](https://github.com/seb-alliot/runique/blob/main/PROJECT_STATUS.md)
- 🧪 [Test Reports](https://github.com/seb-alliot/runique/blob/main/TEST_REPORT.md)
- 📋 [Changelog](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- 📖 [Documentation Guide](https://github.com/seb-alliot/runique/blob/main/README.fr.md)

---

## 📝 Licence

MIT License[LICENCE](https://github.com/seb-alliot/runique/blob/main/LICENCE) voir [SECURITY.md](https://github.com/seb-alliot/runique/blob/main/SECURITY.md)

---

## 🚀 Prêt pour la production

Le framework Runique est **stable, testé et documenté**, prêt pour une utilisation en production.


**Démarrer maintenant** → [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/01-installation.md)

---

🌍 **Disponible en** : [English](https://github.com/seb-alliot/runique/blob/main/README.md) | [🇫🇷 Français](#)