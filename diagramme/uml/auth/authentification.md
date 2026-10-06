# UML — Authentification (traits, session, lockout, reset password)

[`auth/session.rs`](../../../runique/src/auth/session.rs),
[`auth/guard.rs`](../../../runique/src/auth/guard.rs),
[`auth/password.rs`](../../../runique/src/auth/password.rs),
[`auth/user.rs`](../../../runique/src/auth/user.rs)

## Trait & comptes (`eihwaz_users`, seul modèle utilisateur)

```mermaid
classDiagram
    class RuniqueUser {
        <<trait>>
        +user_id() Pk
        +email() &str
        +password_hash() &str
        +is_staff() / is_superuser() bool
    }
    class BuiltinUserEntity {
        <<unit>>
        +find_by_id/email/username(...)
        +update_password_by_id(...)
        +activate_pending(...) / set_password_and_activate(...)
    }
    class authenticate_admin {
        <<fn>>
        password vérifié d'abord, puis actif et staff/superuser
    }
    authenticate_admin ..> BuiltinUserEntity
    BuiltinUserEntity ..> RuniqueUser : Model eihwaz_users
```

## Lockout (process-local)

```mermaid
classDiagram
    class LoginGuard {
        -Store store [Arc Mutex HashMap]
        +u32 max_attempts
        +u64 lockout_secs
        +new() / max_attempts() / lockout_secs()
        +spawn_cleanup(period)
        +record_failure / is_locked / reset
    }
    note for LoginGuard "Store en mémoire process-local"
```

## Reset password (token haché, single-use)

```mermaid
classDiagram
    class PasswordResetConfig {
        +String forgot_route / reset_route
        +String forgot_template / reset_template
        +Option~String~ email_template
        +String success_redirect
        +Option~String~ base_url
        +u64 max_requests / retry_after
        +Duration token_ttl [défaut 1h]
    }
    class ForgotPasswordForm {
        +email
    }
    class PasswordResetForm {
        +token
        +new_password / confirm
    }
    class ResetToken {
        <<eihwaz_reset_tokens>>
        +bigint id
        +String token_hash UK
        +Pk user_id FK
        +NaiveDateTime expires_at
        +generate() / consume() / peek()
    }
    class build_router {
        <<fn>>
    }
    build_router ..> BuiltinUserEntity
    build_router ..> PasswordResetConfig
    build_router ..> ResetToken
    ForgotPasswordForm ..> ResetToken : génère (hashé)
    PasswordResetForm ..> ResetToken : consume (single-use)
```

Sécurité reset (vérifié) : token **stocké haché** (jamais brut), **single-use strict**
(delete + `rows_affected == 1`), `user_id` dérivé **du token** (jamais d'un champ URL/form)
→ IDOR-safe.

## Anomalies / flux suspects

### 🟠 AU1 — Lockout `LoginGuard` en mémoire process-local
[`guard.rs:84`](../../../runique/src/auth/guard.rs#L84)
Le compteur d'échecs vit dans une `HashMap` mémoire. En multi-process/multi-instance,
le lockout d'une instance n'est pas vu par les autres → un attaquant réparti sur N instances
multiplie les tentatives par N avant blocage. Cohérent avec le modèle mono-process actuel,
mais à acter.

### 🟡 AU2 — Cache permissions sans invalidation cross-instance (rappel AM4) — ✅ CORRIGÉ (2026-10-03)
Cache supprimé : `load_admin_user` relit l'état du compte (`BuiltinUserEntity::find_by_id`) et les droits
(`pull_groupes_db`, 2 requêtes) à chaque requête admin. Plus aucune copie à invalider.

### Rappels (déjà listés ailleurs)
- **AM2** double écriture `eihwaz_sessions` au login (`session_id` divergent).
- **AM3** TTL 24h codé en dur en double.
- **S1** sessions anonymes (et leur CSRF) perdues au restart.
