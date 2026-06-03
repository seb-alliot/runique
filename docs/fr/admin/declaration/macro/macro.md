# La macro `admin!`

## Syntaxe complète

```rust
admin! {
    // Optionnel : configure l'affichage de n'importe quelle ressource (déclarée ou builtin)
    configure {
        users:  { list_display: [["id", "ID"], ["username", "Nom"], ["email", "Email"]] },
        droits: { list_display: [["id", "ID"], ["nom", "Nom"]] },
        blog:   { list_display: [["id", "ID"], ["titre", "Titre"], ["created_at", "Créé le"]] },
    }

    // Champs obligatoires uniquement
    articles: articles::Model => ArticleForm {
        title: "Articles",
    },

    // Tous les champs optionnels
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
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

### Optionnels — comportement

| Champ | Valeur par défaut | Description |
| --- | --- | --- |
| `id_type` | `I32` | Type de la clé primaire dans les routes — `I32`, `I64`, `Uuid` |
| `create_form` | *(même que `form`)* | Formulaire distinct pour la création |
| `edit_form` | *(même que `form`)* | Formulaire distinct pour l'édition |
| `list_display` | *(vide — toutes colonnes)* | Colonnes visibles et leurs libellés dans la vue liste |
| `list_filter` | *(vide — pas de sidebar)* | Champs disponibles dans la barre de filtre latérale (limite optionnelle par colonne en 3ème élément, défaut `10`) |
| `extra` | *(vide)* | Variables supplémentaires injectées dans les templates Tera de cette ressource |

### Bloc `configure {}`

Le bloc `configure {}` contrôle l'affichage de **n'importe quelle** ressource enregistrée — qu'elle soit déclarée dans `admin!{}` ou injectée automatiquement en tant que builtin (users, droits, groupes).

```rust
admin! {
    configure {
        users:  { list_display: [["id", "ID"], ["username", "Nom"], ["email", "Email"]] },
        droits: { list_display: [["id", "ID"], ["nom", "Nom"]] },
    }
    // ... déclarations de ressources
}
```

Clés disponibles dans chaque entrée :

| Clé | Type | Description |
| --- | --- | --- |
| `list_display` | `[["col", "Libellé"], …]` | Colonnes visibles ordonnées dans la vue liste |
| `list_exclude` | `["col1", "col2", …]` | Colonnes à masquer (exclusif avec `list_display`) |
| `list_filter` | `[["col", "Libellé"], …]` | Filtres de la barre latérale |
| `group_action` | `[["champ", "Libellé"], …]` | Actions de masse (disponible aussi ici pour les builtins) |
| `hidden` | `true` / `false` | Supprime la ressource builtin du registry — utile quand `extend!{}` prend le relai |

`list_display` et `list_exclude` sont mutuellement exclusifs — le daemon rejette les deux déclarés pour la même ressource.

Quand `hidden: true` est déclaré, toutes les autres clés de cette entrée sont ignorées — la ressource est supprimée du registry.

Le daemon génère des appels `registry.configure("key", DisplayConfig::new()...)` **après** tous les `registry.register()`, ce qui permet de configurer aussi les ressources builtin enregistrées en amont. Pour `hidden: true`, il génère `registry.remove("key")`.

**Exemple typique — `extend!{}` prend ownership d'`eihwaz_users` :**

```rust
admin! {
    configure {
        users: { hidden: true }
    }
    user_profile: user_profile::Model => user_profile::AdminForm {
        title: "Profils utilisateurs",
        list_display: [
            ["username", "Utilisateur"],
            ["email", "Email"],
            ["bio", "Bio"],
            ["is_verified", "Vérifié"],
        ],
    }
}
```

Le panel builtin "Utilisateurs" disparaît ; "Profils utilisateurs" le remplace avec toute la table (`eihwaz_users` + colonnes étendues).

#### `id_type`

Par défaut, le segment `{id}` des routes admin est converti en `i32`. Déclarer un type différent génère la conversion appropriée dans les closures CRUD :

```rust
admin! {
    posts: posts::Model => PostForm {
        title: "Articles",
        id_type: I64
    }
}
```

Types supportés : `I32` (défaut), `I64`, `Uuid`.

> La colonne en base est définie par l'entité SeaORM, pas par `id_type`. Ce champ ne génère ni migration ni changement de schéma — il ajuste seulement la conversion String → type natif dans le handler admin.

#### `create_form`

Utiliser un formulaire différent pour la création (cas courant : un `CheckboxField` multi-sélection pour créer plusieurs enregistrements en une fois, combiné avec `bulk_create`) :

```rust
admin! {
    horaires: horaire::Model => horaire::AdminForm {
        title: "Horaires",
        create_form: crate::formulaire::HorairesGroupeForm,
        bulk_create: jour,
    }
}
```

Lorsque déclaré, la vue create utilise `create_form` ; la vue edit continue d'utiliser le formulaire principal (ou `edit_form` si déclaré séparément). La méthode `save()` du wrapper create retourne `Ok(())` — la persistance est assurée par `create_fn`.

> `create_form` active implicitement `inject_password: true` sur la ressource, ce qui préserve le traitement du champ mot de passe si présent dans le formulaire.

---

#### `edit_form`

Utiliser un formulaire différent pour la création et l'édition (cas courant : le formulaire de création inclut un champ mot de passe, pas l'édition) :

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
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
        list_display: [
            ["username", "Nom d'utilisateur"],
            ["email", "Email"],
            ["is_superuser", "Superuser"],
            ["is_active", "Actif"],
        ]
    }
}
```

