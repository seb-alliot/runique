🌍 **Languages**: [English](KNOWN_ISSUES.md) | [Français](../fr/KNOWN_ISSUES.fr.md)

# Known issues

The known defects of each published version of Runique, their workaround and the release that fixes them. Security vulnerabilities are in [SECURITY.md](../../SECURITY.md), fixes in [CHANGELOG.md](CHANGELOG.md).

## 4.0.0

### The admin cannot create a row with a UUID key (`pk-uuid`)

- **What it does**: on the create page of a `model!{}`, saving is refused with "Action not allowed", whatever the account's rights. The admin inserts the row without its key, and no database generates a UUID key.
- **Who is affected**: projects built with the `pk-uuid` feature that use the admin. Integer keys (the default and `big-pk`) are not affected. Updating and deleting work.
- **Workaround**: create these rows from a view of the project, giving the key (`Uuid::now_v7()`).
- **Fix**: Runique 4.1.0. A model with a UUID key gets its key on insert when it is missing.

### The admin does not save list fields (`checkbox`, `multichoice`)

- **What it does**: the values ticked in a list field are not saved, with no error message, and the field does not show on the edit page.
- **Who is affected**: projects where a `model!{}` registered in the admin declares a `checkbox` or `multichoice` field.
- **Workaround**: write these values from a view of the project, with the model's `set_<field>` method.
- **Fix**: Runique 4.1.0. The admin writes, shows and traces list fields like columns, with the same rights.

### A form refused before the write keeps the path of the uploaded file

- **What it does**: when `save` refuses a form without attempting the write, the returned form keeps in its file field the path the upload would have gone to (`avatars/photo.png`), while the file never left the staging folder. Shown again, the form announces a file that doesn't exist there. Nothing is written to the database or moved on disk.
- **Who is affected**: update forms holding a file field, when the route parameter of the row (`{<form>_<key>}`) can't be read as the key type: `abc` for an integer key, a number for a UUID key. A missing row whose key reads fine is not affected.
- **Workaround**: have the parameter checked before the form, by asking for it typed in the handler (`Path(id): Path<Pk>`): an unreadable value is then refused before `save`.
- **Fix**: Runique 4.1.0. The upload is rolled back on every refusal of `save`, of `delete` and of the gate's write: the field takes its former value back.

### SQLite: a foreign key added to or removed from an existing table crashes `migration up`

- **What it does**: `runique migration up` stops on a panic of the SQL library ("Sqlite does not support modification of foreign key constraints to existing tables"). The statements placed before it in the same migration are already applied.
- **Who is affected**: SQLite projects with a migration that adds, removes or changes a `belongs_to` on a table already created. PostgreSQL and MySQL are not affected. Neither are migrations that only create tables.
- **Workaround**: on a database whose data needn't be kept, delete the database and the migration files, then run `runique makemigrations` again: migrations written again from the models only hold creations, which SQLite accepts. On a database to keep, the table has to be rebuilt by hand.
- **Fix**: Runique 4.1.0 for the crash: the migration is refused at its first statement, with a message naming the table, and nothing is applied. Migrations written by 4.0.0 keep their former text and must be generated again to benefit from it. The table rebuild, which will make this change possible under SQLite, is not written yet.

### PostgreSQL: renaming a value of an enum several columns use makes `migration up` fail

- **What it does**: `runique migration up` stops on a PostgreSQL error saying the former value doesn't exist in the type. `makemigrations` writes the rename once per column using the enum, while under PostgreSQL these columns share a single type: the first rename passes, the next one no longer finds the former value.
- **Who is affected**: PostgreSQL projects with a model that has several fields on the same enum (two `checkbox` or `multichoice` fields, for instance, or a `choice` field and a list field), when a value of that enum is renamed. MySQL and SQLite are not affected: each table updates its own rows there. Adding a value is not affected either.
- **Workaround**: after `runique makemigrations` and before `runique migration up`, keep the rename in the migration of one of the tables concerned and remove, from the migrations of the others, the block that renames the value (in `up` and in `down`). Don't run `makemigrations` again in between: it would rewrite these migrations. This workaround holds for a PostgreSQL database only.
- **Fix**: not fixed yet. Planned for Runique 4.1.0: the rename will be written once per enum.
