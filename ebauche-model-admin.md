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
| Builder | `search_condition::<E>(db, &self.search, term)` — sa liste blanche |
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
| `derive_form` (`model!{}`) | — | Ajoute `impl AdminModel for Entity`, en regroupant ce qu'il génère déjà |

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
| Un bloc `admin!{}` dans `src/admin.rs` | Un dossier `src/admin/` : un fichier par table + `mod.rs` |
| `runique start` lance le daemon | Plus de daemon |
| `src/admins/` généré, à ne pas toucher | Supprimé |
| `.with_admin(\|a\| a.routes(admins::routes("/admin")))` | `.with_admin(admin::site)` |

Aide envisagée : une commande qui convertit un `admin!{}` existant en fichiers `src/admin/<table>.rs`.

### Ordre possible

| Étape | Rupture | Contenu |
| --- | --- | --- |
| 1 | Non | `search_condition` typée, seule implémentation (ci-dessus) |
| 2 | Non | Trait `AdminResource` ; l'admin actuel l'implémente via ses fermetures (rien ne change pour le dev) |
| 3 | Non | `model!{}` implémente `AdminModel` ; builder `ModelAdmin` utilisable **à côté** de `admin!{}` (prototype sur `blog`) |
| 4 | Oui | Ressources intégrées passées au builder, suppression du daemon et de `admin!{}`, guide de migration |

Les étapes 1 à 3 se livrent en 3.x sans rien casser ; seule la 4 demande une version majeure. Le test golden de l'admin actuel sert de filet à chaque étape.

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

### Colonnes non affichées

- **Jamais lues pour la vue** : `select_only().columns([...])` ne sélectionne que les colonnes affichées. Une colonne qui n'est pas lue ne peut fuiter ni dans un template, ni dans une réponse HTMX, ni dans un export.
- Les colonnes nécessaires aux contrôles côté serveur (propriétaire pour `_own`, clé du parent, `updated_at` du verrouillage optimiste) peuvent être lues pour la porte, **jamais transmises au template**.
- **Jamais envoyées ni modifiées** : absentes du formulaire, donc des données, donc `NotSet` en modification.
- **Vérification au démarrage** : à `register`, chaque colonne obligatoire sans valeur par défaut (`Column::def()`) doit être dans le formulaire de création ou remplie automatiquement (clé primaire, `auto_now`, défaut). Sinon l'application refuse de démarrer : « plat : la colonne `prix` est obligatoire mais absente du formulaire de création ».

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

1. `AdminSite` + un `ModelAdmin` : liste et fiche, `list_display` typé, projection des colonnes, vérification au démarrage.
2. Création, suppression, actions groupées — par la porte existante (`GateCtx`, `GateAuthorization`).
3. Clés étrangères, many-to-many, recherche générique.
4. Migration de toutes les ressources de demo-app (le builder cohabite avec `admin!{}` pendant la 3.x).
5. Migration de Campanile, puis 4.0 : retrait de `admin!{}` et du daemon.
6. Parité Django (`fieldsets`, `readonly_fields`, inlines…), une option à la fois.
