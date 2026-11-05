# Création d'utilisateur via le panel admin

## Flux complet

Quand l'admin crée un utilisateur via le panel, le cycle est le suivant :

```
Admin remplit le formulaire
        ↓
Compte créé en base, en attente (is_active = false, activated_at vide)
Hash aléatoire injecté dans le champ password
        ↓
Email de bienvenue / reset envoyé à l'utilisateur
        ↓
L'utilisateur clique le lien
Définit son mot de passe
        ↓
Activation : is_active = true, activated_at rempli
        ↓
L'utilisateur peut se connecter
```

---

## Ce que fait le framework automatiquement

| Étape | Comportement builtin |
| --- | --- |
| Création via admin | `is_active = false` — le compte est inactif à la création |
| Champ `password` | Hash aléatoire injecté par le gate — le staff ne le voit jamais |
| Email | Lien « choisir mon mot de passe » envoyé à l'adresse du formulaire |
| Définition du mot de passe | Le compte en attente est activé : `is_active = true` et `activated_at` rempli. Un compte déjà activé (désactivé depuis) n'est **pas** réactivé |
| Connexion | Refusée tant que le compte n'est pas actif et activé (`can_sign_in()`) |
| Email perdu | L'utilisateur redemande un lien via « mot de passe oublié » |
| Activation par l'admin | Impossible : cocher `is_active` sur un compte en attente est refusé — la première activation appartient au propriétaire de l'email |

---

## Formulaire comptes — plus de `create_form:` séparé

Avec le builder `ModelAdmin`, le formulaire de création / édition des comptes vient du modèle (table `eihwaz_users` étendue via `extend!`). Tu configures les champs visibles :

```rust
fn accounts() -> ModelAdmin<user_profile::Entity> {
    use user_profile::Column::*;
    ModelAdmin::new()
        .title("Utilisateurs")
        .list(|l| {
            l.display([Username, Email, IsActive])
                .search([Username, Email])
        })
        .form(|f| f.fields([Username, Email]))
}

// dans AdminSite :
site.accounts_users(accounts())
```

Points importants :

- `password` est une colonne **secrète** : refusée dans `display`, `search`, `filter`, `fields`, etc.
- les drapeaux de rang (`is_superuser`, `is_dirigeant`, `is_admin`) ne sont pas des champs du formulaire. Le rang se change depuis la fiche du compte. `is_superuser` se pose avec `runique create-superuser`. Détail : [Permissions](/docs/fr/admin/permission).
- à la création d'un compte, le gate injecte un hash inconnu et le handler envoie le lien de bienvenue (`AccountLink::Welcome`)

Pas de `create_form:` / `edit_form:` / flag `inject_password` : c'est le comportement natif de la table comptes.

---

## Envoi de l'email

Le lien de bienvenue n'est envoyé que si les trois conditions sont réunies :

- un mailer est configuré (variables `SMTP_*` dans `.env`, voir [Mailer](/docs/fr/mailer)) ;
- la réinitialisation de mot de passe est activée (`.with_password_reset(…)` dans le builder) : le lien mène à sa page ;
- l'adresse publique est connue (`.with_public_url(…)`, sinon `http://127.0.0.1:{PORT}` en dev).

Sinon le compte est créé quand même, et l'admin voit un avertissement « lien indisponible ». **Le lien n'est jamais affiché à l'admin**, même en développement : celui qui crée le compte ne doit pas pouvoir choisir le mot de passe à la place de son propriétaire. L'utilisateur peut toujours demander un lien plus tard par « mot de passe oublié ».

Templates :

| Email | Template |
| --- | --- |
| Bienvenue (création depuis l'admin) | `admin/user_created_email.html` |
| Réinitialisation (bouton de la fiche du compte) | `admin/reset_password_email.html` |

Contexte Tera disponible dans ces templates :

| Variable | Valeur |
| --- | --- |
| `username` | Nom d'utilisateur |
| `reset_url` | Lien complet, absolu (sur l'adresse publique) |
| `t_greeting`, `t_body`, `t_btn`… | Textes traduits du mail (bienvenue : `t_greeting`, `t_body`, `t_btn`, `t_validity` ; réinitialisation : `t_title`, `t_greeting`, `t_body`, `t_btn`, `t_ignore`) |

L'adresse email n'est jamais mise dans le lien : la page de réinitialisation la vérifie contre le compte lié au token.

---

## URL du lien

```
{public_url}/{reset_route}/{token}
```

`public_url` vient de `.with_public_url(…)` ; `reset_route` est celle de `with_password_reset()` (`/reset-password` par défaut). En production, l'application refuse de démarrer sans adresse publique quand l'admin ou la réinitialisation sont actifs. En debug seulement, l'adresse est construite depuis l'en-tête `Host` de la requête.

---

## Voir aussi

- [Builder `ModelAdmin`](/docs/fr/admin/declaration-macro) — `.form()`, `accounts_users`
- [Déclaration](/docs/fr/admin/declaration) — `AdminSite`
- [Réinitialisation de mot de passe](/docs/fr/auth/password-reset) — flux forgot/reset

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
