//! `test` command: checks the project is set up for `runique_test`, then runs
//! its tests through `cargo test`, one at a time and with their output shown.
use crate::utils::trad::{t, tf};
use anyhow::{Context, Result, bail};
use std::{
    fs,
    io::{IsTerminal, Read, Write},
    path::Path,
    process::{Command, Stdio},
};

const TEST_DIR: &str = "src/runique_test";

/// Set by `runique test` when it writes to a terminal. Its output goes through
/// a pipe, so the test code can't tell it's a terminal and would drop colors.
pub const COLOR_ENV: &str = "RUNIQUE_TEST_COLOR";

/// Entry point of `runique test [file] [test]`, run from the project root.
///
/// No argument runs every test in `src/runique_test/`, `file` only the tests of
/// `src/runique_test/<file>.rs`, and `file` + `test` that single test.
pub fn run_tests(file: Option<&str>, test: Option<&str>) -> Result<()> {
    let cargo_toml = fs::read_to_string("Cargo.toml").context(t("test_runner.no_cargo_toml"))?;
    match test_utils_placement(&cargo_toml) {
        TestUtils::DevOnly => {}
        TestUtils::Missing => bail!("{}", t("test_runner.test_utils_missing")),
        TestUtils::Shipped => bail!("{}", t("test_runner.test_utils_shipped")),
    }

    let dir = Path::new(TEST_DIR);
    let mod_src = fs::read_to_string(dir.join("mod.rs")).context(t("test_runner.no_mod_file"))?;

    let declaration = ["src/main.rs", "src/lib.rs"]
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .find_map(|src| test_module_declaration(&src));
    match declaration {
        None => bail!("{}", t("test_runner.module_not_declared")),
        Some(false) if !module_file_is_gated(&mod_src) => {
            bail!("{}", t("test_runner.module_not_gated"))
        }
        Some(_) => {}
    }

    let declared = declared_modules(&mod_src);
    let files = test_files(dir)?;
    for stem in files.iter().filter(|stem| !declared.contains(stem)) {
        eprintln!(
            "{}",
            tf("test_runner.file_not_declared_warning", &[stem, stem])
        );
    }
    if let Some(file) = file {
        if !files.iter().any(|stem| stem == file) {
            bail!(
                "{}",
                tf(
                    "test_runner.no_test_file",
                    &[file, files.join(", ").as_str()]
                )
            );
        }
        if !declared.iter().any(|stem| stem == file) {
            bail!("{}", tf("test_runner.file_not_declared", &[file, file]));
        }
    }

    let (filter, exact) = cargo_filter(file, test);
    if let (Some(file), Some(test)) = (file, test) {
        check_test_exists(file, test, &filter)?;
    }

    let color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    let mut args = vec!["test".to_string(), "--".to_string(), filter];
    if exact {
        args.push("--exact".to_string());
    }
    args.extend(["--nocapture".to_string(), "--test-threads=1".to_string()]);
    if color {
        args.extend(["--color".to_string(), "always".to_string()]);
    }

    let mut command = Command::new("cargo");
    command.args(&args).stdout(Stdio::piped());
    if color {
        command.env(COLOR_ENV, "1");
    }
    let mut child = command
        .spawn()
        .context(t("test_runner.cargo_spawn_failed"))?;
    if let Some(output) = child.stdout.take() {
        forward_spaced(output, &mut std::io::stdout(), color)
            .context(t("test_runner.output_failed"))?;
    }
    let status = child.wait().context(t("test_runner.cargo_wait_failed"))?;
    if !status.success() {
        bail!("{}", t("test_runner.tests_failed"));
    }
    Ok(())
}

/// How cargo opens each test's line, and what ends that opening part.
const HEADER_PREFIX: &[u8] = b"test runique_test::";
const HEADER_END: &[u8] = b" ... ";
/// A soft orange (256-color palette): easy on the eyes on a dark terminal,
/// and apart from the cyan step numbers of the trace underneath.
const TEST_NAME_COLOR: &str = "\x1b[38;5;215m";

