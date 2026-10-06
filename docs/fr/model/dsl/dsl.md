# DSL `model!` & `extend!`

## Macros exposées

- `model! { ... }` — déclare un modèle (entité SeaORM + migrations + formulaire admin)
- `extend! { ... }` — ajoute des colonnes à une table framework existante
- `#[form(...)]` — lie un formulaire Rust à un `model!` (voir [Formulaires & enjeux](/docs/fr/model/formulaires))

Toutes sont disponibles via `use runique::prelude::*`.

---

## Structure du DSL `model!`

Le parseur attend les blocs **dans cet ordre strict** (les blocs optionnels peuvent être absents mais pas réordonnés) :

```rust
model! {
    NomModele,              // 1. Nom (PascalCase)
    table: "nom_table",     // 2. Nom de la table SQL
    pk: champ => type,      // 3. Clé primaire
    enums: { ... },         // 4. Optionnel — enums locaux
    { ... },                // 5. Champs — bloc anonyme, types sémantiques
    relations: { ... },     // 6. Optionnel — relations SeaORM
    meta: { ... },          // 7. Optionnel — contraintes & tri
}
```

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    {
        titre:      text      [required, max_length: 150],
        contenu:    textarea  [required],
        is_active:  bool      [default: true],
        created_at: datetime  [auto_now],
    }
}
```

> Le bloc de champs est **toujours** un bloc anonyme `{ ... }`, jamais précédé du mot-clé
> `fields:`. Ne pas confondre avec `extend!{}`, qui lui exige `fields:` — deux macros
> différentes, deux grammaires différentes (voir plus bas).

---

## Clé primaire (`pk`)

```
pk: nom_champ => type
```

| Type   | SQL Postgres          | SQL MySQL               | Auto-incrément | Création                        |
|--------|-----------------------|--------------------------|----------------|----------------------------------|
| `i32`  | `SERIAL`              | `INT AUTO_INCREMENT`     | ✅ Oui          | séquence DB                      |
| `i64`  | `BIGSERIAL`           | `BIGINT AUTO_INCREMENT`  | ✅ Oui          | séquence DB                      |
| `uuid` | `UUID`                | `VARCHAR(36)`            | ❌ Non          | `Uuid::now_v7()` côté Rust        |
| `Pk`   | alias `i32`/`i64`/`Uuid` | idem                   | selon le type   | selon la feature active          |

**L'alias `Pk`** défère au type global de l'application, résolu par feature Cargo — **une seule**
peut être active à la fois (`compile_error!` si deux sont déclarées ensemble) :

```toml
[dependencies]
# rien de déclaré → Pk = i32 (défaut)
runique = { version = "3.0.0", features = ["big-pk"] }    # Pk = i64
runique = { version = "3.0.0", features = ["pk-uuid"] }   # Pk = Uuid (généré via Uuid::now_v7())
```

Utilisez `big-pk` quand vous anticipez plus de ~2 milliards de lignes dans une table, ou pour interopérer avec un schéma existant utilisant des clés primaires `BIGINT`. Utilisez `pk-uuid` pour des identifiants non séquentiels (multi-tenant, génération côté client, exposition publique des ids sans révéler le volume de lignes).

### `Pk` sur un champ normal (pas seulement la PK)

Le mot-clé `Pk` est aussi utilisable sur un **champ FK ordinaire**, pas uniquement dans
`pk: id => Pk`. C'est la façon recommandée de déclarer une colonne qui référence la clé
primaire d'une autre table : elle suit alors automatiquement la même feature, sans jamais se
désynchroniser si vous changez `big-pk`/`pk-uuid` plus tard.

```rust
model! {
    Chapitre,
    table: "chapitre",
    pk: id => Pk,
    {
        cour_id: Pk [required],   // suit automatiquement le type de Cours.id
        titre:   text [required],
    },
    relations: {
        belongs_to: Cour via cour_id,
    }
}
```

**À éviter** : déclarer une FK avec un type figé (`cour_id: int [required]`) quand la table
référencée utilise `pk: id => Pk`. Ça compile et fonctionne tant que `Pk` vaut `i32`, mais se
désynchronise silencieusement dès qu'une feature (`big-pk`/`pk-uuid`) change — le même type de
bug que documenté plus bas pour `big-pk`. Utiliser `Pk` sur le champ FK élimine le risque à la
source, sans code de conversion (`.try_into()`) à maintenir.

**Contrainte lors de l'activation de `big-pk`/`pk-uuid`** : chaque colonne FK pointant vers une
clé primaire `Pk` doit rester cohérente avec elle. La forme `cour_id: Pk` (ci-dessus) le garantit
automatiquement ; une colonne figée en `bigint`/`int`/`uuid` doit être mise à jour manuellement
si vous changez de feature après coup.

> **Le choix de `big-pk`/`pk-uuid` doit être fait avant la première migration.**
> Une fois les migrations appliquées, basculer de mode est un changement cassant : les colonnes
> en base ont déjà un type concret, et changer la feature ne modifie que le type Rust — le
> schéma reste intact. Changer après coup nécessite une migration manuelle pour `ALTER` chaque
> colonne PK et FK, avec un risque de troncature (`big-pk` → défaut) ou d'incompatibilité totale
> de format (`pk-uuid` ↔ n'importe quel type entier). Choisissez un mode au démarrage du projet.

---

## Types de champs

Le type Rust indiqué est celui d'une colonne NOT NULL ; une colonne `nullable` donne `Option<T>`.

| Type DSL          | Type Rust généré          | Colonne SQL créée par `makemigrations` |
|--------------------|---------------------------|---------------------------------|
| `text`             | `String`                  | `VARCHAR`, ou `VARCHAR(n)` avec `max_length: n` |
| `char`             | `String`                  | `VARCHAR`                       |
| `email`            | `String`                  | `VARCHAR(254)` — format validé  |
| `password`         | `String`                  | `VARCHAR` — haché automatiquement |
| `richtext`         | `String`                  | `TEXT` — éditeur HTML (nettoyé) |
| `textarea`         | `String`                  | `TEXT` — multi-ligne            |
| `url`              | `String`                  | `VARCHAR` — format validé       |
| `slug`             | `String`                  | `VARCHAR`                       |
| `color`            | `String`                  | `VARCHAR` — couleur hex         |
| `phone`            | `String`                  | `VARCHAR(20)`, ou `VARCHAR(n)` avec `max_length: n` |
| `i8`               | `i8`                      | `TINYINT`                       |
| `i16`              | `i16`                     | `SMALLINT`                      |
| `int`              | `i32`                     | `INTEGER`                       |
| `bigint`           | `i64`                     | `BIGINT`                        |
| `u32`              | `u32`                     | `INTEGER UNSIGNED`              |
| `u64`              | `u64`                     | `BIGINT UNSIGNED`               |
| `f32`              | `f32`                     | `FLOAT`                         |
| `float`            | `f64`                     | `DOUBLE`                        |
| `percent`          | `f64`                     | `DOUBLE` — formulaire borné à 0–100 |
| `decimal`          | `Decimal`                 | `DECIMAL`                       |
| `bool`             | `bool`                    | `BOOLEAN`                       |
| `date`             | `NaiveDate`                | `DATE`                          |
| `time`             | `NaiveTime`                | `TIME`                          |
| `datetime`         | `NaiveDateTime`            | `DATETIME`                      |
| `timestamp`        | `NaiveDateTime`            | `DATETIME`                      |
| `timestamp_tz`     | `DateTime<Utc>`            | `TIMESTAMP WITH TIME ZONE`      |
| `uuid`             | `Uuid`                     | `UUID`                          |
| `Pk`               | `i32`/`i64`/`Uuid`         | selon la feature — voir ci-dessus |
| `json`             | `serde_json::Value`        | `JSON`                          |
| `json_binary`      | `serde_json::Value`        | `JSON`                          |
| `binary`           | `Vec<u8>`                  | `BINARY(n)` — 255 par défaut, `max_length: n` |
| `var_binary`       | `Vec<u8>`                  | `VARBINARY(n)` — 255 par défaut, `max_length: n` |
| `blob`             | `Vec<u8>`                  | `BLOB`                          |
| `ip`               | `String`                   | `VARCHAR` — format validé       |
| `cidr`             | `String`                   | `VARCHAR`                       |
| `mac_address`      | `String`                   | `VARCHAR`                       |
| `interval`         | `String`                   | `VARCHAR`                       |
| `image`            | `String`                   | `VARCHAR` — chemin du fichier   |
| `document`         | `String`                   | `VARCHAR` — chemin du fichier   |
| `file`             | `String`                   | `VARCHAR` — chemin du fichier   |
| `choice`           | `NomEnum`                  | colonne de l'enum — requiert `enum(NomEnum)`, rendu `<select>` |
| `radio`            | `NomEnum`                  | idem `choice`, rendu en boutons radio |
| `multichoice`      | — (liste, voir plus bas)   | table à part `{table}_{champ}` — rendu `<select multiple>` |
| `checkbox`         | — (liste, voir plus bas)   | idem `multichoice`, rendu en cases à cocher |

`VARCHAR` sans longueur prend la valeur par défaut du moteur : illimité sous Postgres,
`VARCHAR(255)` sous MySQL/MariaDB, `TEXT` sous SQLite. `ip`, `cidr`, `mac_address` et
`interval` restent des `VARCHAR` : leurs types natifs n'existent que sous Postgres.

**Limites des moteurs**, refusées à la compilation quand une seule feature de moteur est
active : `i8` et `u32` sous Postgres, `u64` sous Postgres et SQLite (la valeur ne peut pas être
relue dans le type Rust). Sous MySQL/MariaDB, `binary` est une colonne à **longueur fixe**,
complétée par des `0x00` : utilisez `var_binary` pour relire exactement les octets écrits.

> **Non disponible** : `decimal(precision, scale)` inline (ex. `decimal(10, 2)`) n'a pas
> d'équivalent actuel — seul `decimal` sans paramètres est supporté. Contournement : appliquer
> la précision/l'échelle côté validation applicative plutôt que dans le schéma.

---

## Options de champ

Dans un bloc `[...]`, séparées par des virgules, valeur après `:` quand l'option en prend une :

```rust
username: text [required, max_length: 150, unique],
```

| Option                   | Description                                                     |
|--------------------------|-------------------------------------------------------------------|
| `required`               | Champ **obligatoire dans le formulaire**. Ne dit rien de la colonne |
| `nullable`               | Colonne `NULL` — type Rust `Option<T>`                           |
| `unique`                 | Contrainte `UNIQUE`                                              |
| `max_length: n`          | Longueur max : validation **et** taille de colonne (`VARCHAR(n)`, `BINARY(n)`, `VARBINARY(n)`) |
| `min_length: n`          | Longueur min (validation)                                        |
| `min: n` / `max: n`      | Bornes entières (validation)                                     |
| `min: n.0` / `max: n.0`  | Bornes flottantes (validation)                                   |
| `default: valeur`        | Valeur par défaut SQL (`true`, `0`, `"draft"`, etc.)             |
| `auto_now`               | Rempli à la création — exclu des formulaires                    |
| `auto_now_update`        | Rempli à chaque enregistrement — exclu des formulaires          |
| `readonly`               | Exclu de la migration générée (colonne existe côté Rust, non gérée par `derive_form`) |
| `label: "str"`           | Libellé personnalisé dans les formulaires                        |
| `upload_to: "path"`      | Champ fichier — dossier d'upload                                 |
| `max_size: n MB`         | Champ fichier — taille max (`KB`/`MB`/`GB`)                      |
| `rows: n`                | `textarea`/`richtext` — hauteur du widget                        |
| `step: n`                | Champs numériques — pas du widget                                |
| `enum(NomEnum)`          | Lie le champ à un enum déclaré dans `enums:`                     |
| `renamed_from: "x"`      | Renomme la colonne (voir plus bas)                                |
| `skip`                   | Exclu des formulaires générés                                    |
| `no_hash`                | Champs `password` uniquement — désactive le hachage automatique  |

Un attribut inconnu ou non valide pour le type est une **erreur de compilation**, qui nomme le
champ et l'attribut. `makemigrations` lit le DSL avec le même parseur que la macro : un modèle
refusé par l'un est refusé par l'autre, avec le fichier, la ligne et la colonne.

> **Le nom d'une colonne ne décide de rien.** Une colonne `created_at` sans `auto_now` n'a pas de
> valeur par défaut, une colonne `cache_key` est migrée comme les autres : seuls les attributs
> comptent.

> **`readonly`** (DB-level) est distinct du `readonly` de `#[form]`
> (`field_readonly()`, désactive un champ dans le rendu HTML d'une instance de formulaire
> précise). `readonly` sur le champ du modèle exclut la colonne de la migration générée ;
> `field_readonly()` désactive juste un widget au runtime. Les deux peuvent coexister.

