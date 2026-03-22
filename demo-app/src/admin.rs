use crate::entities::{
    blog, changelog_entry, chapitre, code_example, cour, cour_block, demo_category, demo_page,
    demo_section, doc_block, doc_page, doc_section, form_field, known_issue, page_doc_link,
    roadmap_entry, site_config, users,
};
use crate::formulaire::{
    BlogForm, ChangelogEntryForm, ChapitreForm, CodeExampleForm, CourBlockForm, CourForm,
    DemoCategoryForm, DemoPageForm, DemoSectionForm, DocBlockForm, DocPageForm, DocSectionForm,
    FormFieldForm, KnownIssueForm, PageDocLinkForm, RegisterForm, RoadmapEntryForm, SiteConfigForm,
};

admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"],
        edit_form: crate::formulaire::UserEditForm,
        list_display: [
            ["username", "Nom d'utilisateur"],
            ["email", "Email"],
            ["is_superuser", "Superuser"],
            ["is_active", "Actif"],
        ],
        list_filter: [
            ["is_superuser", "Superuser"],
            ["is_active", "Actif"],
        ]
    }
    blog: blog::Model => BlogForm {
        title: "Articles",
        permissions: ["admin"]
    }
    changelog_entry: changelog_entry::Model => ChangelogEntryForm {
        title: "Changelog",
        permissions: ["admin"]
    }
    roadmap_entry: roadmap_entry::Model => RoadmapEntryForm {
        title: "Roadmap",
        permissions: ["admin"]
    }
    known_issue: known_issue::Model => KnownIssueForm {
        title: "Problèmes connus",
        permissions: ["admin"]
    }
    demo_category: demo_category::Model => DemoCategoryForm {
        title: "Catégories",
        permissions: ["admin"]
    }
    demo_page: demo_page::Model => DemoPageForm {
        title: "Pages",
        permissions: ["admin"]
    }
    demo_section: demo_section::Model => DemoSectionForm {
        title: "Sections",
        permissions: ["admin"]
    }
    code_example: code_example::Model => CodeExampleForm {
        title: "Exemples de code",
        permissions: ["admin"]
    }
    page_doc_link: page_doc_link::Model => PageDocLinkForm {
        title: "Liens documentation",
        permissions: ["admin"]
    }
    form_field: form_field::Model => FormFieldForm {
        title: "Champs formulaire",
        permissions: ["admin"]
    }
    doc_section: doc_section::Model => DocSectionForm {
        title: "Doc — Sections",
        permissions: ["admin"]
    }
    doc_page: doc_page::Model => DocPageForm {
        title: "Doc — Pages",
        permissions: ["admin"],
        list_filter: [
            ["lang", "Langue"],
        ]
    }
    doc_block: doc_block::Model => DocBlockForm {
        title: "Doc — Blocs",
        permissions: ["admin"],
        list_filter: [
            ["page_id", "page", 10],
            ["block_type", "type", 5],
            ["heading", "En-tête", 1],
        ]
    }
    site_config: site_config::Model => SiteConfigForm {
        title: "Configuration site",
        permissions: ["admin"]
    }
    cour: cour::Model => CourForm {
        title: "Cours",
        permissions: ["admin"],
        list_display: [
            ["slug", "Slug"],
            ["theme", "Thème"],
            ["difficulte", "Difficulté"],
            ["ordre", "Ordre"],
        ],
        list_filter: [
            ["theme", "Thème"],
            ["difficulte", "Difficulté"],
        ]
    }
    chapitre: chapitre::Model => ChapitreForm {
        title: "Chapitres",
        permissions: ["admin"],
        list_filter: [
            ["cour_id", "Cours", 10],
        ]
    }
    cour_block: cour_block::Model => CourBlockForm {
        title: "Cours — Blocs",
        permissions: ["admin"],
        list_filter: [
            ["chapitre_id", "Chapitre", 10],
            ["block_type", "Type", 5],
        ]
    }
}
