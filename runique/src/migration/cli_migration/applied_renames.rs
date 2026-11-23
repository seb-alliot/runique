//! Once `migration up` applied a rename, the `renamed_from` that asked for it
//! is taken out of the entity file: left there, it would only say what the
//! table or the field used to be called.
//!
//! The file is the developer's: only the directive goes, with the comma that
//! set it apart, and what is left is checked token for token before it is
//! written.
use crate::migration::parse::builder::MacroCollector;
use crate::utils::trad::tf;
use anyhow::{Context, Result};
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use runique_dsl::{ast::ModelInput, extend::ExtendDsl};
use std::{fs, ops::Range, path::PathBuf};
use syn::visit::Visit;

/// The tokens of a macro call, as they are written.
struct Raw(TokenStream);

impl syn::parse::Parse for Raw {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse()?))
    }
}

/// The entity files that lose their `renamed_from`, with their new content.
/// `targets` are the tables the applied migrations changed, as (`extend!{}`
/// or not, table): a model loses every one of its directives when its table,
/// or the table of one of its list fields, is among them.
pub(super) fn cleared_files(
    entities_path: &str,
    targets: &[(bool, String)],
) -> Result<Vec<(PathBuf, String)>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(entities_path)
        .with_context(|| format!("Cannot read entities directory: {entities_path}"))?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<std::io::Result<_>>()?;
    paths.sort();

    let mut files = Vec::new();
    for path in paths {
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path)
            .with_context(|| format!("Cannot read file: {}", path.display()))?;
        let ranges = applied_ranges(&source, targets);
        if ranges.is_empty() {
            continue;
        }
        let cleared = cut(&source, &ranges);
        if same_but(&source, &cleared, &ranges) {
            files.push((path, cleared));
        } else {
            println!("{}", tf("migrate.renamed_from_kept", &[path.display()]));
        }
    }
    Ok(files)
}

/// Where the `renamed_from` to take out of `source` are, in bytes, sorted.
/// Nothing when the file can't be read as Rust, or when a place found isn't
/// one the text can be cut at.
fn applied_ranges(source: &str, targets: &[(bool, String)]) -> Vec<Range<usize>> {
    let Ok(file) = syn::parse_str::<syn::File>(source) else {
        return Vec::new();
    };
    let mut ranges = Vec::new();

    let mut models = MacroCollector::<Raw>::new("model");
    models.visit_file(&file);
    for Raw(tokens) in models.found.into_iter().flatten() {
        if let Ok(model) = syn::parse2::<ModelInput>(tokens.clone())
            && targets.iter().any(|(extend, table)| {
                !extend
                    && (*table == model.table
                        || model
                            .lists
                            .iter()
                            .any(|list| *table == format!("{}_{}", model.table, list.name)))
            })
        {
            collect(tokens, &mut ranges);
        }
    }

    let mut extends = MacroCollector::<Raw>::new("extend");
    extends.visit_file(&file);
    for Raw(tokens) in extends.found.into_iter().flatten() {
        if let Ok(block) = syn::parse2::<ExtendDsl>(tokens.clone())
            && targets
                .iter()
                .any(|(extend, table)| *extend && *table == block.table)
        {
            collect(tokens, &mut ranges);
        }
    }

    ranges.sort_by_key(|range| range.start);
    let cuttable = ranges
        .iter()
        .all(|range| source.get(range.clone()).is_some());
    if cuttable { ranges } else { Vec::new() }
}

/// Notes where each `renamed_from: "old"` of `tokens` is, with the comma that
/// sets it apart: the one after it, or else the one before.
fn collect(tokens: TokenStream, ranges: &mut Vec<Range<usize>>) {
    let trees: Vec<TokenTree> = tokens.into_iter().collect();
    let comma = |tree: Option<&TokenTree>| {
        matches!(tree, Some(TokenTree::Punct(punct)) if punct.as_char() == ',')
    };
    for (at, tree) in trees.iter().enumerate() {
        if let TokenTree::Group(group) = tree {
            collect(group.stream(), ranges);
        } else if let TokenTree::Ident(word) = tree
            && word == "renamed_from"
            && let (Some(TokenTree::Punct(colon)), Some(TokenTree::Literal(old))) =
                (trees.get(at + 1), trees.get(at + 2))
            && colon.as_char() == ':'
        {
            let mut range = word.span().byte_range().start..old.span().byte_range().end;
            if comma(trees.get(at + 3)) {
                range.end = trees[at + 3].span().byte_range().end;
            } else if at > 0 && comma(trees.get(at - 1)) {
                range.start = trees[at - 1].span().byte_range().start;
            }
            ranges.push(range);
        }
    }
}