### Nullabilité

Une colonne est **NOT NULL**, sauf si elle est déclarée `nullable`. `required` ne concerne que
le formulaire. Le parseur refuse, à la compilation :

- `required` et `nullable` ensemble ;
- `nullable` sur `auto_now` / `auto_now_update` : le framework les remplit toujours ;
- un champ qui n'est pas de type texte (nombre, date, choix, uuid, json…) déclaré sans
  `required`, `nullable` ni `default` — un champ facultatif laissé vide n'aurait rien à
  enregistrer dans une colonne NOT NULL. Les champs texte, fichier, binaire et `bool` peuvent
  rester facultatifs : ils envoient toujours une valeur enregistrable (texte vide, aucun fichier,
  case décochée).

```rust
{
    titre:      text  [required],          // NOT NULL, obligatoire dans le formulaire
    sous_titre: text,                      // NOT NULL, facultatif (texte vide)
    resume:     text  [nullable],          // NULL possible — Option<String>
    note:       int   [default: 0],        // NOT NULL, 0 si rien n'est saisi
    publie_le:  date  [nullable],          // NULL possible — Option<NaiveDate>
}
```

### `auto_now` / `auto_now_update`

Remplis par l'entité elle-même (`ActiveModelBehavior::before_save`), de la même façon sur tous
les moteurs : `auto_now` à la création s'il n'est pas déjà posé, `auto_now_update` à chaque
enregistrement. La migration ajoute aussi `DEFAULT CURRENT_TIMESTAMP`, pour une insertion faite
hors de SeaORM. Le type Rust est `T` (jamais `Option<T>`), et ces champs sont exclus de
`admin_from_form` et d'`admin_partial_update`.

