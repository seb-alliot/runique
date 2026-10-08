# Ébauche — builder `ModelAdmin<Entity>` (refonte de l'admin)

> Statut : **ébauche de conception, rien n'est implémenté.** Cible : version majeure (rupture).
> Calqué sur le `ModelAdmin` de Django ([référence 5.2](https://docs.djangoproject.com/en/5.2/ref/contrib/admin/)).

## Pourquoi

| Aujourd'hui | Cible |
| --- | --- |
| `admin!{}` + daemon : ≈ 9 000 lignes générées par application (demo-app) | Logique écrite **une fois** dans le framework, générique |
| Chaque ressource = 9 fermetures `Arc<dyn Fn(...)>` qui répètent liste, comptage, lecture, création, mise à jour | Un fichier court par table, écrit à la main, en Rust ordinaire |
| Lignes en `serde_json::Value`, identifiants en `String`, colonnes désignées par des chaînes | Colonnes typées (`Column::Title`), vérifiées à la compilation |
| Code généré à ne pas toucher | Rien de généré : rust-analyzer navigue, complète, vérifie |

## Principes

| Principe | Conséquence |
| --- | --- |
| **Colonnes typées** | Une faute de frappe ou une colonne d'une autre table est une erreur de compilation |
| **Listes blanches uniquement**, pas d'`exclude` | Une colonne ajoutée plus tard au modèle n'apparaît nulle part tant qu'elle n'est pas déclarée |
| **Le builder est la seule porte** | Un paramètre d'URL (`?sort=`, `?filter=`) est cherché dans la liste déclarée et converti en `Column`, jamais transmis en chaîne |
| **`fields` borne l'affichage et l'écriture** | Le formulaire et le POST ne peuvent pas diverger (assignation de masse impossible) |
| **Ce qui dépend de la requête** est une closure | Équivalent des `get_*(request)` de Django : elle reçoit l'utilisateur courant |
| **Le modèle reste la source** | Type et contraintes d'un champ déclarés une fois dans `model!{}` ; le builder choisit **lesquels** apparaissent et **comment** (le partage `models.py` / `admin.py`) |

Statuts utilisés plus bas : **existe** (déjà dans l'admin actuel, à porter) · **P1** (utile tout de suite) · **plus tard** (backlog).

## Organisation des fichiers

`main.rs` ne grossit pas avec le nombre de tables.

```rust
// main.rs
RuniqueAppBuilder::new(config)
    .with_admin(admins::site)
```

```rust
// src/admins/mod.rs
mod blog;
mod contribution;

pub fn site(a: AdminSite) -> AdminSite {
    a.site_title("Administration")
        .register(blog::admin())
        .register(contribution::admin())
}
```

```rust
// src/admins/blog.rs
use crate::entities::blog::{Column::*, Entity};
use runique::prelude::*;

pub fn admin() -> ModelAdmin<Entity> {
    ModelAdmin::new()
        .list_display([Title, Email, CreatedAt])
        .search([Title, Summary])
        .fields([Title, Summary, Content, Image])
}
```

| Fichier | Rôle |
| --- | --- |
| `main.rs` | Une ligne, quel que soit le nombre de tables |
| `src/admins/mod.rs` | Sommaire : titre du site, une ligne `register` par table |
| `src/admins/<table>.rs` | La configuration d'une table (équivalent d'un `admin.py`) |

## Page liste

```rust
ModelAdmin::new()
    .list_display([Title, Email, CreatedAt])
    .list_display_links([Title])
    .sortable_by([Title, CreatedAt])
    .ordering([CreatedAt.desc()])
    .search([Title, Summary])
    .search_help("Titre ou résumé")
    .list_filter([Email])
    .per_page(50)
    .empty_value("—")
    .date_hierarchy(CreatedAt)
    .show_full_count(true)
    .list_editable([IsPublished])
```

| Méthode | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `list_display([..])` | `list_display` | existe | Colonnes de la liste |
| `list_display_links([..])` | `list_display_links` | P1 | Colonnes cliquables vers le détail (défaut : la première) |
| `sortable_by([..])` | `sortable_by` | P1 | Seules colonnes triables depuis l'URL |
| `ordering([..])` | `ordering` | existe | Tri par défaut (aujourd'hui via le `meta` du modèle) |
| `search([..])` | `search_fields` | existe | Colonnes où cherche `?search=` |
| `search_help(..)` | `search_help_text` | P1 | Texte sous la barre de recherche |
| `list_filter([..])` | `list_filter` | existe | Seuls filtres acceptés dans l'URL |
| `per_page(n)` | `list_per_page` | existe | Lignes par page |
| `empty_value(..)` | `empty_value_display` | P1 | Affichage d'une valeur vide |
| `date_hierarchy(col)` | `date_hierarchy` | plus tard | Navigation année → mois → jour |
| `show_full_count(bool)` | `show_full_result_count` | plus tard | Total complet sur une liste filtrée |
| `list_editable([..])` | `list_editable` | plus tard | Édition directe dans la liste (⊂ `fields`) |

## Actions groupées

```rust
ModelAdmin::new()
    .action(IsPublished, true, "Publier")
    .action_fn("archiver", "Archiver", |db, ids| async move {
        // …
        Ok(())
    })
```

| Méthode | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `action(col, valeur, libellé)` | `actions` | existe | Met une colonne à une valeur sur la sélection (`group_action` actuel) |
| `action_fn(clé, libellé, fn)` | `actions` | plus tard | Action libre sur les id sélectionnés |

## Formulaire (ajout / modification)

```rust
ModelAdmin::new()
    .fields([Title, Summary, Content, Image])
    .readonly([CreatedAt])
    .label(Summary, "Résumé")
    .field(Content, |f| f.rows(12))
    .form::<BlogAdminForm>()
    .fieldsets([
        ("Contenu", [Title, Summary, Content]),
        ("Média", [Image]),
    ])
    .prepopulated(Slug, [Title])
    .save_as_new(true)
    .save_on_top(true)
```

| Méthode | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `fields([..])` | `fields` | P1 | Champs affichés **et** seuls écrits par le POST |
| `readonly([..])` | `readonly_fields` | P1 | Affiché, jamais écrit |
| `label(col, ..)` | `verbose_name` / `formfield_overrides` | P1 | Surcharge le `[label]` du modèle |
| `field(col, \|f\| ..)` | `formfield_overrides` | P1 | Réglage du widget d'un champ |
| `form::<F>()` | `form` | P1 | `#[form]` écrit à la main (`clean()` métier) ; les clés écrites restent bornées par `fields` |
| `fieldsets([..])` | `fieldsets` | plus tard | Mise en page en groupes, sans effet sur ce qui est écrit |
| `prepopulated(col, [..])` | `prepopulated_fields` | plus tard | Pré-remplissage (slug depuis le titre) |
| `save_as_new(bool)` | `save_as` | plus tard | Bouton « enregistrer comme nouveau » |
| `save_on_top(bool)` | `save_on_top` | plus tard | Boutons d'enregistrement en haut aussi |

## Relations

```rust
ModelAdmin::new()
    .autocomplete([AuthorId])
    .inline::<comment::Entity>(|i| i
        .fk(comment::Column::BlogId)
        .fields([comment::Column::Body]))
```

| Méthode | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `autocomplete([..])` | `autocomplete_fields` | plus tard | Recherche au lieu d'une liste déroulante pour une clé étrangère |
| `inline::<E>(..)` | `inlines` | plus tard | Édition des enfants dans le formulaire du parent |

## Accès

```rust
ModelAdmin::new()
    .owner(AuthorId)
    .queryset(|q, user| {
        if user.is_superuser {
            q
        } else {
            q.filter(Region.eq(user.region()))
        }
    })
    .can_delete(|user, _obj| user.is_superuser)
```

| Méthode | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `owner(col)` | — | existe | Colonne propriétaire pour `can_update_own` / `can_delete_own` (`own_field` actuel) |
| `queryset(\|q, user\| ..)` | `get_queryset` | P1 | Filtre métier appliqué partout : liste, détail, édition, actions (backlog n°38) |
| `can_delete(\|user, obj\| ..)` | `has_delete_permission` | plus tard | Règle en plus des droits de groupe, jamais à leur place |

## Hooks et templates

```rust
ModelAdmin::new()
    .before_save(|model, user, is_new| async move { Ok(model) })
    .after_delete(|id, user| async move { Ok(()) })
    .list_template("admin/blog_list.html")
```

| Méthode | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `before_save(..)` | `save_model` | plus tard | Avant l'enregistrement (création ou modification) |
| `after_delete(..)` | `delete_model` | plus tard | Après une suppression |
| `list_template(..)` | `change_list_template` | existe en partie | Surcharge du template de liste |

## Vérifié par `register()` au démarrage

Le serveur refuse de démarrer si une de ces règles est violée :

| Règle | Pourquoi |
| --- | --- |
| Aucune colonne de mot de passe ou de hash dans `list_display`, `search`, `list_filter`, `fields`, `readonly` | Jamais affichée, cherchée ni écrite par l'admin |
| `is_superuser` jamais dans `fields` ni `list_editable` | Un compte superuser ne s'accorde pas depuis un formulaire |
| `readonly`, `list_editable`, `fieldsets`, `field()`, `label()` ⊂ `fields` | Sinon on configure un champ absent du formulaire |
| `list_display_links`, `sortable_by` ⊂ `list_display` | Sinon on lie ou trie une colonne non affichée |
| Pas de colonne à la fois dans `list_editable` et `list_display_links` | Règle Django : une cellule ne peut être lien et champ |

## Volontairement absent

| Django | Pourquoi pas ici |
| --- | --- |
| `exclude` | Liste noire : une colonne ajoutée plus tard fuirait par oubli |
| Colonnes de liste en chaîne, ou fonction quelconque | Remplacé plus tard par une colonne calculée typée : `.computed("auteur", \|row: &Model\| row.author_name.clone())` |
| Macro procédurale `admin!{}` | Écartée pour garder rust-analyzer |
