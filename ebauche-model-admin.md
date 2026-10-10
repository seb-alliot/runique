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

### Brique à poser en premier : une seule recherche typée

Aujourd'hui, la même condition de recherche (`LOWER(CAST(col AS TEXT)) LIKE '%terme%'`, type de conversion selon le moteur, valeur en paramètre lié) existe en plusieurs copies :

| Où | Forme |
| --- | --- |
| `search_cond!` (`runique/src/macros/bdd/filter.rs`), public | Macro, colonnes en chaîne ou `all_columns` |
| `ilike(db, col, pattern)` (`runique/src/admin/helper/sql_dialect.rs`) | Fonction, colonne en `&str` |
| `runique/src/admin/builtin/user.rs` et `groupe.rs` | La même boucle copiée 4 fois |
| Code généré par le daemon (`write_search_conditions`) | Une copie par ressource |

Cible : une fonction générique et typée, seule implémentation.

```rust
pub fn search_condition<E: EntityTrait>(db: &ADb, cols: &[E::Column], term: &str) -> Condition {
    let pattern = format!("%{}%", term.to_lowercase());
    cols.iter().fold(Condition::any(), |cond, col| {
        cond.add(Expr::expr(Func::lower(Expr::col(*col).cast_as(text_cast_type(db)))).like(&pattern))
    })
}
```

| Utilisateur | Appel |
| --- | --- |
| Builder | `search_condition::<E>(db, &self.search, term)` : sa liste blanche, et sans `.search(..)` les colonnes texte de `list_display` (voir **Décisions › Recherche**) |
| `search_cond!` (vues publiques des devs) | Fine couche au-dessus ; `all_columns` passe `E::Column::iter()` |
| Ressources intégrées (utilisateurs, groupes) | Les 4 copies disparaissent |

Les détails délicats (conversion par moteur, minuscules, paramètre lié) ne vivent plus qu'à un endroit : un correctif s'applique partout d'un coup. Faisable avant le reste, sans rupture.

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

## Ce que ça engendre

Carte des conséquences, côté framework puis côté dev. Tailles relevées le 2026-10-09.

### Disparaît

| Fichier | Lignes | Remplacé par |
| --- | --- | --- |
| `runique/src/admin/daemon/generator.rs` | 1 354 | Le code générique, monomorphisé par le compilateur |
| `runique/src/admin/daemon/parser.rs` | 1 101 | Rien : plus de DSL `admin!{}` à lire |
| `runique/src/admin/daemon/watcher.rs` | 28 | Rien : plus de fichier à régénérer |
| `runique/src/admin/resource.rs` (DSL `admin!{}`) | 296 | Le builder `ModelAdmin` |
| `demo-app/src/admins/admin.rs` (généré) | 9 155 | Un fichier court par table, écrit à la main |

### Change

| Fichier | Lignes | Ce qui change |
| --- | --- | --- |
| `runique/src/admin/helper/resource_entry.rs` | 317 | Les 9 fermetures (`ListFn`, `GetFn`, `CountFn`…) deviennent un trait `AdminResource` |
| `runique/src/admin/admin_main/handle_*.rs` | ≈ 1 850 | Appellent le trait au lieu des fermetures ; la logique (pagination, retours, messages) reste |
| `runique/src/admin/builtin/user.rs`, `groupe.rs`, `droit.rs` | 1 143 | Réécrits avec le builder (ou le trait directement pour les cas particuliers) |
| `runique/src/admin/registry.rs` | — | Stocke des `Box<dyn AdminResource>` |
| `derive_form` (`model!{}`) | — | Ajoute `impl ModelMeta for Entity`, un trait sur l'entité existante (comme `HasLists`), pas un second modèle |

### Reste tel quel

