# Déclarer l'admin

L'admin se déclare en Rust pur : un `AdminStaging` pour le site, un `AdminSite` pour les modèles, un `ModelAdmin<E>` par entité.

## Flux

```text
main.rs
  .with_admin(admin::config)
       │
       ▼
admin.rs :: config(AdminStaging) -> AdminStaging
  .site_title / .path / .page_size / …
  .site(|site| site.register(…).accounts_users(…))
       │
       ▼
Au build : validate() puis montage des routes CRUD
```

Aucune génération de code, aucun daemon, aucun `src/admins/`.

## `AdminStaging` — le site

Reçu par la closure `.with_admin(...)`. Méthodes principales :

```rust
pub fn config(admin: AdminStaging) -> AdminStaging {
    admin
        .site_title("Administration")
        .path("/admin/")
        .prefix("backoffice")   // optionnel
        .page_size(20)
        .with_rate_limiter(RateLimiter::new().max_requests(20).retry_after(3600))
        .with_login_guard(LoginGuard::new().max_attempts(20).lockout_secs(300))
        .templates(|t| t.with_dashboard("admin/mon_dashboard.html"))
        .site(site)
}
```

## `AdminSite` — les modèles

```rust
pub fn site(site: AdminSite) -> AdminSite {
    site.accounts_users(accounts())  // remplace l'admin comptes framework
        .register(blog())
        .register(article())
        .register(ModelAdmin::<tag::Entity>::new()) // défaut OK
}
```

| Méthode | Rôle |
| --- | --- |
| `.register(ModelAdmin<E>)` | Ajoute un modèle métier |
| `.accounts_users(ModelAdmin<E>)` | Remplace l'admin des comptes (extension `eihwaz_users`) |

Comptes et groupes sont toujours présents. Enregistrer un second modèle sous la clé comptes est refusé au boot.

## `ModelAdmin<E>` — un modèle

```rust
fn blog() -> ModelAdmin<blog::Entity> {
    use blog::Column::*;
    ModelAdmin::new()
        .title("Blog")
        .list(|l| { /* ListConfig */ })
        .form(|f| { /* FormConfig */ })
        .templates(|t| { /* surcharge pages de ce modèle */ })
}
```

Sans `.list().display(...)`, le framework affiche toutes les colonnes non secrètes (sur les comptes : username, email, is_active).

Sans `.form().fields(...)`, le formulaire expose tous les champs du formulaire du modèle (`ModelMeta::Form`), réduits aux droits de l'utilisateur.

## Routes admin déclarées dans `urlpatterns!`

Une page à vous, servie avec la protection de l'admin, se déclare dans `urlpatterns!` avec le mot `admin` devant le chemin :

```rust
use runique::admin::AdminView;

urlpatterns! {
    "/" => get(views::index), name = "index",

    // Page à part, rattachée au modèle des articles
    admin "/articles/stats" => get(views::article_stats),
        name = "article_stats",
        for_model = article::Entity,

    // Remplace la page de détail des articles
    admin "/articles/{id}" => get(views::article_detail),
        name = "article_detail",
        for_model = article::Entity,
        overrides = AdminView::Detail,
}
```

- `name` et `for_model` sont obligatoires ; le modèle doit être enregistré dans l'admin.
- Sans `overrides`, la page s'ajoute (`AdminView::Standalone`), autant que vous voulez par modèle.
- `overrides = AdminView::Detail` remplace la page de détail du modèle : les liens « voir » de la liste mènent à votre route. Le chemin doit contenir exactement un segment `{id}`, et rien d'autre. C'est la seule page qui se remplace par une route ; les autres se remplacent par un template (voir [Surcharge](/docs/fr/admin/template-surcharge)).
- La route garde son propre chemin, hors du préfixe de l'admin, et ne doit chevaucher ni une route publique ni l'espace de l'admin.
- Accès : connecté avec accès à l'admin → votre handler ; connecté sans accès → renvoyé vers `/` avec un message, session gardée ; anonyme → `/` (ou la connexion de l'admin s'il venait de l'admin). Le préfixe de l'admin n'est jamais révélé à un inconnu.
- La protection ne vérifie que l'**accès à l'admin**. Ce que votre handler lit ou écrit passe par la porte, comme dans le reste du framework : voir [Permissions](/docs/fr/admin/permission).

Pour une page sur le layout de l'admin (feuilles de style, menu avec les droits appliqués, barre du haut), appelez `admin_page_context` puis étendez `admin_base.html` :

```rust
use runique::prelude::*;

pub async fn article_stats(
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(site): Extension<Arc<AdminSite>>,
    mut request: Request,
) -> AppResult<Response> {
    admin_page_context(&mut request, &admin, &site, Page::Detail, "articles");
    request.render("admin/article_stats.html")
}
```

`Page` choisit les feuilles de style chargées, et le dernier argument le modèle mis en avant dans le menu. Cette fonction ne prépare que l'affichage : elle ne donne aucun droit.

Si l'admin est désactivé, déclarer une route `admin` refuse le démarrage.

## Imports utiles

```rust
use runique::admin::{AdminSite, ModelAdmin};
use runique::app::staging::AdminStaging;
use sea_orm::Order;
```

## Voir aussi

| Section | Description |
| --- | --- |
| [Builder ModelAdmin](/docs/fr/admin/declaration-macro) | Détail de l'API |
| [Validation au boot](/docs/fr/admin/declaration-daemon) | Ce qui est contrôlé au démarrage |
| [Mise en place](/docs/fr/admin/setup) | Câblage dans main |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
