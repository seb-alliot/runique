# Réinitialisation de mot de passe

[← Authentification](/docs/fr/auth)

---

> **Mode `Auto` requis pour le reset built-in :** la route intégrée (`with_password_reset`) lit la valeur du formulaire après `finalize()` et l'écrit en base. En mode `Auto`, `finalize()` hache automatiquement le mot de passe — tout est correct. En mode `Manual`, `Custom` ou `Delegated`, `finalize()` ne hache pas : le mot de passe serait stocké en clair. Si tu n'utilises pas `PasswordConfig::auto()`, écris ta propre route de reset ou implémente `UserEntity::update_password` de façon à hacher la valeur reçue. Voir → [Configuration des mots de passe](/docs/fr/configuration/password)

## Ce que le framework fournit

Runique intègre un système complet, prêt à l'emploi :

- 2 routes auto-enregistrées (formulaire oubli + réinitialisation)
- 2 formulaires fournis (`ForgotPasswordForm`, `PasswordResetForm`)
- Génération et validation de tokens (UUID, TTL 1h, usage unique)
- Envoi automatique de l'email si le mailer est configuré
- Rate limiting intégré (5 requêtes / 5 min par défaut)
- Messages i18n inclus
- Déconnexion automatique à l'accès au formulaire de reset

---

## Activation

```rust
RuniqueAppBuilder::new(config)
    .with_mailer_from_env()
    .with_password_reset::<BuiltinUserEntity>(|pr| pr
        .base_url("https://monsite.com")  // optionnel en dev, obligatoire en prod
    )
    .build()
    .await?
```

> **`BuiltinUserEntity`** — utilise la table `eihwaz_users` fournie par le framework. Si ton projet a un modèle utilisateur custom, implémente le trait `UserEntity` (voir ci-dessous).

---

## Configuration

Toutes les options sont optionnelles — les valeurs par défaut fonctionnent sans rien changer.

```rust
.with_password_reset::<BuiltinUserEntity>(|pr| pr
    .forgot_route("/mot-de-passe-oublie")       // défaut : /forgot-password
    .reset_route("/reinitialiser")               // défaut : /reset-password
    .forgot_template("auth/oubli.html")          // défaut : auth/forgot_password.html
    .reset_template("auth/reset.html")           // défaut : auth/reset_password.html
    .email_template("emails/reset.html")         // défaut : template intégré
    .success_redirect("/connexion")              // défaut : /
    .base_url("https://monsite.com")             // pour construire le lien dans l'email
)
```

---

## Modèle utilisateur custom — trait `UserEntity`

Si tu n'utilises pas `BuiltinUserEntity`, implémente ce trait sur ton entité :

```rust
use runique::prelude::UserEntity;

#[async_trait::async_trait]
impl UserEntity for user::Model {
    async fn find_by_email(db: &DatabaseConnection, email: &str) -> Option<Self> {
        user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(db)
            .await
            .ok()?
    }

    async fn update_password(
        db: &DatabaseConnection,
        email: &str,
        hash: &str,
    ) -> Result<(), sea_orm::DbErr> {
        user::Entity::update_many()
            .col_expr(user::Column::Password, sea_orm::sea_query::Expr::value(hash))
            .filter(user::Column::Email.eq(email))
            .exec(db)
            .await?;
        Ok(())
    }

    fn username(&self) -> &str {
        &self.username
    }
}
```

Puis dans le builder :

```rust
.with_password_reset::<user::Model>(|pr| pr)
```

---

## Templates

Deux templates sont à créer dans `templates/`. Si absents, le framework utilise ses propres templates par défaut.

### `auth/forgot_password.html` — formulaire "email oublié"

```html
{% extends "base.html" %}
{% block content %}
<form method="post">
    {{ form.html | safe }}
    <button type="submit">Envoyer le lien</button>
</form>

{% if form.errors.email %}
    <p class="error">{{ form.errors.email }}</p>
{% endif %}
{% endblock %}
```

Variables disponibles dans le contexte Tera :

| Variable | Description |
| --- | --- |
| `form` | Formulaire avec champ `email` |
| `form.errors` | Erreurs de validation |

### `auth/reset_password.html` — formulaire de reset

```html
{% extends "base.html" %}
{% block content %}
<form method="post">
    {{ form.html | safe }}
    <button type="submit">Changer le mot de passe</button>
</form>

{% if form.errors.password %}
    <p class="error">{{ form.errors.password }}</p>
{% endif %}
{% endblock %}
```

Variables disponibles :

| Variable | Description |
| --- | --- |
| `form` | Formulaire avec champs `password` + `confirm` (token et email sont en hidden) |
| `form.errors` | Erreurs de validation |

> **Validations automatiques :** mot de passe minimum 10 caractères, confirmation identique, token valide et non expiré.

---

## Flux complet

```text
1. Utilisateur clique "Mot de passe oublié"
        ↓
2. GET /forgot-password → affiche le formulaire email
        ↓
3. POST /forgot-password → email soumis
   - Si l'email existe : génère un token (1h), envoie un email avec le lien
   - Si l'email est inconnu : même réponse (sécurité — ne révèle pas les comptes existants)
        ↓
4. Utilisateur clique le lien reçu par email
        ↓
5. GET /reset-password/{token}/{email_chiffré} → affiche le formulaire de reset
   - Token invalide ou expiré → erreur
        ↓
6. POST /reset-password/{token}/{email_chiffré}
   - Valide le formulaire (mot de passe min 10 chars, confirmation)
   - Consomme le token (usage unique)
   - Met à jour le mot de passe en DB
   - Redirige vers success_redirect (défaut : /)
```

> **Sessions actives après reset :** la mise à jour du mot de passe **n'invalide pas** les sessions déjà ouvertes. Si vous voulez forcer la déconnexion partout au changement de mot de passe, appelez `invalidate_all(user_id)` dans votre propre route de reset — voir [Révocation des sessions](/docs/fr/auth/session#révocation-des-sessions).

---

## Intégration admin

Si l'admin est activé avec `.user_resource()`, le panel peut envoyer des liens de reset directement depuis la fiche utilisateur.

Configuration dans `config_admin.rs` (généré par le daemon) :

```rust
admin_config
    .user_resource("users")
    .reset_password_url("https://monsite.com/reset-password")
    // optionnel — si absent, le lien s'affiche dans un flash message
```

Template email admin (optionnel, sinon template par défaut) :
- `templates/admin/reset_password_email.html`
- `templates/admin/user_created_email.html`

Variables disponibles dans ces templates : `username`, `email`, `reset_url`.

---

## Sans mailer configuré

Le mailer est optionnel. Si absent :
- En développement : le lien de reset s'affiche dans un flash message dans l'admin
- Côté formulaire oublié : le message de succès s'affiche mais aucun email n'est envoyé

> Pour tester en dev sans serveur SMTP : `EMAIL_BACKEND=console` imprime l'email dans le terminal.

---

← [**Authentification**](/docs/fr/auth) | [**Mailer**](/docs/fr/mailer) →
