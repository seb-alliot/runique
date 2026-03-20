use crate::entities::blog::Entity as BlogEntity;
use crate::entities::changelog_entry::Entity as ChangelogEntryEntity;
use crate::entities::contribution::Entity as ContributionEntity;
use crate::entities::known_issue::Entity as KnownIssueEntity;
use crate::entities::roadmap_entry::Entity as RoadmapEntryEntity;
use crate::entities::{code_example, demo_category, demo_page, form_field, page_doc_link};
use crate::formulaire::*;
use runique::middleware::auth::login as auth_login;
use runique::prelude::user::Entity as UserEntity;
use runique::prelude::*;

#[derive(serde::Serialize)]
struct FieldGroup {
    type_name: String,
    fields: Vec<form_field::Model>,
}

// ─── Helper partagé — fetch code_examples + doc_links par slug ───────────────
async fn fetch_page_examples(
    slug: &str,
    db: &sea_orm::DatabaseConnection,
) -> (Vec<code_example::Model>, Vec<page_doc_link::Model>) {
    let page = demo_page::Entity::find()
        .filter(demo_page::Column::Slug.eq(slug))
        .one(db)
        .await
        .unwrap_or(None);

    let Some(page) = page else {
        return (vec![], vec![]);
    };

    let examples = code_example::Entity::find()
        .filter(code_example::Column::PageId.eq(page.id))
        .order_by_asc(code_example::Column::SortOrder)
        .all(db)
        .await
        .unwrap_or_default();

    let links = page_doc_link::Entity::find()
        .filter(page_doc_link::Column::PageId.eq(page.id))
        .order_by_asc(page_doc_link::Column::SortOrder)
        .all(db)
        .await
        .unwrap_or_default();

    (examples, links)
}

// ─── Vue générique DB — page_type = "code" ────────────────────────────────────
async fn demo_code_page(slug: &str, request: &mut Request) -> AppResult<Response> {
    inject_auth(request).await;

    let db = request.engine.db.clone();

    let page = demo_page::Entity::find()
        .filter(demo_page::Column::Slug.eq(slug))
        .one(&*db)
        .await
        .unwrap_or(None);

    let Some(page) = page else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let code_examples = code_example::Entity::find()
        .filter(code_example::Column::PageId.eq(page.id))
        .order_by_asc(code_example::Column::SortOrder)
        .all(&*db)
        .await
        .unwrap_or_default();

    let doc_links = page_doc_link::Entity::find()
        .filter(page_doc_link::Column::PageId.eq(page.id))
        .order_by_asc(page_doc_link::Column::SortOrder)
        .all(&*db)
        .await
        .unwrap_or_default();

    let category = demo_category::Entity::find_by_id(page.category_id)
        .one(&*db)
        .await
        .unwrap_or(None);

    context_update!(request => {
        "title"         => &page.title,
        "page"          => &page,
        "code_examples" => &code_examples,
        "doc_links"     => &doc_links,
        "category"      => &category,
    });

    request.render("demo/generic.html")
}

// ─── Utilitaire : injecter l'état auth dans le contexte Tera ─────────────────
async fn inject_auth(request: &mut Request) {
    let connected = is_authenticated(&request.session).await;
    let username = get_username(&request.session).await;
    request.context.insert("connected", &connected);
    request.context.insert("current_user", &username);
}

// ───  Index  ─────────────────────────────────────────────────────────────────
pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Bienvenue sur Runique",
        "description" => "Un framework web inspiré de Django",
        "status" => "Status: Framework en cours de développement...",
        "backend" => "Rust , Axum",
        "template" => "Moteur de template: Tera",
        "tokio" => "Runtime asynchrone tokio",
        "session" => "Session: tower avec memory store, evolution prévue a l'avenir",
        "orm" => "ORM: sea-orm/sea-migration pour la gestion de la base de données",
        "migration" => "Migrations: système intégré via macro model!",
    });

    request.render("index.html")
}

// ─── Inscription ──────────────────────────────────────────────────────────────

pub async fn soumission_inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;

    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/profil").into_response());
    }
    let template = "auth/inscription.html";
    let db = request.engine.db.clone();
    let (code_examples, doc_links) = fetch_page_examples("inscription", &db).await;

    if request.is_get() {
        context_update!(request => {
            "title"         => "Inscription utilisateur",
            "inscription_form" => &form,
            "code_examples" => &code_examples,
            "doc_links"     => &doc_links,
        });
        return request.render(template);
    }

    if request.is_post() && form.is_valid().await {
        match form.save(&request.engine.db).await {
            Ok(user) => {
                auth_login(&request.session, user.id, &user.username)
                    .await
                    .ok();
                success!(request.notices => format!("Bienvenue {} ! Votre compte est créé.", user.username));
                return Ok(Redirect::to("/profil").into_response());
            }
            Err(err) => {
                form.get_form_mut().database_error(&err);
            }
        }
    }
    context_update!(request => {
        "title"         => "Erreur de validation",
        "inscription_form" => &form,
        "code_examples" => &code_examples,
        "doc_links"     => &doc_links,
        "messages"      => flash_now!(error => "Veuillez corriger les erreurs"),
    });
    request.render(template)
}

