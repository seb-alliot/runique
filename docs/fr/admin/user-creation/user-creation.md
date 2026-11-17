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
| Email perdu | Le staff renvoie le lien depuis la fiche du compte ; l'utilisateur le redemande lui-même via « mot de passe oublié » si le site a ouvert ce formulaire (`with_password_reset`) |
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

L'admin n'a besoin d'aucune déclaration dans `main.rs` pour ouvrir les comptes qu'il crée. Il lui faut deux choses :

- **un mailer complet dans le `.env`** (`EMAIL_BACKEND`, ou `SMTP_HOST`, `SMTP_USER` et `SMTP_PASS`, voir [Mailer](/docs/fr/mailer)) ;
- **l'adresse publique** (`.with_public_url(…)` ; avec `RUNIQUE_ENV=dev`, c'est `http://127.0.0.1:{PORT}`).

La page où le titulaire choisit son mot de passe est servie par l'admin lui-même, avec le gabarit de Runique. `with_password_reset(…)` sert à la personnaliser et à ouvrir aux visiteurs le formulaire « mot de passe oublié ».

**Sans mailer complet, l'admin ne crée pas de compte.** Le staff ne tient jamais de mot de passe : un compte sans lien envoyé ne pourrait être ouvert par personne. La page de création renvoie donc à la liste avec le message « l'envoi de courriel n'est pas configuré sur ce site », et rien n'est écrit. La relance d'un mot de passe depuis la fiche d'un compte est refusée de la même façon.

Si l'envoi échoue alors que le mailer est configuré (serveur SMTP injoignable, par exemple), le compte existe : le staff relance le lien depuis sa fiche.

**Le lien n'est jamais affiché à l'admin**, même en développement : celui qui crée le compte ne doit pas pouvoir choisir le mot de passe à la place de son propriétaire.

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

## Personnaliser dans `main.rs`

Sans rien déclarer, l'admin utilise les réglages de base :

| Élément | Valeur de base |
| --- | --- |
| Page où le titulaire choisit son mot de passe | `/reset-password/{token}`, gabarit `auth/reset_password.html` de Runique |
| Durée de vie du lien | 1 heure |
| Limite de débit de la page | 5 requêtes par 5 minutes et par adresse |
| Formulaire « mot de passe oublié » pour les visiteurs | non servi |

`with_password_reset(…)` remplace ces réglages et ouvre le formulaire aux visiteurs :

```rust
use std::time::Duration;

builder::new(config)
    .with_public_url("https://monsite.com")
    .with_password_reset(|mut pr| {
        // Limite de débit des deux pages
        pr.max_requests = 10;       // défaut : 5
        pr.retry_after = 600;       // défaut : 300 secondes
        pr.reset_route("/reinitialiser")            // défaut : /reset-password
            .reset_template("auth/reset.html")      // défaut : gabarit de Runique
            .forgot_route("/mot-de-passe-oublie")   // défaut : /forgot-password
            .forgot_template("auth/oubli.html")     // défaut : gabarit de Runique
            .token_ttl(Duration::from_secs(1800))   // défaut : 1 heure
    })
    .with_admin(admin::config)
```

Les liens envoyés par l'admin suivent ces réglages : même route, même durée de vie. Un gabarit du même nom que celui de Runique, posé dans votre dossier `templates/`, le remplace sans rien déclarer (`templates/auth/reset_password.html`).

Les deux emails de l'admin ont leurs propres gabarits, listés plus haut. Toutes les options de `with_password_reset` sont décrites dans [Réinitialisation de mot de passe](/docs/fr/auth/password-reset).

---

## URL du lien

```
{public_url}/{reset_route}/{token}
```

`public_url` vient de `.with_public_url(…)` ; `reset_route` vaut `/reset-password`, ou celle donnée à `with_password_reset()`. En production, l'application refuse de démarrer sans adresse publique quand l'admin ou la réinitialisation sont actifs. En développement (`RUNIQUE_ENV=dev`), l'adresse est toujours `http://127.0.0.1:{PORT}`, jamais l'en-tête `Host` de la requête.

---

## Voir aussi

- [Builder `ModelAdmin`](/docs/fr/admin/declaration-macro) — `.form()`, `accounts_users`
- [Déclaration](/docs/fr/admin/declaration) — `AdminSite`
- [Réinitialisation de mot de passe](/docs/fr/auth/password-reset) — flux forgot/reset

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
