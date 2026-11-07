//! What each `{% dev_block "…" %}` of Runique's templates becomes when the
//! developer replaced that block. Tera names an included template once, when
//! it loads it: the preprocessor writes the developer's path in, behind a test
//! on the page's model, Runique's template as the fallback. Tera then checks
//! every include, a missing template refusing the boot.
use super::block::{Block, GlobalBlock};
use super::page::{AdminTemplate, ModelTemplates};
use super::stylesheet::Page;
use crate::app::templates::BlockDispatch;

/// `text` as a Tera string literal's content, refused when it could close
/// the literal or open a tag: a key or a path is written into the template.
fn literal(text: &str) -> Result<&str, String> {
    let unsafe_char = |c: char| matches!(c, '"' | '\\' | '{' | '}' | '%' | '#') || c.is_control();
    if text.is_empty() || text.chars().any(unsafe_char) {
        return Err(format!(
            "admin template override `{text}` holds a character a template can't take"
        ));
    }
    Ok(text)
}

/// A developer's template, refused unless named `.html` or `.xml`: Tera
/// escapes by the template's extension, another one would render unescaped.
fn template(path: &str) -> Result<&str, String> {
    if !(path.ends_with(".html") || path.ends_with(".xml")) {
        return Err(format!(
            "admin template override `{path}` must be a `.html` or `.xml` template, escaped by Tera"
        ));
    }
    literal(path)
}