| Élément | Pourquoi |
| --- | --- |
| `admin_main/gate.rs` (qui peut faire quoi, quelles clés) | Indépendant de la façon dont la ressource est déclarée |
| Droits de groupe (`eihwaz_groupes_droits`), CSRF, connexion admin | Inchangés |
| Templates admin, historique, actions groupées | Mêmes données, typées en amont |
| `model!{}` côté dev | Rien à réécrire dans les modèles |

### Côté dev (projet qui passe à cette version)

| Avant | Après |
| --- | --- |
| Un bloc `admin!{}` dans `src/admin.rs` | Le dossier `src/admins/`, écrit à la main : un fichier par table + `mod.rs` |
| `runique start` lance le daemon | Plus de daemon |
| `src/admins/` généré, à ne pas toucher | Même dossier, mais c'est le dev qui l'écrit |
| `.with_admin(\|a\| a.routes(admins::routes("/admin")))` | `.with_admin(admins::site)` |

Aide envisagée : une commande qui convertit un `admin!{}` existant en fichiers `src/admins/<table>.rs`.

L'ordre de livraison est dans **Jalons**, en fin de document.

## Vérifié par `register()` au démarrage

Le serveur refuse de démarrer si une de ces règles est violée. C'est la liste unique : toute règle de démarrage évoquée ailleurs dans ce document doit figurer ici.

| Règle | Pourquoi |
| --- | --- |
| Aucune colonne secrète dans `list_display`, `search`, `list_filter`, `fields`, `readonly`, `action` | Jamais affichée, cherchée, filtrée ni écrite par l'admin |
| `is_superuser` jamais dans `fields`, `list_editable` ni `action` | Un compte superuser ne s'accorde pas depuis un formulaire ni par lot |
| Chaque colonne obligatoire sans défaut est dans `fields` ou remplie automatiquement (clé primaire, `auto_now`, défaut) | Sinon la création échoue en base, devant l'utilisateur |
| `autocomplete`, `m2m` et `inline` visent une entité elle aussi enregistrée | Sinon les droits de la cible sont introuvables (règle M2M n°1) |
| `readonly`, `list_editable`, `fieldsets`, `field()`, `label()` ⊂ `fields` | Sinon on configure un champ absent du formulaire |
| `list_display_links`, `sortable_by` ⊂ `list_display` | Sinon on lie ou trie une colonne non affichée |
| Pas de colonne à la fois dans `list_editable` et `list_display_links` | Règle Django : une cellule ne peut être lien et champ |

**Source des vérifications** : `runique_dsl` connaît chaque modèle en entier. `model!{}` génère `impl ModelMeta for Entity` (un trait sur l'entité existante, pas un nouveau modèle ; nom générique car ces faits décrivent le modèle, pas l'admin ; le DSL les donne déjà aux formulaires, `ModelMeta` les expose depuis la même source, sans seconde description) avec tout ce dont `register` a besoin : colonnes obligatoires sans défaut, colonnes secrètes, clés étrangères et leur cible, relations M2M. Le builder lit ces listes, il ne déduit rien lui-même.

**Colonne secrète** : un champ de type `password` dans `model!{}`. Le DSL la publie dans `ModelMeta` (`const SECRET: &[Column]`), le builder n'a pas à la deviner par son nom. Un hash stocké sous un autre type (jeton, clé d'API) se déclare avec un attribut du DSL, à nommer.

## Volontairement absent

| Django | Pourquoi pas ici |
| --- | --- |
| `exclude` | Liste noire : une colonne ajoutée plus tard fuirait par oubli |
| Colonnes de liste en chaîne, ou fonction quelconque | Remplacé plus tard par une colonne calculée typée : `.computed("auteur", \|row: &Model\| row.author_name.clone())` |
| Macro procédurale `admin!{}` | Écartée pour garder rust-analyzer |

## Leçons des failles de la 3.0.3

**Une opération d'écriture fait une seule chose.** La porte autorise selon le nom de l'opération ; une fonction d'écriture qui en fait une autre échappe à la vérification. Exemple corrigé en 3.0.3 — la création générée pour `bulk_create` mettait à jour la ligne existante :

