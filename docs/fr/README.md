# 📚 Documentation Runique - Français

Documentation complète du framework web Runique.

## 📖 Sections de la documentation

### 1️⃣ [Installation](01-installation.md)
Démarrer avec Runique. Installation, dépendances et premiers pas.

**Sujets couverts:**
- Prérequis
- Étapes d'installation
- Configuration du projet
- Première application

👉 **Aller à** : [Guide d'installation](01-installation.md)

---

### 2️⃣ [Architecture](02-architecture.md)
Comprendre l'architecture interne de Runique.

**Sujets couverts:**
- Structure du projet
- Vue d'ensemble des composants
- Motifs de conception
- Comment ça marche

👉 **Aller à** : [Guide d'architecture](02-architecture.md)

---

### 3️⃣ [Configuration](03-configuration.md)
Configurer votre application Runique.

**Sujets couverts:**
- Configuration du serveur
- Mise en place de la BD
- Variables d'environnement
- Paramètres de sécurité

👉 **Aller à** : [Guide de configuration](03-configuration.md)

---

### 4️⃣ [Routage](04-routing.md)
Routage des URL et traitement des requêtes.

**Sujets couverts:**
- Modèles d'URL
- Définition des routes
- Gestionnaires de requêtes
- Paramètres d'URL

👉 **Aller à** : [Guide de routage](04-routing.md)

---

### 5️⃣ [Formulaires](05-forms.md)
Création et gestion de formulaires.

**Sujets couverts:**
- Extracteur Prisme
- Déclaration manuelle via `RuniqueForm`
- Déclaration basée modèle/schéma (AST) puis formulaire automatique
- Types de champs (FieldBuilder)
- Validation et sauvegarde
- Rendu dans les templates

👉 **Aller à** : [Guide des formulaires](05-forms.md)

---

### 6️⃣ [Templates](06-templates.md)
Travailler avec les templates Tera.

**Sujets couverts:**
- Tags Django-like ({% static %}, {% form.xxx %}, {% link %}, {% csrf %}, {% messages %}, {% csp_nonce %})
- Filtres Tera (static, media, form, csrf_field)
- Fonctions Tera (csrf(), nonce(), link())
- Macro context_update!
- Héritage de templates
- Variables auto-injectées

👉 **Aller à** : [Guide des templates](06-templates.md)

---

### 7️⃣ [ORM](07-orm.md)
Opérations de base de données avec SeaORM.

**Sujets couverts:**
- Définition de modèles
- Requêtes
- Relations
- Migrations

👉 **Aller à** : [Guide ORM](07-orm.md)

---

### 8️⃣ [Middlewares](08-middleware.md)
Sécurité et middlewares de requête.

**Sujets couverts:**
- Stack middleware avec système de slots
- Protection CSRF (Double Submit Cookie)
- Content Security Policy (CSP) avec nonce
- Validation Allowed Hosts
- Headers de sécurité
- Configuration des sessions
- Builder Intelligent vs Builder classique

👉 **Aller à** : [Guide des middlewares](08-middleware.md)

---

### 9️⃣ [Flash Messages](09-flash-messages.md)
Retours utilisateur et notifications.

**Sujets couverts:**
- Macros de redirection : success!, error!, info!, warning!
- Macro immédiate : flash_now!
- Affichage avec {% messages %}
- Pattern flash vs flash_now
- Comportement de consommation (une seule lecture)

👉 **Aller à** : [Guide Flash Messages](09-flash-messages.md)

---

### 🔟 [Exemples](10-examples.md)
Exemples de code complets et projets.

**Sujets couverts:**
- Application blog
- Authentification
- Upload de fichiers
- API REST

👉 **Aller à** : [Guide des exemples](10-examples.md)

---
### 11. Admin

---

##  Vue d’administration (bêta)

Runique intègre une **vue d’administration en version bêta**, basée sur une macro déclarative `admin!` et un système de génération automatique.

Les ressources administrables sont déclarées dans `src/admin.rs`.
À partir de cette déclaration, Runique génère automatiquement une interface CRUD complète (routes, handlers, formulaires) sous forme de **code Rust standard**, lisible et auditable.

Cette approche met l’accent sur :

* la **sécurité de typage** (vérification à la compilation des modèles et formulaires)
* la **transparence** (pas de logique cachée, pas de macro procédurale)
* le **contrôle développeur** sur le code généré

Le daemon (`runique start`) permet une régénération automatique, tandis qu’un workflow `cargo run` peut être utilisé lorsque des modifications manuelles sont nécessaires.

>  La vue admin est actuellement en **bêta** et pose volontairement des bases simples, déclaratives et sûres. Des évolutions sont prévues (permissions plus fines, meilleur feedback, protections supplémentaires).

---

## 🎯 Navigation rapide

| Section | Fichier | Sujets |
|---------|---------|--------|
| Setup | [Installation](01-installation.md) | Prérequis, install, premiers pas |
| Apprentissage | [Architecture](02-architecture.md) | Structure, conception, fonctionnement |
| Config | [Configuration](03-configuration.md) | Paramètres, environnement, sécurité |
| Routes | [Routage](04-routing.md) | Modèles URL, gestionnaires, paramètres |
| Formulaires | [Formulaires](05-forms.md) | Prisme, FieldBuilder, `#[form(...)]` |
| Vues | [Templates](06-templates.md) | Tags Django-like, filtres, fonctions Tera |
| Données | [ORM](07-orm.md) | Modèles, requêtes, impl_objects! |
| Sécurité | [Middlewares](08-middleware.md) | Slots, CSRF, CSP, sessions |
| Retours | [Flash Messages](09-flash-messages.md) | success!, flash_now!, {% messages %} |
| Code | [Exemples](10-examples.md) | Projets complets |
| Code | [Admin](11-Admin.md) | Admin beta |
---

## 🚀 Par où commencer ?

1. **Nouveau sur Runique ?** → Commencez par [Installation](01-installation.md)
2. **Vous voulez comprendre ?** → Lisez [Architecture](02-architecture.md)
3. **Prêt à coder ?** → Consultez [Exemples](10-examples.md)
4. **Besoin d'aide ?** → Cherchez la section correspondante ci-dessus

---

## 📋 Caractéristiques de la documentation

- ✅ Complète et détaillée
- ✅ Exemples de code inclus
- ✅ Bonnes pratiques mises en évidence
- ✅ Problèmes courants adressés
- ✅ Liens et références

---

## 🌍 Langue

- 🇬🇧 **[English](../en/README.md)**
- 📖 **Français** (vous êtes ici)

---

## 💡 Conseils

- Chaque guide contient des exemples
- Suivez les sections dans l'ordre
- Consultez les exemples pour du code réel
- Utilisez la recherche du navigateur

---

**Besoin d'aide ?** Consultez [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/10-examples.md) ou relisez la section pertinente.

Bon codage ! 🚀