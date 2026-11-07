use crate::entities::known_issue::Entity as KnownIssueEntity;
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
pub struct MarkdownSection {
    pub key: String,
    pub heading: String,
    pub content: String,
}

/// Splits a Markdown file on its `## ` headings. What precedes the first one (title,
/// links to the other language) belongs to the file, not to the page.
pub fn split_markdown_sections(markdown: &str) -> Vec<MarkdownSection> {
    let mut sections: Vec<MarkdownSection> = Vec::new();
    for line in markdown.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            let heading = heading.replace(['[', ']'], "");
            sections.push(MarkdownSection {
                key: heading.split(" - ").next().unwrap_or_default().to_string(),
                heading,
                content: String::new(),
            });
        } else if let Some(current) = sections.last_mut() {
            current.content.push_str(line);
            current.content.push('\n');
        }
    }
    sections
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_section_keeps_its_own_content_in_file_order() {
        let sections = split_markdown_sections(
            "# Title\n\nIntro\n\n## [4.0.0] - unreleased\n\n### Fixed\n\n- a\n\n## [3.0.3] - 2026-10-10\n\n- b\n",
        );
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].key, "4.0.0");
        assert_eq!(sections[0].heading, "4.0.0 - unreleased");
        assert_eq!(sections[0].content, "\n### Fixed\n\n- a\n\n");
        assert_eq!(sections[1].key, "3.0.3");
        assert_eq!(sections[1].heading, "3.0.3 - 2026-10-10");
        assert_eq!(sections[1].content, "\n- b\n");
    }

    #[test]
    fn a_text_without_section_gives_nothing() {
        assert!(split_markdown_sections("# Title\n\n### Fixed\n\nNothing.").is_empty());
    }

    #[test]
    fn a_heading_without_date_is_its_own_key() {
        let sections = split_markdown_sections("## En cours\n\n- a\n");
        assert_eq!(sections[0].key, "En cours");
        assert_eq!(sections[0].heading, "En cours");
    }
}