/// `source` without `ranges`. A directive that had its line to itself takes
/// the line with it; one cut with the comma after it takes the blanks that
/// followed. Everything else is kept byte for byte.
fn cut(source: &str, ranges: &[Range<usize>]) -> String {
    let mut out = String::with_capacity(source.len());
    let mut kept_from = 0;
    for range in ranges {
        let (mut start, mut end) = (range.start, range.end);
        let line_start = source[..start].rfind('\n').map_or(0, |at| at + 1);
        let after = &source[end..];
        let blanks = after.len() - after.trim_start_matches([' ', '\t']).len();
        let line_end = if after[blanks..].starts_with("\r\n") {
            Some(2)
        } else if after[blanks..].starts_with('\n') {
            Some(1)
        } else {
            None
        };
        if let Some(line_break) = line_end
            && source[line_start..start].trim().is_empty()
        {
            start = line_start;
            end += blanks + line_break;
        } else if source[..end].ends_with(',') {
            end += blanks;
        }
        // Two directives that share a comma: what is already gone stays gone.
        let start = start.max(kept_from);
        out.push_str(&source[kept_from..start]);
        kept_from = end.max(start);
    }
    out.push_str(&source[kept_from..]);
    out
}

/// Whether `cleared` is `source` less the tokens of `ranges` and nothing
/// else: the last check before a developer's file is rewritten.
fn same_but(source: &str, cleared: &str, ranges: &[Range<usize>]) -> bool {
    let (Ok(before), Ok(after)) = (
        source.parse::<TokenStream>(),
        cleared.parse::<TokenStream>(),
    ) else {
        return false;
    };
    let (mut was, mut is) = (Vec::new(), Vec::new());
    leaves(before, &mut was);
    leaves(after, &mut is);
    let expected = was.iter().filter(|(_, at)| {
        !ranges
            .iter()
            .any(|range| range.start <= at.start && at.end <= range.end)
    });
    expected.map(|(text, _)| text).eq(is.iter().map(|(text, _)| text))
}

