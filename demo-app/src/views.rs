use crate::backend::doc::{doc_index, doc_page, doc_section_index};
use crate::backend::{
    auth::{find_user_by_username, get_profile_user, handle_inscription, handle_login},
    blog::{get_article, handle_blog_save, list_articles},
    contribution::{handle_contribution_submit, list_contributions},
    demo_code_page, fetch_changelog, fetch_known_issues, fetch_roadmap,
    forms::{extract_helpers_data, get_field_groups, handle_upload_image},
    inject_auth,
};
use crate::formulaire::*;
use runique::prelude::*;

// ─── Index ────────────────────────────────────────────────────────────────────

pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title"       => "Bienvenue sur Runique",
        "description" => "Runique — framework web Rust inspiré de Django, construit sur Axum, SeaORM et Tera. Formulaires typés, sécurité, ORM, admin généré.",
        "status"      => "Status: Framework en cours de développement...",
        "backend"     => "Rust , Axum",
        "template"    => "Moteur de template: Tera",
        "tokio"       => "Runtime asynchrone tokio",
        "session"     => "Session: tower avec memory store, evolution prévue a l'avenir",
        "orm"         => "ORM: sea-orm pour la gestion de la base de données",
        "migration"   => "Migrations: système intégré via macro model!",
    });
    request.render("index.html")
}

// ─── Auth ─────────────────────────────────────────────────────────────────────

pub async fn soumission_inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    handle_inscription(&mut request, &mut form).await
}

pub async fn login_user(
    mut request: Request,
    Prisme(form): Prisme<LoginForm>,
) -> AppResult<Response> {
    handle_login(&mut request, &form).await
}

pub async fn deconnexion(request: Request) -> AppResult<Response> {
    logout(&request.session).await.ok();
    Ok(Redirect::to("/").into_response())
}

pub async fn profil(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    if !is_authenticated(&request.session).await {
        warning!(request.notices => "Connectez-vous pour accéder à votre profil.");
        return Ok(Redirect::to("/login").into_response());
    }
    let user_id = get_user_id(&request.session).await;
    let username = get_username(&request.session).await;
    let user_opt = get_profile_user(user_id, &request.engine.db).await;
    context_update!(request => {
        "title"        => "Mon profil",
        "username"     => &username,
        "profile_user" => &user_opt,
        "connected"    => &true,
    });
    request.render("profile/profile.html")
}

pub async fn info_user(
    mut request: Request,
    Prisme(mut form): Prisme<UsernameForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let template = "profile/view_user.html";
    if request.is_get() && form.is_valid().await {
        let username_val = form.cleaned_string("username").unwrap_or_default();
        let user_opt = find_user_by_username(&request.engine.db, &username_val).await;
        match user_opt {
            Some(user) => {
                context_update!(request => {
                    "title"      => "Vue utilisateur",
                    "username"   => &user.username,
                    "email"      => &user.email,
                    "found_user" => &user,
                    "user"       => &form,
                    "messages"   => flash_now!(success => "Utilisateur trouvé !"),
                });
            }
            None => {
                context_update!(request => {
                    "title"    => "Vue utilisateur",
                    "user"     => &form,
                    "messages" => flash_now!(warning => "Utilisateur introuvable."),
                });
            }
        }
        return request.render(template);
    }
    context_update!(request => { "title" => "Vue utilisateur", "user" => &form });
    request.render(template)
}

// ─── Blog ─────────────────────────────────────────────────────────────────────

pub async fn blog_list(
    mut request: Request,
    Prisme(form): Prisme<SearchDemoForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let search = form.cleaned_string("search");
    let articles = list_articles(&request.engine.db, search.as_deref()).await;
    context_update!(request => {
        "title"       => "Blog — Articles",
        "articles"    => &articles,
        "search"      => &search,
        "search_form" => &form,
    });
    request.render("blog/blog_list.html")
}

pub async fn blog_save(
    mut request: Request,
    Prisme(mut blog): Prisme<BlogForm>,
) -> AppResult<Response> {
    handle_blog_save(&mut request, &mut blog).await
}

pub async fn blog_detail(Path(id): Path<i32>, mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    match get_article(&request.engine.db, id).await {
        Some(a) => {
            context_update!(request => { "title" => &a.title, "article" => &a });
            request.render("blog/blog_detail.html")
        }
        None => {
            warning!(request.notices => "Article introuvable.");
            Ok(Redirect::to("/blog/liste").into_response())
        }
    }
}

