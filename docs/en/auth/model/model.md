# User Model

## Built-in Model

Runique includes a ready-to-use user model that requires no configuration.

**Generated table:** `eihwaz_users`

| Field | Type | Description |
|---------------|----------|----------------------------------------|
| `id` | `Pk` | Primary key (`i32` by default, `i64` with `big-pk`, `Uuid` with `pk-uuid`) |
| `username` | `String` | Unique username |
| `email` | `String` | Email address |
| `password` | `String` | Hash (the `PasswordConfig`'s algorithm) — never stored in plain text |
| `is_active` | `bool` | Account active |
| `is_admin` | `bool` | Admin rank: panel access, with its own rights (table × column) |
| `is_dirigeant` | `bool` | Dirigeant rank: every right on the project's models |
| `is_superuser` | `bool` | Superuser rank: full access |
| `created_at` | datetime | Creation timestamp |
| `updated_at` | datetime | Last update timestamp |
| `activated_at` | datetime | Activation by the owner (empty while the account is pending) |
| `session_version` | `i32` | Raised to close every session of the account (`close_user_sessions`, a successful reset): a session opened with an older version ends |

To create the first superuser:

```bash
runique create-superuser
```

### i64 or UUID primary key

By default, the primary key is `i32`. To switch to `i64` or `Uuid` — **only one feature at a
time**, enabling both together is a deliberate compile-time error:

```toml
# project Cargo.toml
runique = { version = "4.0.0", features = ["postgres", "big-pk"] }    # Pk = i64
runique = { version = "4.0.0", features = ["postgres", "pk-uuid"] }   # Pk = Uuid (Uuid::now_v7())
```

The choice must be made before the first migration — see
[`model!` DSL & `extend!`](/docs/en/model/dsl) for full details (compatible types, FK
constraints, switching mode after the fact).

---

## Account states

`eihwaz_users` is the only user model: to add fields to it, use
`extend!{ table: "eihwaz_users", ... }`.

Two columns describe an account's state: `is_active` (is it active?) and `activated_at` (when
did its owner activate it, through the emailed link?).

| `is_active` | `activated_at` | State | Sign-in | "Forgot password" |
| --- | --- | --- | --- | --- |
| `false` | empty | **Pending** — created, never activated | Refused | Link that **activates** the account |
| `true` | set | **Active** | Allowed | Reset link |
| `false` | set | **Deactivated** from the admin | Refused | "Account blocked" email, no link |
| `true` | empty | Impossible — refused by the database | — | — |

- The **first activation** belongs to the email's owner: an admin can't tick `is_active` on a
  pending account.
- **Deactivating and reactivating** an account activated before is done from the admin, by an account allowed to.
- Changing the password **never reactivates** a deactivated account.
- The "forgot password" page answers the same whatever the account's state or existence.
- To cancel an invitation (pending account), delete the account.

The guarantee "no `is_active` without `activated_at`" is set **in the database**: a `CHECK`
constraint created with the `eihwaz_users` table, on all three engines. It applies to raw SQL too. A second `CHECK` constraint refuses an account holding more than one rank at a time (`is_superuser`, `is_dirigeant`, `is_admin`).

In code, the model implements the `RuniqueUser` trait:

| Method | Description |
| --- | --- |
| `can_sign_in()` | `is_active` **and** `activated_at` set |


Admin panel access is read on the signed-in account (`request.user`): `can_access_admin()` is true for rank admin, dirigeant or superuser, or for a member of a group that opens the admin. See [Middlewares & CurrentUser](/docs/en/auth/middleware).

---

## See also

| Section | Description |
| --- | --- |
| [Session helpers](/docs/en/auth/session) | `login`, `logout` |
| [Middlewares & CurrentUser](/docs/en/auth/middleware) | Route protection |

## Back to summary

- [Authentication](/docs/en/auth)
