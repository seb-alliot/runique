# Mailer — envoi d'emails

[← Retour](/docs/fr)

---

## Configuration

Le mailer se configure dans le `.env`, et nulle part ailleurs : il n'y a rien à déclarer dans le builder de l'application. Runique lit ces variables au premier envoi.

```env
# Backend : "smtp" (défaut) ou "console" (dev — affiche dans le terminal)
EMAIL_BACKEND=smtp

SMTP_HOST=smtp.example.com
SMTP_PORT=587                   # optionnel, défaut = 587
SMTP_USER=user@example.com
SMTP_PASS=secret
SMTP_FROM=noreply@example.com   # optionnel, défaut = SMTP_USER
SMTP_STARTTLS=true              # optionnel, défaut = true
```

**La configuration doit être complète.** Avec le backend `smtp`, `SMTP_HOST`, `SMTP_USER` et `SMTP_PASS` sont tous les trois requis : s'il en manque un, le mailer est considéré comme absent, exactement comme un `.env` vide. Le backend `console` n'a besoin de rien d'autre.

Un mailer absent n'empêche pas l'application de démarrer : un site qui n'envoie aucun email n'a rien à configurer. Ce qui en dépend le dit au moment où il en a besoin :

| Qui envoie | Sans mailer complet |
| --- | --- |
| Votre code (`Email::new()…send()`, `dispatch_email`) | `send()` rend une erreur |
| L'admin (création d'un compte, relance d'un mot de passe) | l'action est refusée avant toute écriture, avec un message |
| La page « mot de passe oublié » déclarée par `with_password_reset` | en production, l'application refuse de démarrer |

`mailer_configured()` dit si le mailer est complet, pour qu'un handler choisisse quoi faire avant d'envoyer.

---

## Backends

| Backend | Comportement |
| --- | --- |
| `smtp` | Envoi réel via SMTP (production) |
| `console` | Affiche l'email dans le terminal (développement) |

En dev, `EMAIL_BACKEND=console` évite d'avoir un serveur SMTP local. L'email complet (from, to, subject, body) s'affiche dans les logs.

---

## L'admin et le mailer

L'admin envoie lui-même des emails : il ne tient jamais un mot de passe, donc chaque compte qu'il crée reçoit un lien pour choisir le sien.

**Fonctionnement de base.** Avec un mailer complet dans le `.env`, rien d'autre n'est à écrire. L'admin envoie le lien, et sert lui-même la page où le titulaire choisit son mot de passe, avec le gabarit de Runique.

**Sans mailer complet, l'admin refuse.** Avant toute écriture, il vérifie que le mailer est configuré en entier :

| Action dans l'admin | Sans mailer complet |
| --- | --- |
| Créer un compte | refusé : aucun compte n'est créé, la page renvoie à la liste |
| Relancer le mot de passe d'un compte | refusé : aucun lien n'est créé |

Dans les deux cas, l'admin affiche « L'envoi de courriel n'est pas configuré sur ce site : action impossible. » Un compte sans lien envoyé ne pourrait être ouvert par personne : mieux vaut ne pas le créer.

**Personnaliser.** Les gabarits de la page, la route, la durée de vie du lien et la limite de débit se règlent dans `main.rs`, avec `with_password_reset(…)`. Voir [Création d'utilisateurs](/docs/fr/admin/user-creation).

---

## Envoyer un email

### Raccourci — texte brut

```rust
use runique::prelude::*;

dispatch_email("user@example.com", "Bienvenue", "Votre compte a été créé.").await?;
```

### Builder complet — HTML, templates, reply-to

```rust
use runique::prelude::Email;

Email::new()
    .to("user@example.com")
    .subject("Bienvenue sur la plateforme")
    .html("<h1>Bonjour !</h1><p>Votre compte est actif.</p>")
    .reply_to("support@example.com")
    .send()
    .await?;
```

### Avec un template Tera

```rust
use runique::prelude::Email;
use runique::context;

let ctx = context! { "username" => "Alice", "url" => "https://example.com/confirm/abc" };

Email::new()
    .to("alice@example.com")
    .subject("Confirmez votre adresse")
    .template(&tera, "emails/confirmation.html", ctx.into())?
    .send()
    .await?;
```

Le template est un fichier Tera standard dans ton dossier `templates/` :

```html
{# templates/emails/confirmation.html #}
<h1>Bonjour {{ username }} !</h1>
<p><a href="{{ url }}">Confirmez votre adresse email</a></p>
```

---

## Méthodes du builder `Email`

| Méthode | Description |
| --- | --- |
| `.to(address)` | Destinataire |
| `.subject(text)` | Objet |
| `.html(body)` | Corps HTML |
| `.text(body)` | Corps texte brut |
| `.reply_to(address)` | En-tête Reply-To |
| `.template(&tera, name, ctx)` | Rend un template Tera comme corps HTML. Retourne `Result`. |
| `.send().await` | Envoie l'email. Retourne `Result<(), String>`. |

> **Priorité** : si `.html()` et `.template()` sont tous les deux appelés, le dernier gagne. `.text()` est utilisé uniquement si aucun corps HTML n'est défini.

---

## Utilisation dans un handler

```rust
use runique::prelude::*;

pub async fn register(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form("register");
    let form = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "register_form" => &form });
            return request.render("register.html");
        }
    };

    let email = form.cleaned::<String>("email").unwrap_or_default();
    let username = form.cleaned::<String>("username").unwrap_or_default();

    // ... sauvegarder en DB ...

    // L'échec d'envoi n'annule pas l'inscription : il est seulement journalisé.
    if let Err(e) = dispatch_email(
        &email,
        "Bienvenue !",
        &format!("Bonjour {username}, votre compte est actif."),
    )
    .await
    {
        tracing::warn!("Erreur email : {e}");
    }

    Ok(Redirect::to("/").into_response())
}
```

> **Erreurs d'envoi** : `.send()` et `dispatch_email` retournent `Result<(), String>`. En production, logue l'erreur et continue — un email raté ne doit pas planter la requête.

---

← [**Retour**](/docs/fr)
