use crate::entities::changelog_entry::Entity as ChangelogEntryEntity;
use crate::entities::known_issue::Entity as KnownIssueEntity;
use crate::entities::roadmap_entry::Entity as RoadmapEntryEntity;
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

pub async fn fetch_changelog(db: &sea_orm::DatabaseConnection) -> Vec<CardSection> {
    let all = search!(ChangelogEntryEntity => desc Version, asc SortOrder)
        .all(db)
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
    sections
}

pub async fn fetch_known_issues(db: &sea_orm::DatabaseConnection) -> Vec<CardSection> {
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
    sections
}

pub async fn fetch_roadmap(db: &sea_orm::DatabaseConnection) -> Vec<CardSection> {
    let all = search!(RoadmapEntryEntity => asc SortOrder)
        .all(db)
        .await
        .unwrap_or_default();

    let status_sections = [
        ("active", "🔧 In progress", "roadmap-active"),
        ("planned", "📋 Planned", "roadmap-planned"),
        ("future", "🔭 Future", "roadmap-future"),
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
