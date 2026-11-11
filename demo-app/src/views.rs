use crate::backend::lang::{msg, visitor_lang};
use crate::backend::{
    account::{handle_delete_account, render_profile},
    auth::{handle_activate, handle_inscription, handle_login},
    blog::{get_article, handle_blog_save, list_articles},
    comparatif::{comparatif_detail_page, comparatif_intro_page, limites_page},
    contribution::{handle_contribution_submit, list_contributions},
    cours::{ExerciceInput, cours_detail, cours_exercice, cours_index, cours_root},
    demo_code_page,
    doc::{doc_index, doc_page, doc_section_index},
    fetch_known_issues,
    forms::{extract_helpers_data, get_field_groups, handle_upload_image},
    inject_globals, split_markdown_sections,
};
use crate::formulaire::{
    BlogForm, ContributionForm, DeleteAccountForm, ImageForm, LoginForm, RegisterForm,
    SearchDemoForm,
};
use runique::prelude::*;

pub async fn erreur_502(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    request.render("erreur_502.html")
}
// ─── Index ────────────────────────────────────────────────────────────────────
pub async fn index(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    request.render("index.html")
}

#[derive(serde::Deserialize)]
pub struct LangQuery {
    #[serde(default)]
    next: Option<String>,
}

pub async fn set_language(Path(code): Path<String>, request: Request) -> AppResult<Response> {
    let next = request.query::<LangQuery>()?.next;
    Ok(crate::backend::lang::switch_language(&code, next.as_deref(), &request).await)
}

// ─── Auth ─────────────────────────────────────────────────────────────────────

pub async fn soumission_inscription(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form("inscription");
    let form = form.in_language(&request.headers);
    handle_inscription(&mut request, form).await
}

pub async fn activate_account(
    Path(token): Path<String>,
    mut request: Request,
) -> AppResult<Response> {
    handle_activate(&mut request, token).await
}

pub async fn login_user(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form("login");
    let form = form.in_language(&request.headers);
    handle_login(&mut request, form).await
}

/// Runique checks a POST's CSRF token when the view reads its form; this one
/// reads none, so it checks the token itself.
pub async fn deconnexion(request: Request) -> AppResult<Response> {
    if !request.prisme.csrf_valid {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    logout(&request.session, None).await?;
    Ok(Redirect::to("/").into_response())
}

pub async fn supprimer_compte(mut request: Request) -> AppResult<Response> {
    if !is_authenticated(&request.session).await {
        return Ok(Redirect::to("/login").into_response());
    }
    let form: DeleteAccountForm = request.form("delete_account");
    let form = form.in_language(&request.headers);
    handle_delete_account(&mut request, form).await
}

pub async fn profil(mut request: Request) -> AppResult<Response> {
    if !is_authenticated(&request.session).await {
        warning!(request.notices => msg(&request.headers, "Please log in to access your profile.", "Connectez-vous pour accéder à votre profil."));
        return Ok(Redirect::to("/login").into_response());
    }
    let delete_form: DeleteAccountForm = request.form("delete_account");
    let delete_form = delete_form.in_language(&request.headers);
    render_profile(&mut request, delete_form, false).await
}

// ─── Blog ─────────────────────────────────────────────────────────────────────

pub async fn blog_list(mut request: Request) -> AppResult<Response> {
    let form: SearchDemoForm = request.form("search");
    let form = form.in_language(&request.headers);
    inject_globals(&mut request).await;
    let form = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated.into_form(),
        Err(form) => form,
    };
    let search = form.cleaned::<String>("search");
    let articles = list_articles(&request.engine.db, search.as_deref()).await;
    context_update!(request => {
        "title"       => "Blog — Articles",
        "description" => "The demo blog built with Runique: a model, its form and a search over the articles.",
        "articles"    => &articles,
        "search"      => &search,
        "search_form" => &form,
    });
    request.render("blog/blog_list.html")
}

pub async fn blog_save(mut request: Request) -> AppResult<Response> {
    let blog: BlogForm = request.form("blog");
    let blog = blog.in_language(&request.headers);
    if !is_authenticated(&request.session).await {
        warning!(request.notices => msg(&request.headers, "Please log in to access your profile.", "Connectez-vous pour accéder à votre profil."));
        return Ok(Redirect::to("/login").into_response());
    }
    handle_blog_save(&mut request, blog).await
}