// ─── Connexion ────────────────────────────────────────────────────────────────

pub async fn login_user(
    mut request: Request,
    Prisme(form): Prisme<LoginForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;

    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/profile").into_response());
    }

    let template = "auth/login.html";
    let db = request.engine.db.clone();
    let (code_examples, doc_links) = fetch_page_examples("login", &db).await;

    if request.is_get() {
        context_update!(request => {
            "title"         => "Connexion",
            "login_form"    => &form,
            "code_examples" => &code_examples,
            "doc_links"     => &doc_links,
        });
        return request.render(template);
    }

    if request.is_post() {
        let username_val = form.get_form().get_value("username").unwrap_or_default();
        let password_val = form.get_form().get_value("password").unwrap_or_default();

        if !username_val.is_empty() && !password_val.is_empty() {
            let db = request.engine.db.clone();
            let user_opt = UserEntity::find()
                .filter(runique::prelude::user::Column::Username.eq(&username_val))
                .one(&*db)
                .await
                .unwrap_or(None);

            match user_opt {
                Some(user) if user.is_active && verify(&password_val, &user.password) => {
                    auth_login(&request.session, user.id, &user.username)
                        .await
                        .ok();
                    success!(request.notices => format!("Bienvenue {} !", user.username));
                    return Ok(Redirect::to("/profil").into_response());
                }
                _ => {}
            }
        }

        flash_now!(error => "Identifiants invalides");
        context_update!(request => {
            "title"         => "Connexion",
            "login_form"    => &form,
            "auth_error"    => &true,
            "code_examples" => &code_examples,
            "doc_links"     => &doc_links,
        });
        return request.render(template);
    }

    request.render(template)
}

// ─── Déconnexion ─────────────────────────────────────────────────────────────

pub async fn deconnexion(request: Request) -> AppResult<Response> {
    logout(&request.session).await.ok();
    Ok(Redirect::to("/").into_response())
}

// ─── Profil ───────────────────────────────────────────────────────────────────

pub async fn profil(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;

    if !is_authenticated(&request.session).await {
        warning!(request.notices => "Connectez-vous pour accéder à votre profil.");
        return Ok(Redirect::to("/login").into_response());
    }

    let user_id = get_user_id(&request.session).await;
    let username = get_username(&request.session).await;

    let user_opt = if let Some(id) = user_id {
        UserEntity::find_by_id(id)
            .one(&*request.engine.db)
            .await
            .unwrap_or(None)
    } else {
        None
    };

    context_update!(request => {
        "title" => "Mon profil",
        "username" => &username,
        "profile_user" => &user_opt,
        "connected" => &true,
    });

    request.render("profile/profile.html")
}

// ─── Recherche utilisateur ────────────────────────────────────────────────────

pub async fn info_user(
    mut request: Request,
    Prisme(mut form): Prisme<UsernameForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let template = "profile/view_user.html";

    if request.is_get() && form.is_valid().await {
        let db = request.engine.db.clone();

        let username_val = form.cleaned_string("username").unwrap_or_default();

        let user_opt = UserEntity::find()
            .filter(runique::prelude::user::Column::Username.eq(&username_val))
            .one(&*db)
            .await
            .unwrap_or(None);

        match user_opt {
            Some(user) => {
                context_update!(request => {
                    "title" => "Vue utilisateur",
                    "username" => &user.username,
                    "email" => &user.email,
                    "found_user" => &user,
                    "user" => &form,
                    "messages" => flash_now!(success => "Utilisateur trouvé !"),
                });
            }
            None => {
                context_update!(request => {
                    "title" => "Vue utilisateur",
                    "user" => &form,
                    "messages" => flash_now!(warning => "Utilisateur introuvable."),
                });
            }
        }

        request.render(template)
    } else {
        // Always return a response
        context_update!(request => {
            "title" => "Vue utilisateur",
            "user" => &form,
        });
        request.render(template)
    }
}

// ─── À propos ────────────────────────────────────────────────────────────────

