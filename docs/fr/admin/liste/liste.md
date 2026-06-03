# Vue liste — Pagination, tri, recherche et filtres

La vue liste admin gère toutes ses opérations **au niveau SQL** : pagination, tri par colonne, recherche plein-texte et filtres par valeur. Rien n'est chargé en mémoire.

## Sommaire

- [Colonnes affichées — list_display](#colonnes-affichées--list_display)
- [Résolution des clés étrangères (FK)](#résolution-des-clés-étrangères-fk)
- [Filtres latéraux — list_filter](#filtres-latéraux--list_filter)
- [Tri par colonne](#tri-par-colonne)
- [Recherche](#recherche)
- [Pagination](#pagination)
- [Paramètres d'URL](#paramètres-durl)

## Colonnes affichées — list_display

Par défaut, toutes les colonnes de l'entité sont affichées. Déclarer `list_display` restreint l'affichage à une sélection ordonnée :

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        list_display: [
            ["username", "Nom d'utilisateur"],
            ["email", "Email"],
            ["is_active", "Actif"],
        ]
    }
}
```

Chaque entrée est une paire `["nom_colonne", "Libellé"]`. Les colonnes déclarées dans `list_display` servent aussi de **whitelist pour le tri** : seules ces colonnes (et `id`) acceptent un `sort_by`.

Disponible dans le contexte Tera via `visible_columns` (liste des noms) et `column_labels` (libellés).

## Résolution des clés étrangères (FK)

Un troisième élément optionnel dans chaque entrée de `list_display` permet d'afficher le libellé d'une table liée au lieu de l'ID brut :

```rust
admin! {
    commandes: commande::Model => commande::AdminForm {
        title: "Commandes",
        list_display: [
            ["numero", "N°"],
            ["user_id", "Client", "eihwaz_users.username"],   // ← FK
            ["statut", "Statut"],
        ],
    }
}
```

La syntaxe du 3ème élément est `"table.colonne"` — le daemon génère automatiquement :

- Dans la **vue liste** : une requête `SELECT CAST(id AS TEXT), colonne FROM table WHERE id IN (...)` post-traitement qui remplace l'ID par le libellé affiché.
- Dans les **formulaires create et edit** : un `<select>` peuplé depuis la table liée, avec pré-sélection de la valeur existante en mode édition.

La résolution est compatible `i32`, `i64` et UUID — l'identifiant est converti en `TEXT` avant la comparaison.

La colonne FK est automatiquement **exclue de la recherche plein-texte** (chercher dans un ID brut n'a pas de sens). Les colonnes non-FK restent indexées normalement.

### Exemples courants

```rust
["menu_id",   "Menu",   "menus.titre"]
["theme_id",  "Thème",  "themes.libelle"]
["user_id",   "Client", "eihwaz_users.username"]
```

### Configurer les builtins — configure {}

`list_display` dans le corps d'une ressource s'applique uniquement aux ressources déclarées. Pour configurer l'affichage des **ressources builtin** (users, droits, groupes) ou pour centraliser toute la config d'affichage, utiliser le bloc `configure {}` au niveau supérieur :

```rust
admin! {
    configure {
        users:  { list_display: [["id", "ID"], ["username", "Nom"], ["email", "Email"]] },
        droits: { list_display: [["id", "ID"], ["nom", "Nom"]] },
    }
    blog: blog::Model => BlogForm { title: "Blog" }
}
```

Le bloc `configure {}` supporte `list_display`, `list_exclude` et `list_filter`. `list_display` et `list_exclude` sont mutuellement exclusifs.

## Filtres latéraux — list_filter

Déclarer `list_filter` active une barre latérale avec les valeurs distinctes de chaque champ :

```rust
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        list_filter: [
            ["is_active", "Actif"],                     // défaut : 10 valeurs par page
            ["is_superuser", "Superuser"],              // défaut : 10 valeurs par page
            ["username", "Nom d'utilisateur", 5],       // limite explicite : 5 valeurs par page
        ]
    }
}
```

- Pour chaque colonne, le daemon génère une requête SQL qui charge les valeurs distinctes avec `LIMIT` / `OFFSET`.
- La pagination de la sidebar (`‹ 1/N ›`) est **entièrement serveur** via le paramètre `fp_{colonne}` dans l'URL — aucun JavaScript requis.
- La limite par page est configurée **par colonne** via le 3ème élément optionnel (défaut : `10`).
- Les valeurs sont affichées telles quelles (`CAST(colonne AS TEXT)`). **Ne pas utiliser `list_filter` sur des clés étrangères (FK) ou des colonnes `id`** — la valeur brute (`35`, `128`…) n'est pas lisible. Utiliser la barre de recherche à la place.
- Bons candidats : booléens, énumérations, codes courts (`lang`, `status`, `block_type`).
- Cliquer sur une valeur applique un filtre `WHERE colonne = valeur` à la requête SQL.
- Plusieurs filtres sur des colonnes différentes se cumulent (`AND`).

## Tri par colonne

Chaque en-tête de colonne est cliquable. L'URL résultante :

```text
/admin/users/list?sort_by=email&sort_dir=asc&page=1
```

- `sort_by` : nom de la colonne (doit être dans `list_display` ou être `id`)
- `sort_dir` : `asc` ou `desc`
- Changer le tri remet automatiquement à la **page 1**

La direction actuelle est indiquée par ▲ / ▼ dans l'en-tête. Cliquer une seconde fois sur la même colonne inverse le sens.

## Recherche

La barre de recherche filtre les enregistrements sur toutes les colonnes visibles via `LOWER(CAST(col AS TEXT)) LIKE LOWER('%terme%')` — compatible SQLite, Postgres et MariaDB. Elle préserve les paramètres de tri actifs.

```text
/admin/users/list?search=alice&sort_by=email&sort_dir=asc
```

Le compteur d'entrées reflète le résultat filtré.

## Pagination

La pagination est calculée côté SQL (`LIMIT` / `OFFSET`). La taille de page est configurée au niveau de l'application :

```rust
.with_admin(|a| {
    a.site_title("Administration")
     .auth(RuniqueAdminAuth::new())
     .page_size(15)   // ← entrées par page (liste ET historique)
})
```

`page_size` s'applique à la **vue liste des ressources** et à la **vue historique** (`/admin/history`). La valeur par défaut est `10`.

Les filtres et la recherche actifs sont préservés dans les liens de pagination.

## Paramètres d'URL

| Paramètre | Valeur | Description |
| --- | --- | --- |
| `page` | entier ≥ 1 | Page courante (défaut : 1) |
| `sort_by` | nom de colonne | Colonne de tri |
| `sort_dir` | `asc` \| `desc` | Direction du tri (défaut : `asc`) |
| `search` | chaîne | Terme de recherche |
| `filter_{colonne}` | valeur | Filtre actif sur une colonne |
| `fp_{colonne}` | entier ≥ 0 | Page courante du groupe de filtre sidebar (0-indexé) |

Tous les paramètres sont combinables. L'ordre de priorité : filtres → recherche → tri → pagination.

## Voir aussi

- [Macro `admin!`](/docs/fr/admin/declaration-macro) — syntaxe `list_display`, `list_filter`
- [Contexte des templates](/docs/fr/admin/template/clef/context) — clés Tera de la vue liste
- [Surcharge des templates](/docs/fr/admin/template/surcharge) — personnaliser le rendu de la liste

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
