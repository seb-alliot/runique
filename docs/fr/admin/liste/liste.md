# Vue liste — Pagination, tri, recherche et filtres

La vue liste admin travaille **au SQL** : pagination, tri, recherche et filtres. Rien n'est chargé en mémoire pour paginer.

La config se déclare sur `ModelAdmin` via `.list(|l| …)`.

## Sommaire

- [Colonnes affichées](#colonnes-affichees)
- [Liens vers le détail](#liens-vers-le-detail)
- [Tri](#tri)
- [Recherche](#recherche)
- [Filtres](#filtres)
- [Actions groupées](#actions-groupees)
- [Pagination](#pagination)
- [Paramètres d'URL](#parametres-d-url)

## Colonnes affichées

```rust
.list(|l| l.display([Title, Status, Views]))
```

Sans `display`, toutes les colonnes non secrètes apparaissent (sauf comptes : username, email, is_active).

L'ordre du tableau est l'ordre d'affichage des en-têtes.

## Liens vers le détail

```rust
.list(|l| l.display([Title, Status]).links([Title]))
```

Sans `links`, la première colonne de `display` devient le lien.

## Tri

Tri par défaut (SQL) :

```rust
.list(|l| {
    l.display([Title, CreatedAt])
        .ordering([(CreatedAt, Order::Desc), (Title, Order::Asc)])
})
```

Tri cliquable depuis l'UI / l'URL — **uniquement** les colonnes déclarées :

```rust
.sortable([Title, Status])
```

Tout autre `?sort_by=` est ignoré.

## Recherche

```rust
.list(|l| {
    l.search([Title, Summary])
        .search_help("Titre ou résumé")
})
```

Comportement :

- minimum **3** caractères (sinon la recherche est ignorée)
- longueur max bornée par `.search_max_len` du staging
- sans `.search(...)` : les colonnes texte de `display` sont utilisées
- pour désactiver : passe une liste vide via l'API search si besoin métier

## Filtres

```rust
.list(|l| l.filter([Status, Lang]))
```

Chaque filtre propose les valeurs **distinctes** présentes dans le périmètre autorisé (max 100). L'URL utilise le préfixe `filter_` :

```text
?filter_status=Published&filter_lang=fr
```

## Actions groupées

```rust
.list(|l| {
    l.display([Title, Status])
        .action(Status, "Published", "Publier")
        .action(Status, "Archived", "Archiver")
})
```

L'UI envoie l'**index** de l'action (`a0`, `a1`…), jamais la valeur brute. La valeur est fixée dans le code.

Contraintes (validées au boot) :

- colonne writable à l'édition
- pas de secret, pas d'owner, pas de drapeaux de rang comptes

## Pagination

| Niveau | Méthode | Défaut |
| --- | --- | --- |
| Site | `AdminStaging::page_size(n)` | 10 |
| Modèle | `ListConfig::per_page(n)` | hérite du site |

```rust
// site
admin.page_size(15)

// modèle
.list(|l| l.per_page(30))
```

## Paramètres d'URL

| Param | Rôle |
| --- | --- |
| `page` | Numéro de page (1-based) |
| `sort_by` | Nom de colonne (doit être dans `sortable`) |
| `sort_dir` | `asc` ou `desc` |
| `search` | Terme plein-texte |
| `filter_<col>` | Valeur de filtre |

Exemple :

```text
/admin/blog/?page=2&sort_by=title&sort_dir=asc&search=runique&filter_status=Published
```

## Valeur vide

```rust
.list(|l| l.empty_value("—"))
```

Remplace les cellules vides / NULL dans le rendu liste.

## Voir aussi

| Section | Description |
| --- | --- |
| [Builder ModelAdmin](/docs/fr/admin/declaration-macro) | Syntaxe `.list` / `.form` |
| [Permissions](/docs/fr/admin/permission) | Colonnes coupées selon les droits |
| [Templates](/docs/fr/admin/template) | Surcharge du HTML liste |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
