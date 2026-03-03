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
- Extracteur `Request`, pipeline `Prisme`
- Macros principales (`urlpatterns!`, `view!`, `context_update!`)

👉 **Aller à** : [Guide d'architecture](02-architecture.md)

---

### 3️⃣ [Configuration](03-configuration.md)
Configurer votre application Runique.

**Sujets couverts:**
- Configuration du serveur
- Mise en place de la base de données
- Variables d'environnement
- Paramètres de sécurité
- Builder `RuniqueApp`

👉 **Aller à** : [Guide de configuration](03-configuration.md)

---

### 4️⃣ [Routage](04-routing.md)
Routage des URL et traitement des requêtes.

**Sujets couverts:**
- Macro `urlpatterns!` et `view!`
- Paramètres d'URL
- Routes nommées et `link()`
- Gestionnaires de requêtes

👉 **Aller à** : [Guide de routage](04-routing.md)

---

### 5️⃣ [Formulaires](05-forms.md)
Création et gestion de formulaires.

**Sujets couverts:**
- Extracteur `Prisme<T>` (Sentinel → Aegis → CSRF Gate → Construction)
- Déclaration via `RuniqueForm` + `impl_form_access!()`
- Déclaration automatique via `#[derive(DeriveModelForm)]`
- Types de champs (`TextField`, `NumericField`, `FileField`…)
- `PasswordConfig` — hachage Argon2/Bcrypt/Scrypt, `pre_hash_hook`
- Validation, helpers typés (`get_string()`, `get_i32()`…)
- Sauvegarde et rendu dans les templates

👉 **Aller à** : [Guide des formulaires](05-forms.md)

---

### 6️⃣ [Templates](06-templates.md)
Travailler avec les templates Tera.

**Sujets couverts:**
- Tags Django-like (`{% static %}`, `{% form.xxx %}`, `{% link %}`, `{% csrf %}`, `{% messages %}`, `{% csp_nonce %}`)
- Filtres et fonctions Tera
- Macro `context_update!`
- Héritage de templates
- Variables auto-injectées

👉 **Aller à** : [Guide des templates](06-templates.md)

---

### 7️⃣ [ORM](07-orm.md)
Opérations de base de données avec SeaORM.

**Sujets couverts:**
- Définition de modèles
- Macro `impl_objects!`
- Requêtes, filtres, relations
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
- Macros de redirection : `success!`, `error!`, `info!`, `warning!`
- Macro immédiate : `flash_now!`
- Affichage avec `{% messages %}`
- Pattern flash vs flash_now
- Comportement de consommation (une seule lecture)

👉 **Aller à** : [Guide Flash Messages](09-flash-messages.md)

---

### 🔟 [Exemples](10-examples.md)
Exemples de code complets.

**Sujets couverts:**
- Structure d'application complète
- Authentification (inscription, connexion)
- Upload de fichiers
- Mise à jour de profil

👉 **Aller à** : [Guide des exemples](10-examples.md)

---

### 1️⃣1️⃣ [Admin](11-Admin.md)
Interface d'administration générée automatiquement (bêta).

**Sujets couverts:**
- Macro déclarative `admin!`
- Génération automatique CRUD
- Routes, handlers et formulaires générés
- Sécurité de typage et transparence du code

👉 **Aller à** : [Guide Admin](11-Admin.md)

---

### 1️⃣2️⃣ [Modèles](12-model.md)
Définition des modèles de données.

**Sujets couverts:**
- Structure des entités SeaORM
- Définition des schémas
- Intégration avec les formulaires

👉 **Aller à** : [Guide des modèles](12-model.md)

---

## 🎯 Navigation rapide

| Section | Fichier | Sujets |
|---------|---------|--------|
| Setup | [Installation](01-installation.md) | Prérequis, install, premiers pas |
| Apprentissage | [Architecture](02-architecture.md) | Structure, Request, Prisme, macros |
| Config | [Configuration](03-configuration.md) | Paramètres, environnement, sécurité |
| Routes | [Routage](04-routing.md) | urlpatterns!, view!, paramètres URL |
| Formulaires | [Formulaires](05-forms.md) | Prisme, TextField, PasswordConfig, DeriveModelForm |
| Vues | [Templates](06-templates.md) | Tags Django-like, filtres, context_update! |
| Données | [ORM](07-orm.md) | Modèles, requêtes, impl_objects! |
| Sécurité | [Middlewares](08-middleware.md) | Slots, CSRF, CSP, sessions |
| Retours | [Flash Messages](09-flash-messages.md) | success!, flash_now!, {% messages %} |
| Code | [Exemples](10-examples.md) | Projets complets, auth, upload |
| Admin | [Admin](11-Admin.md) | Admin bêta, CRUD généré |
| Modèles | [Modèles](12-model.md) | Entités, schémas, formulaires |

---

## 🚀 Par où commencer ?

1. **Nouveau sur Runique ?** → Commencez par [Installation](01-installation.md)
2. **Vous voulez comprendre ?** → Lisez [Architecture](02-architecture.md)
3. **Prêt à coder ?** → Consultez [Exemples](10-examples.md)
4. **Besoin d'aide ?** → Cherchez la section correspondante ci-dessus

---

## 🌍 Langue

- 🇬🇧 **[English](../en/README.md)**
- 📖 **Français** (vous êtes ici)

---

**Besoin d'aide ?** Consultez [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/10-examples.md) ou relisez la section pertinente.