### Longueurs de colonne

`max_length` donne la taille de la colonne, suivie par les snapshots : l'agrandir produit un
simple `ALTER`, la réduire demande `--force` (des valeurs plus longues seraient tronquées ou
refusées). Un snapshot écrit avant la 3.0 reprend une seule fois les longueurs du modèle, sans
produire de migration.

### Bornes et `customize`

`min_length`, `max_length`, `min` et `max` passent dans les formulaires générés depuis le
modèle. Un formulaire peut les **resserrer** dans `customize`, jamais les **desserrer** : un
`customize` qui allonge un `max_length`, abaisse un `min`, etc. provoque un panic à la
construction du formulaire, avec un message qui nomme le champ et la borne.

### Renommer une colonne — `renamed_from`

Renommer un champ sans cette option produit un `DROP COLUMN` + `ADD COLUMN` → **perte de données**.
L'outil étant non interactif, il ne peut pas deviner l'intention : il faut le signaler explicitement.

```rust
// avant :  job_title: text,
// après :
title: text [renamed_from: "job_title"],
```

`makemigrations` génère alors un `ALTER TABLE … RENAME COLUMN job_title TO title` (supporté par
PostgreSQL, MySQL/MariaDB et SQLite), sans perte de données. L'attribut est une directive de
migration uniquement : il n'a aucun effet sur l'entité ou le formulaire générés. Garde-fou : si
l'ancienne colonne existe toujours dans le snapshot (hint périmé), aucun rename n'est émis.