pub async fn blog_detail(Path(id): Path<Pk>, mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    if let Some(a) = get_article(&request.engine.db, id).await {
        context_update!(request => { "title" => &a.title, "article" => &a });
        request.render("blog/blog_detail.html")
    } else {
        warning!(request.notices => msg(&request.headers, "Article not found.", "Article introuvable."));
        Ok(Redirect::to("/blog/liste").into_response())
    }
}

// ─── Forms ───────────────────────────────────────────────────────────────────

pub async fn upload_image_submit(mut request: Request) -> AppResult<Response> {
    let form: ImageForm = request.form("image");
    let form = form.in_language(&request.headers);
    if !is_authenticated(&request.session).await {
        warning!(request.notices => msg(&request.headers, "Please log in to access your profile.", "Connectez-vous pour accéder à votre profil."));
        return Ok(Redirect::to("/login").into_response());
    }
    handle_upload_image(&mut request, form).await
}

pub async fn test_fields(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    let field_groups = get_field_groups(&request.engine.db).await;
    context_update!(request => { "title" => "Available fields", "field_groups" => &field_groups });
    request.render("forms/field_test.html")
}

pub async fn formulaires_hub(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    context_update!(request => { "title" => "Forms", "description" => "Runique's form engine: typed forms from a model, validation, file and media fields, rendering in Tera templates." });
    request.render("formulaires/index.html")
}

pub async fn formulaires_champs(mut request: Request) -> AppResult<Response> {
    demo_code_page("formulaires_champs", &mut request).await
}

pub async fn formulaires_templates(mut request: Request) -> AppResult<Response> {
    demo_code_page("formulaires_templates", &mut request).await
}

pub async fn formulaires_helpers(mut request: Request) -> AppResult<Response> {
    let form: SearchDemoForm = request.form("helpers");
    let form = form.in_language(&request.headers);
    inject_globals(&mut request).await;
    let data = extract_helpers_data(&request, form.cleaned::<String>("search"));
    context_update!(request => {
        "title"          => "Helpers & URL access",
        "description"    => "Form helpers in Runique: cleaned values, URL parameters, clear() and the helpers available after validation.",
        "path_id"        => &data.path_id,
        "search_value"   => &data.search_value,
        "cleaned_search" => &data.cleaned_search,
    });
    request.render("formulaires/helpers.html")
}

// ─── Contribution ─────────────────────────────────────────────────────────────

pub async fn contribution_submit(mut request: Request) -> AppResult<Response> {
    let form: ContributionForm = request.form("contribution");
    let form = form.in_language(&request.headers);
    if !is_authenticated(&request.session).await {
        warning!(request.notices => msg(&request.headers, "Please log in to access your profile.", "Connectez-vous pour accéder à votre profil."));
        return Ok(Redirect::to("/login").into_response());
    }
    handle_contribution_submit(&mut request, form).await
}

pub async fn contribution_list(mut request: Request) -> AppResult<Response> {
    if !is_authenticated(&request.session).await {
        warning!(request.notices => msg(&request.headers, "Please log in to access contributions.", "Connectez-vous pour accéder aux contributions."));
        return Ok(Redirect::to("/login").into_response());
    }
    inject_globals(&mut request).await;
    let contributions = list_contributions(&request.engine.db).await;
    context_update!(request => { "title" => "Contributions", "contributions" => &contributions });
    request.render("contribution/contribution_list.html")
}

// ─── Info / Static ───────────────────────────────────────────────────────────

pub async fn about(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    success!(request.notices => msg(&request.headers, "Action successful.", "Action réussie."));
    info!(request.notices => msg(&request.headers, "Information message.", "Message d'information."));
    warning!(request.notices => msg(&request.headers, "Attention required.", "Attention requise."));
    error!(request.notices => msg(&request.headers, "An error occurred.", "Une erreur est survenue."));
    context_update!(request => {
        "title"   => "Flash messages",
        "description" => "Flash messages in Runique: success, info, warning and error notices shown after a redirect, with their macros and template tag.",
        "content" => "Runique is a web framework inspired by Django, built on Axum and Tera.",
    });
    request.render("about/about.html")
}

pub async fn rgpd(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    context_update!(request => { "title" => "Privacy Policy", "description" => "Privacy policy of runique.io: data collected, cookies, hosting, retention and account deletion." });
    request.render("rgpd/rgpd.html")
}