Chaque entrée est une paire `["nom_colonne", "Libellé"]`. Le nom de colonne doit correspondre **exactement** au nom du champ dans l'entité SeaORM générée (en général identique au nom déclaré dans `model!`). Utiliser le mauvais nom produit une erreur SQL au runtime.

> Si le champ DSL est `sort_order`, le nom de colonne à déclarer est `"sort_order"` — pas `"ordre"` ni un alias quelconque.

**Résolution FK** — Pour afficher le libellé d'une relation au lieu de l'ID brut, ajouter un 3ème élément `"table.colonne"` :

```rust
list_display: [
    ["theme_id", "Thème", "themes.nom"],   // affiche themes.nom au lieu de l'ID
    ["user_id",  "Auteur", "eihwaz_users.username"],
]
```

Le daemon génère une jointure SQL `LEFT JOIN themes ON menus.theme_id = themes.id` et expose la valeur résolue dans le contexte Tera.

Si `list_display` est absent, **toutes les colonnes** de l'entité sont affichées (comportement par défaut).

Les colonnes sont injectées dans le contexte Tera sous les clés `visible_columns` (noms) et `column_labels` (libellés correspondants).

#### `list_filter`

Déclare les champs disponibles dans la barre de filtre latérale :

```rust
admin! {
    doc_page: doc_page::Model => DocPageForm {
        title: "Doc — Pages",
        list_filter: [
            ["lang", "Langue"],          // défaut : 10 valeurs par page
            ["block_type", "Type", 5],   // limite explicite : 5 valeurs par page
        ]
    }
}
```

Chaque entrée est un triplet `["nom_colonne", "Libellé", limite_optionnelle]`. Le nom de colonne suit la même règle que `list_display` : il doit correspondre exactement au champ de l'entité SeaORM. Le 3ème élément est optionnel — si absent, la limite par défaut est `10` valeurs par page.

Les colonnes de type `enum` sont de bons candidats pour `list_filter` : les valeurs distinctes affichées correspondent aux variantes sérialisées (capitalisées). Voir [DSL `model!`](/docs/fr/model/dsl) pour le comportement des enums.

Pour chaque champ, le daemon génère une requête qui charge les valeurs distinctes depuis la base avec pagination serveur (`LIMIT` / `OFFSET`). La navigation `‹ 1/N ›` dans la sidebar recharge la page via des paramètres `fp_{colonne}` dans l'URL, sans JavaScript.

La barre latérale s'affiche uniquement si au moins un filtre est déclaré. Si `list_filter` est absent, la mise en page reste en colonne unique.

> Ne pas utiliser `list_filter` sur des clés étrangères (FK) ou des colonnes `id` — la valeur brute (`35`, `128`…) n'est pas lisible. Bons candidats : booléens, énumérations, codes courts (`lang`, `status`, `block_type`).

Les filtres actifs sont transmis à la requête SQL et injectés dans le contexte Tera sous les clés `filter_values`, `active_filters` et `filter_meta`. Voir [contexte de la vue liste](/docs/fr/admin/template/clef/context) pour le détail.

#### `group_action`