```rust
// À NE PAS REPRODUIRE : un update caché dans une création.
// La porte n'a vérifié que le droit de *créer*.
if let Some(id) = existing_id {
    admin_from_form(&row, Some(id))?.update(&db).await?;   // modification sans droit de modification
} else {
    admin_from_form(&row, None)?.insert(&db).await?;
}
```

Pour le builder : chaque opération a sa propre fonction d'écriture, et une seule. Un « upsert », s'il revient, est une opération à part, qui exige les droits de création **et** de modification, et passe par `member_gate` pour chaque ligne existante.

**Protéger l'objet, pas seulement le champ.** Le champ `is_superuser` était intouchable depuis l'admin, mais le **compte** d'un superutilisateur (email, réinitialisation, suppression) restait modifiable par un staff de rang inférieur. La règle vit maintenant dans la porte (`protects_a_superuser`) ; le builder doit la conserver telle quelle, comme `GateCtx` et `GateAuthorization`.

**Une modification ne touche que ce que le formulaire montre.** `update_fn` écrit avec `admin_partial_update` : une colonne absente du formulaire reste intacte au lieu d'être remise à zéro.

## Décisions de conception (2026-10-10)

### Structure

- Un builder maître, `AdminSite`, qui enregistre des builders enfants : `.register(ModelAdmin::<plat::Entity>::new() …)`.
- Chaque `ModelAdmin<E>` est tiré de l'entité SeaORM générée par `model!` ; la clé de ressource et ses droits se déduisent de la table.
- **Colonnes typées** : `plat::Column::Nom`, jamais une chaîne. Une colonne inconnue ne compile pas.
- **Le code décrit, l'administrateur autorise** : tout ce qui est paramétrable (colonnes, filtres, recherche, formulaires, actions) est dans le builder ; les groupes et droits restent en base, gérés depuis l'interface.
- Toute requête passe par SeaORM (entités, `sea_query` typé) ou `search!` / `search_cond!` — jamais de SQL brut.

### Composition du builder

`ModelAdmin<E>` est composé d'une struct par responsabilité. L'API passe par une closure par bloc, comme le reste du builder Runique (`.middleware(|m| ..)`, `.with_csp(|c| ..)`). Les exemples à plat des sections précédentes (`.list_display(..)`, `.fields(..)`) suivent la nomenclature de Django ; leur forme réelle est celle-ci :

```rust
ModelAdmin::new()
    .list(|l| l.display([Title, CreatedAt]).search([Title]).per_page(50))
    .form(|f| f.fields([Title, Summary]).readonly([CreatedAt]))
    .action(IsPublished, true, "Publier")
```

```rust
pub struct ModelAdmin<E: EntityTrait + ModelMeta> {
    list: ListConfig<E>,
    form: FormConfig<E>,
    actions: Vec<GroupAction<E>>,
    access: AccessConfig<E>,
}

pub struct ListConfig<E: EntityTrait> {
    display: Vec<E::Column>,
    links: Vec<E::Column>,
    sortable: Vec<E::Column>,
    ordering: Vec<(E::Column, Order)>,
    search: Option<Vec<E::Column>>,
    filters: Vec<E::Column>,
    per_page: u64,
}

pub struct FormConfig<E: EntityTrait> {
    fields: Vec<E::Column>,
    readonly: Vec<E::Column>,
    labels: Vec<(E::Column, String)>,
}

pub struct AccessConfig<E: EntityTrait> {
    owner: Option<E::Column>,
    queryset: Option<Arc<dyn Fn(Select<E>, &CurrentUser) -> Select<E> + Send + Sync>>,
}

pub struct GroupAction<E: EntityTrait> {
    column: E::Column,
    value: sea_orm::Value,
    label: String,
}
```

