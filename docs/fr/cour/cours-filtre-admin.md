## Système de filtres admin — Comment ça marche 

Runique · Axum + SeaORM + Tera · 2026-03-22 

## Vue d'ensemble 

Le système de filtres affiche dans une sidebar les valeurs distinctes d'une colonne, et filtre la liste en cliquant dessus. Tout est SQL — aucune donnée en mémoire. 

```
Déclaration (admin.rs)
       ↓
   Parser          → lit la macro admin!
       ↓
   Générateur      → produit admin_panel.rs
       ↓
   Handler         → orchestre les requêtes
       ↓
   Template        → rend la sidebar
```

## Étape 1 — Déclarer un filtre 

Dans `src/admin.rs` , le dev déclare les colonnes à filtrer : 

```
list_filter: [
    ["lang", "Langue"],         // défaut : 10 valeurs par page
    ["block_type", "Type", 5],  // limite explicite : 5 valeurs par page
]
```

Chaque entrée = `["colonne", "Libellé"]` ou `["colonne", "Libellé", limite]` . 

**Règle :** ne jamais filtrer sur une FK ou un `id` . Valeurs brutes illisibles. Bons candidats : booléens, énumérations, codes courts. 

## Étape 2 — Le parser 

Le daemon ( `runique start` ) lit `src/admin.rs` token par token et construit une structure intermédiaire : 

```
pubstructResourceDef {
pub list_filter: Vec<(String, String, u64)>,
//                    col     label   limit
}
```

Le 3ème élément est optionnel — si absent, la limite par défaut est `10` . 

## Étape 3 — Le générateur 

À partir de la structure, le daemon génère deux blocs dans `admin_panel.rs` . 

## La configuration d'affichage 

```
let meta = meta.display(
    DisplayConfig::new()
        .list_filter(vec![
            ("lang", "Langue", 10u64),
            ("block_type", "Type", 5u64),
        ])
);
```

Cette config est sérialisée et accessible dans Tera via `resource.display.list_filter` . 

## La closure de filtres 

Pour chaque colonne, deux requêtes SQL sont générées : 

**Comptage** — pour savoir combien de pages existent : 

```
SELECTCOUNT(DISTINCT lang) FROM doc_page WHERE lang ISNOTNULL
```

**Valeurs paginées** — seulement ce qui est affiché : 

```
SELECTDISTINCTCAST(lang ASTEXT)
FROM doc_page
WHERE lang ISNOTNULL
ORDERBY lang ASC
LIMIT10OFFSET20-- page 2 × 10
```

`CAST(... AS TEXT)` uniformise le type : booléens, entiers et chaînes passent tous par là. 

Le résultat est un `HashMap<String, (Vec<String>, u64)>` : chaque colonne → ses valeurs + son total distinct. 

## Étape 4 — Le handler 

Dans `admin_main.rs` , deux séries de paramètres URL sont parsées : 

```
filter_lang=fr          → filtre actif sur la colonne lang
fp_lang=2               → page 2 dans la sidebar du groupe lang
```

Les trois requêtes tournent **en parallèle** grâce à `tokio::join!` : 

```
tokio::join!(
    list_fn(db, list_params),    // entrées de la table
    count_fn(db, search),        // total pour la pagination principale
    filter_fn(db, filter_pages), // valeurs distinctes par colonne
)
```

## Étape 5 — La pagination des filtres 

C'est la partie centrale. Plutôt que charger 100 valeurs en JS, on pagine côté serveur. 

Pour chaque colonne, le handler calcule `filter_meta` : 

```
current_page  → page affichée actuellement
total_pages   → nombre total de pages
prev_qs       → query string complet du lien "page précédente"
next_qs       → query string complet du lien "page suivante"
```

`prev_qs` et `next_qs` sont précalculés en Rust car Tera ne peut pas construire des query strings complexes. Le template n'a plus qu'à écrire : 

```
<a href="?{{ meta.prev_qs }}&page=1">‹</a>
<a href="?{{ meta.next_qs }}&page=1">›</a>
```

Ces liens préservent automatiquement le tri, la recherche et les autres filtres actifs. 

## Étape 6 — Le template 

La sidebar s'affiche uniquement si `list_filter` est non vide : 

```
{% for filter_entry in resource.display.list_filter %}
  {% set col    = filter_entry[0] %}
  {% set label  = filter_entry[1] %}
  {% set values = filter_values[col] %}
  {% set meta   = filter_meta[col] %}
  {% if values | length > 0 %}
<!-- groupe visible -->
  {% endif %}
{% endfor %}
```

Chaque valeur est un lien qui ajoute `filter_{col}={val}` à l'URL. Cliquer applique un `WHERE col = val` côté SQL. 

## Étape 7 — Repli par groupe 

Chaque groupe peut être replié individuellement. L'état est sauvegardé dans `localStorage` par ressource + colonne : 

```
clé : runique_fg_doc_page_lang
      → '1' = ouvert
      → '0' = replié
```

Au chargement, chaque groupe restaure son état. Au clic sur le titre, le corps est caché/montré et l'état est sauvegardé. 

## Étape 8 — Diagnostic 

Si une colonne n'existe pas en base, la requête échoue. Au lieu de passer silencieusement, le code généré log une erreur : 

```
ERROR [runique admin] list_filter `doc_block.lang` :
      colonne introuvable en DB — column "lang" does not exist
```

Le dev voit immédiatement quelle colonne poser dans `list_filter` est invalide. 

## Résumé des choix de conception 

|**Choix**||**Alternative écartée**|**Pourquoi**|
|---|---|---|---|
|Pagination|serveur|Charger 100 valeurs en<br>JS|Scalable, sans limite arbitraire|

||**Choix**|**Alternative écartée**|**Pourquoi**||
|---|---|---|---|---|
||Limite per-colonne|Limite globale|Chaque colonne a des besoins<br>différents||
||`tokio::join!`|Requêtes séquentielles|Les 3 sont indépendantes||
||`tracing::error!`sur colonne<br>absente|`unwrap_or`silencieux|Diagnostique immédiat||
||localStorage par ressource+colonne|État global|Deux ressources indépendantes||
