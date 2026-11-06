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

Sans `display`, toutes les colonnes non secrètes apparaissent (sauf comptes : username, email, is_active), avec les dates et les heures en dernier. Un `display` déclaré garde son ordre. La fiche d'une ligne reprend cet ordre, mais y met toujours les dates et les heures à la fin.

L'ordre du tableau est l'ordre d'affichage des en-têtes.

## Liens vers le détail

```rust
.list(|l| l.display([Title, Status]).links([Title]))
```

**Sans effet pour l'instant** : `links` est accepté et vérifié (ses colonnes doivent être dans `display`), mais la liste affiche chaque ligne en carte avec son propre lien vers le détail (et son menu), quelles que soient les colonnes déclarées.

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
.list(|l| l.search([Title, Summary]))
```

Comportement :

- minimum **3** caractères (sinon la recherche est ignorée)
- au-delà de `.search_max_len(n)` du staging (150 par défaut), le terme est tronqué
- sans `.search(...)` : les colonnes texte de `display` sont utilisées
- `.search([])` désactive la recherche
- `.search_help("…")` est accepté mais pas encore affiché

## Filtres

```rust
.list(|l| l.filter([Status, Lang]))
```

Chaque filtre propose les valeurs **distinctes** présentes dans le périmètre autorisé, page par page. L'URL utilise le préfixe `filter_` :

```text
?filter_status=Published&filter_status=Draft&filter_lang=fr
```

Plusieurs valeurs d'une colonne s'additionnent (`Published` **ou** `Draft`), plusieurs colonnes se cumulent (**et**). Cliquer sur une valeur l'active ; cliquer de nouveau la retire, les autres restent. Une colonne accepte au plus 100 valeurs depuis l'URL.

Chaque groupe du panneau a ses flèches, et un seul sélecteur, sous le titre du panneau, règle le nombre de valeurs de tous les groupes :

| Niveau | Méthode | Défaut |
| --- | --- | --- |
| Site | `AdminStaging::filter_size(n)` | 10 |
| Modèle | `ListConfig::filter_size(n)` | hérite du site |

```rust
// site
admin.filter_size(8)

// modèle
.list(|l| l.filter([PageId]).filter_size(15))
```

Le choix fait dans le sélecteur (10, 25, 50 ou 100) l'emporte sur ces réglages.

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

Le sélecteur de la liste (10, 25, 50 ou 100) l'emporte sur ces réglages. L'URL ne peut pas demander une autre taille.

## Paramètres d'URL

| Param | Rôle |
| --- | --- |
| `page` | Numéro de page (1-based) |
| `sort_by` | Nom de colonne (doit être dans `sortable`) |
| `sort_dir` | `asc` ou `desc` |
| `search` | Terme plein-texte |
| `per_page` | Lignes par page (10, 25, 50 ou 100) |
| `filter_<col>` | Valeur de filtre, répétable |
| `fpage_<col>` | Page de valeurs du groupe de filtre `<col>` |
| `fsize` | Valeurs par page des groupes de filtre (10, 25, 50 ou 100) |

Exemple :

```text
/admin/blog/list?page=2&sort_by=title&sort_dir=asc&search=runique&filter_status=Published
```

## Valeur vide

```rust
.list(|l| l.empty_value("—"))
```

**Sans effet pour l'instant** : accepté, mais la liste n'utilise pas encore ce texte.

## Voir aussi

| Section | Description |
| --- | --- |
| [Builder ModelAdmin](/docs/fr/admin/declaration-macro) | Syntaxe `.list` / `.form` |
| [Permissions](/docs/fr/admin/permission) | Colonnes coupées selon les droits |
| [Templates](/docs/fr/admin/template) | Surcharge du HTML liste |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
