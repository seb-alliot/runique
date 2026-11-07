# Roles and permissions

## Overview

Admin access rests on the account's **rank**, three flags of `eihwaz_users` of which at most one is true (a `CHECK` constraint in the database):

| Rank | Flag | What it can do |
| --- | --- | --- |
| **Superuser** | `is_superuser` | Everything, the only one bypassing rights (never the invariants). The developer's account, to delete at delivery |
| **Dirigeant** | `is_dirigeant` | Every right on the models, with no bypass. Grants the admins' rights |
| **Admin** | `is_admin` | What the dirigeant ticks **on its account**, table by table and column by column |
| **Lambda** | none | A site account. Enters the admin only as a member of a staff group |

`can_access_admin()` is true for rank admin or above, or for a lambda member of an `is_staff` group. An account must also be active and activated.

---

## Where rights come from

`CurrentUser::rights_on("resource")` is the only way to ask what an account may do, in the admin as in your views. The source depends on the rank alone:

| Rank | Source of rights |
| --- | --- |
| Superuser | Everything, with bypass |
| Dirigeant | Everything, no bypass |
| Admin | Its own grid, set by the dirigeant (`eihwaz_groupes_droits`, rows tied to its account) |
| Lambda | The union (logical OR) of its groups' rights — **never** on the framework's tables (accounts, groups, rights, memberships) |

Rights are set per table and per column:

| Right | Level | Effect |
| --- | --- | --- |
| `can_read` | table + columns | See the resource and the ticked columns |
| `can_create` | table + columns | Create, filling the ticked columns |
| `can_update` | table + columns | Update the ticked columns |
| `can_delete` | table | Delete |
| `can_update_own` / `can_delete_own` | table | Own rows only, when the model declares an owner (`meta: { owner: … }`) |

A column added to the model later is readable or writable only once ticked. Secret columns (password) are never offered.

Rights are **not** declared in `ModelAdmin`: the whole grid lives in the database.

---

## Groups and staff

Groups are for **lambda accounts**: public site roles, and rights on the **developer's** tables in the admin. A lambda member of a group marked `is_staff` enters the admin and sees its allowed resources there, plus a read-only summary of its own groups and rights. No group ever has a right on the framework's tables, whoever configures it.

### Immediate revocation

The account (active, rank), its groups and rights are read from the database on every admin request: the session only keeps the user's identity. Removing a right or a group, deactivating an account or lowering its rank takes effect on its next request, whatever path made the change (admin, CLI, SQL, another instance). A deleted or deactivated account has its session closed. An account that only loses admin access (rank lowered, staff group removed) keeps its session on the site: the admin sends it to `/`.

---

## Framework tables: `users`, `groupes`, `groupes_droits`

Accounts, groups, rights and memberships are reserved to **ranks**: the superuser, the dirigeant, and an admin only for the tables and columns the dirigeant ticked for it. No group can get a right on those tables, and nobody can grant it one, superuser included: a lambda, staff or not, does no operation on them.

One exception: a lambda reading a row linked to an account (through a foreign key) sees that account's `username`, read-only.

---

## Access logic

```text
signed in?
  └─ no  → the admin's sign-in page
  └─ yes → account still there, active and activated?
             └─ no  → session closed, sign-in page
             └─ yes → can_access_admin()? (rank ≥ admin, or member of a staff group)
               └─ no  → back to the site (/) with a message; the session stays open
               └─ yes → rights_on(resource)
                          superuser → everything, bypass included
                          dirigeant → everything
                          admin     → its grid (table × column)
                          lambda    → union of its groups, never a framework table
                        └─ can_read? no  → resource missing from the nav
                                     yes → resource in the nav
                                           at least one readable column?
                                             no  → list refused (nothing to show)
                                             yes → list, ticked columns only
                                                   can_create / can_update / can_delete
                                                   (_own: own rows)
```

---

## Rank flags

`is_superuser`, `is_dirigeant` and `is_admin` are not admin form fields. Boot refuses them in `fields`. Rank is changed from the account page: a dirigeant may give admin or lambda, a superuser may give dirigeant, admin or lambda. The rank given stays strictly below the one who writes it. That action takes groups back when rank is lost, drops the rights grid of an admin who steps down, and keeps at least one active dirigeant while an admin remains. `is_superuser` is not given there: it is set with `runique create-superuser`, by a direct insert.

The rank filter limits which rows a proof may touch, from the rank already stored: an admin touches neither an admin, nor a dirigeant, nor a superuser; a dirigeant touches neither a dirigeant nor a superuser; a superuser does not touch another superuser.

`write::insert` refuses a `Set(true)` on one of these flags. `write::update` refuses any `Set`, `true` or `false`: a promotion or a demotion through that path would skip the rules above. A SeaORM `insert` or `update` on the connection never enters the gate.

## Notes

- `ModelAdmin` does not declare permissions: configuration is entirely in the database.
- Permissions are aggregated with logical OR across all of the user's groups.
- A user can have `can_read` without `can_create` — they see the list but cannot create.
- Ticking `can_read` on a table **without ticking any column** makes the list unreachable: there is nothing to show (a page of bare keys), so it is refused. The resource still appears in the menu. Tick at least one column for reading.
- Secret columns and ranks stay out of forms, including for a superuser (refused at boot). The rank-flag detail is above.

---

## See also

| Section | Description |
| --- | --- |
| [Setup](/docs/en/admin/setup) | Wire the admin, create a superuser |
| [Declaration](/docs/en/admin/declaration) | `AdminStaging`, `AdminSite`, registration |
| [Templates](/docs/en/admin/template) | Template hierarchy, blocks, visual override |
| [Roadmap](/docs/en/admin/evolution) | Planned features |

## Back to summary

- [Admin Summary](/docs/en/admin)