pub async fn about(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    success!(request.notices => "Action réussie.");
    info!(request.notices => "Message d'information.");
    warning!(request.notices => "Attention requise.");
    error!(request.notices => "Une erreur est survenue.");

    context_update!(request => {
        "title" => "À propos du Framework Runique",
        "content" => "Runique est un framework web inspiré de Django, construit sur Axum et Tera.",
    });

    request.render("about/about.html")
}

// ─── CSRF test ────────────────────────────────────────────────────────────────

pub async fn test_csrf(request: Request) -> AppResult<Response> {
    success!(request.notices => "CSRF token validé avec succès !");
    Ok(Redirect::to("/").into_response())
}

// ─── Upload image ─────────────────────────────────────────────────────────────

pub async fn upload_image_submit(
    mut request: Request,
    Prisme(mut form): Prisme<ImageForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let template = "forms/upload_image.html";
    let db = request.engine.db.clone();
    let (code_examples, doc_links) = fetch_page_examples("upload_image", &db).await;

    if request.is_get() {
        context_update!(request => {
            "title"         => "Uploader un fichier",
            "image_form"    => &form,
            "code_examples" => &code_examples,
            "doc_links"     => &doc_links,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            success!(request.notices => "Fichier uploadé avec succès !");
        } else {
            let errors = form.get_form().errors();
            let msg = if errors.is_empty() {
                "Erreur de validation".to_string()
            } else {
                errors.values().cloned().collect::<Vec<_>>().join(" | ")
            };
            error!(request.notices => &msg);
        }
        return Ok(Redirect::to("/upload-image").into_response());
    }

    request.render(template)
}

// ─── Blog ─────────────────────────────────────────────────────────────────────

pub async fn blog_list(
    mut request: Request,
    Prisme(form): Prisme<SearchDemoForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;

    let search = form.cleaned_string("search");

    let mut query = BlogEntity::find().order_by_desc(crate::entities::blog::Column::Id);

    if let Some(ref term) = search
        && !term.is_empty()
    {
        use sea_orm::Condition;
        query = query.filter(
            Condition::any()
                .add(crate::entities::blog::Column::Title.contains(term.as_str()))
                .add(crate::entities::blog::Column::Summary.contains(term.as_str())),
        );
    }

    let articles = query.all(&*request.engine.db).await.unwrap_or_default();

    context_update!(request => {
        "title"   => "Blog — Articles",
        "articles" => &articles,
        "search"  => &search,
        "search_form" => &form,
    });

    request.render("blog/blog_list.html")
}

pub async fn blog_save(
    mut request: Request,
    Prisme(mut blog): Prisme<BlogForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let template = "blog/blog.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Créer un article de blog",
            "blog_form" => &blog,
        });
        return request.render(template);
    }

    if request.is_post() && blog.is_valid().await {
        match blog.save(&request.engine.db).await {
            Ok(_) => {
                success!(request.notices => "Article sauvegardé !");
                return Ok(Redirect::to("/blog/liste").into_response());
            }
            Err(err) => {
                blog.get_form_mut().database_error(&err);
                context_update!(request => {
                    "title" => "Erreur base de données",
                    "blog_form" => &blog,
                });
                return request.render(template);
            }
        }
    }

    context_update!(request => {
        "title" => "Erreur de validation",
        "blog_form" => &blog,
        "messages" => flash_now!(error => "Veuillez corriger les erreurs ci-dessous"),
    });
    request.render(template)
}

pub async fn blog_detail(Path(id): Path<i32>, mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let article = BlogEntity::find_by_id(id)
        .one(&*request.engine.db)
        .await
        .unwrap_or(None);

    match article {
        Some(a) => {
            context_update!(request => {
                "title" => &a.title,
                "article" => &a,
            });
            request.render("blog/blog_detail.html")
        }
        None => {
            warning!(request.notices => "Article introuvable.");
            Ok(Redirect::to("/blog/liste").into_response())
        }
    }
}

// ─── Test des champs ─────────────────────────────────────────────────────────

pub async fn test_fields(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;

    let db = request.engine.db.clone();
    let page = demo_page::Entity::find()
        .filter(demo_page::Column::Slug.eq("formulaires_champs"))
        .one(&*db)
        .await
        .unwrap_or(None);
    let form_fields = if let Some(ref p) = page {
        form_field::Entity::find()
            .filter(form_field::Column::PageId.eq(p.id))
            .order_by_asc(form_field::Column::SortOrder)
            .all(&*db)
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };

    let mut field_groups: Vec<FieldGroup> = vec![];
    for field in form_fields {
        if field_groups
            .last()
            .map(|g: &FieldGroup| g.type_name.as_str())
            != Some(&field.field_type)
        {
            field_groups.push(FieldGroup {
                type_name: field.field_type.clone(),
                fields: vec![],
            });
        }
        field_groups.last_mut().unwrap().fields.push(field);
    }

    context_update!(request => {
        "title"        => "Champs disponibles",
        "field_groups" => &field_groups,
    });
    request.render("forms/field_test.html")
}

