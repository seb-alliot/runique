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
            ["username", "Nom d'utilisateur", 10],
            ["email", "Email", 10],
            ["is_superuser", "Superuser", 10],
            ["is_active", "Actif", 10],
        ]
    }
    blog: blog::Model => BlogForm {
        title: "Articles",
        permissions: ["admin"],
        list_display: [
            ["title", "Titre"],
            ["email", "email"],
            ["website", "lien url"],
            ["summary", "Sujet"],
            ["content", "contenu"]
        ],
        list_filter : [
            ["title", "Titre", 10],
            ["email", "email", 10],
            ["website", "lien url", 10],
            ["summary", "Sujet", 10],
            ["content", "contenu", 10],
        ]
    }
    changelog_entry: changelog_entry::Model => ChangelogEntryForm {
        title: "Changelog",
        permissions: ["admin"],
        list_display: [
            ["version", "Version"],
            ["release_date", "Date de sortie"],
            ["category", "Catégorie"],
            ["title", "Titre"],
            ["description", "Description"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["version", "Version", 10],
            ["release_date", "Date", 10],
            ["category", "Catégorie", 10],
            ["title", "Titre", 10],
            ["description", "Description", 10],
            ["sort_order", "Ordre d'affichage", 10],

        ],
    }
    roadmap_entry: roadmap_entry::Model => RoadmapEntryForm {
        title: "Roadmap",
        permissions: ["admin"],
        list_display: [
            ["status", "Statut"],
            ["title", "Titre"],
            ["description", "Description"],
            ["link_url", "URL"],
            ["link_label", "Label"],
            ["link_url_2", "URL 2"],
            ["link_label_2", "Label 2"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["status", "Statut", 10],
            ["title", "Titre", 10],
            ["description", "Description", 10],
            ["link_url", "URL", 10],
            ["link_label", "Label", 10],
            ["link_url_2", "URL 2", 10],
            ["link_label_2", "Label 2", 10],
            ["sort_order", "Ordre d'affichage", 10],
        ]
    }
    known_issue: known_issue::Model => KnownIssueForm {
        title: "Problèmes connus",
        permissions: ["admin"],
        list_display: [
            ["version", "Version"],
            ["title", "Titre"],
            ["description", "Description"],
            ["issue_type", "Type"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["version", "Version", 10],
            ["title", "Titre", 10],
            ["description", "Description", 10],
            ["issue_type", "Type", 10],
            ["sort_order", "Ordre d'affichage", 10],
        ]
    }
    demo_category: demo_category::Model => DemoCategoryForm {
        title: "Catégories",
        permissions: ["admin"]
    }
    demo_page: demo_page::Model => DemoPageForm {
        title: "Pages",
        permissions: ["admin"],
        list_display: [
            ["category_id", "Catégorie"],
            ["slug", "Slug"],
            ["title", "Titre"],
            ["lead", "Lead"],
            ["page_type", "Type"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["category_id", "Catégorie", 10],
            ["slug", "Slug", 10],
            ["title", "Titre", 10],
            ["lead", "Lead", 10],
            ["page_type", "Type", 10],
            ["sort_order", "Ordre d'affichage", 10],

        ]
    }
    demo_section: demo_section::Model => DemoSectionForm {
        title: "Sections",
        permissions: ["admin"],
        list_display: [
            ["page_id", "Page"],
            ["title", "Titre"],
            ["content", "Contenu"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["page_id", "Page", 10],
            ["title", "Titre", 10],
            ["content", "Contenu", 10],
            ["sort_order", "Ordre d'affichage", 10],
        ]
    }
    code_example: code_example::Model => CodeExampleForm {
        title: "Exemples de code",
        permissions: ["admin"],
        list_display: [
            ["page_id", "Page"],
            ["title", "Titre"],
            ["language", "Langage"],
            ["code", "Code"],
            ["context", "Contexte"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["page_id", "Page", 10],
            ["title", "Titre", 10],
            ["language", "Langage", 10],
            ["code", "Code", 10],
            ["context", "Contexte", 10],
            ["sort_order", "Ordre d'affichage", 10],
        ]
    }
    page_doc_link: page_doc_link::Model => PageDocLinkForm {
        title: "Liens documentation",
        permissions: ["admin"],
        list_display: [
            ["page_id", "Page"],
            ["label", "Label"],
            ["url", "URL"],
            ["link_type", "Type"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["page_id", "Page", 10],
            ["label", "Label", 10],
            ["url", "URL", 10],
            ["link_type", "Type", 10],
            ["sort_order", "Ordre d'affichage", 10],
        ]
    }
    form_field: form_field::Model => FormFieldForm {
        title: "Champs formulaire",
        permissions: ["admin"],
        list_display: [
            ["page_id", "Page"],
            ["name", "Nom"],
            ["field_type", "Type"],
            ["description", "Description"],
            ["example", "Exemple"],
            ["html_preview", "Aperçu HTML"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["page_id", "Page", 10],
            ["name", "Nom", 10],
            ["field_type", "Type", 10],
            ["description", "Description", 10],
            ["example", "Exemple", 10],
            ["html_preview", "Aperçu HTML", 10],
            ["sort_order", "Ordre d'affichage", 10],
        ]
    }
    doc_section: doc_section::Model => DocSectionForm {
        title: "Doc — Sections",
        permissions: ["admin"],
        list_display: [
            ["slug", "Slug"],
            ["lang", "Langue"],
            ["title", "Titre"],
            ["theme", "Thème"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["lang", "Langue", 10],
            ["theme", "Thème", 10],
        ]
    }
    doc_page: doc_page::Model => DocPageForm {
        title: "Doc — Pages",
        permissions: ["admin"],
        list_display: [
            ["section_id", "Section"],
            ["slug", "Slug"],
            ["lang", "Langue"],
            ["title", "Titre"],
            ["lead", "Lead"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["section_id", "Section", 10],
            ["slug", "Slug", 10],
            ["lang", "Langue", 10],
            ["title", "Titre", 10],
            ["lead", "Lead", 10],
            ["sort_order", "Ordre d'affichage", 10],
        ]
    }
    doc_block: doc_block::Model => DocBlockForm {
        title: "Doc — Blocs",
        permissions: ["admin"],
        list_display: [
            ["page_id", "Page"],
            ["content", "Contenu"],
            ["block_type", "Type"],
            ["heading", "En-tête"],
            ["sort_order", "Ordre"],
        ],
        list_filter: [
            ["page_id", "page", 10],
            ["heading", "En-tête", 10],
            ["content", "Contenu", 10],
            ["block_type", "type", 10],
            ["sort_order", "Ordre", 10],
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
            ["lang", "Langue"],
            ["titre", "Titre"],
            ["theme", "Thème"],
            ["difficulte", "Difficulté"],
            ["ordre", "Ordre"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["slug", "Slug", 10],
            ["lang", "Langue", 10],
            ["titre", "Titre", 10],
            ["theme", "Thème", 10],
            ["difficulte", "Difficulté", 10],
            ["ordre", "Ordre", 10],
            ["sort_order", "Ordre d'affichage", 10],
        ]
    }
    chapitre: chapitre::Model => ChapitreForm {
        title: "Chapitres",
        permissions: ["admin"],
        list_display: [
            ["cour_id", "Cours"],
            ["slug", "Slug"],
            ["titre", "Titre"],
            ["ordre", "Ordre"],
        ],
        list_filter: [
            ["cour_id", "Cours", 10],
            ["slug", "Slug", 10],
            ["titre", "Titre", 10],
            ["sort_order", "Ordre", 10],
        ]
    }
    cour_block: cour_block::Model => CourBlockForm {
        title: "Cours — Blocs",
        permissions: ["admin"],
        list_display: [
            ["chapitre_id", "Chapitre"],
            ["block_type", "Type"],
            ["heading", "En-tête"],
            ["sort_order", "Ordre"],
        ],
        list_filter: [
            ["chapitre_id", "Chapitre", 10],
            ["block_type", "Type", 10],
            ["heading", "En-tête", 10],
            ["sort_order", "Ordre", 10],
        ]
    }
}
