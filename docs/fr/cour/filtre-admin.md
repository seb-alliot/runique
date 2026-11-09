# Système de filtres admin — Comment ça marche
> Runique · Axum + SeaORM + Tera — de la déclaration `.filter([...])` à la barre latérale, avec le code réel et les techniques Rust utilisées à chaque étape.

## Objectifs

- Comprendre le trajet d'un filtre : déclaration, vérification au démarrage, requêtes SQL, URL, template
- Voir comment les types (`Column`) empêchent les fautes de frappe
- Construire des conditions SeaORM composées (`Condition::all`, `Condition::any`)
- Paginer des valeurs distinctes sans les compter
- Traiter les paramètres d'URL sans faire confiance au visiteur
- Fabriquer des liens qui ajoutent ou retirent un filtre avec une chaîne d'itérateurs

## Table des matières

1. Vue d'ensemble
2. Étape 1 — Déclarer un filtre
3. Étape 2 — Vérification au démarrage
4. Étape 3 — Les valeurs proposées
5. Étape 4 — Appliquer les filtres choisis
6. Étape 5 — Les liens de la barre latérale
7. Étape 6 — Le template
8. Étape 7 — Repli des groupes côté navigateur
9. Sécurité
10. Résumé des choix de conception
11. Exercices
12. Points clés

## 1. Vue d'ensemble

La barre latérale de la liste admin affiche, pour chaque colonne filtrable, ses **valeurs distinctes**. Cliquer sur une valeur filtre la liste. Tout se fait en SQL : aucune table n'est chargée en mémoire.

```
src/admin.rs           ModelAdmin::new().list(|l| l.filter([Lang, SectionId]))
      │
démarrage              vérification : aucune colonne secrète
      │
requête GET            /admin/doc_page?filter_lang=fr&filter_lang=en&page=1
      │
handler liste          1. valeurs distinctes de chaque colonne filtrable (barre latérale)
                       2. conditions des filtres choisis + liste + total
      │
template Tera          groupes repliables, liens qui ajoutent ou retirent une valeur
```

## 2. Étape 1 — Déclarer un filtre

```rust
// src/admin.rs
fn doc_page() -> ModelAdmin<doc_page::Entity> {
    use doc_page::Column::*;
    ModelAdmin::new().list(|l| {
        l.display([Title, Lang, SectionId])
            .filter([Lang, SectionId])
            .filter_size(10)              // valeurs par groupe avant les flèches de pagination
    })
}
```

```rust
// runique/src/admin/builder/list.rs (extrait)
pub fn filter(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
    self.filter = cols.into_iter().collect();
    self
}
```

Ce que montre la signature :

- `E::Column` : le type des colonnes **de l'entité administrée**. Passer `users::Column::Email` à un `ModelAdmin<doc_page::Entity>` ne compile pas, une colonne mal orthographiée (`Lnag`) non plus ;
- `impl IntoIterator<Item = ...>` : on peut passer un tableau, un `Vec`, ou n'importe quel itérateur (cours `generics`) ;
- `mut self -> Self` : un builder par valeur (cours `orm`, section 2.7).

> **Conseil** : filtre sur des colonnes à **peu de valeurs** : booléens, énumérations, codes courts, clés étrangères. Filtrer sur un titre ou une date à la seconde produit autant de valeurs que de lignes.

## 3. Étape 2 — Vérification au démarrage

Qu'une colonne existe, le compilateur le garantit. Ce qu'il ne peut pas savoir, c'est qu'une colonne est **secrète**. Chaque entité déclare ses colonnes secrètes (`E::SECRET`, par exemple le mot de passe), et le builder les refuse dans toutes les listes :

```rust
// runique/src/admin/builder/admin_builder.rs (extrait)
for secret in E::SECRET {
    let name = secret.as_str();
    for list in self
        .list
        .lists_using(name)
        .into_iter()
        .chain(self.form.lists_using(name))
    {
        errors.push(format!(
            "{list}: `{name}` is a secret column, never shown, searched, filtered, sorted or written by the admin"
        ));
    }
}
```

Un `.filter([Password])` arrête donc le **démarrage** avec un message qui nomme la liste fautive. Le choix : une erreur au démarrage plutôt qu'à la première requête, pour qu'aucune configuration dangereuse n'atteigne la production.

