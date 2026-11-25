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
