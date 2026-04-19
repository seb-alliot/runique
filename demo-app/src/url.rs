use crate::views::{
    about, activate_account, admin_declaration, admin_hub, admin_setup, admin_surcharge,
    blog_detail, blog_list, blog_save, changelog, comparatif_demo, configuration_demo,
    contribution_list, contribution_submit, database_demo, deconnexion, docs_index_en,
    docs_index_fr, docs_page, docs_section, erreur_502, force_not_found, force_server_error,
    force_to_many_requests, formulaires_champs, formulaires_helpers, formulaires_hub,
    formulaires_templates, google_verify, i18n_demo, index, info_user, installation_demo,
    login_user, macros_demo, middleware_csp, middleware_csrf, middleware_hosts, middleware_https,
    middleware_hub, middleware_login_guard, middleware_rate_limit, migrations_demo, model_demo,
    orm_demo, probleme_connu, profil, propos_template_error, readme_en, readme_fr, rgpd, roadmap,
    router_demo, security_txt, session_demo, sitemap_xml, soumission_inscription,
    surcharge_exemple, template_demo, test_csrf, test_fields, upload_image_submit,
    view_cours_detail, view_cours_exercice, view_cours_index,
};

use runique::prelude::*;

pub fn routes() -> Router {
    urlpatterns! {
        "/"                              => view!{ index },                   name = "index",

        // Auth
        "/logout"                        => view! { deconnexion },            name = "logout",
        "/activate/{token}/{encrypted_email}" => view! { activate_account },  name = "activate_account",

        // Profil
        "/profil"                        => view! { profil },                 name = "profil",
        "/view-user"                     => view! { info_user },              name = "search-user",

        // Blog
        "/blog/liste"                    => view! { blog_list },              name = "blog_list",
        "/blog/nouveau"                  => view! { blog_save },              name = "blog_info",
        "/blog/{id}"                     => view! { blog_detail },            name = "blog_detail",

        // Tools & Demo
        "/about"                         => view! { about },                  name = "about",
        "/admin-demo"                    => view! { admin_hub },              name = "admin_hub",
        "/admin-demo/declaration"        => view! { admin_declaration },      name = "admin_declaration",
        "/admin-demo/setup"              => view! { admin_setup },            name = "admin_setup",
        "/admin-demo/surcharge"          => view! { admin_surcharge },        name = "admin_surcharge",
        "/admin-demo/surcharge/exemple"  => view! { surcharge_exemple },      name = "admin_surcharge_exemple",
        "/i18n"                          => view! { i18n_demo },              name = "i18n_demo",
        "/session"                       => view! { session_demo },           name = "session_demo",
        "/macros"                        => view! { macros_demo },            name = "macros_demo",
        "/database"                      => view! { database_demo },          name = "database_demo",
        "/installation"                  => view! { installation_demo },      name = "installation_demo",
        "/migrations"                    => view! { migrations_demo },        name = "migrations_demo",
        "/templates"                     => view! { template_demo },          name = "template_demo",
        "/configuration"                 => view! { configuration_demo },     name = "configuration_demo",
        "/orm"                           => view! { orm_demo },               name = "orm_demo",
        "/comparatif"                    => view! { comparatif_demo },        name = "comparatif_demo",
        "/routeur"                       => view! { router_demo },            name = "router_demo",
        "/modeles"                       => view! { model_demo },             name = "model_demo",
        "/roadmap"                       => view! { roadmap },                name = "roadmap",
        "/changelog"                     => view! { changelog },              name = "changelog",
        "/problemes-connus"              => view! { probleme_connu },         name = "problemes_connus",
        "/rgpd"                          => view! { rgpd },                   name = "rgpd",
        "/test-csrf"                     => view! { test_csrf },              name = "test_csrf",

        // Forms
        "/test-fields"                   => view! { test_fields },            name = "test_fields",
        "/formulaires"                   => view! { formulaires_hub },        name = "formulaires_hub",
        "/formulaires/champs"            => view! { formulaires_champs },     name = "formulaires_champs",
        "/formulaires/rendu"             => view! { formulaires_templates },  name = "formulaires_templates",
        "/formulaires/helpers"           => view! { formulaires_helpers },    name = "formulaires_helpers",
        "/formulaires/helpers/{id}"      => view! { formulaires_helpers },    name = "formulaires_helpers_detail",

        // Middlewares
        "/middleware"                    => view! { middleware_hub },         name = "middleware_hub",
        "/middleware/csrf"               => view! { middleware_csrf },        name = "middleware_csrf",
        "/middleware/csp"                => view! { middleware_csp },         name = "middleware_csp",
        "/middleware/rate-limiter"       => view! { middleware_rate_limit },  name = "middleware_rate_limit",
        "/middleware/login-guard"        => view! { middleware_login_guard }, name = "middleware_login_guard",
        "/middleware/host-validation"    => view! { middleware_hosts },       name = "middleware_hosts",
        "/middleware/https"              => view! { middleware_https },       name = "middleware_https",

        // Errors — check that Runique error pages display correctly
        "/erreurs/propose-error"         => view! { propos_template_error },  name = "propos_template_error",
        "/erreurs/404"                   =>   view! { force_not_found },      name = "force_404",
        "/erreurs/500"                   => view! { force_server_error },     name = "force_500",
        "/erreurs/429"                   => view! { force_to_many_requests }, name = "force_429",
        "/erreurs/502"                   => view!{ erreur_502 },              name = "test-502",

        // Sitemap & SEO
        "/sitemap.xml"                   => view! { sitemap_xml },            name = "sitemap_xml",
        "/google59ae742b6eee40ef.html"   => view! { google_verify },          name = "google_verify",
        "/.well-known/security.txt"      => view! { security_txt },           name = "security_txt",

        // Readme
        "/readme/fr"                     => view! { readme_fr },              name = "readme_fr",
        "/readme/en"                     => view! { readme_en },              name = "readme_en",

        // Rust Courses
        "/cours"                         => view! { view_cours_index },       name = "cours_index",
        "/cours/{slug}"                  => view! { view_cours_detail },      name = "cours_detail",
        "/cours/{slug}/exercice"         => view! { view_cours_exercice },    name = "cours_exercice",

        // Documentation
        "/docs/fr"                       => view! { docs_index_fr },          name = "doc_index_fr",
        "/docs/en"                       => view! { docs_index_en },          name = "doc_index_en",
        "/docs/{lang}/{section}"         => view! { docs_section },           name = "doc_section",
        "/docs/{lang}/{section}/{page}"  => view! { docs_page },              name = "doc_page",

        // Contributions
        "/contribution"                  => view! { contribution_submit },    name = "contribution",
        "/contributions"                 => view! { contribution_list },      name = "contribution_list",


    }
    .rate_limit("/upload-image",  "upload_image",  view!(upload_image_submit),   5,  60)
    .rate_limit("/inscription",   "inscription",   view!(soumission_inscription), 5, 300)
    .rate_limit("/login",         "login",         view!(login_user),             10,  60)
}