Technique Rust : `.into_iter().chain(...)` parcourt deux sources (listes de la vue liste, listes du formulaire) comme une seule, sans construire de collection intermédiaire.

## 4. Étape 3 — Les valeurs proposées

### 4.1 La requête

```rust
// runique/src/admin/builder/list.rs (extrait)
/// The values a filter offers for `col`: the distinct ones of the rows in
/// `scope`, as text (how `resolve_filters` compares them), sorted, NULL
/// left out, `page` of `size` of them. One more than `size` is read: its
/// presence says a next page exists, without counting the values.
pub(crate) fn filter_values(
    backend: DbBackend,
    scope: Condition,
    col: E::Column,
    page: u64,
    size: u64,
) -> Select<E> {
    let cast = Alias::new(text_cast_type_of(backend));
    // The databases take a signed 64-bit offset: a huge page from the URL
    // stays in range instead of failing the query.
    let offset = std::cmp::Ord::min(
        page.saturating_sub(1).saturating_mul(size),
        i64::MAX.unsigned_abs(),
    );
    E::find()
        .filter(scope)
        .filter(col.is_not_null())
        .select_only()
        .column_as(Expr::col((E::default(), col)).cast_as(cast), FILTER_VALUE)
        .distinct()
        .order_by_asc(Expr::col(Alias::new(FILTER_VALUE)))
        .offset(offset)
        .limit(size.saturating_add(1))
}
```

Le SQL produit ressemble à :

```sql
SELECT DISTINCT CAST("lang" AS TEXT) AS filter_value
FROM "doc_page"
WHERE <périmètre autorisé> AND "lang" IS NOT NULL
ORDER BY filter_value ASC
LIMIT 11 OFFSET 0
```

### 4.2 Les choix, un par un

| Élément | Pourquoi |
|---|---|
| `.filter(scope)` | seules les lignes que le compte a le droit de voir proposent leurs valeurs : une valeur ne fuit pas par la barre latérale |
| `is_not_null()` | `NULL` n'est pas une valeur cliquable |
| `cast_as(texte)` | booléens, entiers, énumérations et textes deviennent du texte : un seul code pour tous les types |
| `text_cast_type_of(backend)` | le nom du type texte diffère selon le moteur (Postgres, MySQL, SQLite) |
| `distinct()` | une valeur par ligne de la barre, même si mille lignes la portent |
| `limit(size + 1)` | lire **une valeur de plus** que la page : si elle existe, il y a une page suivante, sans `COUNT` |
| `saturating_sub`, `saturating_mul`, `min(.., i64::MAX)` | une page démesurée dans l'URL (un numéro de page à vingt chiffres) ne provoque ni dépassement d'entier ni erreur de la base |

### 4.3 La technique « un de plus »

```rust
let mut valeurs: Vec<String> = /* résultat de filter_values, size + 1 au plus */ vec![];
let size = 10;
let page_suivante = valeurs.len() as u64 > size;
valeurs.truncate(size as usize);       // n'afficher que la page demandée
```

Compter toutes les valeurs distinctes coûterait une requête de plus et un parcours complet. Lire `size + 1` lignes suffit pour savoir s'il faut afficher la flèche « suivant ».

### 4.4 Arithmétique saturante

```rust
let page: u64 = 0;                         // une page 0 venue de l'URL
let offset = page.saturating_sub(1)        // 0 au lieu d'un dépassement (0 - 1)
    .saturating_mul(10);                   // reste dans les bornes d'un u64
```

Toute valeur numérique venue d'une URL peut être 0, énorme ou absurde : `saturating_*` la garde dans les bornes sans paniquer (cours `variables-et-fonctions`, section 5).

## 5. Étape 4 — Appliquer les filtres choisis

### 5.1 L'URL

```
?filter_lang=fr&filter_lang=en&filter_section_id=3
```

Une valeur est un paramètre `filter_<colonne>=<valeur>`. Plusieurs valeurs d'une même colonne se répètent.

### 5.2 La condition

