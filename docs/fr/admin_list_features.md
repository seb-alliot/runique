# Implémentation des fonctionnalités de la vue liste admin

> Pagination · Tri par colonne · Recherche · `list_display` configurable
> Toutes les opérations se font **au niveau SQL** (pas en mémoire).

---

## 1. Pourquoi SQL-level ?

La première tentation est de charger tous les enregistrements puis de trier/filtrer en Rust. C'est simple mais inutilisable dès que la table grossit. L'approche correcte consiste à laisser la base faire le travail :

```
SELECT ... FROM users
WHERE username ILIKE '%alice%'
ORDER BY created_at DESC
LIMIT 20 OFFSET 40;
```

Ça implique que les paramètres (page, tri, recherche) doivent traverser toute la pile avant d'atteindre la requête SeaORM.

---

## 2. Le vecteur de données : `ListParams`

La première chose à faire est de définir une struct qui regroupe tous les paramètres de liste. Elle sert de contrat entre le router HTTP et les closures de base de données.

**Fichier : `runique/src/admin/resource_entry.rs`**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    pub fn as_str(&self) -> &'static str {
        match self { SortDir::Asc => "asc", SortDir::Desc => "desc" }
    }
    pub fn toggle(&self) -> &'static str {
        match self { SortDir::Asc => "desc", SortDir::Desc => "asc" }
    }
}

pub struct ListParams {
    pub offset: u64,
    pub limit: u64,
    pub sort_by: Option<String>,
    pub sort_dir: SortDir,
    pub search: Option<String>,
}
```

`toggle()` est une petite astuce : dans le template, chaque en-tête de colonne doit pointer vers le sens opposé au tri actuel. Plutôt que de faire la logique en Tera, on calcule la valeur côté Rust et on l'injecte dans le contexte (`sort_dir_toggle`).

---

## 3. Mise à jour de la signature `ListFn`

La closure de listage avait la signature `|db: ADb|`. Elle devient :

```rust
// Avant
pub type ListFn = Arc<dyn Fn(ADb) -> BoxFuture<Result<Vec<Value>, DbErr>> + Send + Sync>;

// Après
pub type ListFn = Arc<dyn Fn(ADb, ListParams) -> BoxFuture<Result<Vec<Value>, DbErr>> + Send + Sync>;
```

`CountFn` reçoit un `Option<String>` pour la recherche (même logique, compte les lignes filtrées) :

```rust
pub type CountFn = Arc<dyn Fn(ADb, Option<String>) -> BoxFuture<Result<u64, DbErr>> + Send + Sync>;
```

---

## 4. Extraction des paramètres dans le handler HTTP

**Fichier : `runique/src/admin/admin_main.rs`**

Le handler `admin_get()` récupère les query params de l'URL :

```
GET /admin/users/list?page=2&sort_by=created_at&sort_dir=desc&search=alice
```

```rust
async fn admin_get(/* ... */) -> AppResult<Response> {
    let page: u64 = params.get("page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
        .max(1);

    let sort_by: Option<String> = params.get("sort_by").cloned();

    let sort_dir = match params.get("sort_dir").map(|s| s.as_str()) {
        Some("desc") => SortDir::Desc,
        _ => SortDir::Asc,
    };

    let search: Option<String> = params.get("search")
        .filter(|s| !s.is_empty())
        .cloned();

    handle_list(req, entry, state, page, sort_by, sort_dir, search).await
}
```

---

## 5. `handle_list` : sécurité et construction du contexte

Avant de passer `sort_by` à la base, il faut le **valider** contre les colonnes autorisées. Sinon, un utilisateur malveillant pourrait injecter n'importe quelle expression dans l'`ORDER BY`.

```rust
async fn handle_list(
    req: Request,
    entry: &ResourceEntry,
    state: Arc<PrototypeAdminState>,
    page: u64,
    sort_by: Option<String>,
    sort_dir: SortDir,
    search: Option<String>,
) -> AppResult<Response> {
    let page_size = state.config.page_size;
    let offset = (page - 1) * page_size;

    // Calcul des colonnes visibles via ColumnFilter
    let visible_columns: Vec<String> = match &entry.meta.display.columns {
        ColumnFilter::All => vec![],   // le template itère toutes les clés JSON
        ColumnFilter::Include(cols) => cols.clone(),
        ColumnFilter::Exclude(excl) => { /* filtrage à partir des clés du premier enregistrement */ }
    };

    // Validation du sort_by contre la whitelist des colonnes autorisées
    let safe_sort_by = sort_by.filter(|col| {
        col == "id"
            || visible_columns.iter().any(|c| c == col)
            || visible_columns.is_empty() // si All, on autorise (le générateur filtre côté DB)
    });

    let params = ListParams {
        offset,
        limit: page_size,
        sort_by: safe_sort_by.clone(),
        sort_dir: sort_dir.clone(),
        search: search.clone(),
    };

    // Appel des closures SQL
    let entries = (list_fn)(db.clone(), params).await?;
    let total = (count_fn)(db.clone(), search.clone()).await?;

    // Calcul pagination
    let page_count = (total + page_size - 1) / page_size;

    // Injection dans le contexte Tera
    req.insert("visible_columns", &visible_columns)
       .insert("sort_by", &safe_sort_by)
       .insert("sort_dir", &sort_dir.as_str())
       .insert("sort_dir_toggle", &sort_dir.toggle())
       .insert("search", &search.unwrap_or_default())
       .insert("page", &page)
       .insert("page_count", &page_count)
       .insert("has_prev", &(page > 1))
       .insert("has_next", &(page < page_count))
       .insert("prev_page", &(page - 1))
       .insert("next_page", &(page + 1))
       // ...
```

Le point clé ici : `sort_dir_toggle` est calculé **une seule fois en Rust** et injecté en tant que variable Tera. Le template n'a pas besoin de logique conditionnelle pour ça.

---

## 6. Le générateur : `generator.rs`

La fermeture `list_fn` générée par `runique start` utilise SeaORM de manière générique grâce à `sea_query` :

```rust
let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
    Box::pin(async move {
        use sea_orm::sea_query::{Alias, Expr, Order};

        let mut query = users::Entity::find();

        // Tri dynamique : le nom de colonne vient de params (déjà validé par le handler)
        if let Some(ref col) = params.sort_by {
            let order = if params.sort_dir == SortDir::Desc {
                Order::Desc
            } else {
                Order::Asc
            };
            query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
        }

        let rows = query
            .offset(params.offset)
            .limit(params.limit)
            .all(&*db)
            .await?;

        Ok(rows.into_iter()
            .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
            .collect())
    })
});
```

### Pourquoi `Alias::new` ?

SeaORM expose `order_by_column()` qui requiert de connaître le type `Column` au moment de la compilation. Ici on veut trier sur une colonne **connue seulement à l'exécution** (c'est une `String` qui vient de l'URL).

`Alias::new("created_at")` construit un identifiant SQL brut. SeaORM l'utilise tel quel dans la clause `ORDER BY`. La sécurité repose sur la **validation préalable** dans `handle_list` (la colonne doit être dans `visible_columns` ou être `"id"`).

---

## 7. `ColumnFilter` : list_display configurable

**Fichier : `runique/src/admin/resource.rs`**

```rust
pub enum ColumnFilter {
    All,                    // affiche toutes les clés JSON
    Include(Vec<String>),   // whitelist : seulement ces colonnes
    Exclude(Vec<String>),   // blacklist : toutes sauf celles-ci
}
```

Côté utilisateur (dans `src/admin.rs`) :

```rust
UserResource::new()
    .display(DisplayConfig::new()
        .columns(ColumnFilter::Include(vec![
            "username".into(),
            "email".into(),
            "is_active".into(),
        ]))
    )
