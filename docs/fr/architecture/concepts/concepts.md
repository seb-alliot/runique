# Concepts clés

## 1. RuniqueEngine

**État principal** partagé de l'application, construit par le builder et accessible dans chaque handler via `request.engine` :

```rust
pub struct RuniqueEngine {
    pub config: RuniqueConfig,        // configuration (.env + builder)
    pub tera: ATera,                  // moteur de templates
    pub db: ADb,                      // connexion à la base (SeaORM)
    pub url_registry: ARlockmap,      // routes nommées, pour {% link %}
    pub security_csp: ASecurityCsp,   // politique CSP
    pub security_hosts: ASecurityHosts,
    pub trusted_proxies: Arc<TrustedProxies>,
    // … sessions, réglages des middlewares, connexions additionnelles (with_custom_db)
}
```

Les champs les plus utilisés : `request.engine.db` pour les requêtes, `request.engine.config` pour la configuration, `request.engine.extension::<T>()` pour une connexion déclarée avec `.with_custom_db(...)`.

---

## 2. Request — L'extracteur principal

`Request` est l'extracteur central de Runique : il réunit tout ce dont un handler a besoin.

```rust
pub struct Request {
    pub engine: AEngine,               // Arc<RuniqueEngine>
    pub session: Session,              // session tower-sessions
    pub notices: Message,              // messages flash (success!, error!…)
    pub csrf_token: CsrfToken,         // jeton CSRF de la session
    pub context: Context,              // contexte Tera de la page
    pub method: Method,                // méthode HTTP
    pub headers: HeaderMap,
    pub path_params: StrMap,           // paramètres de chemin (/{id})
    pub query_params: StrMap,          // paramètres de la query string
    pub user: Option<CurrentUser>,     // compte connecté, relu en base à chaque requête
    pub prisme: Prisme,                // données du formulaire envoyé + validité du CSRF
    // …
}
```

`Request` lit le corps de la requête : c'est un extracteur qui le consomme, il doit donc être le **dernier paramètre** du handler.

**Usage dans un handler :**

```rust
pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Accueil",
    });
    request.render("index.html")
}
```

**Méthodes principales :**

- `request.render("template.html")` — rendu avec le contexte courant
- `request.form::<F>("nom")` — construit le formulaire `F` avec les données envoyées (voir ci-dessous)

---

## 3. Formulaires : `request.form("nom")` puis `ValidationForm::try_form`

Le traitement d'un formulaire se fait en deux temps.

**`request.form("nom")` construit le formulaire, sans le valider.** Le corps de la requête a déjà été lu à l'extraction de `Request` (Prisme), et la validité du jeton CSRF déjà calculée. `request.form("nom")` :

1. crée une instance du formulaire avec ses champs ;
2. la remplit avec les données envoyées ;
3. la marque invalide si la méthode exige un jeton CSRF (toute méthode sauf GET et HEAD) et que celui-ci manque ou est faux.

**`ValidationForm::try_form(form, &request)` décide s'il faut valider, et valide.** Le formulaire a reçu son nom de `request.form("nom")` (rendu en champ caché) ; il n'est rempli, puis validé, que si la requête renvoie **ce même nom**. C'est ce qui permet **plusieurs formulaires sur une même page** : seul celui qui a été envoyé est rempli et validé, les autres restent vierges, sans erreur de champ, même s'ils ont un champ du même nom. Il renvoie `Ok(validé)` ou `Err(form)` avec ses erreurs, prêt à être réaffiché.

```rust
pub async fn signup(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form("signup");
    let form = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => match validated.save(&request.engine.db).await {
            Ok(_) => {
                success!(request.notices => "Compte créé !");
                return Ok(Redirect::to("/").into_response());
            }
            Err(form) => form,
        },
        Err(form) => form,
    };

    context_update!(request => {
        "signup_form" => &form,
    });
    request.render("signup.html")
}
```

> Un `ValidationForm` ne s'obtient que par `try_form` : une fonction qui prend un `ValidationForm<F>` prouve, par son type, que son appelant a validé avant d'agir.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Macros](/docs/fr/architecture/macros) | Macros de contexte, flash, routage, erreur |
| [Tags & filtres Tera](/docs/fr/architecture/tera) | Tags Django-like, filtres, fonctions |
| [Stack middleware](/docs/fr/architecture/middleware) | Ordre des slots, injection de dépendances |
| [Lifecycle d'une requête](/docs/fr/architecture/lifecycle) | Cycle de vie, bonnes pratiques |
| [Formulaires](/docs/fr/formulaire) | Champs, validation, rendu |

## Retour au sommaire

- [Architecture](/docs/fr/architecture)
