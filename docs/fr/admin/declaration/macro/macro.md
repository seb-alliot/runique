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
        list_display: [                                      // colonnes visibles dans la vue liste
            ["username", "Nom d'utilisateur"],
            ["email", "Email"],
            ["is_active", "Actif"],
        ],
        list_filter: [                                       // filtres dans la barre latérale
            ["is_active", "Actif"],                          // défaut : 10 valeurs par page
            ["is_superuser", "Superuser", 5],                // limite explicite par colonne
        ],
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
| `permissions` | `[&str; N]` | Rôles déclarés pour cette ressource (⚠️ non appliqués — voir [Permissions](/docs/fr/admin/permission)) |

### Optionnels — comportement

| Champ | Valeur par défaut | Description |
| --- | --- | --- |
| `id_type` | `I32` | Type de la clé primaire dans les routes — `I32`, `I64`, `Uuid` |
| `edit_form` | *(même que `form`)* | Formulaire distinct pour l'édition |
| `list_display` | *(vide — toutes colonnes)* | Colonnes visibles et leurs libellés dans la vue liste |
| `list_filter` | *(vide — pas de sidebar)* | Champs disponibles dans la barre de filtre latérale (limite optionnelle par colonne en 3ème élément, défaut `10`) |
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

#### `list_display`

Déclare les colonnes affichées dans la vue liste et leurs libellés :

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"],
        list_display: [
            ["username", "Nom d'utilisateur"],
            ["email", "Email"],
            ["is_superuser", "Superuser"],
            ["is_active", "Actif"],
        ]
    }
}
```

Chaque entrée est une paire `["nom_colonne", "Libellé"]`. Le nom de colonne doit correspondre à un champ de l'entité SeaORM.

Si `list_display` est absent, **toutes les colonnes** de l'entité sont affichées (comportement par défaut).

Les colonnes sont injectées dans le contexte Tera sous les clés `visible_columns` (noms) et `column_labels` (libellés correspondants).

#### `list_filter`

Déclare les champs disponibles dans la barre de filtre latérale :

```rust
admin! {
    doc_page: doc_page::Model => DocPageForm {
        title: "Doc — Pages",
        permissions: ["admin"],
        list_filter: [
            ["lang", "Langue"],          // défaut : 10 valeurs par page
            ["block_type", "Type", 5],   // limite explicite : 5 valeurs par page
        ]
    }
}
```

Chaque entrée est un triplet `["nom_colonne", "Libellé", limite_optionnelle]`. Le 3ème élément est optionnel — si absent, la limite par défaut est `10` valeurs par page.

Pour chaque champ, le daemon génère une requête qui charge les valeurs distinctes depuis la base avec pagination serveur (`LIMIT` / `OFFSET`). La navigation `‹ 1/N ›` dans la sidebar recharge la page via des paramètres `fp_{colonne}` dans l'URL, sans JavaScript.

La barre latérale s'affiche uniquement si au moins un filtre est déclaré. Si `list_filter` est absent, la mise en page reste en colonne unique.

> Ne pas utiliser `list_filter` sur des clés étrangères (FK) ou des colonnes `id` — la valeur brute (`35`, `128`…) n'est pas lisible. Bons candidats : booléens, énumérations, codes courts (`lang`, `status`, `block_type`).

Les filtres actifs sont transmis à la requête SQL et injectés dans le contexte Tera sous les clés `filter_values`, `active_filters` et `filter_meta`. Voir [contexte de la vue liste](/docs/fr/admin/template/clef/context) pour le détail.

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
- les colonnes visibles dans la vue liste (`list_display`)
- les filtres de la barre latérale (`list_filter`) avec limite optionnelle par colonne
- des variables Tera supplémentaires par ressource (`extra`)

## Ce qui n'est pas déclarable

| Fonctionnalité | Raison de l'exclusion |
| --- | --- |
| Permissions par opération CRUD | Non supporté — les rôles s'appliquent globalement |
| Règles conditionnelles | Relève de la logique métier, à écrire dans le code généré |
| Rendu HTML / templates | Séparation des responsabilités |
| Relations complexes / JOINs | Trop spécifique pour être déclaratif |
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
| [CLI](/docs/fr/admin/declaration) | Fonctionnement de `runique start`
| [Daemon & génération](/docs/fr/admin/declaration) | Fichiers générés

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