fn include(path: &str) -> Result<String, String> {
    Ok(format!(r#"{{% include "{}" %}}"#, template(path)?))
}

/// What the block named `name` becomes, when replaced: the models' own
/// templates, each behind a test on the page's model, then the global one,
/// else Runique's template `runique`.
fn dispatch_of(
    name: &str,
    runique: &str,
    global: Option<&str>,
    own: &[(&str, &str)],
) -> Result<Option<BlockDispatch>, String> {
    if global.is_none() && own.is_empty() {
        return Ok(None);
    }
    let fallback = include(global.unwrap_or(runique))?;
    let mut content = String::new();
    for (i, (key, path)) in own.iter().enumerate() {
        let tag = if i == 0 { "if" } else { "elif" };
        content.push_str(&format!(
            r#"{{% {tag} resource.key == "{}" %}}{}"#,
            literal(key)?,
            include(path)?
        ));
    }
    if own.is_empty() {
        content = fallback;
    } else {
        content.push_str(&format!("{{% else %}}{fallback}{{% endif %}}"));
    }
    Ok(Some(BlockDispatch {
        name: name.to_string(),
        content,
    }))
}

/// The dispatch of every replaced block. Refused: a page replaced for a
/// model that isn't one of a model's, a key or path a template can't hold.
pub(crate) fn dispatch(
    templates: &AdminTemplate,
    models: &[(&'static str, &ModelTemplates)],
) -> Result<Vec<BlockDispatch>, String> {
    for (key, t) in models {
        if let Some(page) = t.pages.keys().find(|p| !p.per_model()) {
            return Err(format!(
                "model `{key}`: the {page:?} page is about no model, it is replaced globally only"
            ));
        }
        for path in t.pages.values() {
            template(path)?;
        }
    }
    for page in Page::ALL {
        if let Some(path) = templates.page_template(page).and_then(|t| t.dev.as_deref()) {
            template(path)?;
        }
    }
    let mut out = Vec::new();
    for block in Block::ALL {
        let own: Vec<(&str, &str)> = models
            .iter()
            .filter_map(|(key, t)| t.blocks.html_of(block).map(|path| (*key, path)))
            .collect();
        out.extend(dispatch_of(
            block.name(),
            block.runique(),
            templates.blocks.html_of(block),
            &own,
        )?);
    }
    for block in GlobalBlock::ALL {
        out.extend(dispatch_of(
            block.name(),
            block.runique(),
            templates.global_blocks.html_of(block),
            &[],
        )?);
    }
    Ok(out)
}

/// Every page the developer replaced, globally or for a model, that Tera did
/// not load. A block's template is checked by Tera itself (it is an
/// `include`); a page is only named at render time, so a missing one would
/// fail on its first request instead.
pub(crate) fn missing_pages(
    templates: &AdminTemplate,
    models: &[(&'static str, &ModelTemplates)],
    loaded: impl Fn(&str) -> bool,
) -> Result<(), String> {
    let mut missing = Vec::new();
    for page in Page::ALL {
        if let Some(path) = templates.page_template(page).and_then(|t| t.dev.as_deref())
            && !loaded(path)
        {
            missing.push(format!("{page:?} page: `{path}`"));
        }
    }
    if let Some(path) = templates.htmx.dev.as_deref()
        && !loaded(path)
    {
        missing.push(format!("List fragment (htmx): `{path}`"));
    }
    for (key, t) in models {
        for (page, path) in &t.pages {
            if !loaded(path.as_str()) {
                missing.push(format!("model `{key}`, {page:?} page: `{path}`"));
            }
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "admin template override not found in the templates folder: {}",
            missing.join(", ")
        ))
    }
}

/// The template Runique renders for `page`, as it ships.
fn runique_page(templates: &AdminTemplate, page: Page) -> &'static str {
    match templates.page_template(page) {
        Some(template) => template.runique,
        None if page == Page::Rights => "admin/rights.html",
        None => "admin/transfer.html",
    }
}

/// What every admin page's request puts in its context, whatever the page.
const REQUEST_KEYS: &[&str] = &[
    "admin_prefix",
    "csp_nonce",
    "csrf_token",
    "current_path",
    "current_user",
    "debug",
    "dev",
    "messages",
    "static_runique",
];

/// The translation sections `insert_admin_messages` gives `page`.
fn message_sections(page: Page) -> &'static [&'static str] {
    match page {
        Page::Login => &["login"],
        Page::Dashboard => &["dashboard", "base"],
        Page::List => &["list", "base"],
        Page::Create => &["create", "base"],
        Page::Edit => &["edit", "base"],
        Page::Detail => &["detail", "list", "base"],
        Page::Delete => &["delete", "base"],
        Page::History | Page::HistoryDiff => &["history", "base"],
        Page::Rights | Page::Transfer => &["base"],
    }
}

/// Keys a page's context holds that Runique's own templates for it may not
/// read. Kept exact by `every_key_an_admin_page_provides_is_allowed`, which
/// reads the live pages' contexts.
fn provided_unread(page: Page) -> Vec<String> {
    let mut keys: Vec<String> = REQUEST_KEYS.iter().map(|k| k.to_string()).collect();
    for section in message_sections(page) {
        let prefix = format!("admin.{section}.");
        keys.extend(
            crate::utils::ADMIN_MESSAGE_KEYS
                .iter()
                .filter(|k| k.starts_with(&prefix))
                .map(|k| k.replace('.', "_")),
        );
    }
    match page {
        Page::List => keys.push("page".to_string()),
        Page::Create | Page::Edit | Page::Delete | Page::Rights => {
            keys.push("resource_key".to_string());
        }
        _ => {}
    }
    keys
}

/// Tera's own variable: the whole context, for debugging a template.
const TERA_CONTEXT: &str = "__tera_context";

/// The keys a template replacing `page`, or one of its blocks, may read: those
/// Runique's own templates for that page read (`pristine` holds them without
/// any replacement), plus the ones its context holds unread.
fn allowed_keys(
    pristine: &tera::Tera,
    templates: &AdminTemplate,
    page: Page,
) -> Result<std::collections::HashSet<String>, String> {
    let mut keys: std::collections::HashSet<String> = pristine
        .get_template_variables(runique_page(templates, page))
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(str::to_string)
        .collect();
    keys.extend(provided_unread(page));
    keys.insert(TERA_CONTEXT.to_string());
    Ok(keys)
}

/// The closest of `known` to `name`, when close enough to be a typo.
fn closest<'a>(name: &str, known: &'a std::collections::HashSet<String>) -> Option<&'a str> {
    fn distance(a: &str, b: &str) -> usize {
        let b: Vec<char> = b.chars().collect();
        let mut row: Vec<usize> = (0..=b.len()).collect();
        for (i, ca) in a.chars().enumerate() {
            let mut previous = row[0];
            row[0] = i + 1;
            for (j, cb) in b.iter().enumerate() {
                let current = row[j + 1];
                row[j + 1] = if ca == *cb {
                    previous
                } else {
                    1 + previous.min(row[j]).min(current)
                };
                previous = current;
            }
        }
        row[b.len()]
    }
    known
        .iter()
        .map(|k| (distance(name, k), k.as_str()))
        .filter(|(d, _)| *d <= 2)
        .min()
        .map(|(_, k)| k)
}

/// Every variable a replaced admin page or block reads that its context never
/// holds: a typo the page would only reveal on render, refused at boot.
/// `loaded` holds the developer's templates, `pristine` Runique's own without
/// any replacement. A block shown on several pages may read what any of them
/// holds.
pub(crate) fn unknown_variables(
    pristine: &tera::Tera,
    loaded: &tera::Tera,
    templates: &AdminTemplate,
    models: &[(&'static str, &ModelTemplates)],
) -> Result<(), String> {
    let mut problems = Vec::new();
    let mut check = |label: String, path: &str, allowed: &std::collections::HashSet<String>| {
        let Ok(vars) = loaded.get_template_variables(path) else {
            return;
        };
        let mut unknown: Vec<&str> = vars.into_iter().filter(|v| !allowed.contains(*v)).collect();
        unknown.sort_unstable();
        for name in unknown {
            match closest(name, allowed) {
                Some(hint) => problems.push(format!(
                    "{label} `{path}`: `{name}` (did you mean `{hint}`?)"
                )),
                None => problems.push(format!("{label} `{path}`: `{name}`")),
            }
        }
    };

    let mut pages = Vec::new();
    for page in Page::ALL {
        pages.push((page, allowed_keys(pristine, templates, page)?));
    }
    let allowed_of = |page: Page| &pages.iter().find(|(p, _)| *p == page).unwrap().1;

    for page in Page::ALL {
        if let Some(path) = templates.page_template(page).and_then(|t| t.dev.as_deref()) {
            check(format!("{page:?} page"), path, allowed_of(page));
        }
    }
    if let Some(path) = templates.htmx.dev.as_deref() {
        check(
            "List fragment (htmx)".to_string(),
            path,
            allowed_of(Page::List),
        );
    }
    for (key, t) in models {
        for (page, path) in &t.pages {
            check(
                format!("model `{key}`, {page:?} page"),
                path.as_str(),
                allowed_of(*page),
            );
        }
    }

    let union_of = |shown_on: &dyn Fn(Page) -> bool| {
        let mut keys = std::collections::HashSet::new();
        for (page, allowed) in &pages {
            if shown_on(*page) {
                keys.extend(allowed.iter().cloned());
            }
        }
        keys
    };
    for block in Block::ALL {
        let allowed = union_of(&|p: Page| p.blocks().contains(&block));
        if let Some(path) = templates.blocks.html_of(block) {
            check(format!("{block:?} block"), path, &allowed);
        }
        for (key, t) in models {
            if let Some(path) = t.blocks.html_of(block) {
                check(format!("model `{key}`, {block:?} block"), path, &allowed);
            }
        }
    }
    for block in GlobalBlock::ALL {
        let allowed = union_of(&|p: Page| p.global_blocks().contains(&block));
        if let Some(path) = templates.global_blocks.html_of(block) {
            check(format!("{block:?} block"), path, &allowed);
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "admin template override reads a variable its page never provides: {}",
            problems.join("; ")
        ))
    }
}

/// A model's view is served by its own page or by a route replacing it,
/// never both: the route would win, the page never render.
pub(crate) fn check_replaced_views(
    models: &[(&'static str, &ModelTemplates)],
    replaced: impl Iterator<Item = (String, Page)>,
) -> Result<(), String> {
    for (key, page) in replaced {
        if models
            .iter()
            .any(|(k, t)| *k == key && t.pages.contains_key(&page))
        {
            return Err(format!(
                "model `{key}`: its {page:?} page is both replaced by a template and by a route"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: &str = "admin/blocks/list/table.html";

    #[test]
    fn a_replaced_page_found_in_the_templates_passes() {
        let global = AdminTemplate::new().with_list("theme/list.html");
        let notes = ModelTemplates::default().page(Page::Detail, "notes/detail.html");
        let loaded = |p: &str| matches!(p, "theme/list.html" | "notes/detail.html");
        assert!(missing_pages(&global, &[("notes", &notes)], loaded).is_ok());
        assert!(
            missing_pages(&AdminTemplate::new(), &[], |_| false).is_ok(),
            "nothing replaced"
        );
    }

    #[test]
    fn a_missing_global_or_model_page_is_named() {
        let global = AdminTemplate::new().with_list("theme/list.html");
        let err = missing_pages(&global, &[], |_| false).unwrap_err();
        assert!(err.contains("List page: `theme/list.html`"), "{err}");

        let notes = ModelTemplates::default().page(Page::Detail, "notes/detail.html");
        let err =
            missing_pages(&AdminTemplate::new(), &[("notes", &notes)], |_| false).unwrap_err();
        assert!(
            err.contains("model `notes`, Detail page: `notes/detail.html`"),
            "{err}"
        );

        // Only the missing one is listed.
        let err =
            missing_pages(&global, &[("notes", &notes)], |p| p == "theme/list.html").unwrap_err();
        assert!(
            !err.contains("theme/list.html") && err.contains("notes/detail.html"),
            "{err}"
        );
    }

    #[test]
    fn nothing_replaced_dispatches_nothing() {
        assert!(dispatch(&AdminTemplate::new(), &[]).unwrap().is_empty());
    }

    #[test]
    fn a_global_block_is_included_directly() {
        let t = AdminTemplate::new().block(Block::ListTable, "theme/table.html");
        let out = dispatch(&t, &[]).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "list_table");
        assert_eq!(out[0].content, r#"{% include "theme/table.html" %}"#);
    }

    #[test]
    fn a_models_block_goes_behind_a_test_then_the_global_one() {
        let t = AdminTemplate::new().block(Block::ListTable, "theme/table.html");
        let notes = ModelTemplates::default().block(Block::ListTable, "notes/table.html");
        let out = dispatch(
            &t,
            &[("notes", &notes), ("bare", &ModelTemplates::default())],
        )
        .unwrap();
        assert_eq!(
            out[0].content,
            r#"{% if resource.key == "notes" %}{% include "notes/table.html" %}{% else %}{% include "theme/table.html" %}{% endif %}"#
        );
        // Without a global one, the others get Runique's.
        let out = dispatch(&AdminTemplate::new(), &[("notes", &notes)]).unwrap();
        assert!(out[0].content.ends_with(&format!(
            r#"{{% else %}}{{% include "{TABLE}" %}}{{% endif %}}"#
        )));
    }

    #[test]
    fn a_layout_block_is_replaced_globally() {
        let t = AdminTemplate::new().global_block(GlobalBlock::Sidebar, "theme/sidebar.html");
        let out = dispatch(&t, &[]).unwrap();
        assert_eq!(out[0].content, r#"{% include "theme/sidebar.html" %}"#);
    }

    #[test]
    fn a_path_or_key_that_could_break_out_of_the_template_is_refused() {
        for path in [
            r#"x" %}{{ secret }}{% include "y.html"#,
            "a{{b}}.html",
            "",
            "a\nb.html",
            "theme/table",
            "theme/table.txt",
        ] {
            let t = AdminTemplate::new().block(Block::ListTable, path);
            assert!(dispatch(&t, &[]).is_err(), "{path:?}");
        }
        let notes = ModelTemplates::default().block(Block::ListTable, "notes/table.html");
        assert!(dispatch(&AdminTemplate::new(), &[("no\"tes", &notes)]).is_err());
    }

    #[test]
    fn a_page_replaced_must_be_a_html_template() {
        assert!(dispatch(&AdminTemplate::new().with_list("theme/list"), &[]).is_err());
        assert!(dispatch(&AdminTemplate::new().with_list("theme/list.html"), &[]).is_ok());
        let notes = ModelTemplates::default().page(Page::List, "notes/list.txt");
        assert!(dispatch(&AdminTemplate::new(), &[("notes", &notes)]).is_err());
    }

    #[test]
    fn a_page_about_no_model_is_never_replaced_for_one() {
        let notes = ModelTemplates::default().page(Page::Dashboard, "notes/dashboard.html");
        assert!(dispatch(&AdminTemplate::new(), &[("notes", &notes)]).is_err());
        let notes = ModelTemplates::default().page(Page::List, "notes/list.html");
        assert!(dispatch(&AdminTemplate::new(), &[("notes", &notes)]).is_ok());
    }

    #[test]
    fn a_view_is_replaced_by_a_page_or_a_route_never_both() {
        let notes = ModelTemplates::default().page(Page::Detail, "notes/detail.html");
        let models = [("notes", &notes)];
        assert!(
            check_replaced_views(&models, [("notes".to_string(), Page::Detail)].into_iter())
                .is_err()
        );
        assert!(
            check_replaced_views(&models, [("other".to_string(), Page::Detail)].into_iter())
                .is_ok()
        );
        assert!(check_replaced_views(&models, std::iter::empty()).is_ok());
    }
}