Fonctionne aussi bien dans `model!{}` que dans `extend!{}`.

---

## Enums

Les enums se déclarent dans un bloc `enums: { ... }` distinct des champs, puis sont référencés via `enum(NomEnum)`.

```rust
model! {
    Commande,
    table: "commandes",
    pk: id => i32,
    enums: {
        StatutCommande: [
            EnAttente  = ("en_attente",  "En attente"),
            EnCours    = ("en_cours",    "En cours"),
            Livree     = ("livree",      "Livrée"),
            Annulee    = ("annulee",     "Annulée"),
        ],
        Priorite: i32 [Basse = 0, Normale = 1, Haute = 2, Urgente = 9],
    },
    {
        statut:   choice [enum(StatutCommande), required],
        priorite: choice [enum(Priorite), required],
    },
}
```

### Quatre formes de variant — à ne jamais confondre

> **Attention, piège fréquent** : `:` et `=` ne font **pas** la même chose. Une seule
> différence de symbole change complètement le comportement, sans erreur de compilation pour
> vous avertir. Toujours vérifier sur ce tableau, ne pas deviner par analogie.

| Syntaxe                              | Valeur stockée en DB | Libellé affiché (`Display`)                  |
|---------------------------------------|----------------------|-----------------------------------------------|
| `Variant`                             | `"Variant"` (le nom)  | `"Variant"` — retombe sur la valeur DB         |
| `Variant: "Libellé"`                  | `"Variant"` (**inchangé**) | `"Libellé"`                               |
| `Variant = "valeur_db"`               | `"valeur_db"`         | `"valeur_db"` — retombe sur la valeur DB, **pas** sur `Variant` |
| `Variant = ("valeur_db", "Libellé")`  | `"valeur_db"`         | `"Libellé"`                                    |

