use crate::entities::{
    blog, changelog_entry, code_example, demo_category, demo_page, demo_section, form_field,
    known_issue, page_doc_link, roadmap_entry, users,
};
use crate::formulaire::{
    BlogForm, ChangelogEntryForm, CodeExampleForm, DemoCategoryForm, DemoPageForm, DemoSectionForm,
    FormFieldForm, KnownIssueForm, PageDocLinkForm, RegisterForm, RoadmapEntryForm,
};

admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"]
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
}
