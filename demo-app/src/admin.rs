//! Admin declared with the typed `ModelAdmin` builder, served under
//! `{admin}/{resource}/…`. Every `model!{}` of the project is registered:
//! the rights of a model left out would be pruned at boot.
//!
//! One function per model that declares its columns: its `use …::Column::*`
//! then names them alone, and two models' columns of the same name
//! (`Email`, `Title`…) never meet.
use crate::entities::{
    blog, changelog_entry, chapitre, code_example, contribution, demo_category, demo_page,
    demo_section, doc_block, doc_page, doc_section, known_issue, page_doc_link, roadmap_entry,
    runique_release, site_config, user_profile,
};
use runique::admin::{AdminSite, ModelAdmin};
use runique::app::staging::AdminStaging;
use runique::auth::LoginGuard;
use runique::middleware::rate_limit::RateLimiter;
use sea_orm::Order;

/// The whole admin, handed to `with_admin` by `main`: the boot test below
/// checks this very configuration, not a copy.
pub fn config(admin: AdminStaging) -> AdminStaging {
    admin
        .site_title("Administration")
        .sitemap("https://runique.io/sitemap.xml")
        .path("/admin-runique/")
        .prefix("/prefix-test")
        .with_rate_limiter(RateLimiter::new().max_requests(20).retry_after(3600))
        .with_login_guard(LoginGuard::new().max_attempts(20).lockout_secs(300))
        .page_size(15)
        .site(site)
}

/// The accounts, with the profile columns `user_profile` adds to them.
fn accounts() -> ModelAdmin<user_profile::Entity> {
    use user_profile::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Username, Email, IsActive, Website])
                .sortable([Username])
                .search([Username, Email])
        })
        .form(|f| f.fields([Username, Email, Bio, Website, Phone, BirthDate]))
}

fn blog() -> ModelAdmin<blog::Entity> {
    use blog::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Title, Email, Website, Status])
                .links([Title])
                .sortable([Title, Status])
                .ordering([(Title, Order::Asc)])
                .search([Title, Summary])
                .filter([Status])
                .empty_value("—")
                .action(Status, "Published", "Publier")
                .action(Status, "Archived", "Archiver")
        })
        // `view_count` has its default (0): read-only, a new article starts there.
        .form(|f| {
            f.fields([Title, Email, Website, Summary, Content, Status, ViewCount])
                .readonly([ViewCount])
        })
}

fn changelog_entry() -> ModelAdmin<changelog_entry::Entity> {
    use changelog_entry::Column::*;
    ModelAdmin::new()
        .title("Changelog")
        .list(|l| {
            l.display([Version, ReleaseDate, Category, Title])
                .links([Title])
                .sortable([Version, ReleaseDate])
                .ordering([(SortOrder, Order::Asc)])
                .search([Title, Version])
                .search_help("Titre ou numéro de version")
                .filter([Category, Version])
                .per_page(15)
        })
        .form(|f| {
            f.fields([
                Version,
                ReleaseDate,
                Category,
                Title,
                Description,
                SortOrder,
            ])
            .labels([
                (ReleaseDate, "Date de sortie"),
                (Category, "Catégorie"),
                (SortOrder, "Ordre d'affichage"),
            ])
        })
}

fn known_issue() -> ModelAdmin<known_issue::Entity> {
    use known_issue::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Version, Title, IssueType])
                .sortable([Version])
                .search([Title])
                .filter([IssueType, Version])
                .action(IssueType, "Fix", "Marquer corrigé")
        })
        .form(|f| f.fields([Version, Title, Description, IssueType, SortOrder]))
}

fn contribution() -> ModelAdmin<contribution::Entity> {
    use contribution::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Title, ContributionType, CreatedAt])
                .sortable([CreatedAt])
                .ordering([(CreatedAt, Order::Desc)])
                .search([Title])
                .filter([ContributionType])
        })
        .form(|f| {
            f.fields([ContributionType, Title, Content])
                .create_only([ContributionType])
        })
}

fn chapitre() -> ModelAdmin<chapitre::Entity> {
    use chapitre::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Title, Slug, CourId, SortOrder])
                .links([Title])
                .sortable([Title, SortOrder])
                .ordering([(CourId, Order::Asc), (SortOrder, Order::Asc)])
                .search([Title, Slug])
                .filter([CourId])
        })
        .form(|f| {
            f.fields([CourId, Slug, Title, Lead, SortOrder])
                .labels([(CourId, "Cours"), (Lead, "Chapeau")])
        })
}

fn code_example() -> ModelAdmin<code_example::Entity> {
    use code_example::Column::*;
    ModelAdmin::new().list(|l| {
        l.display([Title, Language, PageId, SortOrder])
            .ordering([(PageId, Order::Asc), (SortOrder, Order::Asc)])
            .search([Title, Code])
            .filter([Language, PageId])
            .per_page(20)
    })
}

fn demo_category() -> ModelAdmin<demo_category::Entity> {
    use demo_category::Column::*;
    ModelAdmin::new().list(|l| {
        l.display([Title, BackLinkLabel, BackLinkUrl, SortOrder])
            .links([Title])
            .ordering([(SortOrder, Order::Asc)])
            .search([Title])
            .empty_value("—")
    })
}