Résumé de la règle :
- `:` (deux-points) ne touche **que l'affichage** — la valeur stockée reste toujours le nom du variant.
- `=` seul (sans parenthèses) ne touche **que la valeur stockée** — l'affichage retombe dessus, jamais sur le nom du variant.
- `= (a, b)` fixe les deux indépendamment — c'est la seule forme qui permet un nom de variant, une valeur DB et un libellé tous différents.

**Le libellé est purement cosmétique.** Il n'affecte :
- ni le stockage réel (`#[sea_orm(string_value = ...)]` / `#[sea_orm(num_value = ...)]` utilisent toujours la valeur DB, jamais le libellé),
- ni la comparaison de parsing (`FromStr` compare en priorité contre la valeur DB et le nom du variant Rust — le libellé n'est accepté qu'en repli, seulement s'il diffère des deux autres).

Changer un libellé (`Published: "Publié"` → `Published: "Mis en ligne"`) n'a donc **aucun** impact sur les données stockées ni sur le code qui compare des valeurs d'enum.

> **La valeur DB est stockée exactement telle qu'écrite.** Aucune transformation automatique.

### Types de backing

| Syntaxe              | Stockage DB                                     |
|----------------------|--------------------------------------------------|
| `NomEnum: [A, B]`     | `ENUM` natif (Postgres) ou `VARCHAR` (MySQL/SQLite) |
| `NomEnum: i8 [...]`   | `TINYINT` — refusé sous Postgres                  |
| `NomEnum: i16 [...]`  | `SMALLINT`                                        |
| `NomEnum: i32 [...]`  | `INTEGER`                                         |
| `NomEnum: i64 [...]`  | `BIGINT`                                          |

Pour un enum entier, `=` fixe la valeur numérique stockée, et **chaque variante doit en avoir
une** (`Basse = 0` ou `Basse = (0, "Basse")`), dans les limites du type, sans doublon. Sinon,
c'est une erreur de compilation : deux variantes avec la même valeur seraient relues l'une pour
l'autre.

### Méthodes générées

| Méthode | Retour | Description |
|---------|--------|-------------|
| `.to_string()` | `String` | Libellé d'affichage |
| `.db_value()` | `&'static str` / `i8` … `i64` | Valeur exacte en base |
| `.form_value()` | `&'static str` | Valeur envoyée par un formulaire (valeur stockée pour un enum texte, nom de la variante pour un enum entier) |
| `::from_str(s)` / `.parse()` | `Result<Self, ()>` | Parsing depuis valeur DB, libellé, ou nom variant |
| `::iter()` | `impl Iterator<Item = Self>` | Itération sur tous les variants |

```rust
use sea_orm::Iterable;

let s = StatutCommande::EnAttente;
s.db_value()   // → "en_attente"
s.to_string()  // → "En attente"

// Pour un <select>
let options: Vec<(String, String)> = StatutCommande::iter()
    .map(|v| (v.db_value().to_string(), v.to_string()))
    .collect();

// Parser depuis une valeur DB
let statut: Option<StatutCommande> = "en_attente".parse().ok();
```

**Dans les templates Tera**, la valeur de comparaison doit correspondre **exactement** à ce qui est stocké en base (sensible à la casse).

---

## Champs fichier

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    {
        image:        image    [upload_to: "media/articles"],
        fichier:      document [upload_to: "docs/"],
        piece_jointe: file     [upload_to: "media/uploads"],
    },
}
```

| Type      | Extensions autorisées          |
|-----------|--------------------------------|
| `image`   | `jpg jpeg png gif webp avif`   |
| `document`| `pdf doc docx txt odt`         |
| `file`    | aucun filtre                   |

`upload_to:` est obligatoire pour les trois types. Le chemin est relatif à `MEDIA_ROOT`.

---

## Relations

```rust
relations: {
    belongs_to: cour via cour_id [cascade],          // clé étrangère
    has_many: Commentaire,
    has_many: Commentaire as commentaires,           // alias optionnel
    has_one: Profil as profil,
    many_to_many: Role through UserRole via user_id,
}
```

| Type             | Contrainte DB   | Description                  |
|------------------|-----------------|-------------------------------|
| `belongs_to`     | ✅ `FOREIGN KEY` + index | Relation N-1             |
| `has_many`       | ❌ code seul     | Relation 1-N                 |
| `has_one`        | ❌ code seul     | Relation 1-1                 |
| `many_to_many`   | ❌ code seul     | Relation N-N via une table pivot déclarée à part |

### `belongs_to` — la clé étrangère

`belongs_to: cible via colonne [on_delete, on_update]` est la **seule** façon de déclarer une
clé étrangère. Elle produit à la fois la contrainte SQL et la relation SeaORM :

- **`cible`** est le module de l'entité visée — le fichier `cible.rs` de `src/entities/` — ou une
  table du framework (`eihwaz_users`, …). La clé vise la **vraie table et la vraie clé
  primaire** de ce modèle, quel que soit son nom (`pk: code => i32` compris). Une cible
  introuvable est une erreur de `makemigrations`.
- **`colonne`** doit être un champ déclaré du modèle (de préférence `Pk`, voir plus haut).
- **Actions** entre crochets : la première pour `ON DELETE`, la seconde (facultative) pour
  `ON UPDATE` — `cascade`, `set_null`, `restrict`, `set_default`, `no_action` (par défaut). Une
  action inconnue est une erreur de compilation, et `set_null` exige une colonne `nullable`.
- **Index** : `makemigrations` crée un index `idx_<table>_<colonne>` sur chaque colonne de
  `belongs_to`, comme Django — sauf si la colonne est déjà `unique` ou en tête d'un index déclaré
  dans `meta`.
- **Cycles** : deux nouvelles tables qui se référencent l'une l'autre sont gérées. Sous SQLite,
  la clé reste dans le `CREATE TABLE` ; sous Postgres et MySQL, celle qui ferme le cycle est
  ajoutée par un `ALTER TABLE` une fois l'autre table créée.

> Depuis la 3.0, l'ancienne option de champ `fk(table.col, action)` n'existe plus : elle produit
> une erreur de compilation qui renvoie vers `belongs_to`.

---

## Champs liste — `multichoice` et `checkbox`

Un champ liste contient **plusieurs valeurs d'un enum**. `multichoice` est rendu en
`<select multiple>`, `checkbox` en cases à cocher ; les deux se stockent et s'utilisent de la
même façon.

```rust
model! {
    Livre,
    table: "livres",
    pk: id => Pk,
    enums: { Genre: [Roman, Policier, Jeunesse = ("jeunesse", "Jeunesse")] },
    {
        titre:  text [required],
        genres: checkbox [enum(Genre), required],   // required = au moins une valeur
    }
}
```

- **Stockage** : pas de colonne dans `livres`, et pas de champ `genres` dans le `Model`. Les
  valeurs vivent dans une table `livres_genres`, créée par `makemigrations` : `id`, `owner_id`
  (FK `ON DELETE CASCADE` vers la ligne propriétaire), `value` (la colonne de l'enum), un index
  unique `(owner_id, value)` et un index `(value, owner_id)` pour les filtres.
- **Attributs** : l'enum est obligatoire ; seuls `required`, `enum(...)` et `label` sont
  acceptés (une liste vide suffit à dire « rien »). Un champ liste ne peut pas être référencé
  dans `meta` ni par `belongs_to`, et n'est pas accepté dans `extend!{}`. Un nom qui masquerait
  une méthode de SeaORM (`get`, `set`, `delete`, `find_related`…) est refusé.

### Lire et écrire

```rust
let genres: Vec<Genre> = livre.genres(&db).await?;          // dans l'ordre d'enregistrement
livre.set_genres(&db, [Genre::Roman, Genre::Policier]).await?; // remplace, en transaction, sans doublon
let par_livre = livre::Model::load_genres(&db, &livres).await?; // une page entière : 1 requête
```

### Filtrer

```rust
search!(livre::Entity => Genres has Genre::Roman)
search!(livre::Entity => Genres has_any [Genre::Roman, Genre::Policier])
search!(livre::Entity => Genres has_all [Genre::Roman, Genre::Policier])
search!(livre::Entity => !Genres has Genre::Roman, Titre icontains "nuit")

