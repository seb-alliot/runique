# Création d'utilisateur via le panel admin

## Flux complet

Quand l'admin crée un utilisateur via le panel, le cycle est le suivant :

```
Admin remplit le formulaire
        ↓
Compte créé en base (is_active = false)
Hash aléatoire injecté dans le champ password
        ↓
Email de reset envoyé à l'utilisateur
        ↓
L'utilisateur clique le lien
Définit son mot de passe
        ↓
update_password → is_active = true
        ↓
L'utilisateur peut se connecter
```

---

## Ce que fait le framework automatiquement

| Étape | Comportement builtin |
| --- | --- |
| Création via admin | `is_active = false` — le compte est inactif à la création |
| Champ `password` | Hash aléatoire injecté — l'utilisateur ne connaît pas ce mot de passe |
| Email | Lien de reset envoyé à l'adresse fournie dans le formulaire |
| Définition du mot de passe | `is_active` passe à `true` automatiquement via `update_password` |
| Connexion | `auth_login()` bloque les comptes avec `is_active = false` |

---

## Formulaire de création builtin — `UserAdminCreateForm`

Le formulaire fourni par Runique (`runique::admin::UserAdminCreateForm`) expose :

| Champ | Type | Description |
| --- | --- | --- |
| `username` | Texte requis | Nom d'utilisateur (min. 3 caractères) |
| `email` | Email requis | Adresse de contact + destinataire de l'email de reset |
| `password` | Caché | Hash aléatoire injecté automatiquement — non visible dans l'interface |
| `is_staff` | Booléen | Accès lecture au panel admin |
| `is_superuser` | Booléen | Accès complet admin |

`is_active` n'est **pas** dans le formulaire — le compte est toujours créé inactif.

---

## Configuration dans `admin!{}`

```rust
admin! {
    users: eihwaz_users::Model => MyForm {
        title: "Utilisateurs",
        permissions: ["admin"],
        create_form: runique::admin::UserAdminCreateForm,
        edit_form: crate::formulaire::UserEditForm,
    }
}
```

Le champ `create_form:` indique au daemon d'utiliser `UserAdminCreateForm` à la création
et d'activer le flag `inject_password` sur la ressource.

---

## Envoi de l'email

L'email est envoyé si le mailer est configuré (variables `SMTP_*` dans `.env`).

Sans mailer (développement), le lien de reset est affiché dans un flash message.

Le template email par défaut est `admin/user_created_email.html`. Il peut être surchargé
via `AdminConfig::reset_password_email_template("mon_template/email.html")`.

Contexte Tera disponible dans le template :

| Variable | Valeur |
| --- | --- |
| `username` | Nom d'utilisateur |
| `email` | Adresse email |
| `reset_url` | Lien complet de reset (absolu si `reset_password_url` configuré) |

---

## URL de reset

L'URL construite suit le schéma :

```
{base_url}/reset-password/{token}/{encrypted_email}
```

En production, configurer `reset_password_url` dans le builder pour générer une URL absolue :

```rust
.with_admin(|a| a
    .reset_password_url("https://monsite.fr/reset-password")
)
```

Sans cette configuration, l'URL est construite depuis le header `Host` de la requête HTTP
(`http://{host}/reset-password/...`).

---

## Modèle custom

Si le projet utilise son propre modèle utilisateur (pas `eihwaz_users`), il doit implémenter
`UserEntity` et gérer `is_active` dans `update_password` lui-même.

`auth_login()` utilise `BuiltinUserEntity` — les projets avec un modèle custom appellent
`login()` directement et contrôlent la vérification `is_active` dans leur propre handler.

---

## Voir aussi

- [Macro `admin!`](/docs/fr/admin/declaration/macro) — `create_form:`, `edit_form:`
- [Réinitialisation de mot de passe](/docs/fr/auth) — flux forgot/reset

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
