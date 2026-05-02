🌍 **Languages**: [English](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md) | [Français](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md)

# Journal des modifications

Toutes les modifications notables de ce projet sont documentées dans ce fichier.

---

## [2.1.1] - 2026-05-02

### Corrigé — `derive_form` 2.0.2

* **`fk()` dans les blocs v2 ignoré silencieusement :** `FormFieldAttr::Fk(FkDef)` ajouté dans l'AST, le parser et la propagation vers `FieldOption::Fk`.
* **Attribut `skip` inconnu :** `FormFieldAttr::Skip` ajouté dans l'AST, le parser et le générateur (champ exclu du rendu formulaire).
* **Syntaxe `many_to_many(target).through(via)` cassée :** corrigée en `many_to_many(target, via)` dans `foreignkey.rs`.
* **`sea_query::ForeignKeyAction` introuvable :** re-exporté sous `runique::migration::ForeignKeyAction` ; chemins du générateur mis à jour.
* **Méthode `.references_column()` inexistante :** remplacée par `.to_column()` dans le builder FK.
* **Noms de modèles PascalCase dans les chemins de relations :** `to_snake_case()` utilisé partout à la place de `.to_lowercase()` dans `relation_enum.rs` et `foreignkey.rs` (ex. `super::menuimage` → `super::menu_image`).
* **`rust_decimal::Decimal` introuvable :** type mappé vers `::runique::sea_orm::prelude::Decimal` dans `sea_model.rs`.
* **`via_self` FK → mauvais variant de relation :** suffixe `_id` supprimé et PascalCase appliqué pour dériver le bon nom de variant dans l'impl `Related` de `ManyToMany`.
* **`Decimal` absent de `generate_partial_update` :** `FieldType::Decimal(_)` ajouté au bras numérique.
* **`Decimal` absent de `generate_from_str_map` :** `FieldType::Decimal(_)` ajouté au bras float/decimal.
* **`unique_together` / `indexes` jamais générés en SQL :** `parser_builder.rs` ignorait silencieusement le bloc `meta`. Désormais parsé et converti en entrées `ParsedIndex` (`{table}_{cols}_uniq` pour les contraintes uniques, `idx_{table}_{cols}` pour les index simples).

### Ajouté — `runique` 2.1.1-alpha.3

* **Enum `OrderDir`** ajoutée dans `migration::schema` (`Asc` / `Desc`).
* **Méthodes builder sur `ModelSchema` :** `order_by()`, `unique_together()`, `verbose_name()`, `verbose_name_plural()`.
* **`ForeignKeyAction` re-exporté** depuis `runique::migration`.
* **`RelationDef::as_name()`** méthode no-op ajoutée pour la compatibilité DSL.

---

## [2.1.0] - 2026-04-20

### Rupture

* **`Prisme<T>` supprimé — extraction via `req.form::<T>()` :**
  Les handlers n'acceptent plus `Prisme<MyForm>` comme paramètre. Utiliser `let form = req.form::<MyForm>()`.
  `Request` doit être le **dernier paramètre** de chaque handler (extracteur body-consuming).
  L'extracteur `AdminBody` est supprimé — les handlers admin POST lisent les données via `req.prisme.data`.

### Ajouté

* **`EihwazSessionsMigration` — table de sessions persistantes :**
  `migrations_table::EihwazSessionsMigration` crée la table `eihwaz_sessions`.
  À ajouter dans le vec du `Migrator` après `EihwazUsersMigration`.
  `eihwaz_sessions` est désormais dans `FRAMEWORK_TABLES` et exclue du scan `makemigrations`.

### Corrigé

* **`auth_login` — sessions persistées en base :**
  `auth_login()` passe maintenant un `RuniqueSessionStore` à `login()`, ce qui crée une ligne
  dans `eihwaz_sessions` à la connexion. Les sessions survivent aux redémarrages serveur via le fallback DB.

---
