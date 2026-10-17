# Ébauche — builder `ModelAdmin<Entity>` (refonte de l'admin)

> Statut (2026-10-17) : **jalon 1 fait** (`runique/src/admin/refonte_admin/builder/`, feature `admin`) : `AdminSite`, `ModelAdmin` avec `ListConfig` et `FormConfig` typés, liste (tri, recherche, filtres, pagination, total en une requête) et fiche par la porte, projection des colonnes, `ModelMeta` généré par `model!{}` (`KEY`, `OWNER`, `SECRET`), vérification au démarrage. Le reste est encore à l'état d'ébauche. Cible : 4.0 (rupture).
> Calqué sur le `ModelAdmin` de Django ([référence 5.2](https://docs.djangoproject.com/en/5.2/ref/contrib/admin/)).
> La porte (droits, opérations, preuves, écritures) est décrite dans [ebauche-crud-operations.md](ebauche-crud-operations.md) ; ce document ne la répète pas.

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
| **Le builder est la seule entrée** | Un paramètre d'URL (`?sort_by=`, `?filter_*=`) est cherché dans la liste déclarée et converti en `Column`, jamais transmis en chaîne ; un nom inconnu est ignoré |
| **`fields` borne l'affichage et l'écriture** | Le formulaire et le POST ne peuvent pas diverger (assignation de masse impossible) |
| **Ce qui dépend de la requête** est une closure | Équivalent des `get_*(request)` de Django : elle reçoit l'utilisateur courant |
| **Le modèle reste la source** | Type et contraintes d'un champ déclarés une fois dans `model!{}` ; le builder choisit **lesquels** apparaissent et **comment** (le partage `models.py` / `admin.py`) |
| **Le code décrit, l'administrateur autorise** | Colonnes, filtres, formulaires, actions dans le builder ; groupes et droits en base, gérés depuis l'interface |
| **Jamais de SQL brut** | SeaORM, `sea_query` typé, `search!` / `search_cond!` |

Statuts utilisés plus bas : **fait** · **P1** (avec l'écriture) · **plus tard** (backlog).

## Organisation des fichiers

`main.rs` ne grossit pas avec le nombre de tables.

```rust
// main.rs
RuniqueAppBuilder::new(config)
    .with_admin(|a| a.site(admins::site))
```

```rust
// src/admins/mod.rs
mod blog;
mod contribution;

pub fn site(site: AdminSite) -> AdminSite {
    site.register(blog::admin())
        .register(contribution::admin())
}
```

```rust
// src/admins/blog.rs
use crate::entities::blog::{Column::*, Entity};
use runique::prelude::*;

pub fn admin() -> ModelAdmin<Entity> {
    ModelAdmin::new()
        .list(|l| l.display([Title, Email, CreatedAt]).search([Title, Summary]))
        .form(|f| f.fields([Title, Summary, Content, Image]))
}
```

| Fichier | Rôle |
| --- | --- |
| `main.rs` | Une ligne, quel que soit le nombre de tables |
| `src/admins/mod.rs` | Sommaire : une ligne `register` par table |
| `src/admins/<table>.rs` | La configuration d'une table (équivalent d'un `admin.py`) |

Pendant la refonte, demo-app déclare le builder dans `src/admin_refonte.rs` (hors de `src/admins/`, que `runique start` régénère), servi sous `{admin}/next/{resource}/list`.

## Composition du builder

`ModelAdmin<E>` est composé d'une struct par responsabilité, configurée par une closure par bloc, comme le reste du builder Runique (`.middleware(|m| ..)`, `.with_csp(|c| ..)`).

```rust
ModelAdmin::new()
    .list(|l| l.display([Title, CreatedAt]).search([Title]).per_page(50))
    .form(|f| f.fields([Title, Summary]).readonly([CreatedAt]))
```

| Choix | Contrainte d'où il vient |
| --- | --- |
| Une struct par bloc | Chaque handler ne reçoit que sa partie ; une option nouvelle s'ajoute dans un seul bloc |
| `E::Column` partout, jamais `String` | Colonnes typées |
| `Vec` plutôt que `HashSet` | L'ordre déclaré est l'ordre affiché |
| `search: Option<Vec<..>>` | `None` (colonnes texte affichées) ≠ `Some(vec![])` (recherche désactivée) |
| `ordering` en `(colonne, Order)` | Le sens du tri est déclaré, pas déduit d'un `-` dans une chaîne |
| `labels` en paires | `HashMap` exigerait `Column: Hash`, non garanti par SeaORM |
| `per_page: Option<u64>` | `None` : la valeur du site ; jamais moins de 1 |
| Une action = une colonne + une valeur | Une action écrit une colonne déclarée, jamais une colonne venue de la requête |
| `queryset` en `Arc<dyn Fn + Send + Sync>` | Dépend de l'utilisateur ; partagé entre requêtes sous Axum |

Validation : `ListConfig::validate()` et `FormConfig::validate()` pour les règles d'un bloc, `ModelAdmin::validate()` pour celles qui en croisent plusieurs, `AdminSite::check()` pour le site (même clé enregistrée deux fois). Toutes sont appelées au démarrage.

Type erasure : `AdminSite` garde des `Box<dyn RegisteredModel>` ; le routeur trouve le modèle par sa clé (clé inconnue → 404, jamais de requête), et `RegisteredModel` appelle lui-même la porte pour chaque modèle.

## Page liste — `.list(|l| ..)`

| Méthode | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `display([..])` | `list_display` | fait | Colonnes de la liste |
| `links([..])` | `list_display_links` | fait | Colonnes cliquables vers le détail |
| `sortable([..])` | `sortable_by` | fait | Seules colonnes triables depuis l'URL |
| `ordering([..])` | `ordering` | fait | Tri par défaut (clé primaire ajoutée en départage) |
| `search([..])` | `search_fields` | fait | Colonnes où cherche `?search=` |
| `filter([..])` | `list_filter` | fait | Seuls filtres acceptés dans l'URL |
| `per_page(n)` | `list_per_page` | fait | Lignes par page |
| `search_help(..)` | `search_help_text` | P1 | Texte sous la barre de recherche |
| `empty_value(..)` | `empty_value_display` | P1 | Affichage d'une valeur vide |
| `date_hierarchy(col)` | `date_hierarchy` | plus tard | Navigation année → mois → jour |
| `list_editable([..])` | `list_editable` | plus tard | Édition directe dans la liste (⊂ `fields`) |

Le total vient de `COUNT(*) OVER ()` dans la même requête (lu par `ListRow`), avec un `COUNT` de secours quand la page est vide.

## Actions groupées

| Méthode | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `action(col, valeur, libellé)` | `actions` | P1 | Met une colonne à une valeur sur la sélection (`group_action` actuel) |
| `action_fn(clé, libellé, fn)` | `actions` | plus tard | Action libre sur les id sélectionnés |

Les actions permises sont calculées en Rust, par action et par ligne ; le template ne fait que les afficher. Une seule requête, tout ou rien (voir la porte, « Tranché »).

## Formulaire — `.form(|f| ..)`

| Méthode | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `fields([..])` | `fields` | fait (lecture) | Champs affichés **et** seuls écrits par le POST |
| `readonly([..])` | `readonly_fields` | fait (lecture) | Affiché, jamais écrit |
| `labels([..])` | `verbose_name` | fait | Surcharge le `[label]` du modèle |
| `field(col, \|f\| ..)` | `formfield_overrides` | P1 | Réglage du widget d'un champ |
| `form::<F>()` | `form` | P1 | `#[form]` écrit à la main (`clean()` métier) ; les clés écrites restent bornées par `fields` |
| `fieldsets([..])` | `fieldsets` | plus tard | Mise en page en groupes, sans effet sur ce qui est écrit |
| `prepopulated(col, [..])` | `prepopulated_fields` | plus tard | Pré-remplissage (slug depuis le titre) |
| `save_as_new(bool)` | `save_as` | plus tard | Bouton « enregistrer comme nouveau » |
| `save_on_top(bool)` | `save_on_top` | plus tard | Boutons d'enregistrement en haut aussi |

## Relations

| Méthode | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `autocomplete([..])` | `autocomplete_fields` | plus tard | Recherche au lieu d'une liste déroulante pour une clé étrangère |
| `inline::<E>(..)` | `inlines` | plus tard | Édition des enfants dans le formulaire du parent |

## Accès

| Élément | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `meta: { owner: .. }` dans `model!{}` → `ModelMeta::OWNER` | — | fait | Colonne propriétaire pour les droits `_own` (jamais déduite des relations) |
| `queryset(\|q, user\| ..)` | `get_queryset` | P1 | Filtre métier appliqué partout : liste, détail, édition, actions ; s'ajoute au propriétaire (ET), jamais à sa place |
| `can_delete(\|user, obj\| ..)` | `has_delete_permission` | plus tard | Règle en plus des droits de groupe, jamais à leur place |

## Hooks et templates

| Méthode | Django | Statut | Rôle |
| --- | --- | --- | --- |
| `before_save(..)` | `save_model` | plus tard | Avant l'enregistrement (création ou modification) |
| `after_delete(..)` | `delete_model` | plus tard | Après une suppression |
| `list_template(..)` | `change_list_template` | plus tard | Surcharge du template de liste |

## Vérifié au démarrage

Le serveur refuse de démarrer si une de ces règles est violée. C'est la liste unique : toute règle de démarrage évoquée ailleurs doit figurer ici.

| Règle | Pourquoi | Statut |
| --- | --- | --- |
| Aucune colonne `SECRET` dans les listes du builder | Jamais affichée, cherchée, filtrée, triée ni écrite par l'admin (même cachée, une colonne cherchable est un oracle) | fait |
| `is_superuser` jamais dans `fields`, `list_editable` ni `action` | Un compte superuser ne s'accorde pas depuis un formulaire ni par lot | fait (`fields`) |
| `readonly`, `labels` ⊂ `fields` | Sinon on configure un champ absent du formulaire | fait |
| `links`, `sortable` ⊂ `display` | Sinon on lie ou trie une colonne non affichée | fait |
| Une clé de ressource enregistrée une seule fois | Sinon le routeur ne sait pas laquelle servir | fait |
| Chaque colonne obligatoire sans défaut est dans `fields` ou remplie automatiquement (clé primaire, `auto_now`, défaut) | Sinon la création échoue en base, devant l'utilisateur | P1 |
| `autocomplete`, `m2m` et `inline` visent une entité elle aussi enregistrée | Sinon les droits de la cible sont introuvables | plus tard |
| Une action vise une colonne de `fields`, jamais secrète ni `server_only` | Une action n'écrit que ce que le formulaire peut écrire | P1 |
| Pas de colonne à la fois dans `list_editable` et `links` | Règle Django : une cellule ne peut être lien et champ | plus tard |

**Source des vérifications** : `model!{}` génère `impl ModelMeta for Entity` (un trait sur l'entité existante, pas un second modèle ; rien sans la feature `admin`). Le builder lit ces faits, il ne déduit rien lui-même. Un `password` est secret d'office.

## Plancher de sécurité de Runique

Exclusions fixées par le framework, ni configurables ni contournables par un builder. Le dev déclare en liste blanche par-dessus.

| Niveau | Exemples | Appliqué par | Visibilité |
| --- | --- | --- | --- |
| Table | `eihwaz_sessions`, `eihwaz_reset_tokens`, `eihwaz_history` : jamais enregistrables | la règle de l'orphelin (`ModelMeta` et ces entités appartiennent à Runique : un projet ne peut pas les enregistrer) | — |
| Colonne du framework | `is_superuser` : jamais dans `fields`, `list_editable`, `action` | le démarrage | `pub(crate)` |
| Colonne secrète | `SECRET` de `ModelMeta` : jamais lue, cherchée, filtrée, triée ni écrite | le démarrage + projection | `ModelMeta` (`pub`, généré) |
| Compte | compte d'un superuser protégé ; en 4.0, la règle des rangs (on n'agit que sur un rang strictement inférieur) | la porte, à chaque requête | `pub(crate)` |

Le plancher ne repose jamais sur `ModelMeta` pour ce qui appartient à Runique : un `impl` écrit à la main pourrait mentir.

**Défense en profondeur**, par des couches de nature différente :

| Couche | État |
| --- | --- |
| Cookie de session `SameSite=Strict` | existe |
| CSRF par Prisme (l'admin abandonne sa comparaison brute) | à faire |
| Vérification `Origin` sur les POST admin | à ajouter |
| Connexion admin séparée + `LoginGuard` | existe |
| Entrée de l'admin (connecté, `can_access_admin()`) | middleware, existe |
| La porte + `Authorized<O, E>` | fait |

## Colonnes non affichées

- **Jamais lues pour la vue** : `select_only().columns(..)` ne sélectionne que les colonnes déclarées, plus la clé primaire (le `SELECT` n'est donc jamais vide). Une colonne non lue ne peut fuiter ni dans un template, ni dans une réponse htmx, ni dans un export.
- Les colonnes nécessaires aux contrôles du serveur (propriétaire, clé du parent, `updated_at`) se lisent pour la porte, **jamais transmises au template**.
- **Jamais envoyées ni modifiées** : absentes du formulaire, donc des données, donc `NotSet` en modification.

## Identifiants

- L'id de l'URL est converti dans le type de la clé primaire (`PkOf<E>`, `FromStr`) avant toute requête. Un id mal formé est refusé sans toucher la base, comme une ligne absente.
- Les id d'une action groupée suivent la même conversion. Un seul id invalide refuse tout le lot.

## Écritures

- Une opération = une fonction d'écriture. Leçon de la 3.0.3 : la création générée pour `bulk_create` mettait à jour la ligne existante, avec le seul droit de créer. Un « upsert », s'il revient, est une opération à part qui exige les deux droits.
- Modification partielle, limitée à `fields` : une colonne absente du formulaire reste intacte.
- Une suppression refusée par une clé étrangère (`restrict`) donne un message clair, jamais une 500. Le nombre et la table enfant (« ce plat est utilisé par 3 commandes ») ne s'affichent que si l'utilisateur peut **lire** cette table ; sinon, message neutre (« cette ligne est utilisée par d'autres données »). L'erreur SQL brute est tracée côté serveur, jamais montrée.

## Recherche

- Condition « OU » de `LOWER(CAST(col AS TEXT)) LIKE %terme%` en `sea_query` typé, `LIKE` échappé, valeur en paramètre lié.
- **Par défaut : uniquement les colonnes texte affichées.** Chercher dans une colonne cachée en ferait un oracle.
- `search([..])` typé pour élargir ; une colonne `SECRET` y est refusée au démarrage.
- À faire : plafond de longueur du terme (~200 caractères).
- Les copies de la même condition dans l'ancien admin (`builtin/user.rs`, `groupe.rs`, code généré par le daemon) disparaissent avec lui.
- Une recherche globale (plusieurs modèles) ne porterait que sur les ressources que l'utilisateur peut **lire**, et sur leurs colonnes autorisées.

## JavaScript des formulaires

| Source | Dans l'admin |
| --- | --- |
| JS d'un type de champ (dans son template, ex. `base_color.html`) | Toujours présent : il fait partie du champ |
| `add_js` du formulaire du modèle | Repris par défaut |
| `.form(\|f\| f.surcharge_js(["js/plat_admin.js"]))` | **Remplace** le `add_js` du formulaire de ce modèle ; `.surcharge_js([])` le retire |

- **Portée : uniquement le formulaire du modèle déclaré**, pas d'effet en cascade (inline, widget M2M, autre modèle).
- **Le JS servi par Runique n'est jamais touché.** L'origine est portée par le type (`enum JsSource { Runique(..), App(..) }`), jamais déduite d'un préfixe de chemin ; `surcharge_js` ne remplace que les entrées `App`. Le JS de sécurité (`csrf.js`) ne peut être ni retiré ni remplacé.
- **Changer le JS d'un champ de Runique** : le dev fournit son propre template de champ (HTML + JS), jamais un script seul.
- Même validation que `add_js` (`validate_js_path`), nonce pris dans la `Request`. La page assemble les listes en une seule, dédoublonnée dans l'ordre.

## Many-to-many

**Dépendance** : le M2M de l'admin s'appuie sur le M2M du DSL (syntaxe dans `model!`, table pivot et migration générées, relation typée). L'admin vient après.

**Bug actuel à ne pas reproduire** : `admin!{}` déclare un M2M entièrement en chaînes ; une faute de frappe compile et ne casse qu'à l'exécution.

**La relation porte son modèle d'origine dans son type** :

```rust
// généré par le DSL pour plat, et seulement pour plat
impl M2mRelation for plat::Allergenes {
    type From = plat::Entity;
    type To = allergene::Entity;
}

impl<E: EntityTrait> ModelAdmin<E> {
    fn m2m<R: M2mRelation<From = E>>(self, relation: R) -> Self { … }
}

ModelAdmin::<plat::Entity>::new().m2m(plat::Allergenes);   // compile
ModelAdmin::<menu::Entity>::new().m2m(plat::Allergenes);   // erreur : la relation part de Plat
```

**Donnée dérivée ≠ M2M** : les allergènes d'un menu (menu → plats → allergènes) s'affichent en colonne calculée, lecture seule, et ne se modifient que depuis les plats.

**Règles** :
1. Cible liable : elle existe, l'utilisateur peut **lire** sa ressource, et elle respecte la portée (`_own`, parent). Aujourd'hui, n'importe quel id valide est accepté.
2. Droit requis : modifier la ressource éditée + lire la cible.
3. Le formulaire envoie l'état final ; ajouts et retraits écrits dans **une transaction** avec la ressource (`write_links`).
4. Table pivot à colonnes supplémentaires (`through`) : un mini-formulaire par lien, traité comme un inline.
5. Historique : « gluten ajouté, lait retiré » visible dans l'historique de la ressource.
6. Suppression d'une cible : cascade sur la table pivot, déclarée en base.

## Templates

- Un seul contrat : le builder construit une struct typée par page (`ListPage` existe déjà), sérialisée dans le contexte.
- Un template par élément (`admin/parts/fieldset.html`, `readonly.html`, `inline.html`), inclus seulement s'il y a des données. Surcharge par ressource : `admin/<ressource>/parts/…` avant le générique.
- Boutons et liens d'action affichés selon les actions permises de la page, par `{% if %}` (absents du HTML, jamais cachés en CSS). La porte bloque de toute façon.
- Erreurs de l'admin rendues dans son propre gabarit, statut exact ; données inaccessibles → refus uniforme (flash + redirection).
- Jamais `| safe` sur une donnée de la base ; scripts dans des fichiers statiques (CSP) ; `readonly` est un affichage, jamais un champ désactivé.
- **Les clés du contexte sont une API publique** dès qu'un dev surcharge un template :
  - un `///` par champ de la struct de page : la doc des clés est celle de rustdoc ;
  - un test rend chaque template par défaut avec une struct pleine, Tera en échec sur une variable inconnue ;
  - renommer ou retirer un champ est une rupture (CHANGELOG « Breaking ») ; en ajouter un ne casse rien.

## Ce que ça engendre

| Disparaît en 4.0 | Remplacé par |
| --- | --- |
| `admin/daemon/` (générateur, parseur, watcher, ≈ 2 500 lignes) | Le code générique, monomorphisé par le compilateur |
| `admin/resource.rs` (DSL `admin!{}`) | Le builder `ModelAdmin` |
| `admin/helper/resource_entry.rs` (les 9 fermetures) | `RegisteredModel` |
| `admin/admin_main/` (handlers et ancienne porte) | `refonte_admin/` (porte, builder, routeur) |
| `demo-app/src/admins/admin.rs` (généré, 9 155 lignes) | Un fichier court par table, écrit à la main |

| Côté dev, avant | Après |
| --- | --- |
| Un bloc `admin!{}` dans `src/admin.rs` | `src/admins/`, écrit à la main : un fichier par table + `mod.rs` |
| `runique start` lance le daemon | Plus de daemon |
| `.with_admin(\|a\| a.routes(admins::routes("/admin")))` | `.with_admin(\|a\| a.site(admins::site))` |
| L'admin toujours compilé | Feature `admin`, désactivée par défaut |

Aide envisagée : une commande qui convertit un `admin!{}` existant en fichiers `src/admins/<table>.rs`.

## Volontairement absent

| Django | Pourquoi pas ici |
| --- | --- |
| `exclude` | Liste noire : une colonne ajoutée plus tard fuirait par oubli |
| Colonnes de liste en chaîne, ou fonction quelconque | Remplacé plus tard par une colonne calculée typée : `.computed("auteur", \|row: &Model\| row.author_name.clone())` |
| Macro procédurale `admin!{}` | Écartée pour garder rust-analyzer |

## Jalons

Tout part en 4.0, sous la feature `admin`. Chaque jalon est « fini » au sens de la ROADMAP.

1. **Fait** — `AdminSite` + `ModelAdmin` : liste et fiche par la porte, colonnes typées, projection, vérification au démarrage, `ModelMeta` généré.
2. Plafond de longueur du terme de recherche.
3. Migration framework 4.0 : `is_admin`, compte protégé, `can_read_own`, et `can_access_admin()` qui les accepte ; règle des rangs dans la porte.
4. Écriture : création, modification, suppression par la porte (`gate/write.rs` existe), formulaire relié au modèle, colonnes écrites recalculées à chaque requête ; puis actions groupées (htmx).
5. Templates sur les structs de page.
6. Ressources intégrées (utilisateurs, groupes, droits), clés étrangères, M2M.
7. Migration de demo-app et de Campanile, retrait de `admin!{}` et du daemon.
8. Parité Django (`fieldsets`, inlines…), une option à la fois.

## Points ouverts

| Question | Pistes |
| --- | --- |
| `per_page` modifiable depuis l'URL ? | Non par défaut. Sinon, plafonné par le builder (`max_per_page`), pour éviter `?per_page=1000000` |
| Export (CSV) | Mêmes colonnes que la liste, même projection, même porte ; plus tard (esquisse dans la porte, « Tranché ») |
| Un admin qui modifie un groupe | Il n'accorde que les droits qu'il possède (ferme le compte prête-nom) ; à confirmer |

## Plus tard : builder v2

Pas dans la première version. La v1 se contente des structs de page typées et de la surcharge par blocs (`{% extends %}` + `{% block %}`).

- **Templates à composants** : Runique expose des fonctions Tera (`admin_form()`, `admin_m2m()`, `admin_actions()`…) qui rendent leur propre HTML depuis la struct de page (droits, CSRF, conditions, JS) ; le dev place les composants et gère mise en page et CSS. Chaque composant laisse une marque (`data-runique="actions"`) vérifiée au démarrage : un composant de sécurité absent (formulaire + CSRF) refuse de démarrer, un autre donne un avertissement. Pas de parser maison au-dessus de Tera.
- **Surcharge à deux portées** : par modèle (`ResourceTemplates`) et pour tout l'admin (`SiteTemplates` : gabarit, tableau de bord, connexion) ; ordre modèle → site → défaut Runique. La navbar et les clés communes sont injectées par `AdminSite`.
- **CSS par bloc** : au démarrage, `AdminSite` choisit HTML et CSS selon ce que le dev a déclaré ; les CSS retenus sont concaténés en un seul fichier gardé en mémoire, servi sous un nom contenant son SHA-256 (cache `immutable`, SRI dans le contexte). Un CSS de bloc remplacé n'est jamais chargé. À soigner : ordre de concaténation fixe ; noms de classes des blocs = API ; reconstruction à chaque requête en debug. En v1 : un `admin.css` unique, complétable par un fichier du dev.
- **Formulaires** : `model!{}` génère un vrai `impl RuniqueForm` et un `save()` réel (équivalent `ModelForm.save()`). Dans l'admin, le formulaire valide et fournit les données limitées à `fields` ; l'écriture reste celle de la porte, jamais le `save()` du formulaire.
- **Retouche du formulaire par utilisateur** : `.form(|f| f.customize(|form, user| …))`, équivalent de `get_fields` / `get_readonly_fields`. Restrictions seulement (`hide`, `readonly`, libellé, widget), appliquées au GET **et** au POST, recalculées à chaque requête.
- **Conversion typée générée** (idée de Sébastien) : `model!{}` génère par modèle `to_active_model(&form)`, qui appelle le `cleaned_*` du bon type pour chaque champ, choisi à la compilation. Même fonction pour le site public et l'admin.
- **`save` à écrivain interchangeable** (idée de Sébastien) : une seule logique `save_with(writer, db)` — transaction, `before_save`, `to_active_model`, écriture, `after_save`, commit. Seule l'écriture change : `Direct` (site public) ou `Gated(preuve)` (admin, conditions rejouées). `Gated` ne se construit qu'avec une preuve.