| Choix | Contrainte d'où il vient |
| --- | --- |
| Une struct par bloc | Chaque handler ne reçoit que sa partie (`handle_list` prend `&ListConfig<E>`) ; une option nouvelle s'ajoute dans un seul bloc |
| `E::Column` partout, jamais `String` | Colonnes typées |
| `Vec` plutôt que `HashSet` | L'ordre déclaré est l'ordre affiché |
| `search: Option<Vec<..>>` | `None` (colonnes texte affichées) ≠ `Some(vec![])` (recherche désactivée) |
| `ordering` en `(colonne, Order)` | Le sens du tri est déclaré, pas déduit d'un `-` dans une chaîne |
| `labels` en paires | Surcharge le libellé du DSL ; `HashMap` exigerait `Column: Hash`, non garanti par SeaORM |
| `GroupAction` = une colonne + une valeur | Une action écrit une colonne déclarée, jamais une colonne venue de la requête |
| `queryset` en `Arc<dyn Fn + Send + Sync>` | Dépend de l'utilisateur ; partagé entre requêtes sous Axum |

Validation : `ListConfig::validate()` et `FormConfig::validate()` pour les règles internes à un bloc, `ModelAdmin::validate()` pour celles qui en croisent plusieurs. `register` les appelle toutes. Les options « plus tard » entrent dans leur bloc au moment où elles sont livrées.

`per_page` : branché directement sur le réglage existant de l'admin, même valeur par défaut.

### Plancher de sécurité de Runique

Exclusions fixées par le framework, ni configurables ni contournables par un builder. Le dev déclare en liste blanche par-dessus.

| Niveau | Exemples | Appliqué par | Visibilité |
| --- | --- | --- | --- |
| Table | `eihwaz_sessions`, `eihwaz_reset_tokens` : jamais enregistrables | `register` refuse de démarrer | `pub(crate)` dans runique |
| Colonne du framework | `is_superuser` : jamais dans `fields`, `list_editable`, `action` | `register` refuse de démarrer | `pub(crate)` dans runique |
| Colonne secrète du dev | champs `password` d'un `model!{}` : jamais lus, cherchés, filtrés ni écrits | `register` + projection | `ModelMeta` (`pub`, généré) |
| Objet | compte d'un superuser, modifiable par un superuser seulement | la porte, à chaque requête (`protects_a_superuser`) | `pub(crate)` dans runique |

Pourquoi deux visibilités : les tables `eihwaz_*` et `is_superuser` appartiennent à Runique, la liste vit donc dans le framework, en `pub(crate)`, hors d'atteinte du dev. Les secrets d'un modèle du dev ne sont connus que de son `model!{}`, qui génère `impl ModelMeta` dans **son** crate : ce trait est forcément `pub`, et un `impl` écrit à la main pourrait mentir. Le plancher ne repose donc jamais sur `ModelMeta` pour ce qui appartient à Runique.

### Handlers admin sur `Request` (décidé 2026-10-10)

- Le router admin est déjà fusionné dans celui du dev (`app/builder/build.rs`). Ce qui change : les handlers admin prennent `Request` comme les vues du dev, au lieu d'extraire `Extension<AdminState>` et `Extension<CurrentUser>` à la main.
- `request.user` donne l'utilisateur, `ResourcePerms::resolve(&user, key)` ses droits, la porte décide, `Authorized<Op, E>` prouve qu'elle a décidé.
- **CSRF** : l'admin abandonne sa comparaison brute (`ct_eq`) et passe par Prisme. Un seul mécanisme, testé des deux côtés (jeton valide accepté, invalide refusé).
- **Défense en profondeur par des couches de nature différente**, pas par deux copies du même CSRF :

| Couche | Indépendante de Prisme | État |
| --- | --- | --- |
| Cookie de session `SameSite=Strict` | Oui | existe (`middleware_staging`) |
| Vérification `Origin` sur les POST admin | Oui | à ajouter |
| Connexion admin séparée + `LoginGuard` | Oui | existe |
| La porte + `Authorized<Op, E>` | Oui | porte existante, jeton à ajouter |

