# 📚 Documentation Runique

Documentation complète et détaillée du framework Runique.

## 🌍 Langues disponibles

- 🇬🇧 **[English](en/README.md)** - English documentation
- 🇫🇷 **[Français](fr/README.md)** - Documentation en français

---

## 📖 10 sections de documentation

### 1. Installation
Installer et configurer Runique pour la première fois.

**Lire** : [English](en/01-installation.md) | [Français](fr/01-installation.md)

---

### 2. Architecture
Comprendre l'architecture et la structure interne du framework.

**Lire** : [English](en/02-architecture.md) | [Français](fr/02-architecture.md)

---

### 3. Configuration
Configurer votre application (serveur, BD, sécurité).

**Lire** : [English](en/03-configuration.md) | [Français](fr/03-configuration.md)

---

### 4. Routage
Définir les routes et les URL patterns.

**Lire** : [English](en/04-routing.md) | [Français](fr/04-routing.md)

---

### 5. Formulaires
Créer et gérer les formulaires avec validation.

**Lire** : [English](en/05-forms.md) | [Français](fr/05-forms.md)

---

### 6. Templates
Utiliser les templates Tera pour les vues.

**Lire** : [English](en/06-templates.md) | [Français](fr/06-templates.md)

---

### 7. ORM
Travailler avec la base de données via SeaORM.

**Lire** : [English](en/07-orm.md) | [Français](fr/07-orm.md)

---

### 8. Middlewares
Intégrer les middlewares de sécurité.

**Lire** : [English](en/08-middleware.md) | [Français](fr/08-middleware.md)

---

### 9. Flash Messages
Utiliser les messages flash pour les retours utilisateur.

**Lire** : [English](en/09-flash-messages.md) | [Français](fr/09-flash-messages.md)

---

### 10. Exemples
Voir des exemples de code complets et de projets.

**Lire** : [English](en/10-examples.md) | [Français](fr/10-examples.md)

---

## 🎯 Guide de navigation

### Je suis nouveau sur Runique
1. Lire [Installation](en/01-installation.md)
2. Lire [Architecture](en/02-architecture.md)
3. Vérifier [Exemples](en/10-examples.md)

### Je veux apprendre X
- Formulaires ? → [Forms](en/05-forms.md)
- Routage ? → [Routing](en/04-routing.md)
- BD ? → [ORM](en/07-orm.md)
- Sécurité ? → [Middleware](en/08-middleware.md)

### Je veux un exemple complet
→ [Examples](en/10-examples.md)

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
let mut form = Forms::new("csrf_token");
form.field(&TextField::text("name").label("Nom"));
```

---

## 🌐 Choix de la langue

### English (EN)
Cliquez sur le lien pour accéder à la documentation en anglais :
[📖 English Documentation](en/README.md)

### Français (FR)
Cliquez sur le lien pour accéder à la documentation en français :
[📖 Documentation Française](fr/README.md)

---

## 📋 Contenu par catégorie

### Mise en route
- [Installation](en/01-installation.md)
- [Configuration](en/03-configuration.md)
- [Architecture](en/02-architecture.md)

### Développement
- [Routage](en/04-routing.md)
- [Formulaires](en/05-forms.md)
- [Templates](en/06-templates.md)

### Données
- [ORM](en/07-orm.md)

### Sécurité & Expérience
- [Middlewares](en/08-middleware.md)
- [Flash Messages](en/09-flash-messages.md)

### Apprentissage pratique
- [Exemples](en/10-examples.md)

---

## 💡 Conseils pour utiliser la documentation

1. **Utilisez la barre de recherche** de votre navigateur (Ctrl+F)
2. **Suivez l'ordre** des sections pour apprendre progressivement
3. **Consultez les exemples** pour du code réel
4. **Revisitez régulièrement** pour mieux comprendre

---

## ❓ FAQ

**Où commence-t-on ?**
→ [Installation](en/01-installation.md)

**Comment créer un formulaire ?**
→ [Forms](en/05-forms.md)

**Comment interroger la BD ?**
→ [ORM](en/07-orm.md)

**Comment déployer en production ?**
→ [Configuration](en/03-configuration.md)

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

[📖 English](en/README.md) | [📖 Français](fr/README.md)