fn demo_page() -> ModelAdmin<demo_page::Entity> {
    use demo_page::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Title, Slug, PageType, CategoryId])
                .links([Title])
                .sortable([Title, Slug])
                .ordering([(CategoryId, Order::Asc), (SortOrder, Order::Asc)])
                .search([Title, Slug, Lead])
                .filter([PageType, CategoryId])
        })
        .form(|f| {
            f.fields([CategoryId, Slug, Title, Lead, PageType, SortOrder])
                .create_only([Slug])
        })
}

fn demo_section() -> ModelAdmin<demo_section::Entity> {
    use demo_section::Column::*;
    ModelAdmin::new().list(|l| {
        l.display([Title, PageId, SortOrder])
            .ordering([(PageId, Order::Asc), (SortOrder, Order::Asc)])
            .search([Title, Content])
            .filter([PageId])
    })
}

fn doc_block() -> ModelAdmin<doc_block::Entity> {
    use doc_block::Column::*;
    ModelAdmin::new().list(|l| {
        l.display([Heading, BlockType, PageId, SortOrder])
            .ordering([(PageId, Order::Asc), (SortOrder, Order::Asc)])
            .search([Heading, Content])
            .filter([BlockType, PageId])
            .empty_value("(sans titre)")
            .action(BlockType, "Text", "Passer en texte")
            .per_page(15)
    })
}

fn doc_page() -> ModelAdmin<doc_page::Entity> {
    use doc_page::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Title, Slug, Lang, SectionId])
                .links([Title])
                .sortable([Title, Lang])
                .ordering([(SectionId, Order::Asc), (SortOrder, Order::Asc)])
                .search([Title, Slug, Lead])
                .search_help("Titre, slug ou chapeau")
                .filter([Lang, SectionId])
        })
        .form(|f| {
            f.fields([SectionId, Slug, Lang, Title, Lead, SortOrder])
                .labels([(SectionId, "Section"), (Lead, "Chapeau")])
        })
}

fn doc_section() -> ModelAdmin<doc_section::Entity> {
    use doc_section::Column::*;
    // `theme` is a nullable enum: its filter offers the values set, its
    // action sets one.
    ModelAdmin::new().list(|l| {
        l.display([Title, Theme, Lang, SortOrder])
            .ordering([(SortOrder, Order::Asc)])
            .search([Title, Slug])
            .filter([Theme, Lang])
            .empty_value("—")
            .action(Theme, "Autres", "Classer dans Autres")
    })
}

fn page_doc_link() -> ModelAdmin<page_doc_link::Entity> {
    use page_doc_link::Column::*;
    ModelAdmin::new().list(|l| {
        l.display([Label, LinkType, Url, PageId])
            .links([Label])
            .ordering([(PageId, Order::Asc), (SortOrder, Order::Asc)])
            .search([Label, Url])
            .filter([LinkType])
            .per_page(10)
    })
}

fn roadmap_entry() -> ModelAdmin<roadmap_entry::Entity> {
    use roadmap_entry::Column::*;
    ModelAdmin::new().title("Roadmap").list(|l| {
        l.display([Title, Status, SortOrder])
            .links([Title])
            .ordering([(SortOrder, Order::Asc)])
            .search([Title])
            .filter([Status])
            .action(Status, "Active", "Activer")
            .action(Status, "Planned", "Planifier")
            .action(Status, "Future", "Reporter")
    })
}

fn runique_release() -> ModelAdmin<runique_release::Entity> {
    use runique_release::Column::*;
    ModelAdmin::new().title("Releases").list(|l| {
        l.display([Version, GithubUrl, CratesUrl, ChangelogFrUrl])
            .links([Version])
            .sortable([Version])
            .ordering([(Version, Order::Desc)])
            .empty_value("—")
    })
}

fn site_config() -> ModelAdmin<site_config::Entity> {
    use site_config::Column::*;
    // `is_public` is a nullable boolean with a default.
    ModelAdmin::new()
        .list(|l| {
            l.display([Key, Value, IsPublic])
                .links([Key])
                .sortable([Key])
                .ordering([(SortOrder, Order::Asc)])
                .search([Key, Description])
                .filter([IsPublic])
                .action(IsPublic, true, "Rendre public")
                .action(IsPublic, false, "Rendre privé")
        })
        .form(|f| {
            f.fields([Key, Value, Description, IsPublic, SortOrder])
                .create_only([Key])
        })
}

pub fn site(site: AdminSite) -> AdminSite {
    site.accounts_users(accounts())
        .register(blog())
        .register(changelog_entry())
        .register(chapitre())
        .register(code_example())
        .register(contribution())
        .register(demo_category())
        .register(demo_page())
        .register(demo_section())
        .register(doc_block())
        .register(doc_page())
        .register(doc_section())
        .register(known_issue())
        .register(page_doc_link())
        .register(roadmap_entry())
        .register(runique_release())
        .register(site_config())
}

#[cfg(test)]
mod tests {
    use runique::app::staging::AdminStaging;

    /// The admin's configuration and every registered model pass the checks
    /// the app runs at boot.
    #[test]
    fn every_registered_model_passes_the_boot_checks() {
        let staging = super::config(AdminStaging::new().enable());
        if let Err(e) = staging.validate() {
            panic!("{e}");
        }
    }
}