Déclare des actions de masse applicables sur une sélection d'entrées dans la vue liste (ex: activer/désactiver en masse) :

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        group_action: [
            ["is_active", "Activer"],   // 2 éléments : champ booléen → soumet "true"
        ]
    }
}
```

Chaque entrée à 2 éléments `["nom_champ", "Libellé"]` cible un **champ booléen** et soumet `"true"` via `partial_update_fn`.

Pour les **champs enum**, utiliser la syntaxe à 3 éléments pour soumettre une valeur exacte :

```rust
admin! {
    avis: avis::Model => avis::AdminForm {
        title: "Avis",
        group_action: [
            ["statut", "Valider", "valide"],   // 3 éléments : soumet "valide"
            ["statut", "Refuser", "refuse"],   // 3 éléments : soumet "refuse"
        ]
    }
}
```

Plusieurs entrées ciblant le **même champ** sont automatiquement fusionnées en un seul select dans l'interface, évitant les conflits de soumission.

Disponible aussi dans `configure {}` pour les ressources builtin.

---

#### `bulk_create`

Quand déclaré sur une ressource, le `create_fn` généré découpe le champ nommé par virgule et effectue un **upsert** par valeur : mise à jour si l'enregistrement existe déjà (même valeur du champ split), insertion sinon. Conçu pour les `CheckboxField` multi-sélection (ex : sélectionner plusieurs jours de la semaine pour créer ou mettre à jour un `horaire` par jour).

```rust
admin! {
    horaires: horaire::Model => horaire::AdminForm {
        title: "Horaires",
        create_form: crate::formulaire::HorairesGroupeForm,
        bulk_create: jour,   // upsert par valeur de data["jour"]
    }
}
```

Seul le champ split se comporte différemment — tous les autres champs du formulaire sont copiés tels quels dans chaque enregistrement inséré ou mis à jour.

**Pipeline de soumission multi-sélection** : une série de checkboxes HTML soumettent plusieurs fois la même clé (`jour=lundi&jour=mardi&jour=jeudi`). Prisme, le parser de corps de requête de Runique, joint automatiquement ces valeurs répétées par une virgule → le champ devient `"lundi,mardi,jeudi"`. C'est cette chaîne que `bulk_create` découpe pour traiter chaque valeur indépendamment.

```rust
// Dans le formulaire de création groupée
form.field(
    &CheckboxField::new("jour")
        .label("Jours")
        .add_choice("lundi", "Lundi")
        .add_choice("mardi", "Mardi")
        // ...
);
```

**Interaction avec `edit_form`** : quand `bulk_create` est déclaré sans `edit_form` explicite, le daemon génère automatiquement un `edit_form_builder` utilisant `module::AdminForm` (formulaire standard mono-enregistrement). La vue edit individuelle n'utilise donc jamais le formulaire multi-sélection de la création groupée.

> Le champ split doit correspondre à une contrainte `unique` dans le modèle pour que l'upsert fonctionne correctement — c'est cette unicité qui permet de retrouver l'enregistrement existant.

---

#### `m2m`

Déclare des relations many-to-many gérées via une table de jonction. Le daemon génère une closure `M2mLoaderFn` qui charge les choix disponibles et pré-sélectionne les associations existantes.

```rust
admin! {
    articles: article::Model => ArticleForm {
        title: "Articles",
        m2m: [
            ["tags", "Tags", "article_tags", "article_id", "tag_id", "tags::Entity", "name"],
        ]
    }
}
```

Chaque entrée est un tableau à 7 éléments :

| Position | Exemple | Description |
| --- | --- | --- |
| 1 | `"tags"` | Nom du champ — utilisé comme préfixe de champ de formulaire (`m2m_tags__`) |
| 2 | `"Tags"` | Libellé affiché dans le formulaire création/édition |
| 3 | `"article_tags"` | Nom de la table de jonction |
| 4 | `"article_id"` | Colonne FK de cette entité dans la table de jonction |
| 5 | `"tag_id"` | Colonne FK de l'entité cible dans la table de jonction |
| 6 | `"tags::Entity"` | Chemin de l'entité SeaORM cible |
| 7 | `"name"` | Colonne de la table cible utilisée comme libellé dans le formulaire |

Dans les formulaires de création/édition, tous les choix disponibles sont chargés depuis la table cible et les associations existantes sont pré-sélectionnées. À la soumission, les valeurs soumises (préfixées `m2m_field__`) sont comparées à l'état courant — seuls les inserts et suppressions nécessaires sont appliqués.

---

#### Édition en masse (bulk edit)

L'édition en masse ne nécessite aucune déclaration DSL. Quand des entrées sont sélectionnées dans la vue liste et que l'action bulk-edit est déclenchée, un formulaire est rendu avec tous les champs éditables.

Les **champs à contrainte unique** sont automatiquement exclus du formulaire d'édition en masse — appliquer la même valeur unique sur plusieurs enregistrements violerait la contrainte. Ces champs sont détectés via la constante `UNIQUE_FIELDS` générée par `derive_form!{}` pour chaque entité.

À la soumission, chaque enregistrement est mis à jour indépendamment. Seuls les champs avec une valeur non vide sont appliqués — laisser un select vide signifie « sans changement ».

Le formulaire d'édition en masse utilise le même type de formulaire que la vue création/édition. Pour personnaliser le template, surcharger `admin/bulk_edit.html`.

---

#### `own_field`

Déclare le champ de l'entité utilisé pour les vérifications de propriété (`can_update_own` / `can_delete_own`). Sans cette déclaration, les droits `own` sont ignorés même si accordés dans les permissions.

```rust
admin! {
    articles: articles::Model => ArticleForm {
        title: "Articles",
        own_field: "author_id",   // vérifié contre l'user_id de la session courante
    }
}
```

Le champ doit contenir l'ID de l'utilisateur propriétaire de l'enregistrement. Le daemon génère un appel `resource.own_field("author_id")` dans `admin_register()`, ce qui active la vérification dans `handle_edit` et `handle_delete`.

---

#### `template_list`, `template_create`, `template_edit`, `template_detail`, `template_delete`

Surcharger le template utilisé pour une opération précise sur cette ressource uniquement. Utile pour personnaliser une vue sans toucher au template global.

```rust
admin! {
    articles: articles::Model => ArticleForm {
        title: "Articles",
        template_list:   "admin/articles/list.html",
        template_create: "admin/articles/create.html",
        template_edit:   "admin/articles/edit.html",
    }
}
```

Seules les opérations déclarées sont surchargées — les autres utilisent le template admin par défaut. Le chemin est relatif au dossier `templates/` du projet.

Voir [Surcharge des templates](/docs/fr/admin/template/surcharge) pour la liste complète des blocks disponibles.

---

#### `extra`

Injecter des variables Tera disponibles dans tous les templates de cette ressource :

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        extra: {
            "icon" => "user",
            "color" => "#3b82f6"
        }
    }
}
```

