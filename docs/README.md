# 📚 Documentation Runique

Documentation complète et détaillée du framework Runique.

## 🌍 Langues disponibles

- 🇬🇧 **[English](en/README.md)** - English documentation
- 🇫🇷 **[Français](fr/README.md)** - Documentation en français

---

## 📖 10 sections de documentation

### 1. Installation

Installer et configurer Runique pour la première fois.

**Lire** : [English](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md) | [Français](https://github.com/seb-alliot/runique/blob/main/docs/fr/01-installation.md)

---

### 2. Architecture

Comprendre l'architecture et la structure interne du framework.

**Lire** : [English](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md) | [Français](https://github.com/seb-alliot/runique/blob/main/docs/fr/02-architecture.md)

---

### 3. Configuration

Configurer votre application (serveur, BD, sécurité).

**Lire** : [English](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md) | [Français](https://github.com/seb-alliot/runique/blob/main/docs/fr/03-configuration.md)

---

### 4. Routage

Définir les routes et les URL patterns.

**Lire** : [English](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md) | [Français](https://github.com/seb-alliot/runique/blob/main/docs/fr/04-routing.md)

---

### 5. Formulaires

Créer et gérer les formulaires avec validation.

**Lire** : [English](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md) | [Français](https://github.com/seb-alliot/runique/blob/main/docs/fr/05-forms.md)

---

### 6. Templates

Utiliser les templates Tera pour les vues.

**Lire** : [English](https://github.com/seb-alliot/runique/blob/main/docs/en/06-templates.md) | [Français](https://github.com/seb-alliot/runique/blob/main/docs/fr/06-templates.md)

---

### 7. ORM

Travailler avec la base de données via SeaORM.

**Lire** : [English](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md) | [Français](https://github.com/seb-alliot/runique/blob/main/docs/fr/07-orm.md)

---

### 8. Middlewares

Intégrer les middlewares de sécurité.

**Lire** : [English](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md) | [Français](https://github.com/seb-alliot/runique/blob/main/docs/fr/08-middleware.md)

---

### 9. Flash Messages

Utiliser les messages flash pour les retours utilisateur.

**Lire** : [English](https://github.com/seb-alliot/runique/blob/main/docs/en/09-flash-messages.md) | [Français](https://github.com/seb-alliot/runique/blob/main/docs/fr/09-flash-messages.md)

---

### 10. Exemples

Voir des exemples de code complets et de projets.

**Lire** : [English](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md) | [Français](https://github.com/seb-alliot/runique/blob/main/docs/fr/10-examples.md)

---

## 🎯 Guide de navigation

### Je suis nouveau sur Runique

1. Lire [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md)
2. Lire [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md)
3. Vérifier [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md)

### Je veux apprendre X

- Formulaires ? → [Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md)
- Routage ? → [Routing](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md)
- BD ? → [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md)
- Sécurité ? → [Middleware](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md)

### Je veux un exemple complet

→ [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md)

---

## 📊 Structure des documents

Chaque document contient :

- 📖 Explications détaillées

- 💻 Exemples de code
- 🎯 Bonnes pratiques
- ⚠️ Pièges à éviter
- 🔗 Références

---

## 🚀 Démarrage rapide

### Installation

```bash
git clone <repo>
cd runique
cargo build
cargo test
```

### Première app

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() {
    let app = RuniqueApp::builder(settings)
        .with_routes(routes)
        .build()
        .await;

    app.run().await;
}
```

### Formulaires

```rust
#[derive(RuniqueForm)]
pub struct MyForm {
    #[field(label = "Nom", required, min_length = 3)]
    pub name: String,
}

// Dans le handler
async fn handle_form(
    Prisme(mut form): Prisme<MyForm>,
    mut template: TemplateContext,
) -> Response {
    if form.is_valid().await {
        // Traiter le formulaire
    }
    template.context.insert("form", form);
    template.render("form.html")
}
```

---

## 🌐 Choix de la langue

### English (EN)

Cliquez sur le lien pour accéder à la documentation en anglais :
[📖 English Documentation](https://github.com/seb-alliot/runique/blob/main/docs/en/README.md)

### Français (FR)

Cliquez sur le lien pour accéder à la documentation en français :
[📖 Documentation Française](https://github.com/seb-alliot/runique/blob/main/docs/fr/README.md)

---

## 📋 Contenu par catégorie

### Mise en route

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/01-installation.md)
- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md)
- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md)

### Développement

- [Routage](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md)
- [Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md)
- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/06-templates.md)

### Données

- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md)

### Sécurité & Expérience

- [Middlewares](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md)
- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/en/09-flash-messages.md)

### Apprentissage pratique

- [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md)

---

## 💡 Conseils pour utiliser la documentation

1. **Utilisez la barre de recherche** de votre navigateur (Ctrl+F)
2. **Suivez l'ordre** des sections pour apprendre progressivement
3. **Consultez les exemples** pour du code réel
4. **Revisitez régulièrement** pour mieux comprendre

---

## ❓ FAQ

**Où commence-t-on ?**
→ [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md)

**Comment créer un formulaire ?**
→ [Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md)

**Comment interroger la BD ?**
→ [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md)

**Comment déployer en production ?**
→ [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md)

---

## 📞 Support

- 📚 Documentation : Vous êtes ici !
- 🧪 Tests : Voir `runique/tests/`
- 🎓 Exemples : Voir `demo-app/`
- 📊 Rapports : Voir `PROJECT_STATUS.md`

---

## ✅ Documentation Status

- ✅ 10 sections complètes
- ✅ Bilingue (EN & FR)
- ✅ Code examples inclus
- ✅ À jour (24/01/2026)

---

**Commencez maintenant !** 🚀

[📖 English](https://github.com/seb-alliot/runique/blob/main/docs/en/README.md) | [📖 Français](https://github.com/seb-alliot/runique/blob/main/docs/fr/README.md)