### JavaScript des formulaires

| Source | Dans l'admin |
| --- | --- |
| JS d'un type de champ (dans son template, ex. `base_color.html`) | Toujours présent : il fait partie du champ |
| `add_js` du formulaire du modèle | Repris par défaut |
| `.form(\|f\| f.surcharge_js(["js/plat_admin.js"]))` | **Remplace** le `add_js` du formulaire de ce modèle par le JS du dev ; `.surcharge_js([])` le retire |

- **Portée : uniquement le formulaire du modèle déclaré.** `.surcharge_js(..)` sur `ModelAdmin<plat>` ne touche ni le formulaire d'un inline, ni le widget d'un M2M, ni un autre modèle (utilisateur…) : chacun garde son JS, ou celui de son propre `ModelAdmin`. Pas d'effet en cascade.
- **Ce que `surcharge_js` ne touche jamais : le JS servi par Runique.** Runique a ses propres chemins statiques (`static_runique_path` / `static_runique_url`, filtre `runique_static`), distincts de ceux du dev (`staticfiles_dirs` / `static_url`, filtre `static`). L'origine est portée par le type dès la déclaration (`enum JsSource { Runique(..), App(..) }`), jamais déduite d'un préfixe de chemin : un dossier du dev qui porterait le même nom ne doit pas changer le résultat. `surcharge_js` ne remplace que les entrées `App`. Conséquence voulue : le JS de sécurité de Runique (`static/js/csrf.js`) ne peut être ni retiré ni remplacé par une surcharge.
- **Changer le JS d'un champ de Runique** (ex. le sélecteur de couleur) : le dev fournit son propre template de champ (`template_name` de `FieldConfig`), avec son HTML et son JS. Jamais un script seul, qui casserait le champ en attendant le HTML de Runique.
- Même validation que `add_js` (`validate_js_path`), nonce pris dans la `Request`.
- La page assemble les listes (champs, formulaire principal, inlines) en une seule, dédoublonnée dans l'ordre, rendue une fois par `js.html`.

### Garanties par le type (ce que Django ne peut pas faire)

Django vérifie à l'exécution des méthodes qu'on peut oublier (`has_change_permission`…). Ici, le chemin dangereux ne compile pas : la règle est portée par un type dont seul le framework a le constructeur (`pub(crate)`).

**Une seule porte d'entrée.** La porte existante (`GateCtx`, `GateAuthorization`, `member_gate`, `bulk_gate`) est la seule fabrique de :

```rust
/// Une ligne précise, vérifiée pour une opération précise.
pub struct Authorized<Op, E> { id: PkOf<E>, _op: PhantomData<Op> }

/// Une requête sur plusieurs lignes (liste, action groupée).
pub enum Scope<E> {
    All,                    // can_read / can_update…
    Own(E::Column, UserId), // can_*_own : WHERE owner = user.id ajouté à la requête
}
```

| Cas | La porte vérifie | Elle délivre |
| --- | --- | --- |
| Ligne, droit complet | `request.user` → groupes → `can_update` | `Authorized<Update, E>` |
| Ligne, droit « own » | groupes → `can_update_own` **et** propriétaire de la ligne = `user.id`, lu en base | le même `Authorized<Update, E>` |
| Liste, action groupée | droit complet ou « own » | `Scope::All` ou `Scope::Own` |

- Les fonctions d'écriture exigent le jeton de **leur** opération : `update` prend `Authorized<Update, E>`, `delete` prend `Authorized<Delete, E>`. Un jeton de création ne modifie rien (la faille `bulk_create` de la 3.0.3 aurait été une erreur de compilation).
- Un seul type de jeton par ligne : une fois la porte passée, droit complet et droit « own » autorisent la même écriture. La distinction ne vit que dans `Scope`, là où elle change la requête.
- La protection du compte superuser reste une règle de la porte : elle refuse simplement de délivrer le jeton.

