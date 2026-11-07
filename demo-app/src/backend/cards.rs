use crate::entities::known_issue::Entity as KnownIssueEntity;
use crate::entities::roadmap_entry::{Entity as RoadmapEntryEntity, RoadmapStatus};
use runique::prelude::*;

#[derive(serde::Serialize)]
pub struct CardEntry {
    pub subtitle: Option<String>,
    pub title: String,
    pub description: String,
    pub link_url: Option<String>,
    pub link_label: Option<String>,
    pub link_url_2: Option<String>,
    pub link_label_2: Option<String>,
}

#[derive(serde::Serialize)]
pub struct CardSection {
    pub heading: String,
    pub heading_class: String,
    pub entries: Vec<CardEntry>,
}

#[derive(serde::Serialize)]
pub struct ChangelogVersion {
    pub version: String,
    pub heading: String,
    pub content: String,
}

/// Splits a Keep a Changelog file on its `## ` headings. What precedes the first
/// one (title, links to the other language) belongs to the file, not to the page.
pub fn split_changelog(markdown: &str) -> Vec<ChangelogVersion> {
    let mut versions: Vec<ChangelogVersion> = Vec::new();
    for line in markdown.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            let heading = heading.replace(['[', ']'], "");
            versions.push(ChangelogVersion {
                version: heading.split(" - ").next().unwrap_or_default().to_string(),
                heading,
                content: String::new(),
            });
        } else if let Some(current) = versions.last_mut() {
            current.content.push_str(line);
            current.content.push('\n');
        }
    }
    versions
}

pub async fn fetch_known_issues(db: &ADb) -> Vec<CardSection> {
    let all = search!(KnownIssueEntity => desc Version, asc SortOrder)
        .all(db)
        .await
        .unwrap_or_default();

    let mut sections: Vec<CardSection> = Vec::new();
    for entry in all {
        let heading = format!("v{}", entry.version);
        if let Some(s) = sections.last_mut()
            && s.heading == heading
        {
            s.entries.push(CardEntry {
                subtitle: Some(entry.issue_type.to_string()),
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
                subtitle: Some(entry.issue_type.to_string()),
                title: entry.title.clone(),
                description: entry.description.clone(),
                link_url: None,
                link_label: None,
                link_url_2: None,
                link_label_2: None,
            }],
        });
    }
    sections
}

pub async fn fetch_roadmap(db: &ADb, lang: &str) -> Vec<CardSection> {
    let all = search!(RoadmapEntryEntity => Lang eq lang, asc SortOrder)
        .all(db)
        .await
        .unwrap_or_default();

    let status_sections = [
        (RoadmapStatus::Active, "In progress", "roadmap-active"),
        (RoadmapStatus::Planned, "Planned", "roadmap-planned"),
        (RoadmapStatus::Future, "Later", "roadmap-future"),
    ];

    status_sections
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
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_version_keeps_its_own_content_in_file_order() {
        let versions = split_changelog(
            "# Title\n\nIntro\n\n## [4.0.0] - unreleased\n\n### Fixed\n\n- a\n\n## [3.0.3] - 2026-10-10\n\n- b\n",
        );
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, "4.0.0");
        assert_eq!(versions[0].heading, "4.0.0 - unreleased");
        assert_eq!(versions[0].content, "\n### Fixed\n\n- a\n\n");
        assert_eq!(versions[1].version, "3.0.3");
        assert_eq!(versions[1].heading, "3.0.3 - 2026-10-10");
        assert_eq!(versions[1].content, "\n- b\n");
    }

    #[test]
    fn a_text_without_version_gives_nothing() {
        assert!(split_changelog("# Title\n\n### Fixed\n\nREADME not found.").is_empty());
    }
}
