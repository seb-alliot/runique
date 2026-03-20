use crate::entities::{blog, changelog_entry, known_issue, roadmap_entry, users};
use crate::formulaire::{
    BlogForm, ChangelogEntryForm, KnownIssueForm, RegisterForm, RoadmapEntryForm,
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
}
