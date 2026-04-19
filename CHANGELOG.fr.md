🌍 **Languages**: [English](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md) | [Français](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md)

# Journal des modifications

Toutes les modifications notables de ce projet sont documentées dans ce fichier.

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