/// Copies cargo's output as it comes, adding a blank line after each test's
/// `ok`/`FAILED` so tests don't run into each other, and showing each test's
/// name without the `runique_test::` every one of them shares (in color when
/// `color` is set). Byte by byte rather than line by line: cargo only ends a
/// test's `test … ...` line once the test is done, so waiting for full lines
/// would hide which test is running.
fn forward_spaced(mut from: impl Read, to: &mut impl Write, color: bool) -> std::io::Result<()> {
    let mut buf = [0u8; 4096];
    let mut line = Vec::new();
    let mut out = Vec::new();
    // While the line so far could still be a test's opening, it's held back
    // (it sits in `line`) so it can be rewritten once complete.
    let mut holding = false;
    loop {
        let read = from.read(&mut buf)?;
        if read == 0 {
            if holding {
                to.write_all(&line)?;
            }
            return to.flush();
        }
        for &byte in &buf[..read] {
            if byte == b'\n' {
                if holding {
                    out.extend_from_slice(&line);
                    holding = false;
                }
                out.push(byte);
                if is_result_line(&String::from_utf8_lossy(&line)) {
                    out.push(b'\n');
                }
                line.clear();
                continue;
            }
            line.push(byte);
            if line.len() == 1 {
                holding = byte == HEADER_PREFIX[0];
            }
            if !holding {
                out.push(byte);
            } else if line.starts_with(HEADER_PREFIX) && line.ends_with(HEADER_END) {
                out.extend_from_slice(&test_header(&line, color));
                holding = false;
            } else if !HEADER_PREFIX.starts_with(&line) && !line.starts_with(HEADER_PREFIX) {
                out.extend_from_slice(&line);
                holding = false;
            }
        }
        to.write_all(&out)?;
        to.flush()?;
        out.clear();
    }
}

/// `test runique_test::blog::add ... ` becomes `test blog::add ... `, with the
/// name in color. Only called on a line that starts and ends that way.
fn test_header(line: &[u8], color: bool) -> Vec<u8> {
    let name = &line[HEADER_PREFIX.len()..line.len() - HEADER_END.len()];
    let mut header = b"test ".to_vec();
    if color {
        header.extend_from_slice(TEST_NAME_COLOR.as_bytes());
    }
    header.extend_from_slice(name);
    if color {
        header.extend_from_slice(b"\x1b[0m");
    }
    header.extend_from_slice(HEADER_END);
    header
}

/// A test's verdict line: `ok`/`FAILED` on its own (after a test's output) or
/// at the end of `test … ...` (for a test that printed nothing).
fn is_result_line(line: &str) -> bool {
    let line = strip_ansi(line);
    let line = line.trim();
    matches!(line, "ok" | "FAILED")
        || (line.starts_with("test ") && (line.ends_with(" ok") || line.ends_with(" FAILED")))
}

/// Removes terminal color codes (`ESC [ … letter`), which wrap `ok` when colors are on.
fn strip_ansi(s: &str) -> String {
    let mut plain = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for code in chars.by_ref() {
                if code.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            plain.push(c);
        }
    }
    plain
}

/// Asks cargo for the tests of `file` and fails with the list of real names
/// when `test` isn't one of them: cargo alone would just report 0 tests run.
fn check_test_exists(file: &str, test: &str, full_name: &str) -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "test",
            "--quiet",
            "--",
            &format!("runique_test::{file}::"),
            "--list",
        ])
        .stderr(Stdio::inherit())
        .output()
        .context(t("test_runner.cargo_spawn_failed"))?;
    if !output.status.success() {
        bail!("{}", t("test_runner.build_failed"));
    }
    let names = parse_test_list(&String::from_utf8_lossy(&output.stdout));
    if !names.iter().any(|name| name == full_name) {
        let available: Vec<&str> = names
            .iter()
            .map(|name| name.rsplit("::").next().unwrap_or(name))
            .collect();
        bail!(
            "{}",
            tf(
                "test_runner.no_such_test",
                &[test, file, available.join(", ").as_str()]
            )
        );
    }
    Ok(())
}

/// Where Cargo.toml turns `test-utils` on.
#[derive(Debug, PartialEq)]
enum TestUtils {
    Missing,
    /// Only in `[dev-dependencies]`: test builds get it, release builds don't.
    DevOnly,
    /// In a section that reaches release builds (`[dependencies]`,
    /// `[workspace.dependencies]`, a `[features]` entry), even if it's also a dev one.
    Shipped,
}