**Structure de la porte** (décidé 2026-10-10) : une fonction privée, des façades publiques, un enum pour la logique, un type par opération pour la preuve.

```rust
pub enum Operation { Create, Read, Update, Delete }

pub trait Op { const KIND: Operation; }
impl Op for Update { const KIND: Operation = Operation::Update; } // idem Create, Read, Delete

// La seule porte. Chaque variante a sa logique.
pub(crate) async fn gate<O: Op, E>(ctx: &GateCtx<'_>, auth: &GateAuthorization<'_>, id: Option<PkOf<E>>)
    -> Result<Authorized<O, E>, Refus>
{
    match O::KIND {
        Operation::Create => { /* droit de création */ }
        Operation::Read   => { /* lecture, jamais bloquée par la protection superuser */ }
        Operation::Update => { /* droit ou own + propriétaire en base + protection superuser */ }
        Operation::Delete => { /* idem + message clair si une clé étrangère bloque */ }
    }
    Ok(Authorized::new(id))
}

// Façades publiques, une par opération : renvoient une preuve, jamais un bool.
pub async fn authorize_update<E>(ctx, auth, id) -> Result<Authorized<Update, E>, Refus>;
```

| Élément | Rôle |
| --- | --- |
| Type `Update` | Ce que la preuve autorise, vérifié par le compilateur |
| `Operation::Update` | La logique propre à l'opération (`match` exhaustif : une opération ajoutée exige sa branche) |
| `Op::KIND` | Lie les deux : `gate::<Update>` exécute forcément la branche `Update` |

**Écriture via une extension trait sur SeaORM** : `insert_with(preuve: Authorized<Create, E>, db)`, `update_with(preuve: Authorized<Update, E>, db)`, `delete_with(preuve: Authorized<Delete, E>, db)`. Une création qui appellerait `update_with` ne compile pas. Une requête incohérente (POST sur `/create` avec un `id` existant, POST sur `/edit` sans ligne) est refusée par la porte avec un message général (« action impossible »), sans dire ce qui existe en base. L'ORM n'est pas bridé (pas de `disallowed-methods`) : l'admin de Runique passe par ce chemin, le code du dev reste libre.

**Signature de l'opération (décidé 2026-10-10)** : le middleware admin déduit l'opération de la route (`/create` → `Create`, `/{id}/edit` → `Update`…) et la signe dans les extensions de la requête ; le handler ne choisit pas sa signature. Le point d'écriture unique (`AdminResource::write`) compare la signature à l'écriture demandée **avant** d'appeler SeaORM ; incohérence → « action impossible ». La vérification n'est pas mise dans le modèle (`ActiveModelBehavior`), partagé avec le code du dev. Écarté : vérifier après écriture (rapport `Inserted`/`Updated` puis annulation) — fonctionnel sous transaction, mais écrit pour rien et consomme des identifiants.

Page par page : détail → `Read` ; formulaire d'édition (GET) et son envoi (POST) → `Update` (afficher le formulaire demande déjà le droit de modifier, comme Django) ; ajout → `Create` ; suppression → `Delete` ; liste et actions groupées → `Scope`.

**Secrets non sérialisables.** Le DSL type les champs `password` en `Secret<String>`, sans `Serialize` ni `Display` : dans un contexte Tera, une réponse JSON ou un `format!`, ça ne compile pas. Faisable avant la refonte, dans le DSL.

**Ligne affichée typée.** Le template reçoit des `ShownRow<E>`, construits uniquement depuis `Projection::shown`. Les colonnes lues pour la porte (`server_only`) n'ont aucun chemin vers le template.

### Colonnes non affichées

