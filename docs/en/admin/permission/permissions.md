# Roles and permissions

## Overview

The admin permission system has two distinct levels:

| Level | Control | Effect |
| --- | --- | --- |
| **Permission (view)** | Scoped droit `access_type = "view"` | Resource visible in the nav |
| **Droit (write)** | Scoped droit `access_type = "write"` | Access to create / edit / delete |
| **Superuser** | `is_superuser = true` | Bypasses both levels |

---

## Access control fields

| Field | Type | Role |
| --- | --- | --- |
| `is_staff` | `bool` | Grants access to the admin interface |
| `is_superuser` | `bool` | Full access, bypasses all checks |
| `is_active` | `bool` | Blocks inactive accounts |

---

## Scoped droits

Each registered resource automatically gets two droits in `eihwaz_droits`, created at startup by the framework if absent:

| Name | `resource_key` | `access_type` | Effect |
| --- | --- | --- | --- |
| `blog.view` | `"blog"` | `"view"` | See the blog resource in the nav |
| `blog.write` | `"blog"` | `"write"` | Create / edit / delete in blog |

These droits are ordinary entries in `eihwaz_droits` — the admin assigns them to users or groups from the panel, exactly like any other droit.

---

## Configuration via the panel

### Granting visibility of a resource

1. Go to **Admin → Droits**
2. Find `blog.view` (created automatically at startup)
3. Go to **Admin → Users**, open the staff profile
4. Assign the `blog.view` droit

The user will now see the `blog` resource in the admin navigation.

### Granting write access

Same procedure with `blog.write`. A user can have `blog.view` without `blog.write` — they see the list but cannot create/edit/delete.

### Immediate revocation

Removing a droit from a user takes effect on their next request — no logout required. Deleting a droit from `eihwaz_droits` clears the permissions cache for **all** users instantly.

---

## Superuser-only resources

The `droits` and `groupes` resources can only be accessed by an `is_superuser`. No scoped droit can unlock their access for a staff user — this is a fixed framework rule.

This prevents privilege escalation: a staff user can never modify their own droits or those of other users.

---

## Access logic (current state)

```text
authenticated?
  └─ no  → redirect to /admin/login
  └─ yes → is_staff OR is_superuser?
               └─ neither → redirect to /admin/login
               └─ is_superuser → GRANTED (full access, all resources)
               └─ is_staff → resource visible if .view droit assigned
                              write operation if .write droit assigned
                              droits/groupes → denied (superuser only)
```

---

## Notes

- The `admin!` macro no longer declares `permissions:` — configuration is entirely in the database.
- Scoped droits are created automatically: the developer has nothing to do on the code side.
- A user with no scoped droits sees no resources in the nav (except superuser).

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
