# Builder `ModelAdmin`

API typée pour déclarer la liste et le formulaire d'un modèle. Les colonnes sont des variantes d'enum SeaORM (`Column::Title`) : une faute de frappe ne compile pas.

## Squelette

```rust
fn articles() -> ModelAdmin<article::Entity> {
    use article::Column::*;
    ModelAdmin::new()
        .title("Articles")
        .list(|l| {
            l.display([Title, Status, Views])
                .links([Title])
                .sortable([Title, Status])
                .ordering([(Title, Order::Asc)])
                .search([Title, Summary])
                .search_help("Titre ou résumé")
                .filter([Status])
                .per_page(15)
                .empty_value("—")
                .action(Status, "Published", "Publier")
                .action(Status, "Archived", "Archiver")
        })
        .form(|f| {
            f.fields([Title, Summary, Body, Status, Views])
                .readonly([Views])
                .create_only([Title])
                .labels([(Summary, "Résumé"), (Body, "Contenu")])
        })
}
```

Le `use …::Column::*` reste **dans** la fonction du modèle : deux modèles avec une colonne `Title` ne se marchent jamais dessus.

## Méthodes de `ModelAdmin`

| Méthode | Rôle |
| --- | --- |
| `.title("…")` | Nom dans la nav et les pages (sinon verbose_name / table) |
| `.list(|l| …)` | Configure la vue liste |
| `.form(|f| …)` | Configure create / edit |
| `.templates(|t| …)` | Surcharge HTML de ce modèle uniquement |

## Liste — `ListConfig`

| Méthode | Effet |
| --- | --- |
| `.display([Col, …])` | Colonnes visibles, dans l'ordre |
| `.links([Col, …])` | Colonnes cliquables vers le détail (défaut : première de display) |
| `.sortable([Col, …])` | Seules colonnes acceptées via `?sort_by=` |
| `.ordering([(Col, Order::Asc), …])` | Tri SQL par défaut |
| `.search([Col, …])` | Colonnes plein-texte (`None` = textes de display ; `Some([])` via search vide désactive) |
| `.search_help("…")` | Texte d'aide sous la barre de recherche |
| `.filter([Col, …])` | Filtres latéraux (`?filter_status=…`) |
| `.per_page(n)` | Taille de page pour ce modèle |
| `.empty_value("—")` | Remplace les valeurs vides |
| `.action(Col, value, "Label")` | Action groupée : fixe `Col` à `value` sur les lignes cochées |

### Actions groupées

```rust
.list(|l| {
    l.display([Title, Status])
        .action(Status, "Published", "Publier")
        .action(Status, "Archived", "Archiver")
})
```

Règles au boot :

- la colonne d'action doit être **writable** à l'édition (dans `fields`, pas `readonly` / `create_only`)
- interdites sur les secrets, le owner, et les drapeaux de rang comptes (`is_superuser`, `is_dirigeant`, `is_admin`, `is_active`)

## Formulaire — `FormConfig`

| Méthode | Effet |
| --- | --- |
| `.fields([Col, …])` | Champs affichés, dans l'ordre |
| `.readonly([Col, …])` | Visibles mais non modifiables |
| `.create_only([Col, …])` | Saisissables à la création, figés ensuite |
| `.labels([(Col, "Libellé"), …])` | Libellés custom |

```rust
.form(|f| {
    f.fields([Slug, Title, Body, ViewCount])
        .create_only([Slug])
        .readonly([ViewCount])
        .labels([(ViewCount, "Vues")])
})
```

Sans `.form()`, tous les champs du modèle (hors secrets / rangs) sont utilisés. Le gate décide encore ce que le compte courant peut lire ou écrire.

## Comptes étendus

Pour exposer les colonnes ajoutées par `extend!` sur `eihwaz_users` :

```rust
fn accounts() -> ModelAdmin<user_profile::Entity> {
    use user_profile::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Username, Email, IsActive, Website])
                .search([Username, Email])
        })
        .form(|f| f.fields([Username, Email, Bio, Website, Phone]))
}

// dans site() :
site.accounts_users(accounts())
```

Les drapeaux de rang (`is_superuser`, `is_dirigeant`, `is_admin`) ne passent **jamais** par le formulaire : le gate les gère.

## Secrets

Les colonnes marquées secrètes sur le modèle (mot de passe, tokens…) sont refusées dans display, search, filter, sortable, links, fields, actions. Le message d'erreur apparaît au `validate()` de boot.

## Exemple réel (demo-app)

```rust
fn blog() -> ModelAdmin<blog::Entity> {
    use blog::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Title, Email, Website, Status])
                .links([Title])
                .sortable([Title, Status])
                .ordering([(Title, Order::Asc)])
                .search([Title, Summary])
                .filter([Status])
                .empty_value("—")
                .action(Status, "Published", "Publier")
                .action(Status, "Archived", "Archiver")
        })
        .form(|f| {
            f.fields([Title, Email, Website, Summary, Content, Status, ViewCount])
                .readonly([ViewCount])
        })
}
```

## Voir aussi

| Section | Description |
| --- | --- |
| [Vue liste](/docs/fr/admin/liste) | Comportement runtime de la liste |
| [Validation au boot](/docs/fr/admin/declaration-daemon) | Contrôles automatiques |
| [Déclaration](/docs/fr/admin/declaration) | AdminStaging et AdminSite |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
