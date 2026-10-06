🌍 **Langues** : [English](/runique/runique_dsl/README.md) | [Français](/runique/runique_dsl/README.fr.md)

# runique_dsl

Lit le DSL de modèles de [Runique](https://runique.io/) — `model!{}` et `extend!{}` — et le
transforme en AST, sans générer de code.

Deux outils lisent le DSL à travers cette crate :

- les macros de `derive_form`, qui génèrent l'entité SeaORM, les formulaires et le schéma ;
- `runique makemigrations`, qui relit les mêmes déclarations dans vos fichiers sources pour
  écrire les migrations SQL.

Comme les deux partagent un seul parseur, un modèle refusé par la macro est refusé par la CLI,
au même endroit et avec le même message (fichier, ligne, colonne). Rien n'est ignoré en silence.

## Le DSL

```rust
model! {
    Book,
    table: "books",
    pk: id => Pk,
    enums: {
        Status: [Draft, Published = ("published", "Publié")],
        Priority: i32 [Low = 1, High = (2, "Haute")],
    },
    {
        title:      text [required, max_length: 200],
        summary:    textarea [nullable],
        status:     choice [enum(Status), required],
        priority:   choice [enum(Priority), required],
        price:      decimal [required, min: 0.0],
        shelf_code: int [nullable],
        cover:      image [nullable, upload_to: "covers/"],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    },
    relations: {
        belongs_to: shelf via shelf_code [set_null, cascade],
    },
    meta: {
        ordering: [-created_at],
        unique_together: [(title, shelf_code)],
        indexes: [(status)],
        verbose_name: "Livre",
    }
}
```

`extend!{}` ajoute des champs à une table du framework, avec la même grammaire de champs et
d'enums :

```rust
extend! {
    table: "eihwaz_users",
    fields: {
        bio:     textarea [nullable],
        website: url [nullable],
    }
}
```

### En-tête

- `table: "nom"` — nom de la table SQL, vérifié comme identifiant.
- `pk: nom => i32 | i64 | uuid | Pk` — `Pk` suit les features `big-pk` / `pk-uuid` (`i32` par
  défaut).

### Enums

- `Nom: [A, B]` — stocké sous le nom de la variante (enum natif sous Postgres, `VARCHAR`
  ailleurs).
- `A = "valeur"`, `A: "libellé"`, `A = ("valeur", "libellé")` — valeur stockée et/ou libellé
  affiché.
- `Nom: i8 [...]`, `i16`, `i32`, `i64` — stocké sous forme de nombre (`i8` refusé sous Postgres). **Chaque variante doit
  avoir une valeur entière** (`Low = 1` ou `Low = (1, "libellé")`), dans les limites du type, et
  deux variantes ne peuvent pas partager la même.

### Attributs de champ

| Attribut | Effet |
|---|---|
| `required` | Le champ du formulaire est obligatoire. Ne dit rien de la colonne. |
| `nullable` | La colonne accepte NULL et le champ Rust est `Option<T>`. |
| `default: litéral` | Valeur par défaut SQL ; le littéral doit correspondre au type. |
| `unique` | Contrainte UNIQUE. |
| `max_length: n` / `min_length: n` | Longueur du texte (et en octets pour `binary`/`var_binary`). `max_length` donne `VARCHAR(n)`. |
| `min: n` / `max: n` | Bornes entières ou flottantes, vérifiées par les formulaires. |
| `auto_now` / `auto_now_update` | Remplis à l'insertion / à chaque enregistrement ; absents des formulaires. |
| `enum(Nom)` | L'enum dont un `choice` / `radio` / `checkbox` tire ses valeurs. |
| `upload_to: "dossier"`, `max_size: 5MB` | Champs fichier (`upload_to` y est obligatoire). |
| `label: "…"`, `rows: n`, `step: x`, `no_hash` | Détails de rendu du formulaire. |
| `readonly` | Exclu des migrations et des formulaires. |
| `skip` | Présent dans le schéma SQL, exclu des formulaires générés. |
| `renamed_from: "ancien"` | Migrations uniquement : renomme la colonne au lieu de la supprimer et la recréer. |

### Nullabilité

Une colonne est **NOT NULL sauf si elle est déclarée `nullable`**. Le parseur refuse, à la
compilation :

- `required` avec `nullable` ;
- `nullable` sur `auto_now` / `auto_now_update` (le framework les remplit toujours) ;
- un champ qui n'est pas de type texte (nombre, date, choix, uuid, json…) déclaré sans
  `required`, `nullable` ni `default` — un champ facultatif laissé vide n'aurait rien à
  enregistrer. Les champs texte, fichier, binaire et `bool` peuvent rester facultatifs : ils
  envoient toujours une valeur enregistrable.

### Relations

- `belongs_to: cible via colonne [on_delete, on_update]` — la **seule** façon de déclarer une clé
  étrangère. Elle produit la contrainte SQL et la relation SeaORM. `cible` est le module de
  l'entité (le fichier `cible.rs`) ou une table du framework (`eihwaz_users`, …) ; la clé vise la
  vraie table et la vraie clé primaire de ce modèle. Actions : `cascade`, `set_null`,
  `restrict`, `set_default`, `no_action` (par défaut). `colonne` doit être un champ déclaré, et
  `nullable` quand une action est `set_null`.
- `has_many: cible` et `has_one: cible`, suivis éventuellement de `as nom`.
- `many_to_many: cible through table_de_jointure via colonne`.

`makemigrations` crée un index `idx_<table>_<colonne>` sur chaque colonne de `belongs_to` (sauf
si elle est déjà `unique` ou en tête d'un index de `meta`), et gère les cycles entre nouvelles
tables qui se référencent l'une l'autre : sous SQLite la clé reste dans le `CREATE TABLE`, sous
Postgres et MySQL celle qui ferme le cycle est ajoutée par un `ALTER TABLE` une fois sa cible
créée.

### Champs liste — `multichoice` et `checkbox`

```rust
genres:   checkbox [enum(Genre), required],   // cases à cocher
humeurs:  multichoice [enum(Genre)],          // <select multiple>
```

Plusieurs valeurs d'un enum, stockées dans une table à part (`{table}_{champ}` : `owner_id` en
`ON DELETE CASCADE`, `value`, unique `(owner_id, value)`), jamais dans une colonne — donc pas de
champ `genres` dans le `Model`. L'enum est obligatoire, et seuls `required` (au moins une
valeur), `enum(...)` et `label` sont acceptés. Interdit dans `extend!{}`, dans `meta` et comme
colonne de `belongs_to`. La macro génère `model.genres(&db)`, `model.set_genres(&db, valeurs)`,
`Model::load_genres(&db, &models)`, et `List::Genres` pour `search!(… => Genres has v)` /
`has_any` / `has_all`.

