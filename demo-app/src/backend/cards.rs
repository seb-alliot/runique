use crate::entities::changelog_entry::Entity as ChangelogEntryEntity;
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

pub async fn fetch_changelog_paged(
    db: &sea_orm::DatabaseConnection,
    page: usize,
) -> (Vec<CardSection>, usize, usize) {
    let all = fetch_changelog(db).await;
    let total = all.len();
    let page = page.max(1).min(total.max(1));
    let sections = all.into_iter().skip(page - 1).take(1).collect();
    (sections, page, total)
}

pub async fn fetch_changelog(db: &sea_orm::DatabaseConnection) -> Vec<CardSection> {
    // `desc Id` orders releases newest-first (id tracks insertion chronology — the
    // version string can't be sorted reliably: "2.1.9" would land above "2.1.18").
    // We then group by version in Rust so that `sort_order` controls the order WITHIN
    // each release. A single SQL `ORDER BY id DESC, sort_order ASC` cannot do this:
    // id is unique, so it fully dominates and sort_order never breaks a tie.
    let all = search!(ChangelogEntryEntity => desc Id)
        .all(db)
        .await
        .unwrap_or_default();

    let mut order: Vec<String> = Vec::new();
    let mut groups: std::collections::HashMap<String, (String, Vec<(i32, CardEntry)>)> =
        std::collections::HashMap::new();

    for entry in all {
        let group = groups.entry(entry.version.clone()).or_insert_with(|| {
            // First time we see this version (highest id, i.e. newest) → keep its rank.
            order.push(entry.version.clone());
            (entry.release_date.clone(), Vec::new())
        });
        group.1.push((
            entry.sort_order,
            CardEntry {
                subtitle: Some(entry.category.to_string()),
                title: entry.title.clone(),
                description: entry.description.clone(),
                link_url: None,
                link_label: None,
                link_url_2: None,
                link_label_2: None,
            },
        ));
    }

    order
        .into_iter()
        .map(|version| {
            let (release_date, mut items) = groups.remove(&version).unwrap_or_default();
            items.sort_by_key(|(sort_order, _)| *sort_order);
            CardSection {
                heading: format!("v{} — {}", version, release_date),
                heading_class: "roadmap-active".into(),
                entries: items.into_iter().map(|(_, entry)| entry).collect(),
            }
        })
        .collect()
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

pub async fn fetch_roadmap(db: &sea_orm::DatabaseConnection) -> Vec<CardSection> {
    let all = search!(RoadmapEntryEntity => asc SortOrder)
        .all(db)
        .await
        .unwrap_or_default();

    let status_sections = [
        (RoadmapStatus::Active, "🔧 In progress", "roadmap-active"),
        (RoadmapStatus::Planned, "📋 Planned", "roadmap-planned"),
        (RoadmapStatus::Future, "🔭 Future", "roadmap-future"),
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