```rust
// runique/src/admin/builder/list.rs (extrait)
pub(crate) fn resolve_filters(
    &self,
    backend: DbBackend,
    filters: &[(String, String)],
) -> Option<Condition> {
    let cast = Alias::new(text_cast_type_of(backend));
    let mut cond = Condition::all();                         // ET entre les colonnes
    for col in &self.filter {                                // seulement les colonnes DÉCLARÉES
        let one_of = filters
            .iter()
            .filter(|(name, _)| name == col.as_str())
            .fold(Condition::any(), |any, (_, value)| {      // OU entre les valeurs d'une colonne
                any.add(
                    Expr::col((E::default(), *col))
                        .cast_as(cast.clone())
                        .eq(value.as_str()),
                )
            });
        if !one_of.is_empty() {
            cond = cond.add(one_of);
        }
    }
    (!cond.is_empty()).then_some(cond)
}
```

Pour l'URL de la section 5.1 :

```sql
WHERE (CAST(lang AS TEXT) = 'fr' OR CAST(lang AS TEXT) = 'en')
  AND (CAST(section_id AS TEXT) = '3')
```

### 5.3 Les techniques Rust

| Technique | Ligne | Effet |
|---|---|---|
| boucle sur `self.filter` | `for col in &self.filter` | un paramètre `filter_password=...` ou `filter_inconnu=...` est **ignoré** : aucune colonne n'est prise dans l'URL |
| `fold` | `.fold(Condition::any(), ...)` | construit une condition à partir d'une liste, sans variable mutable |
| `Condition::all()` / `Condition::any()` | | ET / OU de SeaORM, imbricables |
| `cast_as` des deux côtés | colonne en texte, valeur en texte | les valeurs proposées (section 4) et les valeurs comparées ont la même forme |
| `bool::then_some` | `(!cond.is_empty()).then_some(cond)` | `None` quand aucun filtre n'est actif : pas de `WHERE` inutile |

La valeur venue de l'URL (`value.as_str()`) devient un **paramètre lié** de la requête : jamais du SQL. Un `filter_lang=' OR 1=1 --` est une chaîne comparée à `lang`, qui ne correspond à rien.

### 5.4 Le même filtre écrit autrement

```rust
use sea_orm::Condition;

// Avec une boucle et une variable mutable
fn une_colonne_v1(valeurs: &[&str]) -> Condition {
    let mut any = Condition::any();
    for v in valeurs {
        any = any.add(doc_page::Column::Lang.eq(*v));
    }
    any
}

// Avec fold (forme de Runique)
fn une_colonne_v2(valeurs: &[&str]) -> Condition {
    valeurs.iter().fold(Condition::any(), |any, v| any.add(doc_page::Column::Lang.eq(*v)))
}

// Avec IN : une seule expression
fn une_colonne_v3(valeurs: &[&str]) -> Condition {
    Condition::all().add(doc_page::Column::Lang.is_in(valeurs.iter().copied()))
}
```

`IN` serait plus court, mais Runique compare en **texte** pour traiter tous les types de colonne de la même façon : la forme avec `cast_as` et `OR` garde un seul code pour un booléen, un entier ou une énumération.

## 6. Étape 5 — Les liens de la barre latérale

Chaque valeur affichée est un lien qui **ajoute** le filtre s'il est inactif, ou le **retire** s'il est actif, en gardant tous les autres paramètres.

```rust
// runique/src/admin/router/context.rs (extrait)
pub struct FilterOption {
    pub value: String,
    /// Whether the list is filtered on this value.
    pub active: bool,
    /// `?…` of the list with this value turned on or off.
    pub qs: String,
}

