# Système de filtres admin — Comment ça marche
> Runique · Axum + SeaORM + Tera · 2026-03-22

---

## Vue d'ensemble

Le système de filtres affiche dans une sidebar les valeurs distinctes d'une colonne, et filtre la liste en cliquant dessus. **Tout est SQL** — aucune donnée en mémoire.

```
Déclaration (src/admin.rs)
      ↓
    ModelAdmin.list().filter([...])
      ↓
    Validation au boot (AdminStaging::validate)
      ↓
    Handler liste → requêtes distinctes + liste
      ↓
    Template → sidebar
```

---

## Étape 1 — Déclarer un filtre

Dans `src/admin.rs`, le dev déclare les colonnes à filtrer via le builder typé :

```rust
fn doc_page() -> ModelAdmin<doc_page::Entity> {
    use doc_page::Column::*;
    ModelAdmin::new().list(|l| {
        l.display([Title, Lang, SectionId])
            .filter([Lang, SectionId])
    })
}
```

Les colonnes sont des variants SeaORM (`Column::Lang`) : une typo ne compile pas.

> **Règle :** privilégier booléens, énumérations et codes courts.
> Les secrets et drapeaux de rang sont refusés au boot.

---

## Étape 2 — Validation au démarrage

Plus de parser/daemon. Qu'une colonne de filtre existe, c'est le compilateur qui le garantit (`Column::Lang` est une variante de l'enum de l'entité). Au démarrage, `AdminStaging::validate()` refuse en plus une colonne **secrète** (`password`…) dans `.filter([...])`, comme dans toute autre liste de colonnes.

Une erreur arrête le démarrage avec un message clair.

---

## Étape 3 — Requêtes SQL

Pour chaque colonne filtrable, le handler charge une page de valeurs **distinctes** dans le périmètre autorisé :

```sql
SELECT DISTINCT CAST(lang AS TEXT)
FROM doc_page
WHERE lang IS NOT NULL
ORDER BY lang ASC
LIMIT {taille + 1} OFFSET {décalage}
```

`CAST(... AS TEXT)` uniformise le type : booléens, entiers et chaînes passent tous par là.

Le résultat alimente `filter_values` dans le contexte Tera : colonne → liste de valeurs (`value`, `active`, `qs`).

---

## Étape 4 — Le handler

Dans le router admin, les paramètres URL sont parsés :

```
filter_lang=fr    → filtre actif sur la colonne lang
```

La liste, le total et les valeurs de filtre tournent dans le même flux de page. Seules les colonnes déclarées dans `.filter([...])` sont acceptées ; tout autre `filter_*` est ignoré.

---

## Étape 5 — Le template

La sidebar s'appuie sur `resource.display.list_filter` et `filter_values` :

```html
{% for filter_entry in resource.display.list_filter %}
  {% set col = filter_entry[0] %}
  {% set label = filter_entry[1] %}
  {% set values = filter_values[col] %}
  {% if values %}
    <!-- groupe visible -->
  {% endif %}
{% endfor %}
```

Chaque valeur est un lien (`qs`) qui ajoute `filter_{col}={val}` à l'URL, ou l'en retire si elle est déjà active. Plusieurs valeurs d'une colonne s'additionnent (`col = a OR col = b`) ; plusieurs colonnes se cumulent (`AND`).

Les filtres actifs sont aussi exposés via `active_filters` et `filter_qs` pour préserver l'état dans la pagination.

---

## Étape 6 — Repli par groupe

Chaque groupe peut être replié individuellement. Les groupes sont **repliés par défaut** ; l'état est sauvegardé dans `sessionStorage` par ressource + colonne :

```
clé : runique_fg_doc_page_lang
      → '1'            = ouvert
      → autre ou absent = replié
```

Au chargement, chaque groupe restaure son état. Au clic sur le titre, le corps est caché/montré et l'état est sauvegardé.

---

## Résumé des choix de conception

| Choix                              | Alternative écartée          | Pourquoi                             |
|------------------------------------|------------------------------|--------------------------------------|
| Colonnes typées (`Column::…`)      | Chaînes libres               | Erreur à la compilation              |
| Validation au boot                 | Génération de code           | Pas de `src/admins/`, pas de daemon  |
| Valeurs distinctes SQL             | Charger toute la table       | Scalable                             |
| Whitelist `.filter([...])`         | Accepter tout `filter_*`     | Pas d'injection de nom de colonne    |
| `sessionStorage` par ressource+colonne | État global              | Deux ressources indépendantes        |