pub async fn contribution_submit(
    mut request: Request,
    Prisme(mut form): Prisme<ContributionForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let template = "contribution/contribution_form.html";
    if !is_authenticated(&request.session).await {
        return Ok(Redirect::to("/login").into_response());
    }
    if request.is_get() {
        context_update!(request => {
            "title" => "Soumettre une contribution",
            "contribution_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() && form.is_valid().await {
        let user_id = get_user_id(&request.session).await.unwrap_or(0);
        match form.save(&request.engine.db, user_id).await {
            Ok(_) => {
                success!(request.notices => "Contribution sauvegardée !");
                return Ok(Redirect::to("/").into_response());
            }
            Err(err) => {
                form.get_form_mut().database_error(&err);
                context_update!(request => {
                    "title" => "Erreur base de données",
                    "contribution_form" => &form,
                });
                return request.render(template);
            }
        }
    }

    context_update!(request => {
        "title" => "Erreur de validation",
        "contribution_form" => &form,
        "messages" => flash_now!(error => "Veuillez corriger les erreurs ci-dessous"),
    });
    request.render(template)
}

// ─── Model demo ───────────────────────────────────────────────────────────────

pub async fn model_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("model_demo", &mut request).await
}

// ─── RGPD ─────────────────────────────────────────────────────────────────────

pub async fn rgpd(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => {
        "title" => "Politique de confidentialité",
    });
    request.render("rgpd/rgpd.html")
}

// ─── Roadmap / Changelog ──────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct CardEntry {
    subtitle: Option<String>,
    title: String,
    description: String,
    link_url: Option<String>,
    link_label: Option<String>,
    link_url_2: Option<String>,
    link_label_2: Option<String>,
}

#[derive(serde::Serialize)]
struct CardSection {
    heading: String,
    heading_class: String,
    entries: Vec<CardEntry>,
}

pub async fn changelog(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;

    let all = ChangelogEntryEntity::find()
        .order_by_desc(crate::entities::changelog_entry::Column::Version)
        .order_by_asc(crate::entities::changelog_entry::Column::SortOrder)
        .all(&*request.engine.db)
        .await
        .unwrap_or_default();

    let mut sections: Vec<CardSection> = Vec::new();
    for entry in all {
        let heading = format!("v{} — {}", entry.version, entry.release_date);
        if let Some(s) = sections.last_mut()
            && s.heading == heading
        {
            s.entries.push(CardEntry {
                subtitle: Some(entry.category.clone()),
                title: entry.title.clone(),
                description: entry.description.clone(),
                link_url: None,
                link_label: None,
                link_url_2: None,
                link_label_2: None,
            });
            continue;
        }
        sections.push(CardSection {
            heading,
            heading_class: "roadmap-active".into(),
            entries: vec![CardEntry {
                subtitle: Some(entry.category.clone()),
                title: entry.title.clone(),
                description: entry.description.clone(),
                link_url: None,
                link_label: None,
                link_url_2: None,
                link_label_2: None,
            }],
        });
    }

    context_update!(request => {
        "title"    => "Changelog",
        "sections" => &sections,
        "ext_link_label" => "CHANGELOG complet",
        "ext_link_url"   => "https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md",
    });
    request.render("info/cards.html")
}

pub async fn probleme_connu(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;

    let all = KnownIssueEntity::find()
        .order_by_desc(crate::entities::known_issue::Column::Version)
        .order_by_asc(crate::entities::known_issue::Column::SortOrder)
        .all(&*request.engine.db)
        .await
        .unwrap_or_default();

    let mut sections: Vec<CardSection> = Vec::new();
    for entry in all {
        let heading = format!("v{}", entry.version);
        if let Some(s) = sections.last_mut()
            && s.heading == heading
        {
            s.entries.push(CardEntry {
                subtitle: Some(entry.issue_type.clone()),
                title: entry.title.clone(),
                description: entry.description.clone(),
                link_url: None,
                link_label: None,
                link_url_2: None,
                link_label_2: None,
            });
            continue;
        }
        sections.push(CardSection {
            heading,
            heading_class: "roadmap-active".into(),
            entries: vec![CardEntry {
                subtitle: Some(entry.issue_type.clone()),
                title: entry.title.clone(),
                description: entry.description.clone(),
                link_url: None,
                link_label: None,
                link_url_2: None,
                link_label_2: None,
            }],
        });
    }

    context_update!(request => {
        "title"    => "Problèmes connus",
        "sections" => &sections,
        "ext_link_label" => "CHANGELOG complet",
        "ext_link_url"   => "https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md",
    });
    request.render("info/cards.html")
}

