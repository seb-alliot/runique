# Mise en place de l'admin

Guide pas à pas pour activer l'interface d'administration dans un projet Runique existant.

---

## Prérequis

- Un projet Runique fonctionnel avec une base de données configurée
- Un modèle `users` avec les champs `is_staff` et `is_superuser` (générés par `model!`)
- Le binaire `runique` installé (`cargo install runique --features <moteur> --locked` ou `cargo build` du workspace)

---

## Étape 1 — Créer `src/admin.rs`

Ce fichier déclare les ressources administrables via la macro `admin!` :

```rust
// src/admin.rs
use crate::entities::{users, articles};
use crate::formulaire::{RegisterForm, ArticleForm};

admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
    },
    articles: articles::Model => ArticleForm {
        title: "Articles",
    }
}
```

---

## Étape 2 — Générer `src/admins/` avec le daemon

```bash
runique start
```

Le daemon lit `src/admin.rs`, génère `src/admins/` et lance `cargo run`.
Le dossier `src/admins/` est créé automatiquement — ne pas le modifier manuellement.

```text
src/admins/
  ├── README.md
  ├── mod.rs
  └── admin_panel.rs
```

> Si `src/admins/` existe déjà depuis une génération précédente, `runique start` le régénère.

---

## Étape 3 — Déclarer le module dans `src/main.rs`

```rust
mod admin;
mod admins;  // module généré par runique start
```

---

## Étape 4 — Câbler `.with_admin()` dans le builder

```rust
use runique::app::builder::RuniqueAppBuilder as builder;

RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .with_admin(|a| {
        a.site_title("Administration")
         .routes(admins::routes("/admin"))
         .with_state(admins::admin_state())
    })
    .build()
    .await?
    .run()
    .await?;
```

| Méthode | Rôle |
| --- | --- |
| `.prefix("secret")` | Segment monté **devant** l'admin (défaut : aucun) — voir ci-dessous |
| `.site_title("…")` | Titre affiché dans l'interface |
| `.routes(admins::routes("/admin"))` | Monte les routes CRUD sous `/admin` |
| `.with_state(…)` | État partagé généré par le daemon |
| `.no_robots_txt()` | Désactive le `/robots.txt` automatique |

### `.routes()` et `.prefix()` — deux réglages distincts

`.routes(admins::routes("/site-admin"))` définit **où vit l'admin**.
`.prefix("secret")` définit **ce qui est monté devant**. Ils se composent :

| `routes(...)` | `.prefix(...)` | URL de l'admin |
| --- | --- | --- |
| `/site-admin` | — | `/site-admin/…` |
| `/site-admin` | `secret` | `/secret/site-admin/…` |
| *(non appelé)* | `secret` | `/secret/admin/…` (chemin par défaut) |

L'ordre des deux appels n'a aucune importance, et les slashs sont optionnels :
`secret`, `/secret` et `/secret/` sont équivalents.

> **Note de version** — `.prefix()` *remplaçait* auparavant le chemin de l'admin,
> ce qui la mettait en concurrence avec `.routes()` : le dernier appelé gagnait, et
> inverser les deux lignes changeait silencieusement les URLs servies. Elle préfixe
> désormais, conformément à son nom. Un projet qui utilisait `.prefix("/backoffice")`
> pour *renommer* l'admin doit passer ce chemin à `admins::routes("/backoffice")`.

---

> **robots.txt automatique** — Quand l'admin est actif, Runique génère automatiquement
> une route `/robots.txt` (ligne `Sitemap:` si `.sitemap(…)` est défini, plus les
> signaux `Content-Signal`). Utilisez `.no_robots_txt()` si vous souhaitez gérer ce
> fichier vous-même.
>
> **Le préfixe admin n'y figure pas, volontairement.** `robots.txt` est un fichier
> public : y écrire `Disallow: /mon-prefixe/` publierait justement le chemin qu'un
> préfixe personnalisé cherche souvent à garder discret. Et `Disallow` demande de ne
> pas *parcourir*, pas de ne pas *indexer* — une URL connue par ailleurs peut
> apparaître malgré tout dans les résultats.
>
> L'exclusion est assurée par l'en-tête `X-Robots-Tag: noindex, nofollow`, posé par
> Runique sur **toutes** les réponses de l'admin — page de login incluse, ainsi que
> les redirections, les erreurs et les réponses JSON. Il ne divulgue aucun chemin et
> reste en place même si vous surchargez les templates admin.

---

## Étape 5 — Créer un superuser

```bash
runique create-superuser
```

Suit un assistant interactif pour créer le premier compte admin (`is_superuser = true`).

---

## Accès à l'interface

Une fois le serveur démarré, l'interface est disponible à :

```text
http://localhost:{PORT}/admin/
```

La page `/admin/login` redirige vers le dashboard si les identifiants sont valides.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CLI](/docs/fr/admin/declaration) | Commande `runique start`, workflow général |
| [Permissions](/docs/fr/admin/permission) | Rôles, `is_staff` / `is_superuser`, vérification runtime |
| [Templates](/docs/fr/admin/template) | Hiérarchie de templates, blocks, surcharge du visuel |
| [Évolutions](/docs/fr/admin/evolution) | Axes d'évolution et état bêta |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