// ─── Formulaires ──────────────────────────────────────────────────────────────

pub async fn upload_image_submit(
    mut request: Request,
    Prisme(mut form): Prisme<ImageForm>,
) -> AppResult<Response> {
    handle_upload_image(&mut request, &mut form).await
}

pub async fn test_fields(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let field_groups = get_field_groups(&request.engine.db).await;
    context_update!(request => { "title" => "Champs disponibles", "field_groups" => &field_groups });
    request.render("forms/field_test.html")
}

pub async fn formulaires_hub(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => { "title" => "Formulaires" });
    request.render("formulaires/index.html")
}

pub async fn formulaires_champs(mut request: Request) -> AppResult<Response> {
    demo_code_page("formulaires_champs", &mut request).await
}

pub async fn formulaires_templates(mut request: Request) -> AppResult<Response> {
    demo_code_page("formulaires_templates", &mut request).await
}

pub async fn formulaires_helpers(
    mut request: Request,
    Prisme(form): Prisme<SearchDemoForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let data = extract_helpers_data(&request, form.cleaned_string("search"));
    context_update!(request => {
        "title"          => "Helpers & accès URL",
        "path_id"        => &data.path_id,
        "search_value"   => &data.search_value,
        "cleaned_search" => &data.cleaned_search,
    });
    request.render("formulaires/helpers.html")
}

// ─── Contribution ─────────────────────────────────────────────────────────────

pub async fn contribution_submit(
    mut request: Request,
    Prisme(mut form): Prisme<ContributionForm>,
) -> AppResult<Response> {
    handle_contribution_submit(&mut request, &mut form).await
}

pub async fn contribution_list(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let contributions = list_contributions(&request.engine.db).await;
    context_update!(request => { "title" => "Contributions", "contributions" => &contributions });
    request.render("contribution/contribution_list.html")
}

// ─── Info / Statique ──────────────────────────────────────────────────────────

pub async fn about(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    success!(request.notices => "Action réussie.");
    info!(request.notices => "Message d'information.");
    warning!(request.notices => "Attention requise.");
    error!(request.notices => "Une erreur est survenue.");
    context_update!(request => {
        "title"   => "À propos du Framework Runique",
        "content" => "Runique est un framework web inspiré de Django, construit sur Axum et Tera.",
    });
    request.render("about/about.html")
}

pub async fn rgpd(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => { "title" => "Politique de confidentialité" });
    request.render("rgpd/rgpd.html")
}

pub async fn changelog(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let sections = fetch_changelog(&request.engine.db).await;
    context_update!(request => {
        "title"          => "Changelog",
        "sections"       => &sections,
        "ext_link_label" => "CHANGELOG complet",
        "ext_link_url"   => "https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md",
    });
    request.render("info/cards.html")
}

pub async fn probleme_connu(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let sections = fetch_known_issues(&request.engine.db).await;
    context_update!(request => {
        "title"          => "Problèmes connus",
        "sections"       => &sections,
        "ext_link_label" => "CHANGELOG complet",
        "ext_link_url"   => "https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md",
    });
    request.render("info/cards.html")
}

pub async fn roadmap(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let sections = fetch_roadmap(&request.engine.db).await;
    context_update!(request => {
        "title"          => "Ce qui arrive",
        "sections"       => &sections,
        "ext_link_label" => "Roadmap complète",
        "ext_link_url"   => "https://github.com/seb-alliot/runique/blob/main/ROADMAP.md",
    });
    request.render("info/cards.html")
}

// ─── Sitemap ──────────────────────────────────────────────────────────────────

pub async fn sitemap_xml(_: Request) -> Response {
    let candidates = ["demo-app/sitemap.xml", "sitemap.xml", "/app/sitemap.xml"];
    let xml = candidates
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_else(|| "<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"></urlset>".to_string());

    (
        StatusCode::OK,
        [("content-type", "application/xml; charset=utf-8")],
        xml,
    )
        .into_response()
}

// ─── Readme ───────────────────────────────────────────────────────────────────

fn find_readme(candidates: &[&str]) -> String {
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            return content;
        }
    }
    "README introuvable.".to_string()
}

pub async fn readme_fr(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let content = find_readme(&[
        "docs/fr/README.fr.md",
        "../docs/fr/README.fr.md",
        "/app/docs/fr/README.fr.md",
    ]);
    context_update!(request => {
        "title"   => "README — Français",
        "content" => &content,
        "lang"    => "fr",
    });
    request.render("readme.html")
}