impl FilterOption {
    pub(crate) fn new(params: &[(String, String)], column: &str, value: String) -> Self {
        let name = format!("filter_{column}");
        let is_it = |k: &str, v: &str| k == name && v == value;
        let active = params.iter().any(|(k, v)| is_it(k, v));
        let qs = encoded_query(
            params
                .iter()
                .filter(|(k, v)| k != "page" && !is_it(k, v))      // tous les paramètres sauf la page et cette valeur
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .chain((!active).then_some((name.as_str(), value.as_str())))   // + cette valeur si elle était inactive
                .chain(std::iter::once(("page", "1"))),             // + retour à la première page
        );
        Self { value, active, qs }
    }
}
```

La chaîne d'itérateurs, ligne par ligne :

1. `filter` : garde tous les paramètres actuels, sauf `page` et la valeur elle-même ;
2. `chain(Option)` : une `Option` est itérable (zéro ou un élément) : la valeur est ajoutée **seulement** si elle n'était pas active, d'où le basculement ;
3. `chain(once(..))` : on revient à la page 1, puisque le nombre de résultats change ;
4. `encoded_query` encode le tout pour l'URL (un `&` ou un `=` dans une valeur ne casse pas le lien).

| Technique | Usage ici |
|---|---|
| closure qui capture (`is_it`) | réutiliser le même test dans `any` et dans `filter` |
| `Option` comme itérateur | ajouter un élément conditionnellement dans une chaîne |
| `std::iter::once` | ajouter un élément fixe |
| `bool::then_some` | transformer une condition en `Option` |

Une fonction voisine, `filter_clear_qs`, construit le lien « tout effacer » d'un groupe : mêmes paramètres, sans aucun `filter_<colonne>`.

## 7. Étape 6 — Le template

```html
{# runique/templates/admin/blocks/list/filters.html (extrait) #}
{% for filter_entry in resource.display.list_filter %}
  {% set col = filter_entry[0] %}
  {% set label = filter_entry[1] %}
  {% set values = filter_values[col] %}
  {% if values and values | length > 0 %}
  <div class="admin-filter__group" data-filter-col="{{ col }}">
    <button class="admin-filter__group-title" type="button" aria-expanded="true">{{ label }}</button>
    <div class="admin-filter__group-body">
      {% if filter_clear[col] %}
      <a href="{{ filter_clear[col] }}" hx-get="{{ filter_clear[col] }}"
        hx-target="#list-content" hx-swap="outerHTML" hx-push-url="true"
        class="admin-filter__option admin-filter__option--clear">{{ admin_list_filters_clear }}</a>
      {% endif %}
      {# Each link turns its value on or off, the others kept. #}
      {% for option in values %}
        <a href="{{ option.qs }}" hx-get="{{ option.qs }}"
          hx-target="#list-content" hx-swap="outerHTML" hx-push-url="true"
          class="admin-filter__option {% if option.active %}admin-filter__option--active{% endif %}">
          {{ option.value | replace(from="_", to=" ") }}
        </a>
      {% endfor %}
      {% set pager = filter_pagers[col] %}
      {% if pager.prev_qs or pager.next_qs %}
        {% include "admin/blocks/list/pager.html" %}
      {% endif %}
    </div>
  </div>
  {% endif %}
{% endfor %}
```

- `list_filter` : les colonnes déclarées, avec leur libellé, dans l'ordre de la déclaration ;
- `filter_values[col]` : la page de valeurs (`value`, `active`, `qs`) de la section 6 ; `qs` commence déjà par `?` ;
- `filter_clear[col]` : le lien « tout effacer » du groupe (`filter_clear_qs`) ;
- `filter_pagers[col]` : les flèches de la section 4.3, présentes seulement s'il y a une page avant ou après ;
- `hx-get`, `hx-target`, `hx-push-url` : htmx remplace seulement la liste (`#list-content`) au clic, et met l'URL à jour ; sans JavaScript, le `href` recharge la page entière avec le même résultat ;
- `active_filters` et `filter_qs` : l'état des filtres actifs, repris par les liens de pagination et de tri pour ne pas perdre la sélection ;
- `{{ option.value }}` est échappé par Tera (autoescape actif sur `.html`) : une valeur contenant du HTML s'affiche comme du texte, sans être exécutée.

## 8. Étape 7 — Repli des groupes côté navigateur

Chaque groupe peut être replié. Les groupes sont **repliés par défaut** ; le choix est gardé dans le `sessionStorage` du navigateur, par ressource et par colonne :

```js
// runique/static/js/admin/admin-list.js (extrait)
const KEY = 'runique_fg_' + RESOURCE + '_' + col;
const stored = sessionStorage.getItem(KEY);
// Replié par défaut sauf préférence explicite (sessionStorage)
```

```
clé : runique_fg_doc_page_lang
      '1'               → ouvert
      autre ou absent   → replié
```

`sessionStorage` vit le temps de l'onglet : l'état survit à la navigation entre les pages de la liste, sans rien stocker côté serveur ni dans un cookie. La clé contient la ressource : deux listes ont des groupes indépendants.

## 9. Sécurité

| Risque | Protection |
|---|---|
| filtrer sur une colonne non prévue (`filter_password=...`) | seules les colonnes de `.filter([...])` sont lues (boucle sur la déclaration, section 5.2) |
| déclarer une colonne secrète | refus au démarrage (section 3) |
| injection SQL par une valeur | valeur passée en paramètre lié, jamais en texte SQL |
| fuite de valeurs hors des droits du compte | les valeurs proposées sont calculées dans le périmètre autorisé (`scope`) |
| page démesurée dans l'URL | arithmétique saturante, offset borné |
| HTML dans une valeur | échappement automatique de Tera |
| charger une grosse table en mémoire | `DISTINCT`, `LIMIT` et `OFFSET` côté base |

## 10. Résumé des choix de conception

| Choix | Alternative écartée | Pourquoi |
|---|---|---|
| Colonnes typées (`Column::…`) | chaînes libres | erreur à la compilation |
| Vérification au démarrage | vérification à la requête | une configuration dangereuse n'atteint jamais la production |
| Valeurs distinctes en SQL | charger toute la table | tient la charge sur de grosses tables |
| Lire `size + 1` valeurs | `COUNT(DISTINCT ...)` | une requête de moins |
| Comparaison en texte (`CAST`) | un code par type de colonne | un seul chemin pour tous les types |
| Liste blanche `.filter([...])` | accepter tout `filter_*` | aucun nom de colonne n'est lu dans l'URL |
| Périmètre autorisé dans la requête des valeurs | valeurs de toute la table | aucune valeur ne fuit hors des droits |
| `sessionStorage` par ressource et colonne | état global, cookie | groupes indépendants, rien côté serveur |

## 11. Exercices

### Exercice 1 — lire la condition

Quelle condition produit `?filter_statut=publie&filter_statut=brouillon&filter_auteur=7&filter_password=x` avec `.filter([Statut, Auteur])` ?

Réponse : `(statut = 'publie' OR statut = 'brouillon') AND (auteur = '7')`, en comparaison texte. `filter_password` est ignoré : la colonne n'est pas déclarée.

### Exercice 2 — page suivante

Écris une fonction qui reçoit les valeurs lues (au plus `size + 1`) et renvoie la page à afficher et la présence d'une page suivante.

```rust
// Solution :
fn decouper(mut valeurs: Vec<String>, size: usize) -> (Vec<String>, bool) {
    let suivante = valeurs.len() > size;
    valeurs.truncate(size);
    (valeurs, suivante)
}

assert_eq!(decouper(vec!["a".into(), "b".into(), "c".into()], 2), (vec!["a".into(), "b".into()], true));
assert_eq!(decouper(vec!["a".into()], 2), (vec!["a".into()], false));
```

### Exercice 3 — basculer un paramètre

Écris `basculer(params, nom, valeur) -> Vec<(String, String)>` qui ajoute `(nom, valeur)` s'il est absent, le retire s'il est présent, et garde le reste.

```rust
// Solution :
fn basculer(params: &[(String, String)], nom: &str, valeur: &str) -> Vec<(String, String)> {
    let present = params.iter().any(|(k, v)| k == nom && v == valeur);
    params
        .iter()
        .filter(|(k, v)| !(k == nom && v == valeur))
        .cloned()
        .chain((!present).then(|| (nom.to_string(), valeur.to_string())))
        .collect()
}
```

## 12. Points clés

- Les colonnes filtrables sont typées : une faute de frappe ne compile pas.
- Les colonnes secrètes sont refusées au démarrage, pas à la requête.
- Les valeurs proposées sont distinctes, calculées en SQL, dans le périmètre autorisé, paginées en lisant une valeur de plus.
- Seules les colonnes déclarées sont lues dans l'URL ; les valeurs sont des paramètres liés.
- OU entre les valeurs d'une colonne, ET entre les colonnes.
- Les liens se construisent avec une chaîne d'itérateurs : `filter`, `chain(Option)`, `once`.