Les clés sont accessibles via `{{ resource.extra_context.icon }}`.

> Les templates admin par défaut n'utilisent pas `icon` ni `color`. Ces clés n'ont d'effet que dans un **template personnalisé** qui les lit explicitement. Voir [Surcharge des templates](/docs/fr/admin/template/surcharge).
>
> Les clés réservées du framework (`entries`, `form_fields`, `object_id`, `csrf_token`, etc.) ont la priorité sur les clés `extra`.

---

## Exemple avec plusieurs ressources

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
    },
    articles: articles::Model => ArticleForm {
        title: "Articles",
    },
    comments: comments::Model => CommentForm {
        title: "Commentaires",
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
- le type de la clé primaire (`id_type`)
- un formulaire distinct pour l'édition (`edit_form`)
- les colonnes visibles dans la vue liste (`list_display`)
- les filtres de la barre latérale (`list_filter`) avec limite optionnelle par colonne
- des actions de masse (`group_action`)
- création multi-enregistrements depuis un champ split par virgule (`bulk_create`)
- relations many-to-many via table de jonction (`m2m`)
- des variables Tera supplémentaires par ressource (`extra`)
- la vérification de propriété pour `can_update_own`/`can_delete_own` (`own_field`)
- la surcharge du template pour une opération précise (`template_list/create/edit/detail/delete`)
- la configuration d'affichage de toute ressource y compris les builtins (`configure {}`)

## Ce qui n'est pas déclarable

| Fonctionnalité | Raison de l'exclusion |
| --- | --- |
| Permissions d'accès aux ressources | Gérées depuis le panel via les droits scopés — voir [Permissions](/docs/fr/admin/permission) |
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
| [CLI](/docs/fr/admin/declaration/cli) | Fonctionnement de `runique start`
| [Daemon & génération](/docs/fr/admin/declaration/daemon/generation) | Fichiers générés

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
