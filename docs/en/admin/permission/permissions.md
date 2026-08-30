# Roles and permissions

## Overview

The admin permission system has three levels:

| Level | Control | Effect |
| --- | --- | --- |
| **`is_staff`** | User field | Grants access to the admin login page only |
| **Groups** | `eihwaz_groupes` + `eihwaz_groupes_droits` tables | Granular CRUD permissions per resource |
| **`is_superuser`** | User field | Bypasses all checks |

---

## Access control fields

| Field | Type | Role |
| --- | --- | --- |
| `is_staff` | `bool` | Allows login at `/admin/login` |
| `is_superuser` | `bool` | Full access, bypasses all permission checks |
| `is_active` | `bool` | Blocks inactive accounts |

---

## Granular permissions per group

Permissions are carried by **groups**, not directly by users. A user inherits permissions from all their groups (aggregated with logical OR).

Each group has one entry per resource in `eihwaz_groupes_droits`:

| Field | Effect |
| --- | --- |
| `can_read` | See the resource in the nav + access the list |
| `can_create` | Create a record |
| `can_update` | Edit any record |
| `can_delete` | Delete any record |
| `can_update_own` | Edit only their own records |
| `can_delete_own` | Delete only their own records |

---

## Configuration via the panel

1. Go to **Admin → Groups** → create a group
2. Configure that group's permissions per resource
3. Go to **Admin → Users** → assign the group to the user

A user with no group sees no resources in the nav (except superuser).

### Immediate revocation

Removing a group from a user takes effect on their next request. Deleting a group clears the permissions cache for all its members instantly.

---

## `groupes` and `groupes_droits` resources

These two resources get **no special treatment**: they follow the exact same generic logic as any other resource (`can_read`/`can_create`/`can_update`/`can_delete` via `eihwaz_groupes_droits`). A superuser can therefore deliberately grant `can_update` on `"groupes"` to an "Admin" group, which can then manage other groups' permissions (for example, to delegate to moderators) — this is a legitimate use, not a bypass.

**What this means in practice:** granting rights on `groupes`/`groupes_droits` to a non-superuser group gives it real control over permissions — potentially including its own, if its user also belongs to that group. There's no framework safeguard against self-elevation in that specific case: it's up to whoever configures groups not to grant these rights to a role that shouldn't have them. Only `is_superuser` remains a total, non-delegatable bypass (a bit on the user, never granted through a group).

---

## Access logic

```text
authenticated?
  └─ no  → redirect to /admin/login
  └─ yes → is_staff OR is_superuser?
               └─ neither → redirect to /admin/login
               └─ is_superuser → GRANTED (full access, all resources)
               └─ is_staff → can_read on the resource?
                                └─ no  → resource absent from the nav
                                └─ yes → list visible
                                         can_create / can_update / can_delete
                                         for the corresponding operations
                                         can_update_own / can_delete_own
                                         for their own records only
```

---

## Notes

- The `admin!` macro no longer declares `permissions:` — configuration is entirely in the database.
- Permissions are aggregated with logical OR across all of the user's groups.
- A user can have `can_read` without `can_create` — they see the list but cannot create.

---

## See also

| Section | Description |
| --- | --- |
| [Setup](/docs/en/admin/setup) | Wire the admin, create a superuser |
| [CLI](/docs/en/admin/declaration) | `runique start` command, general workflow |
| [Templates](/docs/en/admin/template) | Template hierarchy, blocks, visual override |
| [Roadmap](/docs/en/admin/evolution) | Planned features |

## Back to summary

- [Admin Summary](/docs/en/admin)