pub async fn rust_django_framework(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    context_update!(request => {
        "title"       => "Rust Web Framework Inspired by Django",
        "description" => "Runique is a Rust web framework inspired by Django: typed ModelAdmin panel, type-safe forms, built-in CSRF/CSP security and persistent sessions. A Django alternative for Rust, built on Axum, SeaORM and Tera.",
    });
    request.render("seo/rust_django_framework.html")
}

pub async fn releases(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    let lang = visitor_lang(&request.headers);
    let content = if lang == "fr" {
        include_str!("../../CHANGELOG.fr.md")
    } else {
        include_str!("../../CHANGELOG.md")
    };
    let versions = split_markdown_sections(content);
    let index = request
        .get_query("version")
        .and_then(|wanted| versions.iter().position(|v| v.key == wanted))
        .unwrap_or(0);
    let newer = index
        .checked_sub(1)
        .and_then(|i| versions.get(i))
        .map(|v| &v.key);
    let older = versions.get(index + 1).map(|v| &v.key);
    context_update!(request => {
        "title"    => if lang == "fr" { "Journal des modifications" } else { "Changelog" },
        "description" => if lang == "fr" {
            "Journal des modifications de Runique, version par version : nouveautés, ruptures et correctifs."
        } else {
            "Runique's changelog, version by version: new features, breaking changes and fixes."
        },
        "versions" => &versions,
        "current"  => versions.get(index),
        "newer"    => newer,
        "older"    => older,
    });
    request.render("info/changelog.html")
}

pub async fn releases_security(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    let lang = visitor_lang(&request.headers);
    let sections = split_markdown_sections(if lang == "fr" {
        include_str!("../../SECURITY.fr.md")
    } else {
        include_str!("../../SECURITY.md")
    });
    context_update!(request => {
        "title"       => if lang == "fr" { "Politique de sécurité" } else { "Security policy" },
        "description" => if lang == "fr" {
            "Politique de sécurité de Runique : versions prises en charge, failles corrigées et signalement d'une vulnérabilité."
        } else {
            "Runique's security policy: supported versions, fixed vulnerabilities and how to report one."
        },
        "sections"    => &sections,
        "release_tab" => "security",
    });
    request.render("info/release_document.html")
}

pub async fn releases_compatibility(mut request: Request) -> AppResult<Response> {
    crate::backend::compatibility::compatibility_page(&mut request).await
}

pub async fn releases_status(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    let lang = visitor_lang(&request.headers);
    let content = if lang == "fr" {
        include_str!("../../docs/fr/PROJECT_STATUS.md")
    } else {
        include_str!("../../docs/en/PROJECT_STATUS.en.md")
    };
    let sections = split_markdown_sections(content);
    context_update!(request => {
        "title"       => if lang == "fr" { "État du projet" } else { "Project status" },
        "description" => if lang == "fr" {
            "Avancement des prochaines versions de Runique : makemigrations, base de données et admin."
        } else {
            "Progress of Runique's next versions: makemigrations, database and admin."
        },
        "sections"    => &sections,
        "release_tab" => "status",
    });
    request.render("info/release_document.html")
}

pub async fn probleme_connu(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    let sections = fetch_known_issues(&request.engine.db).await;
    context_update!(request => {
        "title"          => "Known Issues",
        "description"    => "Known issues of each Runique version, and the release that fixes them.",
        "sections"       => &sections,
        "ext_link_label" => "Full CHANGELOG",
        // The changelog is on the site (`/releases`), no need to leave it.
        "ext_link_url"   => "/releases",
    });
    request.render("info/cards.html")
}

pub async fn roadmap() -> Redirect {
    Redirect::permanent("/releases/limits#a-venir")
}

// ─── Sitemap ──────────────────────────────────────────────────────────────────

pub async fn google_verify(_: Request) -> Response {
    (
        StatusCode::OK,
        [("content-type", "text/html; charset=utf-8")],
        "google-site-verification: google59ae742b6eee40ef.html",
    )
        .into_response()
}