- **Jamais lues pour la vue** : `select_only().columns([...])` ne sélectionne que les colonnes affichées. Une colonne qui n'est pas lue ne peut fuiter ni dans un template, ni dans une réponse HTMX, ni dans un export.
- Les colonnes nécessaires aux contrôles côté serveur (propriétaire pour `_own`, clé du parent, `updated_at` du verrouillage optimiste) peuvent être lues pour la porte, **jamais transmises au template**.
- **Jamais envoyées ni modifiées** : absentes du formulaire, donc des données, donc `NotSet` en modification.
- **Vérification au démarrage** : à `register`, chaque colonne obligatoire sans valeur par défaut (`Column::def()`) doit être dans le formulaire de création ou remplie automatiquement (clé primaire, `auto_now`, défaut). Sinon l'application refuse de démarrer : « plat : la colonne `prix` est obligatoire mais absente du formulaire de création ».

### Identifiants

- L'id de l'URL est converti dans le type de la clé primaire (`<E::PrimaryKey as PrimaryKeyTrait>::ValueType`) avant toute requête. Un id mal formé donne un 404 sans toucher la base ; plus de `String` qui traverse les handlers.
- Les id d'une action groupée suivent la même conversion. Un seul id invalide refuse tout le lot, comme la protection superuser.

### Écritures

- Une opération = une fonction d'écriture (leçon `bulk_create`). Création : `insert`. Modification : `update` partielle, limitée à `fields`. Suppression : `delete`. Action : `update` d'une seule colonne.
- Une suppression refusée par une clé étrangère (`restrict`) donne un message clair (« ce plat est utilisé par 3 commandes »), jamais une 500. Le correctif arrive en 3.0.x ; le builder reprend le même message.

### Recherche

- Une seule fonction générique sur `E::Column` : condition « OU » de `LOWER(col) LIKE %terme%`, en `sea_query` typé.
- **Par défaut : uniquement les colonnes texte affichées.** Chercher dans une colonne cachée en ferait un oracle (taper `$argon2` révèle quels comptes ont ce hash, caractère par caractère on reconstitue la valeur).
- `search_fields` typé pour élargir ; une colonne de mot de passe ou de secret y est refusée au démarrage.
- Une recherche globale (plusieurs modèles) ne porte que sur les ressources que l'utilisateur peut **lire**, et sur leurs colonnes autorisées.

### Many-to-many

**Dépendance** : le M2M de l'admin s'appuie sur le M2M du DSL (chantier 3.x en tête de la ROADMAP) — syntaxe dans `model!`, table pivot et migration générées par makemigrations (suppression en cascade déclarée en base), relation typée et méthodes d'accès sur le modèle. L'admin vient après.

**Bug actuel à ne pas reproduire** : `admin!{}` déclare un M2M entièrement en chaînes (`["allergenes", "Allergènes", "plat_allergene", "plat_id", "allergene_id", …]`). Une faute de frappe, ou un M2M déclaré sur une ressource dont le modèle ne l'a pas, compile et ne casse qu'à l'exécution, devant l'utilisateur.

**Solution : la relation porte son modèle d'origine dans son type.**

```rust
// généré par le DSL pour plat, et seulement pour plat
impl M2mRelation for plat::Allergenes {
    type From = plat::Entity;
    type To = allergene::Entity;
}

impl<E: EntityTrait> ModelAdmin<E> {
    // n'accepte qu'une relation qui part de son propre modèle
    fn m2m<R: M2mRelation<From = E>>(self, relation: R) -> Self { … }
}

ModelAdmin::<plat::Entity>::new().m2m(plat::Allergenes);   // compile
ModelAdmin::<menu::Entity>::new().m2m(plat::Allergenes);   // erreur : la relation part de Plat
// menu::Allergenes n'existe pas tant que le DSL de Menu ne le déclare pas : erreur aussi
```

**Donnée dérivée ≠ M2M** : les allergènes d'un menu (menu → plats → allergènes) ne sont pas un M2M du menu. Ils s'affichent en colonne calculée, lecture seule (`.computed(...)`), et ne se modifient que depuis les plats.