```

Dans `handle_list`, `visible_columns` est calculé à partir de ce filtre. C'est cette liste qui sert aussi de **whitelist pour le tri** : on n'autorise `sort_by` que sur les colonnes effectivement affichées.

---

## 8. Le template : `list.html`

Le template reçoit toutes les variables nécessaires sans logique métier. Il se contente de les assembler dans les URLs.

```jinja
{# En-tête de colonne avec tri #}
<a href="?sort_by={{ col }}
         &sort_dir={% if sort_by == col %}{{ sort_dir_toggle }}{% else %}asc{% endif %}
         {% if search %}&search={{ search | urlencode }}{% endif %}
         &page=1"
   class="th-sort-link">
  {{ col }}
  {% if sort_by == col %}
    <span class="sort-indicator">
      {% if sort_dir == "asc" %}▲{% else %}▼{% endif %}
    </span>
  {% endif %}
</a>
```

Le `&page=1` en fin d'URL est important : changer le tri remet à la première page, sinon on pourrait se retrouver sur une page qui n'existe plus.

La barre de recherche preserve les paramètres de tri via des `<input type="hidden">` :

```html
<form method="GET" action="">
  <input type="text" name="search" value="{{ search }}">
  {% if sort_by %}<input type="hidden" name="sort_by" value="{{ sort_by }}">{% endif %}
  {% if sort_dir %}<input type="hidden" name="sort_dir" value="{{ sort_dir }}">{% endif %}
  <button type="submit">Rechercher</button>
</form>
```

---

## 9. Re-exports : exposer les types dans le prelude

`admin_panel.rs` est généré et utilise `use runique::prelude::*`. Il faut donc que `ListParams` et `SortDir` soient accessibles depuis ce glob import.

La chaîne de re-exports :

```
resource_entry.rs         (définition)
    ↓
admin/mod.rs              pub use resource_entry::{..., ListParams, SortDir, ...};
    ↓
lib.rs (prelude)          pub use crate::admin::{..., ListParams, SortDir, ...};
```

Sans le maillon `admin/mod.rs`, le prelude ne voit pas les types même si `lib.rs` les cite.

---

## 10. Résumé du flux complet

```
URL: /admin/users/list?page=2&sort_by=email&sort_dir=asc&search=alice

    ┌─────────────────────────────────────────────────────────┐
    │ admin_get()                                             │
    │   extrait page, sort_by, sort_dir, search depuis params │
    │   calcule offset = (page-1) * page_size                 │
    └──────────────────────┬──────────────────────────────────┘
                           │
    ┌──────────────────────▼──────────────────────────────────┐
    │ handle_list()                                           │
    │   valide sort_by contre visible_columns (whitelist)     │
    │   construit ListParams { offset, limit, sort_by, ... }  │
    └──────────┬───────────────────────────┬──────────────────┘
               │                           │
    ┌──────────▼──────────┐   ┌────────────▼─────────────────┐
    │ list_fn(db, params) │   │ count_fn(db, search)         │
    │   ORDER BY email ASC│   │   COUNT(*) (avec filtre)     │
    │   LIMIT 20 OFFSET 20│   └──────────────────────────────┘
    └──────────┬──────────┘
               │
    ┌──────────▼──────────────────────────────────────────────┐
    │ contexte Tera                                           │
    │   entries, total, page, page_count, has_prev, has_next  │
    │   visible_columns, sort_by, sort_dir, sort_dir_toggle   │
    │   search                                                │
    └──────────┬──────────────────────────────────────────────┘
               │
    ┌──────────▼──────────────────────────────────────────────┐
    │ list.html                                               │
    │   rendu HTML avec liens de tri et pagination            │
    └─────────────────────────────────────────────────────────┘
```
