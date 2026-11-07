# Réinitialisation de mot de passe

[← Authentification](/docs/fr/auth)

---

> Le reset intégré hache le mot de passe soumis avec la configuration de l'application. Les modes `Auto`, `Manual` et `Custom` sont pris en charge. En mode `Delegated`, le reset intégré refuse de modifier un mot de passe local et conserve le token utilisable ; utilise plutôt le flux de récupération du fournisseur d'identité. Voir → [Configuration des mots de passe](/docs/fr/configuration/password).

## Ce que le framework fournit

Runique intègre un système complet, prêt à l'emploi :

- 2 routes auto-enregistrées (formulaire oubli + réinitialisation)
- 2 formulaires fournis (`ForgotPasswordForm`, `PasswordResetForm`)
- Génération et validation de tokens (UUID, TTL 1h, usage unique)
- Envoi automatique de l'email si le mailer est configuré
- Rate limiting intégré (5 requêtes / 5 min par défaut)
- Messages i18n inclus
- Une fois le reset réussi : toutes les sessions du compte fermées, ainsi que celle du navigateur où il a eu lieu (même connecté sous un autre compte) — dans la même transaction que le changement de mot de passe. Afficher le formulaire ne déconnecte personne
- Selon l'état du compte : lien d'activation (compte en attente), lien de réinitialisation (compte actif), email « compte bloqué » sans lien (compte désactivé) — la page répond toujours la même chose. Voir [États d'un compte](/docs/fr/auth/modele)

---

## Activation

```rust
RuniqueApp::builder(config)
    .with_public_url("https://monsite.com")
    .with_mailer_from_env()
    .with_password_reset(|pr| pr)
    .build()
    .await?
```

Les liens envoyés par email partent de l'URL publique du site, définie dans le builder par `.with_public_url("https://monsite.com")` (voir [Builder](/docs/fr/configuration/builder)).

En production, l'application refuse de démarrer sans elle, et sans mailer. Le lien n'est **jamais** construit à partir de l'en-tête `Host` de la requête, qu'un attaquant choisit (`Host: evil.com` → le jeton de la victime partirait chez lui). En développement (`RUNIQUE_ENV=dev`), l'adresse est toujours `http://127.0.0.1:{PORT}`, quelle que soit celle donnée au builder : un lien de test ne mène jamais au site de production.

> Les comptes sont ceux de `eihwaz_users`, la table utilisateur du framework — à enrichir avec `extend!{ table: "eihwaz_users", ... }`.

---

## Configuration

Toutes les options sont optionnelles — les valeurs par défaut fonctionnent sans rien changer.

```rust
.with_password_reset(|pr| pr
    .forgot_route("/mot-de-passe-oublie")       // défaut : /forgot-password
    .reset_route("/reinitialiser")               // défaut : /reset-password
    .forgot_template("auth/oubli.html")          // défaut : auth/forgot_password.html
    .reset_template("auth/reset.html")           // défaut : auth/reset_password.html
    .email_template("emails/reset.html")         // défaut : template intégré
    .success_redirect("/connexion")              // enregistré, pas encore utilisé (voir le flux)
    .token_ttl(Duration::from_secs(1800))        // défaut : 1h (std::time::Duration)
)
```

---

## Templates

Deux templates peuvent être surchargés dans `templates/`. S'ils sont absents, le framework utilise les siens.

Le formulaire se rend avec `{% form.<nom> %}` : il affiche les champs, leurs erreurs et le jeton CSRF. Rien d'autre à ajouter dans le `<form>`.

### `auth/forgot_password.html` — formulaire "email oublié"

```html
{% extends "base.html" %}
{% block content %}
{% messages %}
<form method="post">
    {% form.forgot_form %}
    <button type="submit">Envoyer le lien</button>
</form>
{% endblock %}
```

Variables disponibles dans le contexte Tera :

| Variable | Description |
| --- | --- |
| `forgot_form` | Formulaire avec le champ `email` (erreurs comprises) |
| `title` | Titre traduit de la page |
| `messages` | Messages flash : « vérifiez votre boîte », ou « indisponible » sans mailer |

### `auth/reset_password.html` — formulaire de reset

```html
{% extends "base.html" %}
{% block content %}
{% messages %}
{% if reset_done %}
    <p>Mot de passe mis à jour. <a href="/">Se connecter</a></p>
{% else %}
    <form method="post">
        {% form.reset_form %}
        <button type="submit">Changer le mot de passe</button>
    </form>
{% endif %}
{% endblock %}
```

Variables disponibles :

| Variable | Description |
| --- | --- |
| `reset_form` | Formulaire avec un token caché et les champs `email`, `password` et `confirm` (erreurs comprises) |
| `token` | Le jeton de l'URL |
| `title` | Titre traduit de la page |
| `reset_done` | `true` une fois le mot de passe changé : afficher la confirmation à la place du formulaire |

Le `<form>` sans `action` poste sur l'adresse de la page, `/reset-password/{token}` : c'est la bonne route.

> **Validations automatiques :** mot de passe d'au moins 10 caractères, avec une majuscule, une minuscule, un chiffre et un caractère spécial ; confirmation identique ; token valide et non expiré ; email saisi correspondant au compte lié au token.

---

## Flux complet

```text
1. Utilisateur clique "Mot de passe oublié"
        ↓
2. GET /forgot-password → affiche le formulaire email
        ↓
3. POST /forgot-password → email soumis
   - Si l'email existe : génère un token (1h), envoie un email avec le lien
   - Compte bloqué par l'équipe : email « compte bloqué », sans lien
   - Si l'email est inconnu : même réponse, dans le même temps (la recherche et l'envoi
     tournent en tâche de fond) — ne révèle pas les comptes existants
   - Redirige vers la page « mot de passe oublié » avec le message « vérifiez votre boîte »
        ↓
4. Utilisateur clique le lien reçu par email
        ↓
5. GET /reset-password/{token} → affiche le formulaire de reset
   - Token invalide ou expiré → message d'erreur, redirection vers /
        ↓
6. POST /reset-password/{token}
   - Valide le formulaire et compare l'email saisi au compte lié au token
   - Consomme le token (usage unique), met à jour le mot de passe et active
     le compte s'il était en attente, ferme ses sessions — en une transaction
   - Réaffiche la page avec reset_done = true (pas de redirection :
     success_redirect n'est pas encore lu)
```

> **Sessions actives après reset :** une fois le nouveau mot de passe enregistré, **toutes les sessions du compte sont fermées**, sur tous les appareils — une session volée comprise —, ainsi que la session du navigateur qui a fait le reset. Le tout se fait dans la transaction du changement de mot de passe : si la base refuse, rien n'est changé et le lien reste utilisable. Un seul lien est actif par compte : une nouvelle demande remplace le précédent. L'utilisateur se reconnecte avec le nouveau mot de passe. Les autres comptes ne sont pas touchés. Seuls les stores de Runique sont concernés : un store branché avec `with_session_store()` (Redis…) ne peut pas être parcouru par utilisateur — voir [Révocation des sessions](/docs/fr/auth/session#revocation-des-sessions).

---

## Intégration admin

Le bouton de réinitialisation de la fiche d'un compte, dans l'admin, envoie le même lien, par le même code. Il ne demande rien de plus que le reset du site : `.with_password_reset(...)`, l'URL publique et un mailer. Sans l'un d'eux, l'admin affiche « lien indisponible » : le lien n'est **jamais** montré à l'équipe.

Templates email de l'admin (optionnels, sinon templates intégrés) :
- `templates/admin/reset_password_email.html` (lien de réinitialisation)
- `templates/admin/user_created_email.html` (lien d'activation d'un compte créé)

Variables disponibles dans ces templates : `username`, `reset_url`, et les textes traduits `t_greeting`, `t_body`, `t_btn` ; plus `t_title` et `t_ignore` pour la réinitialisation, `t_validity` pour la création.

---

## Sans mailer configuré

- **En production :** l'application refuse de démarrer avec `.with_password_reset(...)` sans mailer.
- **En développement :** le formulaire « oublié » répond « réinitialisation indisponible » et n'envoie rien ; l'admin répond « lien indisponible ». Le lien n'est affiché nulle part.

> Pour tester en dev sans serveur SMTP : `EMAIL_BACKEND=console` imprime l'email dans le terminal.

---

← [**Authentification**](/docs/fr/auth) | [**Mailer**](/docs/fr/mailer) →
