\encoding utf8

-- ============================================================
-- Runique demo-app — seed complet
-- PostgreSQL — idempotent (DELETE + INSERT)
-- ============================================================

-- Suppression dans l'ordre (enfants d'abord)
DELETE FROM doc_block;
DELETE FROM doc_page;
DELETE FROM doc_section;
DELETE FROM page_doc_link;
DELETE FROM code_example;
DELETE FROM form_field;
DELETE FROM demo_section;
DELETE FROM demo_page;
DELETE FROM demo_category;
DELETE FROM known_issue;

-- ============================================================
-- demo_category
-- ============================================================
INSERT INTO demo_category (id, title, back_link_url, back_link_label, sort_order) VALUES
(1, 'Démarrage',        NULL,            NULL,            1),
(2, 'Authentification', NULL,            NULL,            2),
(3, 'Formulaires',      '/formulaires',  'Formulaires',   3),
(4, 'Database',         NULL,            NULL,            4),
(5, 'Framework',        NULL,            NULL,            5),
(6, 'Administration',   NULL,            NULL,            6);

-- ============================================================
-- demo_page
-- ============================================================
INSERT INTO demo_page (id, category_id, slug, title, lead, page_type, sort_order) VALUES
(1,  1, 'installation_demo',     'Installation',           'Prérequis, .env, commandes CLI, démarrage rapide.',                                                                'code',   1),
(2,  1, 'configuration_demo',    'Configuration builder',  'RuniqueApp::builder(), middlewares, secrets, assets.',                                                             'code',   2),
(3,  1, 'migrations_demo',       'Migrations',             'macro model!, makemigrations, migration up/down/status.',                                                          'code',   3),
(4,  1, 'comparatif_demo',       'Comparatif Django',      'Equivalences routes, vues, formulaires, ORM, securite.',                                                           'code',   4),
(5,  2, 'inscription',           'Inscription',            'Formulaire base modele, validation, hash mot de passe.',                                                           'form',   1),
(6,  2, 'login',                 'Authentification',       'Login / logout, sessions, protection des routes.',                                                                 'form',   2),
(7,  2, 'profil',                'Profil utilisateur',     'Session active, donnees utilisateur, deconnexion.',                                                                'custom', 3),
(8,  3, 'formulaires_hub',       'Formulaires',            'Declaration, champs disponibles, rendu Tera.',                                                                     'custom', 1),
(9,  3, 'formulaires_champs',    'Declaration des champs', 'Deux façons de declarer un champ — manuellement via register_fields, ou via la macro proc #[form].',               'code',   2),
(10, 3, 'formulaires_helpers',   'Helpers',                'Acces aux valeurs de formulaire et aux parametres d''URL — helpers types et acces whiteliste.',                    'custom', 3),
(11, 3, 'formulaires_templates', 'Rendu templates',        'Tags Tera pour le rendu des formulaires.',                                                                         'code',   4),
(12, 3, 'upload_image',          'Upload fichier',         'Image, validation taille et extension.',                                                                           'form',   5),
(13, 4, 'orm_demo',              'ORM — Requetes',         'objects, filter, paginate, relations, create/update/delete.',                                                      'code',   1),
(14, 4, 'database_demo',         'Base de donnees',        'DatabaseConfig, from_env, from_url, pool, timeouts.',                                                              'code',   2),
(15, 4, 'model_demo',            'Modeles & Schemas',      'Definir un modele, types, contraintes, relations, migrations.',                                                    'code',   3),
(16, 5, 'router_demo',           'Routeur',                'urlpatterns!, parametres de chemin, liens nommes.',                                                                'code',   1),
(17, 5, 'template_demo',         'Templates & Tags',       'Tags Tera — static, link, media, messages, form.xxx.',                                                            'code',   2),
(18, 5, 'middleware_hub',        'Middlewares',            'CSRF, CSP, Rate Limiter, Login Guard, Host Validation, HTTPS.',                                                    'custom', 3),
(19, 5, 'about',                 'Messages flash',         'success!, info!, warning!, error! — macros de messages flash.',                                                    'custom', 4),
(20, 5, 'i18n_demo',             'Internationalisation',   'Langue des templates Runique — 9 langues disponibles.',                                                            'code',   5),
(21, 5, 'session_demo',          'Sessions',               'MemoryStore, limites memoire, nettoyage periodique automatique.',                                                  'code',   6),
(22, 5, 'macros_demo',           'Macros',                 'context_update!, context!, impl_from_error!',                                                                      'code',   7),
(23, 5, 'propos_template_error', 'Pages d''erreur',        '404, 500, 429 — verification des pages Runique.',                                                                  'custom', 8),
(24, 5, 'middleware_csrf',       'CSRF',                   'Protection CSRF — Prisme, contrat POST vers handler.',                                                             'code',   9),
(25, 5, 'middleware_csp',        'CSP',                    'Content Security Policy — builder, directives.',                                                                   'code',  10),
(26, 5, 'middleware_hosts',      'Host Validation',        'Validation des hotes autorises.',                                                                                  'code',  11),
(27, 5, 'middleware_https',      'HTTPS Redirect',         'Redirection HTTP vers HTTPS automatique.',                                                                         'code',  12),
(28, 5, 'middleware_login_guard','Login Guard',             'Protection contre les tentatives de connexion repetees.',                                                          'code',  13),
(29, 5, 'middleware_rate_limit', 'Rate Limit',             'Limitation de requetes par IP avec fenetre glissante.',                                                            'code',  14),
(30, 6, 'admin_hub',             'Administration',         'CRUD auto-genere, permissions, tableau de bord, surcharge templates.',                                             'custom', 1),
(31, 6, 'admin_declaration',     'Declaration admin',      'Declarer une ressource dans le panneau admin.',                                                                    'code',   2),
(32, 6, 'admin_setup',           'Configuration admin',    'Activer et configurer le panneau admin.',                                                                          'code',   3),
(33, 6, 'admin_surcharge',       'Surcharge templates',    'Personnaliser les templates du panneau admin.',                                                                    'custom', 4);

-- ============================================================
-- code_example
-- ============================================================

-- inscription (page_id=5)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(5, 'formulaire/user.rs', 'rust', $$// Dériver un formulaire depuis le modele utilisateur
#[form(schema = eihwaz_users_schema,
       fields = [username, email, password])]
pub struct RegisterForm;

#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if self.get_string("username").len() < 3 {
            errors.insert("username".into(), "3 caracteres minimum".into());
        }
        if !self.get_string("email").contains('@') {
            errors.insert("email".into(), "adresse email invalide".into());
        }
        if self.get_string("password").len() < 10 {
            errors.insert("password".into(), "10 caracteres minimum".into());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}$$, 'formulaire', 1),

(5, 'handler', 'rust', $$pub async fn soumission_inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() && form.is_valid().await {
        match form.save(&request.engine.db).await {
            Ok(user) => {
                auth_login(&request.session, user.id, &user.username).await.ok();
                return Ok(Redirect::to("/profil").into_response());
            }
            Err(err) => form.get_form_mut().database_error(&err),
        }
    }
    context_update!(request => { "inscription_form" => &form });
    request.render("auth/inscription.html")
}$$, 'handler', 2),