/// Reads every `runique` entry of Cargo.toml (inline, dotted, or
/// `[…dependencies.runique]` table) and finds out where `test-utils` is enabled.
fn test_utils_placement(cargo_toml: &str) -> TestUtils {
    // Each entry comes with whether it sits in a dev section.
    let mut entries: Vec<(bool, String)> = Vec::new();
    let mut section: Option<bool> = None;
    let mut table: Option<(bool, String)> = None;
    let mut inline: Option<(bool, String)> = None;
    let mut open_braces = 0i32;

    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if inline.is_none() && trimmed.starts_with('[') {
            entries.extend(table.take());
            section = dependency_section(trimmed, "dependencies");
            table =
                dependency_section(trimmed, "dependencies.runique").map(|dev| (dev, String::new()));
            continue;
        }
        if let Some((_, text)) = &mut table {
            text.push_str(trimmed);
            text.push('\n');
            continue;
        }
        if inline.is_none()
            && let Some(dev) = section
            && trimmed
                .strip_prefix("runique")
                .is_some_and(|rest| rest.trim_start().starts_with(['=', '.']))
        {
            inline = Some((dev, String::new()));
            open_braces = 0;
        }
        if let Some((_, text)) = &mut inline {
            text.push_str(trimmed);
            text.push('\n');
            open_braces += count(trimmed, '{') - count(trimmed, '}');
            if open_braces <= 0 {
                entries.extend(inline.take());
            }
        }
    }
    entries.extend(table);
    entries.extend(inline);

    let enables = |text: &String| text.contains("\"test-utils\"");
    // `[features] x = ["runique/test-utils"]` turns it on for the regular dependency.
    let from_features = cargo_toml.contains("\"runique/test-utils\"")
        || cargo_toml.contains("\"runique?/test-utils\"");
    if from_features || entries.iter().any(|(dev, text)| !dev && enables(text)) {
        TestUtils::Shipped
    } else if entries.iter().any(|(_, text)| enables(text)) {
        TestUtils::DevOnly
    } else {
        TestUtils::Missing
    }
}

/// For a `[…]` header ending in `suffix`: `Some(true)` for a dev section,
/// `Some(false)` for one that reaches release builds, `None` otherwise.
/// Build dependencies never end up in the binary, so they don't count.
fn dependency_section(header: &str, suffix: &str) -> Option<bool> {
    let name = header.strip_prefix('[')?.split(']').next()?;
    let prefix = name.strip_suffix(suffix)?;
    if !(prefix.is_empty() || prefix.ends_with(['.', '-'])) || prefix.ends_with("build-") {
        return None;
    }
    Some(prefix.ends_with("dev-"))
}

fn count(s: &str, c: char) -> i32 {
    i32::try_from(s.matches(c).count()).unwrap_or(i32::MAX)
}

/// `None` when `src` doesn't declare `mod runique_test`, otherwise whether that
/// declaration carries a `cfg(test)`.
fn test_module_declaration(src: &str) -> Option<bool> {
    let file = syn::parse_file(src).ok()?;
    file.items.iter().find_map(|item| match item {
        syn::Item::Mod(module) if module.ident == "runique_test" => {
            Some(module.attrs.iter().any(is_cfg_test))
        }
        _ => None,
    })
}

/// Whether `mod.rs` gates itself with an inner `#![cfg(test)]`.
fn module_file_is_gated(mod_src: &str) -> bool {
    syn::parse_file(mod_src).is_ok_and(|file| file.attrs.iter().any(is_cfg_test))
}

fn is_cfg_test(attr: &syn::Attribute) -> bool {
    match &attr.meta {
        syn::Meta::List(list) if list.path.is_ident("cfg") => list
            .tokens
            .to_string()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .any(|word| word == "test"),
        _ => false,
    }
}