livre::Entity::objects.filter(livre::List::Genres.has(Genre::Roman))   // sans search!
```

Les filtres sont des sous-requêtes portables (Postgres, MySQL/MariaDB, SQLite) et sont typés :
passer une valeur d'un autre enum ne compile pas.

### Postgres : lignes et listes en une requête

Avec la feature `postgres`, `fetch_with` renvoie les lignes d'une requête **avec** leur liste,
en un seul aller-retour (filtre et ordre de la requête conservés) :

```rust
let page: Vec<(livre::Model, Vec<Genre>)> =
    livre::List::Genres.fetch_with(&db, livre::Entity::find().limit(50)).await?;
```

Sur un autre moteur, elle renvoie une erreur ; utilisez `load_genres` après avoir lu la page.

### Formulaires et admin

L'admin affiche le champ (cases ou sélection multiple), enregistre la ligne et sa liste dans la
même transaction, et pré-remplit le formulaire d'édition. Dans un formulaire, les valeurs
cochées se lisent avec `cleaned_enums::<Genre>("genres")`.

---

## Meta

```rust
meta: {
    ordering: [-created_at, titre],
    unique_together: [(slug, lang)],
    indexes: [(lang, sort_order)],
    verbose_name: "Article",
    verbose_name_plural: "Articles",
}
```

| Clé                   | Syntaxe               | Effet                                       |
|-----------------------|-----------------------|---------------------------------------------|
| `ordering`            | `[champ, -champ]`     | Tri par défaut, `-` = `DESC`                |
| `unique_together`     | `[(col1, col2)]`      | Contrainte `UNIQUE` multi-colonnes          |
| `indexes`             | `[(col1, col2)]`      | Index simple multi-colonnes                 |
| `verbose_name`        | `"chaîne"`            | Nom singulier dans l'interface admin        |
| `verbose_name_plural` | `"chaîne"`            | Nom pluriel dans l'interface admin          |

---

## `extend!{}` — extension des tables framework

Ajoute des colonnes à une table Runique et génère une entité SeaORM complète sur cette table.

`extend!{}` produit deux choses :

1. **Schema SQL** — `makemigrations` détecte le bloc et génère des instructions `ALTER TABLE ADD COLUMN`
2. **Entité complète** — `Model`, `Column`, `Entity`, `AdminForm`, `admin_from_form`, `admin_partial_update` couvrant **toutes** les colonnes de la table (colonnes de base + colonnes étendues)

```rust
// src/entities/user_profile.rs
use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        bio:         textarea [nullable],
        avatar:      image    [nullable, upload_to: "avatars/"],
        website:     url      [nullable],
        phone:       phone    [nullable],
        birth_date:  date     [nullable],
        is_verified: bool     [default: false],
    }
}
```

> `extend!{}` exige **toujours** le mot-clé `fields:` avant le bloc de champs — contrairement à
> `model!{}` (bloc anonyme direct). Ce sont deux macros différentes, deux grammaires
> différentes ; ne pas transposer la syntaxe de l'une à l'autre.

Tables autorisées : `eihwaz_users`, `eihwaz_groupes`, `eihwaz_sessions`, `eihwaz_users_groupes`, `eihwaz_groupes_droits`. Tout autre nom provoque une erreur à la compilation.

Les champs déclarés dans `extend!{}` utilisent les mêmes types, options et règles de nullabilité que `model!` (y compris `renamed_from`). Les colonnes ajoutées à une table existante seront remplies pour les lignes déjà présentes : déclarez-les `nullable` ou avec un `default`. Pas de bloc `relations:` ni de champ liste (`multichoice`/`checkbox`) dans `extend!{}`.

### Enums dans `extend!{}`

`extend!{}` accepte un bloc `enums: { ... }` optionnel (entre `table:` et `fields:`), identique à celui de `model!`. La colonne `choice [enum(NomEnum)]` génère le type Rust enum, la colonne typée et le `ChoiceField` peuplé :

```rust
extend! {
    table: "eihwaz_users",
    enums: {
        Seniority: [Junior="junior", Mid="mid", Senior="senior", Lead="lead"],
    },
    fields: {
        job_title: text [nullable],
        seniority: choice [enum(Seniority), nullable],
    }
}
```

`makemigrations` émet la colonne (sur PostgreSQL, un `CREATE TYPE … AS ENUM` ; ailleurs, un `VARCHAR`/`ENUM` natif).

### Workflow complet

```bash
# 1. Déclarer l'extension dans src/entities/
# 2. Générer la migration
runique makemigrations

