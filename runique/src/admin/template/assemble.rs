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
