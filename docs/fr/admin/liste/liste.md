# Vue liste — Pagination, tri, recherche et filtres

La vue liste admin gère toutes ses opérations **au niveau SQL** : pagination, tri par colonne, recherche plein-texte et filtres par valeur. Rien n'est chargé en mémoire.

## Sommaire

- [Colonnes affichées — list_display](#colonnes-affichées--list_display)
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

La barre de recherche filtre les enregistrements sur toutes les colonnes texte visibles (`ILIKE '%terme%'`). Elle préserve les paramètres de tri actifs.

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
     .page_size(15)   // ← entrées par page
})
```

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

- [Macro `admin!`](/docs/fr/admin/declaration) — syntaxe `list_display`, `list_filter`
- [Contexte des templates](/docs/fr/admin/template/clef/context) — clés Tera de la vue liste
- [Surcharge des templates](/docs/fr/admin/template/surcharge) — personnaliser le rendu de la liste

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
