use crate::entities::{
    blog, changelog_entry, chapitre, code_example, contribution, cour, cour_block, demo_category,
    demo_page, demo_section, doc_block, doc_page, doc_section, form_field, known_issue,
    page_doc_link, roadmap_entry, runique_release, site_config,
};
use crate::formulaire::{
    BlogForm, ChangelogEntryForm, ChapitreForm, CodeExampleForm, ContributionForm, CourBlockForm,
    CourForm, DemoCategoryForm, DemoPageForm, DemoSectionForm, DocBlockForm, DocPageForm,
    DocSectionForm, FormFieldForm, KnownIssueForm, PageDocLinkForm, RoadmapEntryForm,
    RuniqueReleaseForm, SiteConfigForm,
};

admin! {
    configure {
        users: {
            group_action: [["is_active", "Actif"], ["is_staff", "Staff"]]
        }
    }
    contribution: contribution::Model => ContributionForm {
        title: "Contribution",
        list_display: [
            ["user_id", "contributeur"],
            ["contribution_type", "type"],
            ["title", "titre"],
            ["content", "contenu"],
        ],
        list_filter: [
            ["user_id", "contributeur", 5],
            ["contribution_type", "type", 5],
            ["title", "titre", 5],
            ["content", "contenu", 5],
        ]
    }
    blog: blog::Model => BlogForm {
        title: "Articles",
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

    }
    demo_page: demo_page::Model => DemoPageForm {
        title: "Pages",

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

    }
    cour: cour::Model => CourForm {
        title: "Cours",
        list_display: [
            ["slug", "Slug"],
            ["lang", "Langue"],
            ["title", "Titre"],
            ["theme", "Thème"],
            ["difficulte", "Difficulté"],
            ["ordre", "Ordre"],
            ["sort_order", "Ordre d'affichage"],
        ],
        list_filter: [
            ["slug", "Slug", 10],
            ["lang", "Langue", 10],
            ["title", "Titre", 10],
            ["theme", "Thème", 10],
            ["difficulte", "Difficulté", 10],
            ["ordre", "Ordre", 10],
            ["sort_order", "Ordre d'affichage", 10],
        ]
    }
    chapitre: chapitre::Model => ChapitreForm {
        title: "Chapitres",
        list_display: [
            ["cour_id", "Cours"],
            ["slug", "Slug"],
            ["title", "Titre"],
            ["sort_order", "Ordre"],
        ],
        list_filter: [
            ["cour_id", "Cours", 10],
            ["slug", "Slug", 10],
            ["title", "Titre", 10],
            ["sort_order", "Ordre", 10],
        ]
    }
    cour_block: cour_block::Model => CourBlockForm {
        title: "Cours — Blocs",
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
            ["sort_order", "Ordre d'affichage", 10],
        ]
    }
    runique_release: runique_release::Model => RuniqueReleaseForm {
        title: "Releases Runique",
        list_display: [
            ["version", "Version"],
            ["github_url", "GitHub"],
            ["crates_url", "Crates.io"],
        ],
        list_filter: [
            ["version", "Version", 10],
        ]
    }
}