## Types

`runique_dsl::types` donne, pour chaque type du DSL, sa colonne, son champ de formulaire et ses
bornes.

| Type DSL | Colonne | Champ de formulaire |
|---|---|---|
| `text`, `char` | `VARCHAR` (`VARCHAR(n)` avec `max_length`) | texte |
| `email` | `VARCHAR(254)` | email |
| `phone` | `VARCHAR(20)` (ou `max_length`) | téléphone |
| `password`, `url`, `slug`, `color` | `VARCHAR` | mot de passe (haché) / url / slug / couleur |
| `textarea`, `richtext` | `TEXT` | zone de texte / texte riche (nettoyé) |
| `image`, `document`, `file` | `VARCHAR` (chemin stocké) | envoi de fichier |
| `int`, `bigint`, `i8`, `i16`, `u32`, `u64` | `INTEGER`, `BIGINT`, `TINYINT`, `SMALLINT`, non signés | entier, limité à la plage du type Rust |
| `float`, `f32`, `percent`, `decimal` | `DOUBLE`, `FLOAT`, `DOUBLE`, `DECIMAL` | nombre (`percent` : 0–100) |
| `bool` | `BOOLEAN` | case à cocher |
| `date`, `time`, `datetime`, `timestamp`, `timestamp_tz` | `DATE`, `TIME`, `DATETIME`, `DATETIME`, `TIMESTAMP WITH TIME ZONE` | sélecteurs de date / heure |
| `uuid` | `UUID` | uuid |
| `json`, `json_binary` | `JSON` | json |
| `binary`, `var_binary`, `blob` | `BINARY(n)`, `VARBINARY(n)` (255 par défaut), `BLOB` | envoi d'octets |
| `ip`, `cidr`, `mac_address`, `interval` | `VARCHAR` | ip / texte |
| `choice`, `radio` + `enum(X)` | la colonne de l'enum | liste / boutons radio |
| `multichoice`, `checkbox` + `enum(X)` | une table à part (voir Champs liste) | sélection multiple / cases à cocher |
| `Pk` | même type que les clés primaires | entier ou uuid |