# 3. Appliquer
runique migration up

# 4. Enregistrer dans admin!{} (src/admin.rs)
```

```rust
admin! {
    configure {
        users: { hidden: true }   // masque le panel builtin "Utilisateurs"
    }
    user_profile: user_profile::Model => user_profile::AdminForm {
        title: "Profils utilisateurs",
        list_display: [
            ["username", "Utilisateur"],
            ["bio", "Bio"],
            ["is_verified", "Vérifié"],
        ],
    }
}
```

### Ce qui est généré

| Symbole | Description |
| ------- | ----------- |
| `Model` | Struct avec toutes les colonnes (base + étendues) |
| `Column` | Enum SeaORM pour les colonnes |
| `Entity` | `EntityTrait` complet — utilisable avec `search!` |
| `AdminForm` | Formulaire admin couvrant toutes les colonnes |
| `admin_from_form` | Construit un `ActiveModel` depuis les données du formulaire |
| `admin_partial_update` | Construit un `ActiveModel` partiel pour la mise à jour |

### Requêtes depuis les vues

L'entité générée est un `EntityTrait` SeaORM standard — `search!` fonctionne directement :

```rust
// Tous les profils vérifiés
let profiles = search!(user_profile::Entity => IsVerified eq true).all(&db).await?;

// Recherche multi-colonnes
let results = search!(user_profile::Entity => or(Username icontains q, Bio icontains q)).all(&db).await?;
```

### Relations vers une table du framework

Une entité pointe vers une table du framework par son nom de table, dans le bloc `relations:` habituel de `model!{}` :

```rust
model! {
    Article,
    table: "articles",
    pk: id => Pk,
    { auteur_id: Pk [required] },
    relations: {
        belongs_to: eihwaz_users via auteur_id [cascade],
    }
}
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Génération & ModelSchema](/docs/fr/model/generation) | Code généré, `schema()`, `ModelSchema` |
| [Formulaires & enjeux](/docs/fr/model/formulaires) | `#[form(...)]`, liaison modèle/formulaire |

## Retour au sommaire

- [Models](/docs/fr/model)
