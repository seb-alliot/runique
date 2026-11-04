🌍 **Languages**: [English](https://github.com/seb-alliot/runique/blob/main/MIGRATION-4.0.md) | [Français](https://github.com/seb-alliot/runique/blob/main/MIGRATION-4.0.fr.md)

# Upgrading from Runique 3.0 to 4.0

This guide lists the 4.0 breaking changes and what to do about them. The [changelog](CHANGELOG.md) has the full list, including fixes and additions that ask nothing of you.

---

## Step by step

1. **Database**: update the framework's tables (see [Framework tables](#framework-tables)).
2. **Admin**: rewrite with `ModelAdmin` in `src/admin.rs` and delete `src/admins/` (see [Admin](#admin)).
3. **Forms**: name every form and move to `try_form` (see [Forms](#forms)).
4. **Models**: give each `decimal` its precision, remove `required` from every `bool` that only meant NOT NULL (see [Model DSL](#model-dsl)).
5. **Code**: fix what the compiler reports, using the sections below.
6. **Behaviour**: read [CSRF](#csrf) and [Files](#files): neither change shows at compile time.
7. **Restart** the application: static files change.

---

## Framework tables

The framework's migrations create their tables with `IF NOT EXISTS`: **an existing table is not modified**. Runique doesn't migrate these tables for you: apply the SQL below to your database, **after a backup**, then run `runique makemigrations` and `runique migration up`, which create the `eihwaz_history` indexes.

To adapt before running it:

- `<ID_COMPTE>`: the `id` of the account responsible for the existing groups.
- `INTEGER` / `INT` of `responsable_id`, `repris_par` and `user_id`: `BIGINT` with `big-pk`, `UUID` with `pk-uuid` (Postgres), like `eihwaz_users.id`.

What this SQL removes:

- `is_staff` on accounts, with no equivalent: then give each account its rank (`UPDATE eihwaz_users SET is_dirigeant = TRUE WHERE id = …`). An `is_admin` has no rights until the dirigeant gives some.
- group rights on the framework's tables, now forbidden.
- pending reset tokens: a link already sent stops working.
- saved sessions: their content is now encrypted, a row in clear is refused. Users sign in again once.

<details>
<summary>PostgreSQL</summary>

```sql
BEGIN;

ALTER TABLE eihwaz_users
    DROP COLUMN is_staff,
    ADD COLUMN is_dirigeant BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT FALSE,
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

These engines don't roll back an `ALTER TABLE`: on an error halfway through, start again from the backup.

```sql
ALTER TABLE eihwaz_users
    DROP COLUMN is_staff,
    ADD COLUMN is_dirigeant BOOL NOT NULL DEFAULT FALSE,
    ADD COLUMN is_admin BOOL NOT NULL DEFAULT FALSE,
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
-- Before dropping the primary key: the groupe_id foreign key needs an index.
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

SQLite can't add a constraint to an existing table: `eihwaz_users`, `eihwaz_groupes` and `eihwaz_groupes_droits` are rebuilt. `PRAGMA foreign_keys` must stay outside the transaction.

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

`admin!{}`, `src/admins/` and `runique start` no longer exist. The admin is declared with the typed `ModelAdmin` builder:

```rust
// 3.0
.with_admin(|a| a.routes(admins::routes("/admin")))

// 4.0: ModelAdmin declared in src/admin.rs
.with_admin(admin::config)
```

- A custom route no longer goes through `.extra_routes()`: it is declared in `urlpatterns!` with the `admin` keyword.
- `urlpatterns!` returns a `Routes`: a `routes() -> Router` function becomes `routes() -> Routes`.
- `AdminTemplate::with_base` and `with_bulk_edit` are removed.
- Sentinel and `GuardRules` are removed: access control by rank or group goes through `admin` routes.

See [admin setup](https://runique.io/docs/en/admin/setup) and [roles and permissions](https://runique.io/docs/en/admin/permission).

---

## Forms

**Every form has a name**, and `try_new` becomes `try_form`:

```rust
// 3.0
let form: MyForm = request.form();
match ValidationForm::try_new(form, &request).await { /* ... */ }

// 4.0
let form: MyForm = request.form("article");
match ValidationForm::try_form(form, &request).await { /* ... */ }
```

The name travels in a hidden `runique_form` field. A hand-written form must carry it itself.

**`save` and `delete` are on the validated form**. `on_save`, `before_save`, `after_save`, `save_with` and `SaveContext` are removed:

```rust
// 4.0: gives the form back, Ok once written, Err with the error set on it
match validated.save(&request.engine.db).await {
    Ok(_) => Ok(Redirect::to("/articles").into_response()),
    Err(form) => { /* render the form again */ }
}
```

Business logic goes into `clean_field`, `clean` or `register_dynamic_fields`; emails and writes to other tables go in the handler after a successful `save`.

Other changes:

| Before | After |
| --- | --- |
| `cleaned_string("x")`, `cleaned_i32("x")`… | `cleaned::<String>("x")`, `cleaned::<i32>("x")`… |
| `FileField::image("x")` | `MediaField::image("x")` |
| `RuniqueForm::build`, `build_with_data` | `request.form::<T>("name")`; in tests, `build_form_for_test::<T>(tera, csrf_token, "name")` |
| `impl_form_access!(model)` | remove it: `#[form(schema = …)]` generates `FormAccess` |
| `#[form(schema = …, model = …)]` | remove `model = …` |
| `#id_{field}` in CSS or JS | `#id_{form}-{field}` |

---

## Model DSL

- **`decimal` declares its precision**, otherwise a compile error: `price: decimal [max_digits: 10, decimal_places: 2]`.
- **`belongs_to` declares its `ON DELETE` action**, otherwise a compile error: `belongs_to: Course via course_id [cascade]`. To keep the previous behaviour, write `[no_action]`.
- **`required` on a `bool` means "must be checked"**: remove it from every `bool` that only meant NOT NULL, or the box can no longer be left unchecked.

Then run `runique makemigrations`: a precision change is flagged as destructive (`--force`).

---

## CSRF

**A form sent from another site or a subdomain gets a 403**, with nothing to turn on. If you legitimately receive some:

- a third party (a payment return): add its path to `csrf_exempt()`, with its own verification;
- a page of one of your subdomains: send the token in `X-CSRF-Token` with `fetch`, or add the path to `csrf_exempt()`.

See [the CSRF docs](https://runique.io/docs/en/middleware/csrf).

---

## Sessions

**`RuniqueSessionStore::new` takes `SECRET_KEY`**: `RuniqueSessionStore::new(db, &secret_key)`, to adapt if you call it directly. `session_data` is encrypted: read it with `open_session_data(&model)`.

---

## Files

**Runique no longer deletes the file an upload replaces.** If your project relied on that, cleaning up the old file is now up to it.

---

## Password reset

- In production, `with_password_reset()` without a mailer stops the boot.
- `AdminConfig.reset_route` becomes `reset_link`, set by the builder.