Limites des moteurs, refusées à la compilation quand une seule feature de moteur est active :
`i8` et `u32` sous Postgres, `u64` sous Postgres et SQLite. Sous MySQL/MariaDB, `binary` est une
colonne à longueur fixe complétée par des `0x00` : utilisez `var_binary` pour relire les octets
exacts.

## Ruptures en 3.0

| Avant | Maintenant |
|---|---|
| Un champ sans `required` était nullable. | NOT NULL sauf `nullable` ; `required` rend seulement le champ du formulaire obligatoire. Ajoutez `nullable` pour garder une colonne facultative existante — sinon `makemigrations` s'arrête sur `nullable -> not_null`. |
| Les champs `auto_now` / `auto_now_update` étaient `Option<T>`. | `T` simple, toujours rempli par le framework. |
| Clés étrangères via `fk(table.colonne, action)` sur le champ. | `relations: { belongs_to: cible via colonne [on_delete, on_update] }`. `fk(...)` est une erreur qui renvoie vers cette syntaxe. |
| `makemigrations` ignorait un modèle illisible et gardait sa propre copie de la grammaire (types v1 `String`, `i32`…). | Un seul parseur strict : type ou attribut inconnu, action de clé étrangère inconnue, deux `model!{}` dans un fichier, cible de `belongs_to` introuvable → erreur avec fichier, ligne et colonne. |
| Les variantes sans valeur d'un enum `i32` / `i64` étaient stockées à `0`. | Chaque variante a une valeur entière unique, dans les limites du type. |
| `customize` qui desserrait un `max_length` déclaré était ramené en silence. | Desserrer `min_length`, `max_length`, `min` ou `max` provoque un panic à la construction du formulaire ; resserrer reste permis. |
| `runique::migration::RelationDef` / `RelationKind`, `ModelSchema::relation()`. | Supprimés (jamais lus). Les relations sont portées par l'entité SeaORM. |
| `checkbox [enum(X)]` stockait une seule valeur dans une colonne. | `checkbox` (et le nouveau `multichoice`) est une liste stockée dans sa propre table. |
| Une colonne nommée `created_at` / `updated_at` recevait `DEFAULT CURRENT_TIMESTAMP`, une colonne `cache_key` était exclue des migrations. | Un nom ne décide de rien : seuls `auto_now`, `auto_now_update` et `readonly` comptent. |

Nouveau aussi, sans rupture : `max_length` arrive jusqu'aux migrations (`VARCHAR(n)`,
`BINARY(n)`, `VARBINARY(n)`), suivi par les snapshots et le diff — agrandir une colonne est un
simple `ALTER`, la réduire demande `--force`. Un snapshot écrit avant la 3.0 reprend une fois
les longueurs du modèle, si bien que la mise à jour ne produit aucune migration à elle seule.
`blob` a son propre type de colonne, et `belongs_to` vise la vraie clé primaire de l'entité
liée, avec ses actions. Chaque colonne de `belongs_to` reçoit un index, les cycles de clés
étrangères entre nouvelles tables fonctionnent sur tous les moteurs, et les enums peuvent aussi
être `i8` / `i16`.

## Licence

MIT