**Règles métier (cahier des charges)** :
1. Cible liable : elle existe, l'utilisateur peut **lire** sa ressource, et elle respecte la portée (`_own`, parent imbriqué). Aujourd'hui, n'importe quel id valide est accepté (relevé par Grok).
2. Droit requis : modifier la ressource éditée + lire la cible.
3. Le formulaire envoie l'état final ; différence (ajouts / retraits) calculée et écrite dans **une transaction** avec la ressource (base : `write_links`).
4. Table pivot à colonnes supplémentaires (`through`) : un mini-formulaire par lien, à traiter comme un inline.
5. Historique : « gluten ajouté, lait retiré » visible dans l'historique de la ressource.
6. Suppression d'une cible : cascade sur la table pivot, déclarée en base.

### Templates

- Un seul contrat : le builder construit une struct typée par page (`EditPage { fieldsets, readonly, inlines, actions }`), sérialisée dans le contexte.
- Un template par élément (`admin/parts/fieldset.html`, `readonly.html`, `inline.html`), inclus seulement s'il y a des données.
- Surcharge par ressource : `admin/<ressource>/parts/…` avant le générique.
- Jamais `| safe` sur une donnée de la base ; scripts dans des fichiers statiques (CSP) ; `readonly` est un affichage, jamais un champ désactivé — c'est la porte qui ignore la valeur.
- **Les clés du contexte sont une API publique** dès qu'un développeur surcharge un template :
  - la struct de chaque page porte un `///` par champ : la doc des clés est celle de rustdoc, jamais en retard sur le code ;
  - un test rend chaque template par défaut avec une struct entièrement remplie, Tera configuré pour échouer sur une variable inconnue : un template qui utilise une clé disparue casse la CI ;
  - renommer ou retirer un champ est une rupture (CHANGELOG « Breaking » + guide de migration) ; en ajouter un ne casse rien ;
  - une page de doc « Surcharger un template » donne, pour chaque template, la struct reçue (lien rustdoc) et l'ordre de recherche des surcharges.

### Jalons (chacun « fini » au sens de la ROADMAP)

Les jalons 0 à 4 se livrent en 3.x sans rien casser : le builder cohabite avec `admin!{}`. Seul le jalon 5 demande une version majeure. Le test golden de l'admin actuel sert de filet à chaque étape.

0. Sans rupture, avant le builder : `search_condition` typée, seule implémentation, puis le trait `AdminResource`, que l'admin actuel implémente avec ses fermetures (rien ne change pour le dev).
1. `AdminSite` + un `ModelAdmin` : liste et fiche, `list_display` typé, projection des colonnes, vérification au démarrage.
2. Création, suppression, actions groupées — par la porte existante (`GateCtx`, `GateAuthorization`).
3. Clés étrangères, many-to-many, recherche générique.
4. Migration de toutes les ressources de demo-app (le builder cohabite avec `admin!{}` pendant la 3.x).
5. Migration de Campanile, puis 4.0 : retrait de `admin!{}` et du daemon.
6. Parité Django (`fieldsets`, `readonly_fields`, inlines…), une option à la fois.

### Points ouverts

| Question | Pistes |
| --- | --- |
| Nom de l'attribut DSL pour un secret qui n'est pas un `password` | `[secret]`, `[sensitive]` |
| `per_page` modifiable depuis l'URL ? | Non par défaut. Sinon, plafonné par le builder (`max_per_page`), pour éviter `?per_page=1000000` |
| Une action groupée sur une colonne absente de `fields` ? | Autorisée (publier sans montrer le champ), mais la colonne doit être déclarée dans `action(..)`, jamais venir de la requête |
| Export (CSV) | Mêmes colonnes que `list_display`, même projection, même porte ; plus tard |
| `queryset` et `owner` en même temps | Les deux filtres s'additionnent (ET), jamais l'un à la place de l'autre |