(5, 'inscription.html', 'html', $$<form method="post" action="/inscription">
    {# Rendu complet — CSRF inclus automatiquement #}
    {% form.inscription_form %}
    <button type="submit">S''inscrire</button>
</form>

{# Rendu champ par champ — CSRF toujours inclus automatiquement #}
<form method="post" action="/inscription">
    {% form.inscription_form.username %}
    {% form.inscription_form.email %}
    {% form.inscription_form.password %}
    <button type="submit">S''inscrire</button>
</form>$$, 'template', 3);

-- login (page_id=6)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(6, 'formulaire/login.rs', 'rust', $$// Formulaire manuel — pas de modele, pas de clean
pub struct LoginForm {
    pub form: Forms,
}

impl RuniqueForm for LoginForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Nom d'utilisateur")
                .required(),
        );
        form.field(
            &TextField::password("password")
                .label("Mot de passe")
                .required(),
        );
    }

    // Pas de clean — impl_form_access!() suffit
    impl_form_access!();
}$$, 'formulaire', 1),

(6, 'handler', 'rust', $$pub async fn login(
    mut request: Request,
    Prisme(form): Prisme<LoginForm>,
) -> AppResult<Response> {
    if request.is_post() {
        let user_opt = UserEntity::find()
            .filter(Column::Username.eq(&username))
            .one(&db).await?;

        match user_opt {
            Some(user) if verify(&password, &user.password) => {
                auth_login(&session, user.id, &user.username).await;
                return Ok(Redirect::to("/profil").into_response());
            }
            _ => {}
        }
    }
    request.render("auth/login.html")
}$$, 'handler', 2);

-- upload_image (page_id=12)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(12, 'formulaire/image.rs', 'rust', $$use runique::prelude::*;

pub struct ImageForm {
    pub form: Forms,
}

impl RuniqueForm for ImageForm {
    impl_form_access!();

    fn register_fields(form: &mut Forms) {
        form.field(
            &FileField::image("image")
                .label("Choisissez une image")
                .upload_to("media/uploads/images")
                .required()
                .max_size_mb(5)
                .max_files(3)
                .max_dimensions(1920, 1080)
                .allowed_extensions(vec!["png", "jpg", "jpeg", "gif"]),
        );
    }
}$$, 'formulaire', 1),

(12, 'Types de FileField', 'rust', $$// Images : jpg jpeg png gif webp avif
FileField::image("avatar")

// Documents : pdf doc docx txt odt
FileField::document("cv")

// Tout type de fichier
FileField::any("fichier")

// Extensions personnalisees
FileField::any("data")
    .allowed_extensions(vec!["csv", "json"])$$, NULL, 2),

(12, 'Chemin d''upload', 'rust', $$// Chemin fixe dans le code
FileField::image("photo")
    .upload_to("media/uploads/photos")

// Depuis la variable MEDIA_ROOT dans .env
FileField::image("photo")
    .upload_to_env()

// Sans upload_to → MEDIA_ROOT directement$$, NULL, 3),

(12, 'handler', 'rust', $$pub async fn upload_image_submit(
    mut request: Request,
    Prisme(mut form): Prisme<ImageForm>,
) -> AppResult<Response> {
    if request.is_post() {
        if form.is_valid().await {
            success!(request.notices => "Fichier uploade avec succes !");
        } else {
            error!(request.notices => "Erreur de validation");
        }
        return Ok(Redirect::to("/upload-image").into_response());
    }
    context_update!(request => { "image_form" => &form });
    request.render("forms/upload_image.html")
}$$, 'handler', 4);

-- formulaires_champs (page_id=9)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(9, 'Declaration manuelle — register_fields', 'rust', $$use runique::prelude::*;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct MonForm {
    pub form: Forms,
}

impl RuniqueForm for MonForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username")
            .label("Nom d'utilisateur"));

        form.field(&TextField::email("email")
            .label("Adresse email"));

        form.field(&TextField::password("password")
            .label("Mot de passe")
            .min_length(8, "8 caracteres minimum"));

        form.field(&NumericField::integer("age")
            .label("Age")
            .min(0.0, "Valeur positive"));

        form.field(&BooleanField::new("actif")
            .label("Compte actif"));

        let roles = vec![
            ChoiceOption::new("admin", "Administrateur"),
            ChoiceOption::new("user", "Utilisateur"),
        ];
        form.field(&ChoiceField::new("role")
            .label("Role")
            .choices(roles));
    }
}$$, 'formulaire', 1),

(9, 'Macro proc #[form] — base sur un schema DB', 'rust', $$use crate::entities::contribution::schema as contribution;
use runique::prelude::*;

// Les champs sont lus depuis le schema SeaORM
// — types, contraintes, nullable deduits automatiquement
#[form(schema = contribution, fields = [title, content])]
pub struct ContributionForm;

#[async_trait]
impl RuniqueForm for ContributionForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let title = self.get_string("title");
        let mut errors = StrMap::new();

        if title.len() < 5 {
            errors.insert(
                "title".to_string(),
                "5 caracteres minimum".to_string(),
            );
        }

        if errors.is_empty() { Ok(()) }
        else { Err(errors) }
    }
}$$, 'formulaire', 2),

