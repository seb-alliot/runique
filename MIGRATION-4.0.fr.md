🌍 **Langues** : [English](https://github.com/seb-alliot/runique/blob/main/MIGRATION-4.0.md) | [Français](https://github.com/seb-alliot/runique/blob/main/MIGRATION-4.0.fr.md)

# Passer de Runique 3.0 à 4.0

Ce guide liste les ruptures de la 4.0 et ce qu'il faut faire. Le [journal des modifications](CHANGELOG.fr.md) donne la liste complète, y compris les correctifs et les ajouts qui ne demandent rien.

---

## Étape par étape

1. **Base** : mettre à jour les tables du framework (voir [Tables du framework](#tables-du-framework)).
2. **Admin** : réécrire avec `ModelAdmin` dans `src/admin.rs` et supprimer `src/admins/` (voir [Admin](#admin)).
3. **Formulaires** : nommer chaque formulaire et passer à `try_form` (voir [Formulaires](#formulaires)).
4. **Modèles** : préciser les `decimal`, retirer `required` des `bool` qui ne voulaient dire que NOT NULL (voir [DSL des modèles](#dsl-des-modèles)).
5. **Code** : corriger ce que le compilateur signale, à l'aide des sections ci-dessous.
6. **Comportement** : relire [CSRF](#csrf) et [Fichiers](#fichiers) : ces deux changements ne se voient pas à la compilation.
7. **Redémarrer** l'application : les fichiers statiques changent.

---

## Tables du framework

Les migrations du framework créent leurs tables avec `IF NOT EXISTS` : **une table existante n'est pas modifiée**. Runique ne migre pas ces tables à votre place : appliquez le SQL ci-dessous sur votre base, **après une sauvegarde**, puis lancez `runique makemigrations` et `runique migration up`, qui créent les index de `eihwaz_history`.

À adapter avant de lancer :

- `<ID_COMPTE>` : l'`id` du compte responsable des groupes existants.
- `INTEGER` / `INT` de `responsable_id`, `repris_par` et `user_id` : `BIGINT` avec `big-pk`, `UUID` avec `pk-uuid` (Postgres), comme `eihwaz_users.id`.

Ce que ce SQL retire :

- `is_staff` des comptes, sans équivalent : nommez ensuite chaque compte à son rang (`UPDATE eihwaz_users SET is_dirigeant = TRUE WHERE id = …`). Un `is_admin` n'a aucun droit tant que le dirigeant ne lui en a pas donné.
- les droits de groupe sur les tables du framework, désormais interdits.
- les jetons de réinitialisation en cours : un lien déjà envoyé ne marche plus.
- les sessions sauvegardées : leur contenu est désormais chiffré, une ligne en clair est refusée. Les utilisateurs se reconnectent une fois.

<details>
<summary>PostgreSQL</summary>

```sql
BEGIN;

ALTER TABLE eihwaz_users
    DROP COLUMN is_staff,
    ADD COLUMN is_dirigeant BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN session_version INTEGER NOT NULL DEFAULT 0,
    ADD CONSTRAINT eihwaz_users_one_rank_at_most CHECK (
        (CASE WHEN is_superuser THEN 1 ELSE 0 END)
      + (CASE WHEN is_dirigeant THEN 1 ELSE 0 END)
      + (CASE WHEN is_admin THEN 1 ELSE 0 END) <= 1);

ALTER TABLE eihwaz_groupes
    ADD COLUMN is_staff BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN responsable_id INTEGER,
    ADD COLUMN repris_par INTEGER;
UPDATE eihwaz_groupes SET responsable_id = <ID_COMPTE>;
ALTER TABLE eihwaz_groupes
    ALTER COLUMN responsable_id SET NOT NULL,
    ADD CONSTRAINT fk_eihwaz_groupes_responsable_id FOREIGN KEY (responsable_id)
        REFERENCES eihwaz_users (id) ON DELETE RESTRICT,
    ADD CONSTRAINT fk_eihwaz_groupes_repris_par FOREIGN KEY (repris_par)
        REFERENCES eihwaz_users (id) ON DELETE SET NULL;

DELETE FROM eihwaz_groupes_droits
    WHERE resource_key IN ('users', 'groupes', 'droits', 'users_groupes');
ALTER TABLE eihwaz_groupes_droits
    DROP CONSTRAINT pk_eihwaz_groupes_droits,
    ADD COLUMN id SERIAL PRIMARY KEY,
    ALTER COLUMN groupe_id DROP NOT NULL,
    ADD COLUMN user_id INTEGER,
    ADD COLUMN colonne VARCHAR NOT NULL DEFAULT '',
    ADD COLUMN can_read_own BOOLEAN NOT NULL DEFAULT FALSE,
    ADD CONSTRAINT fk_eihwaz_groupes_droits_user_id FOREIGN KEY (user_id)
        REFERENCES eihwaz_users (id) ON DELETE CASCADE,
    ADD CONSTRAINT eihwaz_groupes_droits_one_holder CHECK (
        (groupe_id IS NULL AND user_id IS NOT NULL)
        OR (groupe_id IS NOT NULL AND user_id IS NULL)),
    ADD CONSTRAINT eihwaz_groupes_droits_group_never_framework CHECK (
        groupe_id IS NULL
        OR resource_key NOT IN ('users', 'groupes', 'droits', 'users_groupes')),
    ADD CONSTRAINT eihwaz_groupes_droits_column_row_flags CHECK (
        colonne = ''
        OR (NOT can_read_own AND NOT can_update_own AND NOT can_delete AND NOT can_delete_own));
CREATE UNIQUE INDEX uq_eihwaz_groupes_droits_groupe
    ON eihwaz_groupes_droits (groupe_id, resource_key, colonne);
CREATE UNIQUE INDEX uq_eihwaz_groupes_droits_user
    ON eihwaz_groupes_droits (user_id, resource_key, colonne);

ALTER TABLE eihwaz_history ADD COLUMN object_repr VARCHAR NOT NULL DEFAULT '';
UPDATE eihwaz_history SET object_repr = object_pk;
ALTER TABLE eihwaz_history ALTER COLUMN object_repr DROP DEFAULT;

DELETE FROM eihwaz_sessions;

DELETE FROM eihwaz_reset_tokens;
ALTER TABLE eihwaz_reset_tokens ADD CONSTRAINT eihwaz_reset_tokens_user_id_key UNIQUE (user_id);

COMMIT;
```

</details>

<details>
<summary>MySQL / MariaDB</summary>

Ces moteurs n'annulent pas un `ALTER TABLE` : en cas d'erreur à mi-parcours, repartez de la sauvegarde.

```sql
ALTER TABLE eihwaz_users
    DROP COLUMN is_staff,
    ADD COLUMN is_dirigeant BOOL NOT NULL DEFAULT FALSE,
    ADD COLUMN is_admin BOOL NOT NULL DEFAULT FALSE,
    ADD COLUMN session_version INT NOT NULL DEFAULT 0,
    ADD CONSTRAINT eihwaz_users_one_rank_at_most CHECK (
        (CASE WHEN is_superuser THEN 1 ELSE 0 END)
      + (CASE WHEN is_dirigeant THEN 1 ELSE 0 END)
      + (CASE WHEN is_admin THEN 1 ELSE 0 END) <= 1);

ALTER TABLE eihwaz_groupes
    ADD COLUMN is_staff BOOL NOT NULL DEFAULT FALSE,
    ADD COLUMN is_active BOOL NOT NULL DEFAULT TRUE,
    ADD COLUMN responsable_id INT NULL,
    ADD COLUMN repris_par INT NULL;
UPDATE eihwaz_groupes SET responsable_id = <ID_COMPTE>;
ALTER TABLE eihwaz_groupes
    MODIFY responsable_id INT NOT NULL,
    ADD CONSTRAINT fk_eihwaz_groupes_responsable_id FOREIGN KEY (responsable_id)
        REFERENCES eihwaz_users (id) ON DELETE RESTRICT,
    ADD CONSTRAINT fk_eihwaz_groupes_repris_par FOREIGN KEY (repris_par)
        REFERENCES eihwaz_users (id) ON DELETE SET NULL;

DELETE FROM eihwaz_groupes_droits
    WHERE resource_key IN ('users', 'groupes', 'droits', 'users_groupes');
ALTER TABLE eihwaz_groupes_droits
    ADD COLUMN user_id INT NULL,
    ADD COLUMN colonne VARCHAR(255) NOT NULL DEFAULT '',
    ADD COLUMN can_read_own BOOL NOT NULL DEFAULT FALSE;
-- Avant de retirer la clé primaire : la clé étrangère de groupe_id a besoin d'un index.
CREATE UNIQUE INDEX uq_eihwaz_groupes_droits_groupe
    ON eihwaz_groupes_droits (groupe_id, resource_key, colonne);
CREATE UNIQUE INDEX uq_eihwaz_groupes_droits_user
    ON eihwaz_groupes_droits (user_id, resource_key, colonne);
ALTER TABLE eihwaz_groupes_droits
    DROP PRIMARY KEY,
    ADD COLUMN id INT NOT NULL AUTO_INCREMENT PRIMARY KEY FIRST,
    MODIFY groupe_id INT NULL,
    ADD CONSTRAINT fk_eihwaz_groupes_droits_user_id FOREIGN KEY (user_id)
        REFERENCES eihwaz_users (id) ON DELETE CASCADE,
    ADD CONSTRAINT eihwaz_groupes_droits_one_holder CHECK (
        (groupe_id IS NULL AND user_id IS NOT NULL)
        OR (groupe_id IS NOT NULL AND user_id IS NULL)),
    ADD CONSTRAINT eihwaz_groupes_droits_group_never_framework CHECK (
        groupe_id IS NULL
        OR resource_key NOT IN ('users', 'groupes', 'droits', 'users_groupes')),
    ADD CONSTRAINT eihwaz_groupes_droits_column_row_flags CHECK (
        colonne = ''
        OR (NOT can_read_own AND NOT can_update_own AND NOT can_delete AND NOT can_delete_own));

ALTER TABLE eihwaz_history ADD COLUMN object_repr VARCHAR(255) NOT NULL DEFAULT '' AFTER object_pk;
UPDATE eihwaz_history SET object_repr = object_pk;
ALTER TABLE eihwaz_history ALTER COLUMN object_repr DROP DEFAULT;

DELETE FROM eihwaz_sessions;

DELETE FROM eihwaz_reset_tokens;
ALTER TABLE eihwaz_reset_tokens ADD UNIQUE KEY uq_eihwaz_reset_tokens_user_id (user_id);
```

</details>

<details>
<summary>SQLite</summary>

SQLite ne sait pas ajouter une contrainte à une table existante : `eihwaz_users`, `eihwaz_groupes` et `eihwaz_groupes_droits` sont reconstruites. `PRAGMA foreign_keys` doit rester hors de la transaction.

```sql
PRAGMA foreign_keys = OFF;
BEGIN;

CREATE TABLE eihwaz_users_new (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL UNIQUE,
    password VARCHAR NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    is_dirigeant BOOLEAN NOT NULL DEFAULT FALSE,
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    is_superuser BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NULL,
    updated_at DATETIME NULL,
    activated_at DATETIME NULL,
    session_version INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT eihwaz_users_active_needs_activation CHECK (NOT is_active OR activated_at IS NOT NULL),
    CONSTRAINT eihwaz_users_one_rank_at_most CHECK (
        (CASE WHEN is_superuser THEN 1 ELSE 0 END)
      + (CASE WHEN is_dirigeant THEN 1 ELSE 0 END)
      + (CASE WHEN is_admin THEN 1 ELSE 0 END) <= 1)
);
INSERT INTO eihwaz_users_new (id, username, email, password, is_active, is_superuser, created_at, updated_at, activated_at)
    SELECT id, username, email, password, is_active, is_superuser, created_at, updated_at, activated_at FROM eihwaz_users;
DROP TABLE eihwaz_users;
ALTER TABLE eihwaz_users_new RENAME TO eihwaz_users;

CREATE TABLE eihwaz_groupes_new (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    nom VARCHAR NOT NULL UNIQUE,
    is_staff BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    responsable_id INTEGER NOT NULL,
    repris_par INTEGER NULL,
    CONSTRAINT fk_eihwaz_groupes_responsable_id FOREIGN KEY (responsable_id)
        REFERENCES eihwaz_users (id) ON DELETE RESTRICT,
    CONSTRAINT fk_eihwaz_groupes_repris_par FOREIGN KEY (repris_par)
        REFERENCES eihwaz_users (id) ON DELETE SET NULL
);
INSERT INTO eihwaz_groupes_new (id, nom, responsable_id)
    SELECT id, nom, <ID_COMPTE> FROM eihwaz_groupes;
DROP TABLE eihwaz_groupes;
ALTER TABLE eihwaz_groupes_new RENAME TO eihwaz_groupes;

CREATE TABLE eihwaz_groupes_droits_new (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    groupe_id INTEGER NULL,
    user_id INTEGER NULL,
    resource_key VARCHAR NOT NULL,
    colonne VARCHAR NOT NULL DEFAULT '',
    can_create BOOLEAN NOT NULL DEFAULT FALSE,
    can_read BOOLEAN NOT NULL DEFAULT FALSE,
    can_read_own BOOLEAN NOT NULL DEFAULT FALSE,
    can_update BOOLEAN NOT NULL DEFAULT FALSE,
    can_update_own BOOLEAN NOT NULL DEFAULT FALSE,
    can_delete BOOLEAN NOT NULL DEFAULT FALSE,
    can_delete_own BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT eihwaz_groupes_droits_one_holder CHECK (
        (groupe_id IS NULL AND user_id IS NOT NULL)
        OR (groupe_id IS NOT NULL AND user_id IS NULL)),
    CONSTRAINT eihwaz_groupes_droits_group_never_framework CHECK (
        groupe_id IS NULL
        OR resource_key NOT IN ('users', 'groupes', 'droits', 'users_groupes')),
    CONSTRAINT eihwaz_groupes_droits_column_row_flags CHECK (
        colonne = ''
        OR (NOT can_read_own AND NOT can_update_own AND NOT can_delete AND NOT can_delete_own)),
    CONSTRAINT fk_eihwaz_groupes_droits_groupe_id FOREIGN KEY (groupe_id)
        REFERENCES eihwaz_groupes (id) ON DELETE CASCADE,
    CONSTRAINT fk_eihwaz_groupes_droits_user_id FOREIGN KEY (user_id)
        REFERENCES eihwaz_users (id) ON DELETE CASCADE
);
INSERT INTO eihwaz_groupes_droits_new (groupe_id, resource_key, can_create, can_read, can_update, can_delete, can_update_own, can_delete_own)
    SELECT groupe_id, resource_key, can_create, can_read, can_update, can_delete, can_update_own, can_delete_own
    FROM eihwaz_groupes_droits
    WHERE resource_key NOT IN ('users', 'groupes', 'droits', 'users_groupes');
DROP TABLE eihwaz_groupes_droits;
ALTER TABLE eihwaz_groupes_droits_new RENAME TO eihwaz_groupes_droits;
CREATE UNIQUE INDEX uq_eihwaz_groupes_droits_groupe
    ON eihwaz_groupes_droits (groupe_id, resource_key, colonne);
CREATE UNIQUE INDEX uq_eihwaz_groupes_droits_user
    ON eihwaz_groupes_droits (user_id, resource_key, colonne);

ALTER TABLE eihwaz_history ADD COLUMN object_repr VARCHAR NOT NULL DEFAULT '';
UPDATE eihwaz_history SET object_repr = object_pk;

DELETE FROM eihwaz_sessions;

DELETE FROM eihwaz_reset_tokens;
CREATE UNIQUE INDEX uq_eihwaz_reset_tokens_user_id ON eihwaz_reset_tokens (user_id);

COMMIT;
PRAGMA foreign_key_check;
PRAGMA foreign_keys = ON;
```

</details>

---

## Admin

`admin!{}`, `src/admins/` et `runique start` n'existent plus. L'admin se déclare avec le builder typé `ModelAdmin` :

```rust
// 3.0
.with_admin(|a| a.routes(admins::routes("/admin")))

// 4.0 : ModelAdmin déclarés dans src/admin.rs
.with_admin(admin::config)
```

- Une route personnalisée ne passe plus par `.extra_routes()` : elle se déclare dans `urlpatterns!` avec le mot-clé `admin`.
- `urlpatterns!` renvoie un `Routes` : une fonction `routes() -> Router` devient `routes() -> Routes`.
- `AdminTemplate::with_base` et `with_bulk_edit` sont retirés.
- Sentinel et `GuardRules` sont retirés : le contrôle d'accès par rang ou groupe passe par les routes `admin`.

Voir [la mise en place de l'admin](https://runique.io/docs/fr/admin/setup) et [les rôles et permissions](https://runique.io/docs/fr/admin/permission).

---

## Formulaires

**Chaque formulaire porte un nom**, et `try_new` devient `try_form` :

```rust
// 3.0
let form: MonForm = request.form();
match ValidationForm::try_new(form, &request).await { /* ... */ }

// 4.0
let form: MonForm = request.form("article");
match ValidationForm::try_form(form, &request).await { /* ... */ }
```

Le nom part dans un champ caché `runique_form`. Un formulaire écrit à la main doit le porter lui-même.

**`save` et `delete` sont sur le formulaire validé**. `on_save`, `before_save`, `after_save`, `save_with` et `SaveContext` sont retirés :

```rust
// 4.0 : rend le formulaire, Ok une fois écrit, Err avec l'erreur posée dessus
match validated.save(&request.engine.db).await {
    Ok(_) => Ok(Redirect::to("/articles").into_response()),
    Err(form) => { /* réafficher le formulaire */ }
}
```

La logique métier passe dans `clean_field`, `clean` ou `register_dynamic_fields` ; les emails et les écritures dans d'autres tables, dans le handler après un `save` réussi.

Autres changements :

| Avant | Après |
| --- | --- |
| `cleaned_string("x")`, `cleaned_i32("x")`… | `cleaned::<String>("x")`, `cleaned::<i32>("x")`… |
| `FileField::image("x")` | `MediaField::image("x")` |
| `RuniqueForm::build`, `build_with_data` | `request.form::<T>("nom")` ; en test, `build_form_for_test::<T>(tera, csrf_token, "nom")` |
| `impl_form_access!(model)` | à retirer : `#[form(schema = …)]` génère `FormAccess` |
| `#[form(schema = …, model = …)]` | retirer `model = …` |
| `#id_{champ}` en CSS ou JS | `#id_{formulaire}-{champ}` |

---

## DSL des modèles

- **`decimal` déclare sa précision**, sinon erreur de compilation : `prix: decimal [max_digits: 10, decimal_places: 2]`.
- **`belongs_to` déclare son action `ON DELETE`**, sinon erreur de compilation : `belongs_to: Cour via cour_id [cascade]`. Pour garder le comportement d'avant, écrivez `[no_action]`.
- **`required` sur un `bool` signifie « doit être cochée »** : retirez-le de chaque `bool` qui voulait seulement dire NOT NULL, sinon la case devient impossible à décocher.

Lancez ensuite `runique makemigrations` : un changement de précision est signalé comme destructif (`--force`).

---

## CSRF

**Un formulaire envoyé depuis un autre site ou un sous-domaine reçoit un 403**, sans rien à activer. Si vous en recevez légitimement :

- un service tiers (retour de paiement) : ajoutez son chemin à `csrf_exempt()`, avec sa propre vérification ;
- une page d'un de vos sous-domaines : envoyez le jeton dans `X-CSRF-Token` avec `fetch`, ou ajoutez le chemin à `csrf_exempt()`.

Voir [la doc CSRF](https://runique.io/docs/fr/middleware/csrf).

---

## Sessions

**`RuniqueSessionStore::new` prend `SECRET_KEY`** : `RuniqueSessionStore::new(db, &secret_key)`, à adapter si vous l'appelez directement. `session_data` est chiffré : lisez-le avec `open_session_data(&model)`.

**`eihwaz_users.session_version`** ferme les sessions d'un compte, dans tous les stores : `close_user_sessions` et un reset réussi l'augmentent. `update_password_by_id` n'y touche pas : appelez `close_user_sessions` si un changement de mot de passe doit déconnecter les autres appareils. Un modèle `RuniqueUser` écrit à la main peut surcharger `session_version()` ; par défaut elle vaut `0` et ne ferme rien.

---

## Fichiers

**Runique ne supprime plus le fichier remplacé par un upload.** Si votre projet comptait sur cette suppression, le nettoyage de l'ancien fichier est maintenant à sa charge.

---

## Réinitialisation du mot de passe

- En production, `with_password_reset()` sans mailer arrête le démarrage.
- `AdminConfig.reset_route` devient `reset_link`, posé par le builder.
