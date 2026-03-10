# 📚 Documentation Runique - Français

Documentation complète du framework web Runique.

## 📖 Sections de la documentation

### 1️⃣ [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/01-installation.md)
Démarrer avec Runique. Installation, dépendances et premiers pas.

**Sujets couverts:**
- Prérequis
- Étapes d'installation
- Configuration du projet
- Première application

👉 **Aller à** : [Guide d'installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/01-installation.md)

---

### 2️⃣ [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/02-architecture.md)
Comprendre l'architecture interne de Runique.

**Sujets couverts:**
- Structure du projet
- Vue d'ensemble des composants
- Motifs de conception
- Comment ça marche

👉 **Aller à** : [Guide d'architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/02-architecture.md)

---

### 3️⃣ [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/03-configuration.md)
Configurer votre application Runique.

**Sujets couverts:**
- Configuration du serveur
- Mise en place de la BD
- Variables d'environnement
- Paramètres de sécurité

👉 **Aller à** : [Guide de configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/03-configuration.md)

---

### 4️⃣ [Routage](https://github.com/seb-alliot/runique/blob/main/docs/fr/routing/04-routing.md)
Routage des URL et traitement des requêtes.

**Sujets couverts:**
- Modèles d'URL
- Définition des routes
- Gestionnaires de requêtes
- Paramètres d'URL

👉 **Aller à** : [Guide de routage](https://github.com/seb-alliot/runique/blob/main/docs/fr/routing/04-routing.md)

---

### 5️⃣ [Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/05-forms.md)
Création et gestion de formulaires.

**Sujets couverts:**
- Extracteur Prisme
- Déclaration manuelle via `RuniqueForm`
- Déclaration basée modèle/schéma (AST) puis formulaire automatique
- Types de champs (FieldBuilder)
- Validation et sauvegarde
- Rendu dans les templates

👉 **Aller à** : [Guide des formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/05-forms.md)

---

### 6️⃣ [Templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/template/06-templates.md)
Travailler avec les templates Tera.

**Sujets couverts:**
- Tags Django-like ({% static %}, {% form.xxx %}, {% link %}, {% csrf %}, {% messages %}, {% csp_nonce %})
- Filtres Tera (static, media, form, csrf_field)
- Fonctions Tera (csrf(), nonce(), link())
- Macro context_update!
- Héritage de templates
- Variables auto-injectées

👉 **Aller à** : [Guide des templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/template/06-templates.md)

---

### 7️⃣ [ORM](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/07-orm.md)
Opérations de base de données avec SeaORM.

**Sujets couverts:**
- Définition de modèles
- Requêtes
- Relations
- Migrations

👉 **Aller à** : [Guide ORM](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/07-orm.md)

---

### 8️⃣ [Middlewares](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md)
Sécurité et middlewares de requête.

**Sujets couverts:**
- Stack middleware avec système de slots
- Protection CSRF (Double Submit Cookie)
- Content Security Policy (CSP) avec nonce
- Validation Allowed Hosts
- Headers de sécurité
- Configuration des sessions
- Builder Intelligent vs Builder classique

👉 **Aller à** : [Guide des middlewares](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md)

---

### 9️⃣ [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/09-flash-messages.md)
Retours utilisateur et notifications.

**Sujets couverts:**
- Macros de redirection : success!, error!, info!, warning!
- Macro immédiate : flash_now!
- Affichage avec {% messages %}
- Pattern flash vs flash_now
- Comportement de consommation (une seule lecture)

👉 **Aller à** : [Guide Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/09-flash-messages.md)

---

### 🔟 [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/10-examples.md)
Exemples de code complets et projets.

**Sujets couverts:**
- Application blog
- Authentification
- Upload de fichiers
- API REST

👉 **Aller à** : [Guide des exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/10-examples.md)

---
### 11. [Admin](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/11-Admin.md)

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

👉 **Go to**: [Guide des exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/11-Admin.md)
---

### 12. [Modèle](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/12-model.md)

DSL `model!`, génération d'entités SeaORM et de formulaires.

**Sujets couverts:**
- DSL `model!`
- Génération de formulaires depuis le modèle
- Génération de code (entities SeaORM)

👉 **Aller à** : [Guide modèle](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/12-model.md)

---

### 13. [Authentification](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/13-authentification.md)

Authentification utilisateur : sessions, middleware, modèle `is_staff` / `is_superuser`.

**Sujets couverts:**
- Modèle utilisateur
- Helpers de session (login, logout)
- Middleware d'authentification
- Exemple complet

👉 **Aller à** : [Guide authentification](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/13-authentification.md)

---

### 14. [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/14-sessions.md)

Gestion des sessions en mémoire avec protection anti-fuite.

**Sujets couverts:**
- Store (`CleaningMemoryStore`)
- Lecture / écriture en session
- Protection des sessions sensibles

👉 **Aller à** : [Guide sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/14-sessions.md)

---

### 15. [Variables d'environnement](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/15-env.md)

Toutes les variables `.env` reconnues par Runique.

**Sujets couverts:**
- Application et serveur
- Sécurité et sessions
- Assets et médias

👉 **Aller à** : [Guide variables d'environnement](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/15-env.md)

---

## 🎯 Navigation rapide

| Section | Fichier | Sujets |
|---------|---------|--------|
| Setup | [Installation](0https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/01-installation.md) | Prérequis, install, premiers pas |
| Apprentissage | [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/02-architecture.md) | Structure, conception, fonctionnement |
| Config | [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/03-configuration.md) | Paramètres, environnement, sécurité |
| Routes | [Routage](https://github.com/seb-alliot/runique/blob/main/docs/fr/routing/04-routing.md) | Modèles URL, gestionnaires, paramètres |
| Formulaires | [Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/05-forms.md) | Prisme, FieldBuilder, `#[form(...)]` |
| Vues | [Templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/template/06-templates.md) | Tags Django-like, filtres, fonctions Tera |
| Données | [ORM](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/07-orm.md) | Modèles, requêtes, impl_objects! |
| Sécurité | [Middlewares](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md) | Slots, CSRF, CSP, sessions |
| Retours | [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/09-flash-messages.md) | success!, flash_now!, {% messages %} |
| Code | [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/10-examples.md) | Projets complets |
| Code | [Admin](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/11-Admin.md) | Admin beta |
| Modèle | [Modèle](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/12-model.md) | `model!`, entities, génération |
| Auth | [Authentification](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/13-authentification.md) | login, logout, middleware, is_staff |
| Sessions | [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/14-sessions.md) | store, lecture/écriture, protection |
| Env | [Variables d'env](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/15-env.md) | configuration, sécurité, assets |
---

## 🚀 Par où commencer ?

1. **Nouveau sur Runique ?** → Commencez par [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/01-installation.md)
2. **Vous voulez comprendre ?** → Lisez [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/02-architecture.md)
3. **Prêt à coder ?** → Consultez [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/10-examples.md)
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

- 🇬🇧 **[English](https://github.com/seb-alliot/runique/blob/main/README.md)**
- 📖 **[Français](https://github.com/seb-alliot/runique/blob/main/README.fr.md)**

---

## 💡 Conseils

- Chaque guide contient des exemples
- Suivez les sections dans l'ordre
- Consultez les exemples pour du code réel
- Utilisez la recherche du navigateur

---

**Besoin d'aide ?** Consultez [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/10-examples.md) ou relisez la section pertinente.

Bon codage ! 