/// Every token of `tokens`, groups opened up, with where it is.
fn leaves(tokens: TokenStream, out: &mut Vec<(String, Range<usize>)>) {
    for tree in tokens {
        match tree {
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::None => ("", ""),
                };
                out.push((open.to_string(), group.span_open().byte_range()));
                leaves(group.stream(), out);
                out.push((close.to_string(), group.span_close().byte_range()));
            }
            other => {
                let at = other.span().byte_range();
                out.push((other.to_string(), at));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomic::tests::TempDir;

    fn model(table: &str) -> Vec<(bool, String)> {
        vec![(false, table.to_string())]
    }

    /// `source` once the migrations of `targets` were applied; `None` when
    /// nothing is to take out.
    fn cleared(source: &str, targets: &[(bool, String)]) -> Option<String> {
        let ranges = applied_ranges(source, targets);
        if ranges.is_empty() {
            return None;
        }
        let out = cut(source, &ranges);
        assert!(same_but(source, &out, &ranges), "{out}");
        Some(out)
    }

    // On its own line, the directive goes with its line; the lines around it
    // are the same, byte for byte, comments and odd spacing included.
    #[test]
    fn a_table_directive_on_its_own_line_goes_with_the_line() {
        let source = "// é: kept\nmodel! {\n    Shop,\n    table: \"bakery\",\n    renamed_from: \"shop\",\n    pk: id => i32,\n    {\n        title:   text [required],  // odd   spacing\n    }\n}\n";
        let expected = "// é: kept\nmodel! {\n    Shop,\n    table: \"bakery\",\n    pk: id => i32,\n    {\n        title:   text [required],  // odd   spacing\n    }\n}\n";
        assert_eq!(cleared(source, &model("bakery")).as_deref(), Some(expected));
    }

    #[test]
    fn a_table_directive_on_its_own_line_goes_with_a_windows_line_break() {
        let source = "model! {\r\n    Shop,\r\n    table: \"bakery\",\r\n    renamed_from: \"shop\",\r\n    pk: id => i32,\r\n    { title: text }\r\n}\r\n";
        let expected = "model! {\r\n    Shop,\r\n    table: \"bakery\",\r\n    pk: id => i32,\r\n    { title: text }\r\n}\r\n";
        assert_eq!(cleared(source, &model("bakery")).as_deref(), Some(expected));
    }

    // Sharing its line: it goes with its comma and the blanks after it.
    #[test]
    fn a_table_directive_in_a_line_goes_with_its_comma() {
        let source = r#"model! { Shop, table: "bakery", renamed_from: "shop", pk: id => i32, { title: text } }"#;
        let expected = r#"model! { Shop, table: "bakery", pk: id => i32, { title: text } }"#;
        assert_eq!(cleared(source, &model("bakery")).as_deref(), Some(expected));
    }

    // Among the attributes of a field: first, last, alone, before a trailing
    // comma.
    #[test]
    fn a_field_directive_goes_wherever_it_is_among_the_attributes() {
        let cases = [
            (r#"[renamed_from: "label", required]"#, "[required]"),
            (r#"[required, renamed_from: "label"]"#, "[required]"),
            (r#"[required, renamed_from: "label", unique]"#, "[required, unique]"),
            (r#"[renamed_from: "label"]"#, "[]"),
            (r#"[required, renamed_from: "label",]"#, "[required, ]"),
            (r#"[required , renamed_from : "label"]"#, "[required ]"),
        ];
        for (attrs, left) in cases {
            let file = |attrs: &str| {
                format!(r#"model! {{ Shop, table: "shops", pk: id => i32, {{ name: text {attrs}, }} }}"#)
            };
            assert_eq!(
                cleared(&file(attrs), &model("shops")),
                Some(file(left)),
                "{attrs}"
            );
        }
    }

    // Two directives in a row: the second has no comma after it and finds
    // before it the one the first already took. Nothing is cut twice.
    #[test]
    fn two_directives_that_share_a_comma_are_each_cut_once() {
        let source = r#"[required, renamed_from: "a", renamed_from: "b"]"#;
        let mut ranges = Vec::new();
        collect(source.parse().unwrap(), &mut ranges);

        assert_eq!(ranges.len(), 2);
        assert!(ranges[1].start < ranges[0].end, "{ranges:?}");
        let out = cut(source, &ranges);
        assert_eq!(out, "[required, ]");
        assert!(same_but(source, &out, &ranges));
    }

    // A place wholly inside what is already gone changes nothing.
    #[test]
    fn a_range_inside_one_already_cut_changes_nothing() {
        assert_eq!(cut("abcdef", &[1..4, 2..3]), "aef");
        assert_eq!(cut("abcdef", &[1..4]), "aef");
    }

    // What is left is still a model, the same one less its directives.
    #[test]
    fn every_directive_of_the_model_goes_and_the_model_still_reads() {
        let source = r#"model! {
    Book,
    table: "novels",
    renamed_from: "books",
    pk: id => i32,
    enums: { Genre: [A, B] },
    {
        title: text [required, renamed_from: "name"],
        styles: checkbox [enum(Genre), renamed_from: "genres"],
    }
}
"#;
        let out = cleared(source, &model("novels")).unwrap();
        assert!(!out.contains("renamed_from"), "{out}");
        let read = crate::migration::parse_model_from_source(&out)
            .unwrap()
            .unwrap();
        assert_eq!(read.schema.table_name, "novels");
        assert!(read.renames.is_empty());
        assert_eq!(read.lists[0].table_name, "novels_styles");
        assert_eq!(out.lines().count(), source.lines().count() - 1);
    }

    // The migration applied is the one of a list field's table: its
    // directive is in the file of the model that owns it.
    #[test]
    fn a_list_table_clears_the_model_that_owns_it() {
        let source = r#"model! { Book, table: "books", pk: id => i32, enums: { Genre: [A] }, { styles: checkbox [enum(Genre), renamed_from: "genres"] } }"#;
        let out = cleared(source, &model("books_styles")).unwrap();
        assert!(!out.contains("renamed_from"), "{out}");
        assert_eq!(cleared(source, &model("books_genres")), None);
    }

    // Nothing was applied for this table: its file isn't touched. A model is
    // not an extension of the same name, nor the other way round.
    #[test]
    fn only_the_tables_just_migrated_are_cleared() {
        let shop = r#"model! { Shop, table: "bakery", renamed_from: "shop", pk: id => i32, { title: text } }"#;
        assert_eq!(cleared(shop, &model("shop")), None);
        assert_eq!(cleared(shop, &model("other")), None);
        assert_eq!(cleared(shop, &[]), None);
        assert_eq!(cleared(shop, &[(true, "bakery".to_string())]), None);

        let extension = r#"extend! { table: "eihwaz_users", fields: { bio: text [renamed_from: "about"] } }"#;
        assert_eq!(cleared(extension, &model("eihwaz_users")), None);
        assert_eq!(
            cleared(extension, &[(true, "eihwaz_users".to_string())]).as_deref(),
            Some(r#"extend! { table: "eihwaz_users", fields: { bio: text [] } }"#)
        );
    }

    // Two blocks of one file: only the one of the table migrated loses its
    // directive.
    #[test]
    fn one_block_of_a_file_is_cleared_without_the_other() {
        let source = r#"extend! { table: "eihwaz_users", fields: { bio: text [renamed_from: "about"] } }
extend! { table: "eihwaz_groupes", fields: { note: text [renamed_from: "memo"] } }
"#;
        let out = cleared(source, &[(true, "eihwaz_groupes".to_string())]).unwrap();
        assert!(out.contains(r#"renamed_from: "about""#), "{out}");
        assert!(!out.contains("memo"), "{out}");
    }

    // The words elsewhere than in a directive: a field of that name, a label
    // that quotes it, a comment. None of them goes.
    #[test]
    fn the_words_outside_a_directive_stay() {
        let source = r#"// renamed_from: "x",
model! { Shop, table: "shops", pk: id => i32, { renamed_from: text [label: "renamed_from: \"x\""], } }"#;
        assert_eq!(cleared(source, &model("shops")), None);
    }

    // A file that isn't Rust, or without anything to take out: left alone.
    #[test]
    fn a_file_that_cannot_be_read_is_left_alone() {
        assert_eq!(cleared("model! { Shop, table: ", &model("shops")), None);
        assert_eq!(
            cleared(
                r#"model! { Shop, table: "shops", pk: id => i32, { title: text } }"#,
                &model("shops")
            ),
            None
        );
    }

    // The last check: anything but the directives missing, added or changed
    // is refused.
    #[test]
    fn a_file_that_lost_anything_else_is_refused() {
        let source = r#"model! { Shop, table: "bakery", renamed_from: "shop", pk: id => i32, { title: text } }"#;
        let ranges = applied_ranges(source, &model("bakery"));
        let good = cut(source, &ranges);

        assert!(same_but(source, &good, &ranges));
        assert!(same_but(source, &good.replace("{ title", "{   title"), &ranges));
        assert!(!same_but(source, source, &ranges));
        assert!(!same_but(source, &good.replace("title", "titre"), &ranges));
        assert!(!same_but(source, &good.replace("pk: id => i32, ", ""), &ranges));
        assert!(!same_but(source, &format!("{good} x"), &ranges));
        assert!(!same_but(source, "model! { Shop", &ranges));
        assert!(!same_but(source, &good, &[]));
    }

    // From the folder: the files concerned, in order, and only `.rs` ones.
    #[test]
    fn the_files_of_the_folder_are_read_and_only_the_cleared_ones_returned() {
        let dir = TempDir::new();
        let shop = r#"model! { Shop, table: "bakery", renamed_from: "shop", pk: id => i32, { title: text } }"#;
        let plain = r#"model! { Plat, table: "plats", pk: id => i32, { nom: text } }"#;
        let cafe = shop.replace("bakery", "cafe");
        for (name, content) in [
            ("shop.rs", shop),
            ("plat.rs", plain),
            ("notes.txt", shop),
            ("other.rs", cafe.as_str()),
        ] {
            fs::write(dir.join(name), content).unwrap();
        }
        let folder = dir.path().to_str().unwrap();

        let files = cleared_files(folder, &model("bakery")).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, dir.join("shop.rs"));
        assert_eq!(
            files[0].1,
            r#"model! { Shop, table: "bakery", pk: id => i32, { title: text } }"#
        );
        assert!(cleared_files(&format!("{folder}/missing"), &model("bakery")).is_err());
    }
}