(9, 'Difference cle', 'rust', $$// Manuel — controle total, sans entite DB
// -> ideal pour formulaires de contact, login, recherche

// #[form] — branche sur le schema SeaORM
// -> ideal pour CRUD, ModelForm Django-like
// -> form.save(&db).await? disponible automatiquement$$, NULL, 3);

-- formulaires_helpers (page_id=10)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(10, 'Helpers POST — apres is_valid()', 'rust', $$// Valeur par defaut si le champ est vide
form.get_string("username")     // -> String  ("" si vide)
form.get_i32("age")             // -> i32     (0 par defaut)
form.get_f64("price")           // -> f64     (gere , -> .)
form.get_bool("active")         // -> bool    (true/1/on -> true)
form.get_uuid("ref")            // -> Uuid    (Uuid::nil() si vide)

// Option — None si vide
form.get_option("bio")          // -> Option<String>
form.get_option_i32("age")      // -> Option<i32>
form.get_option_f64("note")     // -> Option<f64>
form.get_option_bool("news")    // -> Option<bool>
form.get_option_uuid("id")      // -> Option<Uuid>$$, 'handler', 1),

(10, 'clear() — vider le formulaire apres traitement', 'rust', $$// Re-rendre le formulaire vide apres succes (sans redirect)
if form.is_valid().await {
    let path = form.cleaned_string("image"); // 1. lire avant clear
    // sauvegarder...
    form.clear();                            // 2. vider
    context_update!(request => { "upload_form" => &form });
    return request.render(template);         // 3. form vide affiche
}

// Avec redirect (PRG) : clear() inutile
// la nouvelle requete GET cree une instance fraiche automatiquement$$, 'handler', 2),

(10, 'Helpers date / heure', 'rust', $$form.get_naive_date("birthday")
form.get_naive_time("meeting")
form.get_naive_datetime("event_start")
form.get_datetime_utc("created_at")

// Variantes Option
form.get_option_naive_date("birthday")
form.get_option_naive_datetime("event_start")
form.get_option_datetime_utc("created_at")$$, 'handler', 3),

(10, 'Acces brut aux parametres d''URL — depuis Request', 'rust', $$// Route declaree : "/article/{id}"
let id   = request.path_param("id");  // Option<&str>

// Query string : /article/42?page=2
let page = request.from_url("page"); // Option<&str>$$, 'handler', 4),

(10, 'cleaned_*() — whiteliste, type, toutes sources', 'rust', $$// Priorite : POST -> path param -> query param
// None si le champ n''est pas declare dans le formulaire

form.cleaned_string("search")    // Option<String>
form.cleaned_i32("page")         // Option<i32>
form.cleaned_i64("id")           // Option<i64>
form.cleaned_f64("price")        // Option<f64>  (gere , -> .)
form.cleaned_bool("active")      // Option<bool>
form.cleaned_string("is_admin")  // None — champ inconnu$$, 'handler', 5),

(10, 'Exemple reel — recherche GET /blog/liste?search=rust', 'rust', $$// views.rs — filtre articles selon query string
pub async fn blog_list(
    mut request: Request,
    Prisme(form): Prisme<SearchDemoForm>,
) -> AppResult<Response> {
    let search = form.cleaned_string("search");

    let mut query = BlogEntity::find();
    if let Some(ref term) = search {
        query = query.filter(
            Condition::any()
                .add(Column::Title.contains(term))
                .add(Column::Summary.contains(term)),
        );
    }
    // ...
}

// Exemple reel : recherche user /view-user?username=alice
let username = form.cleaned_string("username").unwrap_or_default();$$, 'handler', 6);

-- middleware_rate_limit (page_id=29)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(29, 'Application sur une route — urls.rs', 'rust', $$use runique::macros::routeur::register_url::register_pending;
use runique::prelude::*;
use std::sync::Arc;

pub fn routes() -> Router {
    let limiter = Arc::new(
        RateLimiter::new()
            .max_requests(5)
            .retry_after(60),
    );

    // register_pending est requis pour que {% link 'nom_route' %}
    // fonctionne dans les templates Tera.
    register_pending("upload_image", "/upload-image");

    let upload_route = Router::new()
        .route("/upload-image", view!(upload_handler))
        .route_layer(middleware::from_fn_with_state(limiter, rate_limit_middleware));

    urlpatterns! {
        // autres routes...
    }.merge(upload_route)
}$$, 'urls', 1),

(29, 'Exemples de configuration', 'rust', $$// 5 requetes par minute
RateLimiter::new().max_requests(5).retry_after(60)

// 3 requetes par 5 minutes
RateLimiter::new().max_requests(3).retry_after(300)

// 100 requetes par minute (defaut : 60/60)
RateLimiter::new().max_requests(100).retry_after(60)$$, NULL, 2),

(29, 'Fonctionnement', 'text', $$// Fenetre glissante par adresse IP.
// Compteur reinitialise apres retry_after secondes.

// Reponse quand la limite est depassee :
HTTP/1.1 429 Too Many Requests
Retry-After: 42

// Le header Retry-After indique
// le delai avant la prochaine fenetre.$$, NULL, 3),

(29, 'Ordre dans le builder', 'text', $$// Les middlewares custom s''inserent au slot 20+.
// Ils s''executent AVANT la session et le CSRF.

Extensions(0)
  -> ErrorHandler(10)
  -> RateLimiter(20)   // ici
  -> Cache(40)
  -> Session(50)
  -> CSRF(60)
  -> routes$$, NULL, 4);

-- installation_demo (page_id=1)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(1, 'Cargo.toml — workspace', 'toml', $$[workspace]
members = ["monapp", "monapp/migration"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[workspace.dependencies]
runique  = { version = "1.1.50", features = ["orm", "postgres"] }
tokio    = { version = "1", features = ["full"] }
serde    = { version = "1", features = ["derive"] }$$, 'Cargo.toml', 1),

(1, '.env', 'bash', $$SECRET_KEY=une_cle_secrete_longue_et_aleatoire
DATABASE_URL=postgres://user:password@localhost:5432/ma_base
DEBUG=true$$, '.env', 2),

(1, 'Commandes CLI', 'bash', $$# Installer le CLI
cargo install runique

# Generer le projet
runique new mon-projet

# Demarrer le serveur (hot reload templates en DEBUG)
runique start

# Migrations
runique makemigrations
runique migrate up
runique migrate down
runique migrate status$$, 'terminal', 3);

-- configuration_demo (page_id=2)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(2, 'main.rs — builder complet', 'rust', $$use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    password_init(PasswordConfig::auto_with(Manual::Argon2));
    set_lang(Lang::Fr);

    let config = RuniqueConfig::from_env();
    let db     = DatabaseConfig::from_env()?.build().connect().await?;

    RuniqueApp::builder(config)
        .routes(url::routes())
        .with_database(db)
        .statics()
        .middleware(|m| {
            m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)
             .with_session_cleanup_interval(5)
             .with_allowed_hosts(|h| {
                 h.enabled(!is_debug())
                  .host("monsite.fr")
                  .host("www.monsite.fr")
             })
             .with_csp(|c| {
                 c.policy(SecurityPolicy::strict())
                  .with_upgrade_insecure(!is_debug())
                  .images(vec!["''self''", "data:"])
             })
        })
        .with_admin(|a| {
            a.site_title("Administration")
             .auth(RuniqueAdminAuth::new())
             .routes(admins::routes("/admin"))
        })
        .build().await?
        .run().await?;

    Ok(())
}$$, 'main.rs', 1),

(2, 'RuniqueConfig — variables .env', 'bash', $$# Obligatoires
SECRET_KEY=cle_secrete_256_bits
DATABASE_URL=postgres://user:pass@localhost/db

# Optionnels
DEBUG=true           # active le hot reload templates + traces debug
HOST=127.0.0.1       # defaut 127.0.0.1
PORT=3000            # defaut 3000$$, '.env', 2),

(2, 'Middleware — options disponibles', 'rust', $$m.with_session_duration(Duration::hours(24))
 .with_session_memory_limit(soft, hard)   // octets
 .with_session_cleanup_interval(minutes)

 .with_allowed_hosts(|h| h.host("...").enabled(true))
 .with_csp(|c| c.policy(SecurityPolicy::strict()))
 .with_https_redirect(true)$$, 'builder', 3);

-- migrations_demo (page_id=3)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(3, 'Declarer un modele — macro model!', 'rust', $$use runique::prelude::*;

model! {
    Article,
    table: "article",
    pk: id => i32,
    fields: {
        title:      String   [required, max_len(255)],
        content:    text     [required],
        author_id:  i32      [required, fk(eihwaz_users.id, cascade)],
        published:  bool     [required, default(false)],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}$$, 'entities/article.rs', 1),

(3, 'Commandes migration', 'bash', $$# Generer les fichiers de migration depuis les entites
runique makemigrations

# Appliquer toutes les migrations en attente
runique migrate up

# Annuler la derniere migration
runique migrate down

# Voir l''etat des migrations
runique migrate status$$, 'terminal', 2),

(3, 'Fichier de migration genere', 'rust', $$use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Alias::new("article"))
                .if_not_exists()
                .col(ColumnDef::new(Alias::new("id")).integer().not_null()
                    .auto_increment().primary_key())
                .col(ColumnDef::new(Alias::new("title")).string().not_null())
                .col(ColumnDef::new(Alias::new("content")).text().not_null())
                .col(ColumnDef::new(Alias::new("published")).boolean().not_null()
                    .default(false))
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Alias::new("article")).to_owned()).await
    }
}$$, 'migration generee', 3);

-- comparatif_demo (page_id=4)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(4, 'Routes', 'rust', $$-- Django (urls.py)
urlpatterns = [
    path("articles/",      views.article_list, name="article_list"),
    path("articles/<id>/", views.article_detail, name="article_detail"),
]

-- Runique (url.rs)
urlpatterns! {
    "/articles"      => view! { article_list },   name = "article_list",
    "/articles/{id}" => view! { article_detail }, name = "article_detail",
}$$, 'urls', 1),

(4, 'Vues', 'rust', $$-- Django (views.py)
def article_list(request):
    articles = Article.objects.all()
    return render(request, "articles.html", {"articles": articles})

-- Runique (views.rs)
pub async fn article_list(mut request: Request) -> AppResult<Response> {
    let db = request.engine.db.clone();
    let articles = article::Entity::find().all(&*db).await.unwrap_or_default();
    context_update!(request => { "articles" => &articles });
    request.render("articles.html")
}$$, 'views', 2),

(4, 'Modeles', 'rust', $$-- Django (models.py)
class Article(models.Model):
    title   = models.CharField(max_length=255)
    content = models.TextField()
    author  = models.ForeignKey(User, on_delete=models.CASCADE)

-- Runique (entities/article.rs)
model! {
    Article,
    table: "article",
    pk: id => i32,
    fields: {
        title:     String [required, max_len(255)],
        content:   text   [required],
        author_id: i32    [required, fk(eihwaz_users.id, cascade)],
    }
}$$, 'models', 3),

(4, 'Formulaires', 'rust', $$-- Django (forms.py)
class ArticleForm(ModelForm):
    class Meta:
        model  = Article
        fields = ["title", "content"]

-- Runique (formulaire/article.rs)
#[form(schema = article, fields = [title, content])]
pub struct ArticleForm;

#[async_trait]
impl RuniqueForm for ArticleForm {
    impl_form_access!(model);
}$$, 'forms', 4),

(4, 'Configuration', 'rust', $$-- Django (settings.py)
SECRET_KEY = "..."
DATABASES  = { "default": { "ENGINE": "django.db.backends.postgresql", ... } }
DEBUG      = True

-- Runique (.env + main.rs)
SECRET_KEY=...
DATABASE_URL=postgres://user:pass@localhost/db
DEBUG=true

// main.rs
let config = RuniqueConfig::from_env();
let db     = DatabaseConfig::from_env()?.build().connect().await?;$$, 'settings', 5);

-- formulaires_templates (page_id=11)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(11, 'Rendu complet du formulaire', 'html', $$<form method="POST" action="/inscription">
    {# Rendu de tous les champs + CSRF automatique #}
    {% form.inscription_form %}
    <button type="submit">S''inscrire</button>
</form>$$, 'template', 1),

(11, 'Rendu champ par champ', 'html', $$<form method="POST" action="/inscription">
    {# CSRF toujours inclus automatiquement — champ par champ ou rendu complet #}
    {% form.inscription_form.username %}
    {% form.inscription_form.email %}
    {% form.inscription_form.password %}

    <button type="submit">S''inscrire</button>
</form>$$, 'template', 2),

(11, 'Messages flash dans template', 'html', $${# Tag Runique — rendu automatique des messages #}
{% messages %}

{# Structure generee (message.html interne) : #}
{% if messages %}
    {% messages }
{% endif %}$$, 'template', 3);

-- orm_demo (page_id=13)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(13, 'Requetes de base — SeaORM', 'rust', $$let db = request.engine.db.clone();

// Tous les enregistrements
let articles = article::Entity::find()
    .all(&*db).await.unwrap_or_default();

// Avec filtre
let article = article::Entity::find()
    .filter(article::Column::Slug.eq("mon-article"))
    .one(&*db).await.unwrap_or(None);

// Par cle primaire
let article = article::Entity::find_by_id(42)
    .one(&*db).await.unwrap_or(None);

// Tri + limite
let recent = article::Entity::find()
    .order_by_desc(article::Column::CreatedAt)
    .limit(10)
    .all(&*db).await.unwrap_or_default();$$, 'handler', 1),

(13, 'Style Django — macro impl_objects!', 'rust', $$// Dans entities/article.rs
model! { Article, ... }
impl_objects!(Entity);   // active le manager objects

// Dans views.rs
let all     = Article::objects.all().all(&db).await?;
let actifs  = Article::objects.filter(Column::Published.eq(true)).all(&db).await?;
let count   = Article::objects.count(&db).await?;
let found   = Article::objects.get(&db, 42).await?;        // Err si absent
let opt     = Article::objects.get_optional(&db, 99).await?; // None si absent
let or_404  = Article::objects.get_or_404(&db, id, &request, "Introuvable").await?;$$, 'handler', 2),

(13, 'Create / Update / Delete', 'rust', $$use sea_orm::ActiveValue::Set;
use crate::entities::article::ActiveModel;

// Creer
let new_article = ActiveModel {
    title:     Set("Mon article".to_string()),
    content:   Set("Contenu...".to_string()),
    published: Set(false),
    ..Default::default()
};
let saved = new_article.insert(&*db).await?;

// Modifier
let mut article: ActiveModel = found.into();
article.title = Set("Nouveau titre".to_string());
article.update(&*db).await?;

// Supprimer
article::Entity::delete_by_id(42).exec(&*db).await?;$$, 'handler', 3);

-- database_demo (page_id=14)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(14, 'DatabaseConfig — depuis .env', 'rust', $$// .env
// DATABASE_URL=postgres://user:pass@localhost:5432/mabase

let config = DatabaseConfig::from_env()?
    .min_connections(1)
    .max_connections(20)
    .build();

let db: DatabaseConnection = config.connect().await?;$$, 'main.rs', 1),

(14, 'DatabaseConfig — depuis URL directe', 'rust', $$let config = DatabaseConfig::from_url(
    "postgres://user:pass@localhost:5432/mabase"
)?
.max_connections(50)
.connect_timeout(Duration::from_secs(10))
.build();

let db = config.connect().await?;$$, 'main.rs', 2),

(14, 'Moteurs supportes', 'text', $$postgres://  ou  postgresql://   → PostgreSQL
mysql://                          → MySQL
mariadb://                        → MariaDB
sqlite://                         → SQLite

// Feature flags Cargo.toml
runique = { version = "1.1.50", features = ["orm", "postgres"] }
runique = { version = "1.1.50", features = ["orm", "mysql"] }
runique = { version = "1.1.50", features = ["orm", "sqlite"] }$$, NULL, 3);

-- model_demo (page_id=15)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(15, 'macro model! — tous les types', 'rust', $$use runique::prelude::*;

model! {
    Produit,
    table: "produit",
    pk: id => i32,
    fields: {
        // Types de base
        nom:        String   [required, max_len(255)],
        slug:       String   [required, unique, max_len(255)],
        description: text    [nullable],
        prix:       f64      [required],
        stock:      i32      [required, default(0)],
        actif:      bool     [required, default(true)],

        // Cle etrangere
        categorie_id: i32    [required, fk(categorie.id, cascade)],

        // Timestamps automatiques
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}$$, 'entities/produit.rs', 1),

(15, 'Modificateurs disponibles', 'text', $$[required]               champ NOT NULL
[nullable]               champ NULL
[unique]                 contrainte UNIQUE
[max_len(N)]             VARCHAR(N)
[default(valeur)]        valeur par defaut SQL
[auto_now]               CURRENT_TIMESTAMP a la creation
[auto_now_update]        CURRENT_TIMESTAMP a chaque modification
[fk(table.col, cascade)] cle etrangere avec ON DELETE CASCADE
[fk(table.col, null)]    cle etrangere avec ON DELETE SET NULL$$, NULL, 2),

(15, 'Activer le manager objects', 'rust', $$model! {
    Article,
    table: "article",
    pk: id => i32,
    fields: { title: String [required] }
}

// Active Article::objects.all(), .filter(), .get(), etc.
impl_objects!(Entity);$$, 'entities/article.rs', 3);

-- router_demo (page_id=16)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(16, 'urlpatterns! — declaration des routes', 'rust', $$use runique::prelude::*;
use crate::views::*;

pub fn routes() -> Router {
    urlpatterns! {
        "/"                 => view! { index },          name = "index",
        "/articles"         => view! { article_list },   name = "article_list",
        "/articles/{id}"    => view! { article_detail }, name = "article_detail",
        "/articles/{id}/edit" => view! { article_edit }, name = "article_edit",
        "/inscription"      => view! { inscription },    name = "inscription",
    }
}$$, 'url.rs', 1),

(16, 'Lire un parametre de chemin', 'rust', $$// Route : "/articles/{id}"
pub async fn article_detail(mut request: Request) -> AppResult<Response> {
    let id = request.path_param("id")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    let db = request.engine.db.clone();
    let article = article::Entity::find_by_id(id)
        .one(&*db).await.unwrap_or(None);

    context_update!(request => { "article" => &article });
    request.render("article/detail.html")
}$$, 'handler', 2),

(16, 'Liens nommes dans les templates', 'html', $${# Route simple — pretraite en {{ link(link=''index'') }} #}
<a href='{% link "index" %}'>Accueil</a>

{# Parametre de route — pretraite en {{ link(link=''article_detail'', id=article.id) }} #}
<a href='{% link "article_detail" id=article.id %}'>{{ article.title }}</a>

{# Sans query — URL propre #}
<a href='{% link "article_list" %}'>Tous les articles</a>

{# Avec query — pretraite en {{ link(link=''article_list'', query={page: 2}) }} #}
<a href='{% link "article_list" query={page: 2} %}'>Page suivante</a>$$, 'template', 3);

-- template_demo (page_id=17)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(17, 'Fichiers statiques et media', 'html', $${# Fichiers dans static/ #}
<link rel="stylesheet" href="{% static "css/style.css" %}">
<script src="{% static "js/app.js" %}"></script>

{# Fichiers dans media/ (uploads utilisateur) #}
<img src="{% media article.image %}" alt="...">$$, 'template', 1),

(17, 'CSRF + nonce CSP', 'html', $$<form method="POST">
    {# CSRF automatique — inclus dans tout rendu de formulaire Runique #}
    {% form.mon_form %}
    ...
</form>

{# Nonce CSP — pour les scripts inline autorises #}
<script nonce="{{ csp_nonce }}">
    console.log(''autorise par la CSP'');
</script>$$, 'template', 2),

(17, 'Messages flash', 'html', $${# Dans le template — tag Runique #}
{% messages %}

{# Dans le handler Rust #}
success!(request.notices => "Enregistre !");
error!(request.notices   => "Erreur.");
info!(request.notices    => "Info.");
warning!(request.notices => "Attention.");$$, 'template', 3),

(17, 'Heritage de templates', 'html', $${# base.html — template parent #}
<html>
<body>
    {% block content %}{% endblock %}
</body>
</html>

{# page.html — template enfant #}
{% extends "base.html" %}
{% block content %}
    <h1>Mon contenu</h1>
{% endblock %}$$, 'template', 4);

-- i18n_demo (page_id=20)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(20, 'Configuration de la langue', 'rust', $$use runique::prelude::*;

// Dans main() — avant de demarrer le serveur
set_lang(Lang::Fr);    // Français
set_lang(Lang::En);    // Anglais (defaut)
set_lang(Lang::De);    // Allemand
set_lang(Lang::Es);    // Espagnol
set_lang(Lang::It);    // Italien
set_lang(Lang::Pt);    // Portugais
set_lang(Lang::Ja);    // Japonais
set_lang(Lang::Zh);    // Chinois
set_lang(Lang::Ru);    // Russe$$, 'main.rs', 1),

(20, 'Langues disponibles', 'text', $$Lang::Fr  — Français
Lang::En  — English    (defaut)
Lang::De  — Deutsch
Lang::Es  — Español
Lang::It  — Italiano
Lang::Pt  — Português
Lang::Ja  — 日本語
Lang::Zh  — 中文
Lang::Ru  — Русский

// Autodetection depuis locale navigateur
let lang = Lang::from("fr-FR");  // → Lang::Fr
let lang = Lang::from("en-US");  // → Lang::En$$, NULL, 2),

(20, 'Traductions dans les templates', 'html', $${# Cle de traduction simple #}
{{ ''forms.required'' | t }}
{# → "Ce champ est obligatoire" (Fr) #}
{# → "This field is required"   (En) #}

{# Cle avec parametre numerique #}
{{ ''forms.too_short'' | t(n=8) }}
{# → "8 caracteres minimum" #}$$, 'template', 3);

-- session_demo (page_id=21)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(21, 'Configuration dans le builder', 'rust', $$RuniqueApp::builder(config)
    .middleware(|m| {
        m
        // Limites memoire : soft 5 Mo, hard 10 Mo
        .with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)

        // Nettoyage automatique toutes les 5 minutes
        .with_session_cleanup_interval(5)

        // Duree de vie des sessions (defaut : 24h)
        .with_session_duration(Duration::hours(12))
    })$$, 'main.rs', 1),

(21, 'Lecture / ecriture dans un handler', 'rust', $$pub async fn mon_handler(mut request: Request) -> AppResult<Response> {
    // Lire
    let user_id = request.session
        .get::<i32>("user_id").await
        .ok().flatten();

    // Ecrire
    request.session.insert("user_id", 42).await.ok();
    request.session.insert("username", "alice").await.ok();

    // Supprimer une cle
    request.session.remove::<i32>("user_id").await.ok();

    // Invalider la session entiere (deconnexion)
    request.session.flush().await.ok();

    request.render("profil.html")
}$$, 'handler', 2);

-- macros_demo (page_id=22)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(22, 'context_update! — injecter des variables Tera', 'rust', $$pub async fn ma_vue(mut request: Request) -> AppResult<Response> {
    let articles = vec!["Article 1", "Article 2"];

    // Injecter dans le contexte Tera
    context_update!(request => {
        "title"    => "Ma page",
        "articles" => &articles,
        "count"    => articles.len(),
    });

    request.render("ma_page.html")
}$$, 'handler', 1),

(22, 'Macros flash — messages utilisateur', 'rust', $$// Dans un handler
success!(request.notices => "Enregistre avec succes !");
error!(request.notices   => "Une erreur est survenue.");
info!(request.notices    => "Verification en cours...");
warning!(request.notices => "Attention : session bientot expiree.");

// flash_now! — message injecte directement dans le contexte
context_update!(request => {
    "messages" => flash_now!(success => "Cree !"),
});$$, 'handler', 2),

(22, 'impl_from_error! — convertir des erreurs', 'rust', $$// Convertit automatiquement DbErr et autres en AppError
impl_from_error!(
    sea_orm::DbErr      => database_error,
    std::io::Error      => internal_error,
    serde_json::Error   => serialization_error,
);

// Permet d''utiliser ? dans les handlers
pub async fn handler(mut request: Request) -> AppResult<Response> {
    let article = Article::find_by_id(1).one(&*db).await?; // DbErr converti
    Ok(request.render("page.html")?)
}$$, 'handler', 3);

-- middleware_csrf (page_id=24)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(24, 'Contrat POST — Prisme extractor', 'rust', $$// Tout formulaire POST doit passer par Prisme<T>
// Prisme valide le token CSRF avant d''entrer dans le handler.
pub async fn inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() && form.is_valid().await {
        // Traitement securise — le CSRF a ete valide
        return Ok(Redirect::to("/profil").into_response());
    }
    context_update!(request => { "form" => &form });
    request.render("auth/inscription.html")
}$$, 'handler', 1),

(24, 'Token CSRF dans le template', 'html', $$<form method="POST" action="{% link "inscription" %}">
    {# Rendu complet — CSRF inclus automatiquement #}
    {% form.inscription_form %}
    <button type="submit">S''inscrire</button>
</form>

{# Rendu champ par champ — CSRF toujours inclus automatiquement #}
<form method="POST" action="{% link "inscription" %}">
    {% form.inscription_form.username %}
    {% form.inscription_form.password %}
    <button type="submit">S''inscrire</button>
</form>$$, 'template', 2),

(24, 'Fonctionnement interne', 'text', $$// GET  → token genere, stocke en session, injecte dans le contexte Tera
// POST → Prisme extrait le token du body, compare en constant-time
//        (via subtle::ConstantTimeEq pour eviter les timing attacks)

// Si le token est invalide ou absent :
//   → formulaire vide + message d''erreur CSRF
//   → le handler n''est jamais execute

// Le token est lie a la session — il change a chaque nouvelle session.$$, NULL, 3);

-- middleware_csp (page_id=25)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(25, 'Configuration dans le builder', 'rust', $$RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_csp(|c| {
            c.policy(SecurityPolicy::strict())
             .with_header_security(true)        // X-Frame-Options, X-Content-Type, etc.
             .with_upgrade_insecure(!is_debug()) // HTTP → HTTPS en prod
             .images(vec!["''self''", "data:"])  // Autorise data: pour les images
        })
    })$$, 'main.rs', 1),

(25, 'Politiques predefinies', 'rust', $$// Strict : bloque tout ce qui n''est pas explicitement autorise
SecurityPolicy::strict()

// Permissive : pour les phases de dev / migration
SecurityPolicy::permissive()

// Header genere (strict) :
// Content-Security-Policy:
//   default-src ''self'';
//   script-src ''self'' ''nonce-abc123'';
//   style-src ''self'';
//   img-src ''self'' data:;
//   object-src ''none'';
//   frame-ancestors ''none'';
//   upgrade-insecure-requests$$, NULL, 2),

(25, 'Nonce CSP pour scripts inline', 'html', $${# Le nonce est regenere a chaque requete.
   Il est injecte automatiquement dans le header CSP. #}
<script nonce="{{ csp_nonce }}">
    // Ce script inline est autorise par la CSP
    document.querySelector(''form'').addEventListener(''submit'', ...);
</script>$$, 'template', 3);

-- middleware_hosts (page_id=26)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(26, 'Configuration dans le builder', 'rust', $$RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_allowed_hosts(|h| {
            h.enabled(!is_debug())  // desactive en dev, actif en prod
             .host("monsite.fr")
             .host("www.monsite.fr")
             .host("monsite.up.railway.app")
        })
    })$$, 'main.rs', 1),

(26, 'Comportement', 'text', $$// Si le header Host de la requete ne correspond
// a aucun hote autorise :
//   → 400 Bad Request

// Comparaison : insensible a la casse, normalisation IPv6
// Wildcards :
//   .example.com  →  accepte foo.example.com, bar.example.com
//   (ne correspond pas a example.com sans sous-domaine)

// Desactive quand enabled(false) — toutes les requetes passent.$$, NULL, 2);

-- middleware_https (page_id=27)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(27, 'Activer la redirection HTTPS', 'rust', $$RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_csp(|c| {
            c.policy(SecurityPolicy::strict())
             // Active "upgrade-insecure-requests" dans le header CSP
             // + redirection 301 HTTP → HTTPS
             .with_upgrade_insecure(!is_debug())
        })
    })$$, 'main.rs', 1),

(27, 'Prerequis — proxy inverse', 'text', $$// Runique doit etre derriere un reverse proxy (Nginx, Caddy, Railway...)
// qui termine le TLS et transmet les requetes en HTTP.

// Le proxy doit envoyer le header :
//   X-Forwarded-Proto: https

// Exemple Nginx :
//   proxy_set_header X-Forwarded-Proto $scheme;

// Sans proxy TLS, with_upgrade_insecure(true) n''a pas d''effet visible.$$, NULL, 2);

-- middleware_login_guard (page_id=28)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(28, 'Creer un LoginGuard', 'rust', $$use runique::prelude::*;
use std::sync::Arc;

// Dans main() ou dans le module de routes
let guard = Arc::new(
    LoginGuard::new()
        .max_attempts(5)    // tentatives avant verrouillage
        .lockout_secs(300), // 5 minutes de verrouillage
);

// Nettoyer les entrees expirees regulierement
guard.spawn_cleanup(tokio::time::Duration::from_secs(60));$$, 'main.rs', 1),

(28, 'Utilisation dans le handler de login', 'rust', $$pub async fn login(
    mut request: Request,
    Prisme(form): Prisme<LoginForm>,
    State(guard): State<Arc<LoginGuard>>,
) -> AppResult<Response> {
    let username = form.get_string("username");
    let ip       = request.client_ip().unwrap_or_default();

    // Cle par username si rempli, sinon par IP
    if guard.is_locked_for(&username, &ip) {
        let secs = guard.retry_after_secs_for(&username, &ip);
        warning!(request.notices => format!("Compte bloque. Reessayez dans {secs}s."));
        return request.render("auth/login.html");
    }

    if request.is_post() {
        match authenticate(&username, &password, &db).await {
            Some(user) => {
                guard.record_success_for(&username, &ip);
                auth_login(&request.session, user.id, &user.username).await;
                return Ok(Redirect::to("/profil").into_response());
            }
            None => {
                guard.record_failure_for(&username, &ip);
                error!(request.notices => "Identifiants incorrects.");
            }
        }
    }
    request.render("auth/login.html")
}$$, 'handler', 2);

-- admin_declaration (page_id=31)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(31, 'admin.rs — macro admin!', 'rust', $$use crate::entities::{users, blog, article};
use crate::formulaire::{RegisterForm, BlogForm, ArticleForm};

// Ce fichier est la SOURCE pour "runique start"
// qui regenere src/admin/generated.rs automatiquement.
admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"]
    }
    blog: blog::Model => BlogForm {
        title: "Articles de blog",
        permissions: ["admin"]
    }
    article: article::Model => ArticleForm {
        title: "Articles",
        permissions: ["admin"]
    }
}$$, 'admin.rs', 1),

(31, 'Regles de la macro admin!', 'text', $$// Syntaxe :
// nom_ressource: entite::Model => FormType {
//     title: "Libelle dans le menu",
//     permissions: ["role1", "role2"],
// }

// nom_ressource  →  segment d''URL (/admin/nom_ressource/)
// entite::Model  →  modele SeaORM genere par model!
// FormType       →  formulaire qui implemente RuniqueForm
// permissions    →  roles autorises a acceder a cette ressource

// runique start regenere src/admin/generated.rs a chaque demarrage.$$, NULL, 2);

-- admin_setup (page_id=32)
INSERT INTO code_example (page_id, title, language, code, context, sort_order) VALUES
(32, 'Activer l''admin dans le builder', 'rust', $$RuniqueApp::builder(config)
    .with_admin(|a| {
        a.site_title("Mon Administration")
         .auth(RuniqueAdminAuth::new())
         .routes(admins::routes("/admin"))
         // Dashboard personnalise (optionnel)
         .templates(|t| t.with_dashboard("admin/mon_dashboard.html"))
         .with_state(admins::admin_state())
    })$$, 'main.rs', 1),

(32, 'Authentification admin — RuniqueAdminAuth', 'rust', $$// RuniqueAdminAuth verifie que l''utilisateur connecte
// a le role declare dans permissions de la ressource.

// Personnaliser l''auth (optionnel) :
pub struct MonAdminAuth;

#[async_trait]
impl AdminAuth for MonAdminAuth {
    async fn is_authorized(&self, session: &Session, role: &str) -> bool {
        let user_role = session.get::<String>("role").await.ok().flatten();
        user_role.as_deref() == Some(role)
    }
}

// Dans le builder :
.auth(MonAdminAuth)$$, 'admins/auth.rs', 2),

(32, 'URL generees automatiquement', 'text', $$// Pour chaque ressource declaree dans admin! :
GET    /admin/                    → tableau de bord
GET    /admin/article/            → liste
GET    /admin/article/create/     → formulaire de creation
POST   /admin/article/create/     → traitement creation
GET    /admin/article/{id}/edit/  → formulaire d''edition
POST   /admin/article/{id}/edit/  → traitement edition
POST   /admin/article/{id}/delete/ → suppression$$, NULL, 3);

-- ============================================================
-- page_doc_link (doc FR/EN uniquement — back link via demo_category)
-- ============================================================
INSERT INTO page_doc_link (page_id, label, url, link_type, sort_order) VALUES
-- installation
(1,  'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/01-installation.md',                   'doc_fr', 1),
(1,  'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md',                   'doc_en', 2),
-- configuration
(2,  'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/02-configuration.md',                  'doc_fr', 1),
(2,  'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/02-configuration.md',                  'doc_en', 2),
-- upload_image
(12, 'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/upload/upload.md', 'doc_fr', 1),
(12, 'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/upload/upload.md', 'doc_en', 2),
-- migrations
(3,  'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/03-migrations.md',                     'doc_fr', 1),
(3,  'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/03-migrations.md',                     'doc_en', 2),
-- auth
(5,  'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/13-authentification.md',          'doc_fr', 1),
(5,  'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/auth/13-authentification.md',          'doc_en', 2),
(6,  'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/13-authentification.md',          'doc_fr', 1),
(6,  'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/auth/13-authentification.md',          'doc_en', 2),
-- formulaires
(9,  'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/05-forms.md',               'doc_fr', 1),
(9,  'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/05-forms.md',               'doc_en', 2),
(10, 'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/helpers/helpers.md',        'doc_fr', 1),
(10, 'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/helpers/helpers.md',        'doc_en', 2),
-- orm / db / model
(13, 'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/07-orm.md',                        'doc_fr', 1),
(13, 'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/orm/07-orm.md',                        'doc_en', 2),
(14, 'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/06-database.md',                   'doc_fr', 1),
(14, 'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/orm/06-database.md',                   'doc_en', 2),
(15, 'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/04-models.md',                     'doc_fr', 1),
(15, 'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/orm/04-models.md',                     'doc_en', 2),
-- middlewares
(24, 'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csrf/csrf.md',               'doc_fr', 1),
(24, 'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csrf/csrf.md',               'doc_en', 2),
(25, 'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/csp.md',                 'doc_fr', 1),
(25, 'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md',                 'doc_en', 2),
(26, 'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/allowed-hosts/allowed-hosts.md', 'doc_fr', 1),
(26, 'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/allowed-hosts/allowed-hosts.md', 'doc_en', 2),
(27, 'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/https/https.md',             'doc_fr', 1),
(27, 'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/https/https.md',             'doc_en', 2),
(28, 'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/login-guard/login-guard.md', 'doc_fr', 1),
(28, 'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/login-guard/login-guard.md', 'doc_en', 2),
(29, 'Doc FR', 'https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/rate-limit/rate-limit.md',   'doc_fr', 1),
(29, 'Doc EN', 'https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/rate-limit/rate-limit.md',   'doc_en', 2);

-- ============================================================
-- form_field (page_id=9 — formulaires_champs)
-- ============================================================
INSERT INTO form_field (page_id, name, field_type, description, example, html_preview, sort_order) VALUES
(9, 'TextField::text',      'TextField',    'Champ texte simple — une ligne.',
 $$TextField::text("username").label("Nom d''utilisateur").required()$$,
 $p$<div class="field-preview"><label>Nom d'utilisateur</label><input type="text" placeholder="John Doe"></div>$p$, 1),

(9, 'TextField::email',     'TextField',    'Email — converti en minuscules automatiquement.',
 $$TextField::email("email").label("Adresse email").required()$$,
 $p$<div class="field-preview"><label>Adresse email</label><input type="email" placeholder="exemple@email.com"></div>$p$, 2),

(9, 'TextField::password',  'TextField',    'Mot de passe — haché via Argon2 par défaut. Désactiver avec .no_hash().',
 $$TextField::password("password").min_length(10, "10 caractères minimum").no_hash()$$,
 $p$<div class="field-preview"><label>Mot de passe</label><input type="password" placeholder="••••••••••"></div>$p$, 3),

(9, 'TextField::textarea',  'TextField',    'Zone de texte multi-ligne.',
 $$TextField::textarea("bio").label("Biographie").rows(5)$$,
 $p$<div class="field-preview"><label>Biographie</label><textarea rows="3" placeholder="Votre texte..."></textarea></div>$p$, 4),

(9, 'TextField::richtext',  'TextField',    'Texte riche — sanitisé côté serveur.',
 $$TextField::richtext("content").label("Contenu").required()$$,
 $p$<div class="field-preview"><label>Contenu</label><textarea rows="4" placeholder="<p>Texte riche sanitisé...</p>"></textarea></div>$p$, 5),

(9, 'TextField::url',       'TextField',    'URL — validée côté serveur.',
 $$TextField::url("website").label("Site web")$$,
 $p$<div class="field-preview"><label>Site web</label><input type="url" placeholder="https://exemple.com"></div>$p$, 6),

(9, 'NumericField::integer', 'NumericField', 'Entier — validation min/max.',
 $$NumericField::integer("age").min(0.0, "Positif").max(120.0, "Max 120").label("Âge")$$,
 $p$<div class="field-preview"><label>Âge</label><input type="number" step="1" placeholder="42" min="0" max="120"></div>$p$, 7),

(9, 'NumericField::float',   'NumericField', 'Nombre décimal.',
 $$NumericField::float("price").min(0.0, "Positif").label("Prix")$$,
 $p$<div class="field-preview"><label>Prix</label><input type="number" step="0.01" placeholder="3.14" min="0"></div>$p$, 8),

(9, 'NumericField::percent', 'NumericField', 'Pourcentage — restreint entre 0 et 100.',
 $$NumericField::percent("discount").label("Réduction (%)")$$,
 $p$<div class="field-preview"><label>Réduction (%)</label><input type="number" min="0" max="100" step="1" placeholder="75"></div>$p$, 9),

(9, 'NumericField::range',   'NumericField', 'Curseur — min, max, valeur par défaut.',
 $$NumericField::range("volume", 0.0, 100.0, 50.0).label("Volume")$$,
 $p$<div class="field-preview"><label>Volume</label><input type="range" min="0" max="100" value="50" class="fp-range"><span class="fp-range-val">50</span></div>$p$, 10),

(9, 'BooleanField::new',    'BooleanField', 'Case à cocher — renvoie true/false.',
 $$BooleanField::new("accept").label("J''accepte les conditions").required()$$,
 $p$<div class="field-preview"><label class="fp-row"><input type="checkbox"> J'accepte les conditions</label></div>$p$, 11),

(9, 'BooleanField::radio',  'BooleanField', 'Bouton radio Oui/Non.',
 $$BooleanField::radio("newsletter").label("S''abonner").checked()$$,
 $p$<div class="field-preview"><label class="fp-row"><input type="radio" checked> S'abonner à la newsletter</label></div>$p$, 12),

(9, 'ChoiceField::new',     'ChoiceField',  'Liste déroulante — options statiques ou dynamiques.',
 $$let opts = vec![ChoiceOption::new("fr", "France"), ChoiceOption::new("be", "Belgique")];
ChoiceField::new("country").label("Pays").choices(opts).required()$$,
 $p$<div class="field-preview"><label>Pays</label><select><option value="">---</option><option value="fr">France</option><option value="be">Belgique</option><option value="ch">Suisse</option></select></div>$p$, 13),

(9, 'RadioField::new',      'ChoiceField',  'Groupe de boutons radio.',
 $$RadioField::new("role").label("Rôle")
    .add_choice("admin", "Administrateur")
    .add_choice("user", "Utilisateur")$$,
 $p$<div class="field-preview"><label>Rôle</label><div class="fp-col"><label class="fp-row"><input type="radio" name="role_p" value="admin"> Administrateur</label><label class="fp-row"><input type="radio" name="role_p" value="user" checked> Utilisateur</label></div></div>$p$, 14),

(9, 'CheckboxField::new',   'ChoiceField',  'Cases à cocher multiples — retourne Vec<String>.',
 $$CheckboxField::new("tags").label("Catégories")
    .add_choice("rust", "Rust").add_choice("web", "Web")$$,
 $p$<div class="field-preview"><label>Catégories</label><div class="fp-col"><label class="fp-row"><input type="checkbox" value="rust" checked> Rust</label><label class="fp-row"><input type="checkbox" value="web"> Web</label></div></div>$p$, 15),

(9, 'DateField::new',       'DateField',    'Date — format YYYY-MM-DD.',
 $$DateField::new("birth_date").label("Date de naissance").required()$$,
 $p$<div class="field-preview"><label>Date de naissance</label><input type="date"></div>$p$, 16),

(9, 'TimeField::new',       'DateField',    'Heure — format HH:MM.',
 $$TimeField::new("meeting_time").label("Heure du rendez-vous")$$,
 $p$<div class="field-preview"><label>Heure du rendez-vous</label><input type="time"></div>$p$, 17),

(9, 'DateTimeField::new',   'DateField',    'Date et heure combinées.',
 $$DateTimeField::new("event_start").label("Début de l''événement").required()$$,
 $p$<div class="field-preview"><label>Début de l'événement</label><input type="datetime-local"></div>$p$, 18),

(9, 'FileField::image',     'FileField',    'Image — jpg jpeg png gif webp avif. Validation dimensions possible.',
 $$FileField::image("avatar").upload_to("media/uploads").max_size_mb(5).max_dimensions(500, 500)$$,
 $p$<div class="field-preview"><label>Avatar</label><input type="file" accept="image/*"></div>$p$, 19),

(9, 'FileField::document',  'FileField',    'Document — pdf doc docx txt odt.',
 $$FileField::document("cv").upload_to("media/docs").max_size_mb(10).required()$$,
 $p$<div class="field-preview"><label>CV</label><input type="file" accept=".pdf,.doc,.docx,.txt,.odt"></div>$p$, 20),

(9, 'FileField::any',       'FileField',    'Tout type de fichier — extensions personnalisables.',
 $$FileField::any("data").allowed_extensions(vec!["csv", "json"]).upload_to("media/imports")$$,
 $p$<div class="field-preview"><label>Fichier de données</label><input type="file" accept=".csv,.json"></div>$p$, 21),

(9, 'SlugField::new',       'SpecialField', 'Slug URL-friendly — validé côté serveur.',
 $$SlugField::new("slug").label("Slug").required()$$,
 $p$<div class="field-preview"><label>Slug</label><input type="text" pattern="[a-z0-9-]+" placeholder="mon-article-en-slug"></div>$p$, 22),

(9, 'ColorField::new',      'SpecialField', 'Sélecteur de couleur hex.',
 $$ColorField::new("theme_color").label("Couleur").default_color("#3b82f6")$$,
 $p$<div class="field-preview"><label>Couleur</label><input type="color" value="#3b82f6"></div>$p$, 23),

(9, 'UUIDField::new',       'SpecialField', 'UUID — validé côté serveur.',
 $$UUIDField::new("ref_id").label("Référence").required()$$,
 $p$<div class="field-preview"><label>Référence</label><input type="text" placeholder="550e8400-e29b-41d4-a716-446655440000"></div>$p$, 24),

(9, 'JSONField::new',       'SpecialField', 'Textarea JSON — validé côté serveur.',
 $$JSONField::new("config").label("Configuration JSON").rows(10)$$,
 $p$<div class="field-preview"><label>Configuration JSON</label><textarea rows="3" placeholder='{"cle": "valeur"}'></textarea></div>$p$, 25),

(9, 'IPAddressField::new',  'SpecialField', 'Adresse IP — IPv4 ou IPv6.',
 $$IPAddressField::new("server_ip").label("IP serveur").ipv4_only().required()$$,
 $p$<div class="field-preview"><label>IP serveur</label><input type="text" placeholder="192.168.1.1"></div>$p$, 26),

(9, 'HiddenField::new',     'HiddenField',  'Champ caché — valeur soumise sans rendu visible.',
 $$HiddenField::new("redirect_to")$$,
 $p$<div class="field-preview"><code class="fp-muted">&lt;input type="hidden" value="..."&gt;</code><span class="fp-small"> — invisible dans le rendu final</span></div>$p$, 27);
-- ============================================================
-- known_issue
-- ============================================================
INSERT INTO known_issue (version, title, description, issue_type, sort_order) VALUES
('1.1.49', 'Upload de fichiers',
 'Les validations d''upload (taille, dimensions) ne sont pas encore appliquees. Les fichiers sont envoyes avant verification, ce qui peut entrainer des uploads inutiles et des erreurs cote client.',
 'Fix', 1),
('1.1.50', 'DX',
 'Cargo watch non present. Le rechargement des templates en dev est actif seulement si DEBUG=true.',
 'Manquant', 2),
('1.1.50', 'Tracing des erreurs en mode DEBUG',
 'Le tracing est complet. Aucun parametrage individuel possible actuellement.',
 'Manquant', 3),
('1.1.50', 'Session',
 'Invalidation automatique des sessions. Deconnexion d''un utilisateur deja connecte sur un autre appareil.',
 'Manquant', 4);

-- ============================================================
-- Reset sequences
-- ============================================================
SELECT setval('demo_category_id_seq', (SELECT MAX(id) FROM demo_category));
SELECT setval('demo_page_id_seq',     (SELECT MAX(id) FROM demo_page));
SELECT setval('code_example_id_seq',  (SELECT MAX(id) FROM code_example));
SELECT setval('page_doc_link_id_seq', (SELECT MAX(id) FROM page_doc_link));