pub async fn roadmap(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;

    let all = RoadmapEntryEntity::find()
        .order_by_asc(crate::entities::roadmap_entry::Column::SortOrder)
        .all(&*request.engine.db)
        .await
        .unwrap_or_default();

    let status_sections = [
        ("active", "🔧 En cours", "roadmap-active"),
        ("planned", "📋 Prévu", "roadmap-planned"),
        ("future", "🔭 Futur", "roadmap-future"),
    ];

    let sections: Vec<CardSection> = status_sections
        .iter()
        .filter_map(|(status, heading, class)| {
            let entries: Vec<CardEntry> = all
                .iter()
                .filter(|e| e.status == *status)
                .map(|e| CardEntry {
                    subtitle: None,
                    title: e.title.clone(),
                    description: e.description.clone(),
                    link_url: e.link_url.clone(),
                    link_label: e.link_label.clone(),
                    link_url_2: e.link_url_2.clone(),
                    link_label_2: e.link_label_2.clone(),
                })
                .collect();
            if entries.is_empty() {
                return None;
            }
            Some(CardSection {
                heading: String::from(*heading),
                heading_class: String::from(*class),
                entries,
            })
        })
        .collect();

    context_update!(request => {
        "title"    => "Ce qui arrive",
        "sections" => &sections,
        "ext_link_label" => "Roadmap complète",
        "ext_link_url"   => "https://github.com/seb-alliot/runique/blob/main/ROADMAP.md",
    });
    request.render("info/cards.html")
}

pub async fn admin_hub(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => {
        "title" => "Administration",
    });
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
    context_update!(request => {
        "title" => "Exemple — template de surcharge",
    });
    request.render("admin/surcharge_exemple.html")
}

pub async fn admin_surcharge(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => {
        "title" => "Surcharge de templates",
    });
    request.render("admin/surcharge.html")
}

pub async fn installation_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("installation_demo", &mut request).await
}

pub async fn database_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("database_demo", &mut request).await
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

// ─── Middleware — hub & pages individuelles ───────────────────────────────────

pub async fn middleware_hub(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => {
        "title" => "Middlewares",
    });
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

// ─── Formulaires — hub & sous-pages ──────────────────────────────────────────

pub async fn formulaires_hub(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => {
        "title" => "Formulaires",
    });
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
    Prisme(form): Prisme<crate::formulaire::SearchDemoForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;

    let path_id = request.path_param("id").map(|s| s.to_string());
    let search_value = request.from_url("search").map(|s| s.to_string());
    let cleaned_search = form.cleaned_string("search");

    context_update!(request => {
        "title"          => "Helpers & accès URL",
        "path_id"        => &path_id,
        "search_value"   => &search_value,
        "cleaned_search" => &cleaned_search,
    });
    request.render("formulaires/helpers.html")
}

// ─── Nouvelles pages démo ─────────────────────────────────────────────────────

pub async fn template_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("template_demo", &mut request).await
}

pub async fn configuration_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("configuration_demo", &mut request).await
}

pub async fn orm_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("orm_demo", &mut request).await
}

pub async fn migrations_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("migrations_demo", &mut request).await
}

pub async fn comparatif_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("comparatif_demo", &mut request).await
}

// ─── Routeur — page démo ──────────────────────────────────────────────────────

pub async fn router_demo(mut request: Request) -> AppResult<Response> {
    demo_code_page("router_demo", &mut request).await
}

// ─── Routes de test d'erreurs ─────────────────────────────────────────────────
//
// Ces routes existent uniquement pour vérifier que les pages d'erreur
// de Runique s'affichent correctement. À ne pas exposer en production.

pub async fn propos_template_error(mut request: Request) -> AppResult<Response> {
    // Forcer le mode debug sur cette page pour tester l'affichage des erreurs de template en mode debug
    context_update!(request => {
        "title" => "Page de test d'erreur de template",
    });
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

// ─── Liste des contributions ──────────────────────────────────────────────────

pub async fn contribution_list(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let contributions = ContributionEntity::find()
        .order_by_desc(crate::entities::contribution::Column::Id)
        .all(&*request.engine.db)
        .await
        .unwrap_or_default();

    context_update!(request => {
        "title" => "Contributions",
        "contributions" => &contributions,
    });

    request.render("contribution/contribution_list.html")
}