/// Names of the `mod x;` declarations in `mod.rs`.
fn declared_modules(mod_src: &str) -> Vec<String> {
    syn::parse_file(mod_src)
        .map(|file| {
            file.items
                .iter()
                .filter_map(|item| match item {
                    syn::Item::Mod(module) => Some(module.ident.to_string()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Stems of the `.rs` files in `dir`, `mod.rs` excluded, sorted.
fn test_files(dir: &Path) -> Result<Vec<String>> {
    let mut stems: Vec<String> = fs::read_dir(dir)
        .with_context(|| tf("test_runner.dir_read_failed", &[dir.display()]))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .filter_map(|path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
        })
        .filter(|stem| stem != "mod")
        .collect();
    stems.sort();
    Ok(stems)
}

/// The `cargo test` name filter, and whether it must match exactly.
fn cargo_filter(file: Option<&str>, test: Option<&str>) -> (String, bool) {
    match (file, test) {
        (Some(file), Some(test)) => (format!("runique_test::{file}::{test}"), true),
        (Some(file), None) => (format!("runique_test::{file}::"), false),
        _ => ("runique_test::".to_string(), false),
    }
}

/// Test names from `cargo test -- --list` (lines like `a::b::c: test`).
fn parse_test_list(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utils_found_in_inline_dev_dependency() {
        let toml = "[dependencies]\nserde = \"1\"\n\n[dev-dependencies]\nrunique = { path = \"../runique\", features = [\"test-utils\"] }\n";
        assert_eq!(test_utils_placement(toml), TestUtils::DevOnly);
    }

    #[test]
    fn test_utils_found_in_table_form() {
        let toml = "[dev-dependencies.runique]\nversion = \"2.3.0\"\nfeatures = [\"test-utils\"]\n";
        assert_eq!(test_utils_placement(toml), TestUtils::DevOnly);
    }

    #[test]
    fn test_utils_found_in_multiline_inline_table() {
        let toml = "[dev-dependencies]\nrunique = {\n  path = \"../runique\",\n  features = [\"test-utils\"],\n}\n";
        assert_eq!(test_utils_placement(toml), TestUtils::DevOnly);
    }

    #[test]
    fn test_utils_found_in_target_dev_dependencies() {
        let toml = "[target.'cfg(unix)'.dev-dependencies]\nrunique = { version = \"2.3.0\", features = [\"test-utils\"] }\n";
        assert_eq!(test_utils_placement(toml), TestUtils::DevOnly);
    }

    #[test]
    fn test_utils_missing() {
        let toml =
            "[dependencies]\nrunique = { version = \"2.3.0\", features = [\"orm\", \"sqlite\"] }\n";
        assert_eq!(test_utils_placement(toml), TestUtils::Missing);
    }

    #[test]
    fn test_utils_on_another_crate_is_ignored() {
        let toml = "[dev-dependencies]\nother = { version = \"1\", features = [\"test-utils\"] }\n";
        assert_eq!(test_utils_placement(toml), TestUtils::Missing);
    }

    #[test]
    fn test_utils_in_dependencies_is_refused() {
        let toml = "[dependencies]\nrunique = { version = \"2.3.0\", features = [\"orm\", \"test-utils\"] }\n";
        assert_eq!(test_utils_placement(toml), TestUtils::Shipped);
    }

    #[test]
    fn test_utils_in_both_sections_is_refused() {
        let toml = "[dependencies]\nrunique = { version = \"2.3.0\", features = [\"test-utils\"] }\n\n[dev-dependencies]\nrunique = { version = \"2.3.0\", features = [\"test-utils\"] }\n";
        assert_eq!(test_utils_placement(toml), TestUtils::Shipped);
    }

    #[test]
    fn test_utils_in_dependencies_table_is_refused() {
        let toml = "[dependencies.runique]\nversion = \"2.3.0\"\nfeatures = [\"test-utils\"]\n\n[dev-dependencies]\nserde = \"1\"\n";
        assert_eq!(test_utils_placement(toml), TestUtils::Shipped);
    }

    #[test]
    fn test_utils_in_dotted_key_is_refused() {
        let toml =
            "[dependencies]\nrunique.version = \"2.3.0\"\nrunique.features = [\"test-utils\"]\n";
        assert_eq!(test_utils_placement(toml), TestUtils::Shipped);
    }

    #[test]
    fn test_utils_in_workspace_dependencies_is_refused() {
        let toml = "[workspace.dependencies]\nrunique = { version = \"2.3.0\", features = [\"test-utils\"] }\n";
        assert_eq!(test_utils_placement(toml), TestUtils::Shipped);
    }

    #[test]
    fn test_utils_through_a_feature_is_refused() {
        let toml = "[dependencies]\nrunique = { version = \"2.3.0\" }\n\n[features]\ntesting = [\"runique/test-utils\"]\n\n[dev-dependencies]\nrunique = { version = \"2.3.0\", features = [\"test-utils\"] }\n";
        assert_eq!(test_utils_placement(toml), TestUtils::Shipped);
    }

    #[test]
    fn test_utils_in_build_dependencies_is_ignored() {
        let toml = "[build-dependencies]\nrunique = { version = \"2.3.0\", features = [\"test-utils\"] }\n";
        assert_eq!(test_utils_placement(toml), TestUtils::Missing);
    }

    #[test]
    fn declaration_gated_in_main() {
        let src = "mod views;\n#[cfg(test)]\nmod runique_test;\nfn main() {}\n";
        assert_eq!(test_module_declaration(src), Some(true));
    }

    #[test]
    fn declaration_not_gated() {
        let src = "mod runique_test;\nfn main() {}\n";
        assert_eq!(test_module_declaration(src), Some(false));
    }

    #[test]
    fn declaration_missing() {
        let src = "mod views;\nfn main() {}\n";
        assert_eq!(test_module_declaration(src), None);
    }

    #[test]
    fn mod_file_gated_by_inner_attribute() {
        assert!(module_file_is_gated("#![cfg(test)]\nmod user;\n"));
        assert!(!module_file_is_gated("mod user;\n"));
    }

    #[test]
    fn declared_modules_lists_mod_items() {
        let src = "pub const ENV: &str = \".env\";\nmod user;\nmod livre;\n";
        assert_eq!(declared_modules(src), ["user", "livre"]);
    }

    #[test]
    fn filters_per_argument() {
        assert_eq!(
            cargo_filter(None, None),
            ("runique_test::".to_string(), false)
        );
        assert_eq!(
            cargo_filter(Some("user"), None),
            ("runique_test::user::".to_string(), false)
        );
        assert_eq!(
            cargo_filter(Some("user"), Some("add_email")),
            ("runique_test::user::add_email".to_string(), true)
        );
    }

    #[test]
    fn blank_line_after_each_verdict() {
        let cargo =
            "running 2 tests\ntest a ... \n     1. x\nok\ntest b ... ok\n\ntest result: ok.\n";
        let mut shown = Vec::new();
        forward_spaced(cargo.as_bytes(), &mut shown, false).unwrap();
        assert_eq!(
            String::from_utf8(shown).unwrap(),
            "running 2 tests\ntest a ... \n     1. x\nok\n\ntest b ... ok\n\n\ntest result: ok.\n"
        );
    }

    #[test]
    fn test_name_loses_its_shared_prefix() {
        let cargo = "test runique_test::blog::add ... \n     1. x\nok\ntest result: ok.\n";
        let mut shown = Vec::new();
        forward_spaced(cargo.as_bytes(), &mut shown, false).unwrap();
        assert_eq!(
            String::from_utf8(shown).unwrap(),
            "test blog::add ... \n     1. x\nok\n\ntest result: ok.\n"
        );
    }

    #[test]
    fn test_name_is_colored_and_verdict_left_alone() {
        let cargo = "test runique_test::user::find ... ok\n";
        let mut shown = Vec::new();
        forward_spaced(cargo.as_bytes(), &mut shown, true).unwrap();
        assert_eq!(
            String::from_utf8(shown).unwrap(),
            format!("test {TEST_NAME_COLOR}user::find\x1b[0m ... ok\n\n")
        );
    }

    #[test]
    fn header_split_across_reads_is_still_rewritten() {
        // A reader handing out one byte at a time: the worst case for a pipe.
        struct OneByte<'a>(&'a [u8]);
        impl Read for OneByte<'_> {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                let Some((first, rest)) = self.0.split_first() else {
                    return Ok(0);
                };
                buf[0] = *first;
                self.0 = rest;
                Ok(1)
            }
        }
        let mut shown = Vec::new();
        forward_spaced(
            OneByte(b"test runique_test::a::b ... \nok\n"),
            &mut shown,
            false,
        )
        .unwrap();
        assert_eq!(String::from_utf8(shown).unwrap(), "test a::b ... \nok\n\n");
    }

    #[test]
    fn other_lines_starting_with_t_are_untouched() {
        let cargo = "test result: ok. 2 passed\ntests\nthe end";
        let mut shown = Vec::new();
        forward_spaced(cargo.as_bytes(), &mut shown, true).unwrap();
        assert_eq!(String::from_utf8(shown).unwrap(), cargo);
    }

    #[test]
    fn colored_verdict_is_recognized() {
        assert!(is_result_line("\x1b[32mok\x1b[0m"));
        assert!(is_result_line("test a ... \x1b[31mFAILED\x1b[0m"));
        assert!(!is_result_line("     1.    0.9 ms  SELECT … FROM \"ok\""));
    }

    #[test]
    fn test_list_is_parsed() {
        let out =
            "runique_test::user::add_email: test\nruntime_check: test\n\n2 tests, 0 benchmarks\n";
        assert_eq!(
            parse_test_list(out),
            ["runique_test::user::add_email", "runtime_check"]
        );
    }
}