pub async fn sitemap_xml(_: Request) -> Response {
    (
        StatusCode::OK,
        [("content-type", "application/xml; charset=utf-8")],
        include_str!("../sitemap.xml"),
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
    "README not found.".to_string()
}

pub async fn readme_fr(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    let content = find_readme(&[
        "docs/fr/README.fr.md",
        "../docs/fr/README.fr.md",
        "/app/docs/fr/README.fr.md",
    ]);
    context_update!(request => {
        "title"   => "README — French",
        "content" => &content,
        "lang"    => "fr",
        "ui_lang" => "fr",
    });
    request.render("readme.html")
}

pub async fn readme_en(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    let content = find_readme(&[
        "docs/en/README.md",
        "../docs/en/README.md",
        "/app/docs/en/README.md",
    ]);
    context_update!(request => {
        "title"   => "README — English",
        "content" => &content,
        "lang"    => "en",
        "ui_lang" => "en",
    });
    request.render("readme.html")
}

// ─── Middleware ───────────────────────────────────────────────────────────────

pub async fn middleware_hub(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    context_update!(request => { "title" => "Middlewares", "description" => "Runique's security middlewares: CSRF, Content Security Policy, rate limiting, login guard, host validation and HTTPS." });
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

// ─── Courses ──────────────────────────────────────────────────────────────────

pub async fn view_cours_root(request: Request) -> AppResult<Response> {
    Ok(cours_root(&request))
}

pub async fn view_cours_index(
    Path(lang): Path<String>,
    mut request: Request,
) -> AppResult<Response> {
    cours_index(&lang, &mut request).await
}

pub async fn view_cours_detail(
    Path((lang, slug)): Path<(String, String)>,
    mut request: Request,
) -> AppResult<Response> {
    cours_detail(&lang, &slug, &mut request).await
}

// Sa route est en commentaire dans url.rs : à retirer avec sa réactivation.
#[allow(dead_code)]
pub async fn view_cours_exercice(
    Path(slug): Path<String>,
    mut request: Request,
) -> AppResult<Response> {
    let input = ExerciceInput {
        message: request
            .prisme
            .checked_data()
            .and_then(|d| d.get("message").cloned())
            .unwrap_or_default(),
    };
    cours_exercice(&slug, input, &mut request).await
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
    inject_globals(&mut request).await;
    context_update!(request => { "title" => "Administration", "description" => "Runique's admin panel: a typed ModelAdmin builder, account ranks and column-level permissions." });
    request.render("admin/hub.html")
}

pub async fn admin_declaration(mut request: Request) -> AppResult<Response> {
    demo_code_page("admin_declaration", &mut request).await
}

pub async fn admin_setup(mut request: Request) -> AppResult<Response> {
    demo_code_page("admin_setup", &mut request).await
}

pub async fn surcharge_exemple(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    context_update!(request => { "title" => "Example — override template", "description" => "A complete example of an overridden Runique admin page: the template, the blocks replaced and the result." });
    request.render("admin/surcharge_exemple.html")
}

pub async fn admin_surcharge(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    context_update!(request => { "title" => "Template overrides", "description" => "Overriding Runique's admin templates: replace a whole page or one block, variables checked at startup." });
    request.render("admin/surcharge.html")
}

// ─── Demo pages ───────────────────────────────────────────────────────────────

pub async fn installation_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("installation_demo", &mut request).await
}

pub async fn configuration_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("configuration_demo", &mut request).await
}

pub async fn migrations_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("migrations_demo", &mut request).await
}

pub async fn comparatif(mut request: Request) -> AppResult<Response> {
    comparatif_intro_page(&mut request).await
}

pub async fn limites(mut request: Request) -> AppResult<Response> {
    limites_page(&mut request).await
}

pub async fn comparatif_detail(
    Path(slug): Path<String>,
    mut request: Request,
) -> AppResult<Response> {
    comparatif_detail_page(&slug, &mut request).await
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

// ─── Errors / CSRF ────────────────────────────────────────────────────────────

pub async fn test_csrf(request: Request) -> AppResult<Response> {
    success!(request.notices => msg(&request.headers, "CSRF token validated successfully!", "Jeton CSRF validé !"));
    Ok(Redirect::to("/").into_response())
}

pub async fn propos_template_error(mut request: Request) -> AppResult<Response> {
    inject_globals(&mut request).await;
    context_update!(request => { "title" => "Template error test page" });
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

pub async fn security_txt() -> &'static str {
    "Contact: mailto:runique.framework.web@gmail.com\n\
        Expires: 2027-12-31T23:59:59.000Z\n\
        Preferred-Languages: fr, en\n\
        Canonical: https://runique.io/.well-known/security.txt\n"
}

pub async fn llms_txt() -> &'static str {
    include_str!("../llms.txt")
}

pub async fn llms_full_txt() -> &'static str {
    include_str!("../llms-full.txt")
}