pub async fn readme_en(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let content = find_readme(&[
        "docs/en/README.md",
        "../docs/en/README.md",
        "/app/docs/en/README.md",
    ]);
    context_update!(request => {
        "title"   => "README — English",
        "content" => &content,
        "lang"    => "en",
    });
    request.render("readme.html")
}

// ─── Middleware ───────────────────────────────────────────────────────────────

pub async fn middleware_hub(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => { "title" => "Middlewares" });
    request.render("middleware/index.html")
}

pub async fn middleware_csrf(mut request: Request) -> AppResult<Response> {
    demo_code_page("middleware_csrf", &mut request).await
}

pub async fn middleware_csp(mut request: Request) -> AppResult<Response> {
    demo_code_page("middleware_csp", &mut request).await
}

pub async fn middleware_rate_limit(mut request: Request) -> AppResult<Response> {
    demo_code_page("middleware_rate_limit", &mut request).await
}

pub async fn middleware_login_guard(mut request: Request) -> AppResult<Response> {
    demo_code_page("middleware_login_guard", &mut request).await
}

pub async fn middleware_hosts(mut request: Request) -> AppResult<Response> {
    demo_code_page("middleware_hosts", &mut request).await
}

pub async fn middleware_https(mut request: Request) -> AppResult<Response> {
    demo_code_page("middleware_https", &mut request).await
}

// ─── Documentation ────────────────────────────────────────────────────────────

pub async fn docs_index_fr(mut request: Request) -> AppResult<Response> {
    doc_index("fr", &mut request).await
}

pub async fn docs_index_en(mut request: Request) -> AppResult<Response> {
    doc_index("en", &mut request).await
}

pub async fn docs_section(
    Path((lang, section)): Path<(String, String)>,
    mut request: Request,
) -> AppResult<Response> {
    doc_section_index(&lang, &section, &mut request).await
}

pub async fn docs_page(
    Path((lang, section, page)): Path<(String, String, String)>,
    mut request: Request,
) -> AppResult<Response> {
    doc_page(&lang, &section, &page, &mut request).await
}

// ─── Admin ────────────────────────────────────────────────────────────────────

pub async fn admin_hub(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => { "title" => "Administration" });
    request.render("admin/hub.html")
}

pub async fn admin_declaration(mut request: Request) -> AppResult<Response> {
    demo_code_page("admin_declaration", &mut request).await
}

pub async fn admin_setup(mut request: Request) -> AppResult<Response> {
    demo_code_page("admin_setup", &mut request).await
}

pub async fn surcharge_exemple(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => { "title" => "Exemple — template de surcharge" });
    request.render("admin/surcharge_exemple.html")
}

pub async fn admin_surcharge(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => { "title" => "Surcharge de templates" });
    request.render("admin/surcharge.html")
}

// ─── Pages démo ───────────────────────────────────────────────────────────────

pub async fn installation_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("installation_demo", &mut request).await
}

pub async fn configuration_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("configuration_demo", &mut request).await
}

pub async fn migrations_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("migrations_demo", &mut request).await
}

pub async fn comparatif_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("comparatif_demo", &mut request).await
}

pub async fn orm_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("orm_demo", &mut request).await
}

pub async fn database_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("database_demo", &mut request).await
}

pub async fn model_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("model_demo", &mut request).await
}

pub async fn router_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("router_demo", &mut request).await
}

pub async fn template_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("template_demo", &mut request).await
}

pub async fn macros_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("macros_demo", &mut request).await
}

pub async fn session_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("session_demo", &mut request).await
}

pub async fn i18n_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("i18n_demo", &mut request).await
}

// ─── Erreurs / CSRF ───────────────────────────────────────────────────────────

pub async fn test_csrf(request: Request) -> AppResult<Response> {
    success!(request.notices => "CSRF token validé avec succès !");
    Ok(Redirect::to("/").into_response())
}

pub async fn propos_template_error(mut request: Request) -> AppResult<Response> {
    context_update!(request => { "title" => "Page de test d'erreur de template" });
    request.render("router/fallback.html")
}

pub async fn force_not_found(_: Request) -> Response {
    StatusCode::NOT_FOUND.into_response()
}

pub async fn force_server_error(_: Request) -> Response {
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

pub async fn force_to_many_requests(_: Request) -> Response {
    StatusCode::TOO_MANY_REQUESTS.into_response()
}
