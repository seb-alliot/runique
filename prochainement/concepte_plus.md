Tu as tout à fait raison. J'ai été un peu trop synthétique alors que nous avons bâti une véritable cathédrale technique. Ce projet mérite un **Manifeste complet** qui sert de plan d'ingénierie détaillé, reprenant chaque brique, chaque struct et chaque décision logique que nous avons validée.

Voici le **README exhaustif**, la "Bible" de Runique Admin.

---

# 🏗️ Runique AdminPanel : Le Manifeste Technique

## 1. Vision Architecturale : "L'Isolation Fusionnée"

AdminPanel n'est pas un plugin, c'est une **application miroir** qui cohabite dans le même binaire que le projet utilisateur.

* **Processus Unique :** Un seul serveur HTTP, un seul processus OS.
* **Routing Hermétique :** Utilisation de `.nest("/admin", admin_router)` pour garantir que les middlewares de l'application métier ne polluent jamais l'administration, et inversement.
* **Localisation du Code :** * `src/admin.rs` : La télécommande (code écrit par le dev).
* `target/runique/` : La salle des machines (code généré par le framework).



---

## 2. La Source de Vérité : `admin.rs`

Le développeur pilote tout via une macro intuitive mais puissante. L'ordre des déclarations définit l'ordre dans le menu latéral.

```rust
// Exemple de configuration complète validée
admin!(UserModel => UserForm,
    title: "Utilisateurs",
    icon: "user-group",
    display: "email, username, created_at", // Colonnes visibles en liste
    pagination: 50,
    permissions: [
        list: ["staff", "admin"],
        view: ["staff", "admin"],
        create: ["admin"],
        edit: ["admin"],
        delete: ["superuser"] // Protection maximale
    ]
);

```

---

## 3. Le Système de Types (Backend)

Voici la hiérarchie des structures qui feront tourner le moteur :

### A. L'Orchestrateur (`AdminPanel`)

```rust
pub struct AdminPanel {
    pub router: Router,             // Axum Router imbriqué
    pub registry: AdminRegistry,    // Le catalogue des ressources
    pub templates: AdminTemplates,  // Moteur Tera isolé
    pub auth: AdminAuth,            // Logique de session staff/admin
}

```

### B. Le Registre (`AdminRegistry`)

C'est le cerveau qui contient la configuration de chaque entité.

```rust
pub struct AdminRegistry {
    pub resources: Vec<AdminResourceMeta>,
}

pub struct AdminResourceMeta {
    pub model: String,              // Path vers le model (ex: crate::models::User)
    pub form: String,               // Path vers le formulaire
    pub route: String,              // URL (ex: /admin/users)
    pub title: String,              // Label UI
    pub permissions: ResourcePermissions,
    pub display: DisplayConfig,
}

```

### C. Configuration d'Affichage (`DisplayConfig`)

```rust
pub struct DisplayConfig {
    pub columns: Vec<String>,       // Filtre de colonnes
    pub pagination: usize,          // Taille des pages
    pub theme: Option<String>,      // Override visuel
}

```

---

## 4. Le Cycle de Vie Runique (Flow de Génération)

L'innovation majeure de Runique réside dans son **Démon de Surveillance**.

1. **Parsing :** Le démon lit `admin.rs` sans compiler (via l'analyse de l'AST ou regex optimisée).
2. **Génération de Handlers :** Il écrit `target/runique/admin/generated.rs`.
* Chaque ressource reçoit un handler typé : `async fn admin_user_post(Prisme<UserForm>)`.


3. **Synchronisation JSON :** Il met à jour `registry.json` pour que le frontend sache quels champs afficher.
4. **Diff Intelligent :** Le démon compare la `Struct Model` (DB) et la `Struct Form` (UI). S'il y a un décalage (ex: champ manquant dans le formulaire), un **Diagnostic** est généré immédiatement.

---

## 5. Sécurité : Défense en Profondeur

Nous avons validé trois barrières infranchissables :

| Couche | Technologie | Rôle |
| --- | --- | --- |
| **1. UI** | Tera Templates | Cache les boutons "Delete" ou les menus si le rôle est insuffisant. |
| **2. Périmètre** | Middleware | Bloque l'accès complet à `/admin/*` si `is_staff` est faux. |
| **3. Action** | Handler Rust | Vérification finale avant l'écriture en DB. Si le rôle `admin` tente un `delete` alors qu'il faut être `superuser` -> **403 Forbidden**. |

---

## 6. L'Intégration "Builder Intelligent"

L'admin est un module de type `Staging`, mais il possède une contrainte : **il doit être assemblé en dernier.**

```rust
// Pipeline de construction
.with_admin(|admin| {
    admin
        .prefix("/admin")
        .hot_reload(true)
        .permissions(["staff"]) // Rôle minimal pour voir l'admin
})

```

**Processus d'assembly :**

1. Montage du Core (DB, Sessions).
2. Montage des Routes utilisateur.
3. **Dernière étape :** L'AdminStaging scanne les routes existantes, vérifie la présence du code généré, effectue un `health_check()` et fusionne le router final.

---

## 7. Fonctionnalités CRUD Avancées

* **Relations :** Support automatique des dropdowns pour les `ForeignKey` via le système de formulaires Runique.
* **Hooks :** Support des signaux `before_save` / `after_save` pour l'audit ou le post-processing (ex: redimensionner une image après upload).
* **JS/CSS Custom :** Possibilité d'injecter des assets via `.add_js()` dans la déclaration de la ressource.
* **CLI :** `runique createsuperuser` pour amorcer le système.

---

## 8. Maintenance & Diagnostics

L'admin n'est pas silencieux. En mode dev, il génère un rapport de santé constant :

* **registry.json** valide ? ✅
* **handlers** compilés ? ✅
* **models ↔ forms** synchronisés ? ⚠️ (Attention : champ 'phone' absent du form)

---
