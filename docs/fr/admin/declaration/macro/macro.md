# La macro `admin!`

## Syntaxe complète

```rust
admin! {
    // Champs obligatoires uniquement
    articles: articles::Model => ArticleForm {
        title: "Articles",
        permissions: ["admin"]
    },

    // Tous les champs optionnels
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"],
        id_type: I32,                                        // I32 | I64 | Uuid
        edit_form: crate::formulaire::UserEditForm,          // formulaire distinct pour l'édition
        template_list: "mon_theme/users_list.html",          // surcharge template liste
        template_create: "mon_theme/users_create.html",      // surcharge template création
        template_edit: "mon_theme/users_edit.html",          // surcharge template édition
        template_detail: "mon_theme/users_detail.html",      // surcharge template détail
        template_delete: "mon_theme/users_delete.html",      // surcharge template suppression
        extra: {
            "icon" => "user",
            "color" => "#3b82f6"
        }
    }
}
```

La macro est parsée par le daemon (`runique start`) qui génère la fonction `admin_register()` dans `src/admins/admin_panel.rs`. Cette fonction construit le `HashMap<String, ResourceEntry>` chargé au boot.

---

## Champs

### Obligatoires

| Champ | Type | Description |
| --- | --- | --- |
| `key` (position) | identifiant | Utilisé dans les routes `/admin/{key}/…` |
| `model` (position) | chemin de type | Ex: `users::Model` |
| `form` (position) | chemin de type | Type du formulaire Runique associé |
| `title` | `&str` | Titre affiché dans l'interface |
| `permissions` | `[&str; N]` | Rôles déclarés pour cette ressource (⚠️ non appliqués — voir [Permissions](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/permission/permissions.md)) |

### Optionnels — surcharge de templates

Ces champs permettent de remplacer un template CRUD par un template personnalisé (voir [Surcharge des templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/template/surcharge/surcharge.md)).

| Champ | Valeur par défaut | Description |
| --- | --- | --- |
| `template_list` | `admin/list.html` | Template de liste |
| `template_create` | `admin/create.html` | Formulaire de création |
| `template_edit` | `admin/edit.html` | Formulaire d'édition |
| `template_detail` | `admin/detail.html` | Page de détail |
| `template_delete` | `admin/delete.html` | Page de confirmation de suppression |

### Optionnels — comportement

| Champ | Valeur par défaut | Description |
| --- | --- | --- |
| `id_type` | `I32` | Type de la clé primaire dans les routes — `I32`, `I64`, `Uuid` |
| `edit_form` | *(même que `form`)* | Formulaire distinct pour l'édition |
| `extra` | *(vide)* | Variables supplémentaires injectées dans les templates Tera de cette ressource |

#### `id_type`

Par défaut, le segment `{id}` des routes admin est converti en `i32`. Déclarer un type différent génère la conversion appropriée dans les closures CRUD :

```rust
admin! {
    posts: posts::Model => PostForm {
        title: "Articles",
        permissions: ["admin"],
        id_type: I64
    }
}
```

Types supportés : `I32` (défaut), `I64`, `Uuid`.

> La colonne en base est définie par l'entité SeaORM, pas par `id_type`. Ce champ ne génère ni migration ni changement de schéma — il ajuste seulement la conversion String → type natif dans le handler admin.

#### `edit_form`

Utiliser un formulaire différent pour la création et l'édition (cas courant : le formulaire de création inclut un champ mot de passe, pas l'édition) :

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"],
        edit_form: crate::formulaire::UserEditForm
    }
}
```

Lorsque déclaré, la vue edit utilise `edit_form` ; la vue create continue d'utiliser le formulaire principal. La méthode `save()` du wrapper edit retourne `Ok(())` — la persistance est assurée par `update_fn` via `admin_from_form`.

#### `extra`

Injecter des variables Tera disponibles dans tous les templates de cette ressource :

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"],
        extra: {
            "icon" => "user",
            "color" => "#3b82f6"
        }
    }
}
```

Les clés sont accessibles via `{{ resource.extra_context.icon }}`.

> Les clés réservées du framework (`entries`, `form_fields`, `object_id`, `csrf_token`, etc.) ont la priorité sur les clés `extra`.

---

## Exemple avec plusieurs ressources

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"]
    },
    articles: articles::Model => ArticleForm {
        title: "Articles",
        permissions: ["admin", "editor"]
    },
    comments: comments::Model => CommentForm {
        title: "Commentaires",
        permissions: ["moderator", "admin"]
    }
}
```

---

## Ce qui est déclarable

La macro `admin!` couvre uniquement les **métadonnées de registre** :

- l'identifiant de route de la ressource
- le modèle SeaORM utilisé
- le formulaire Runique pour la création/modification
- le titre d'affichage
- les rôles autorisés (uniformément sur toutes les opérations CRUD)
- le type de la clé primaire (`id_type`)
- un formulaire distinct pour l'édition (`edit_form`)
- des variables Tera supplémentaires par ressource (`extra`)

## Ce qui n'est pas déclarable

| Fonctionnalité | Raison de l'exclusion |
| --- | --- |
| Permissions par opération CRUD | Non supporté — les rôles s'appliquent globalement |
| Règles conditionnelles | Relève de la logique métier, à écrire dans le code généré |
| Rendu HTML / templates | Séparation des responsabilités |
| Filtres ou relations complexes | Trop spécifique pour être déclaratif |
| Logique d'authentification | Gérée par les middlewares admin |

---

## Erreurs compile-time

La macro inclut une vérification statique : si un type référencé (`Model` ou formulaire) n'existe pas dans le scope, **la compilation échoue** avec un message d'erreur explicite.

```text
error[E0412]: cannot find type `RegisterForm` in module `users`
```

Les champs obligatoires manquants produisent également des erreurs à la compilation, pas au runtime.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CLI](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/declaration/cli.md) | Fonctionnement de `runique start`
| [Daemon & génération](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/declaration/daemon/generation.md) | Fichiers générés

## Revenir au sommaire

- [Sommaire Admin](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/11-Admin.md)
