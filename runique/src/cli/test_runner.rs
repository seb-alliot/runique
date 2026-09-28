//! `test` command: checks the project is set up for `runique_test`, then runs
//! its tests through `cargo test`, one at a time and with their output shown.
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
    let cargo_toml = fs::read_to_string("Cargo.toml")
        .context("No Cargo.toml here: run `runique test` from your project's root.")?;
    if !runique_has_test_utils(&cargo_toml) {
        bail!(
            "The `test-utils` feature isn't enabled for runique in Cargo.toml.\n\
             Add it as a dev-dependency so it never ends up in a release build:\n    \
             cargo add --dev runique --features test-utils"
        );
    }

    let dir = Path::new(TEST_DIR);
    let mod_src = fs::read_to_string(dir.join("mod.rs")).context(
        "No src/runique_test/mod.rs: create it, then declare each test file in it (`mod user;`).",
    )?;

    let declaration = ["src/main.rs", "src/lib.rs"]
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .find_map(|src| test_module_declaration(&src));
    match declaration {
        None => bail!(
            "src/main.rs doesn't declare the test module. Add:\n    #[cfg(test)]\n    mod runique_test;"
        ),
        Some(false) if !module_file_is_gated(&mod_src) => bail!(
            "The runique_test module isn't behind `cfg(test)`, so it would end up in every build.\n\
             Put `#[cfg(test)]` right above `mod runique_test;` in src/main.rs."
        ),
        Some(_) => {}
    }

    let declared = declared_modules(&mod_src);
    let files = test_files(dir)?;
    for stem in files.iter().filter(|stem| !declared.contains(stem)) {
        eprintln!(
            "warning: src/runique_test/{stem}.rs isn't declared in mod.rs, so its tests won't run. \
             Add `mod {stem};` to mod.rs."
        );
    }
    if let Some(file) = file {
        if !files.iter().any(|stem| stem == file) {
            bail!(
                "No src/runique_test/{file}.rs. Test files: {}",
                files.join(", ")
            );
        }
        if !declared.iter().any(|stem| stem == file) {
            bail!("src/runique_test/{file}.rs isn't declared in mod.rs: add `mod {file};` to it.");
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
    let mut child = command.spawn().context("Couldn't run cargo.")?;
    if let Some(output) = child.stdout.take() {
        forward_spaced(output, &mut std::io::stdout()).context("Couldn't show cargo's output.")?;
    }
    let status = child.wait().context("cargo test didn't finish properly.")?;
    if !status.success() {
        bail!("cargo test failed, see the output above.");
    }
    Ok(())
}

/// Copies cargo's output as it comes, adding a blank line after each test's
/// `ok`/`FAILED` so tests don't run into each other. Byte by byte rather than
/// line by line: cargo only ends a test's `test … ...` line once the test is
/// done, so waiting for full lines would hide which test is running.
fn forward_spaced(mut from: impl Read, to: &mut impl Write) -> std::io::Result<()> {
    let mut buf = [0u8; 4096];
    let mut line = Vec::new();
    let mut out = Vec::new();
    loop {
        let read = from.read(&mut buf)?;
        if read == 0 {
            return Ok(());
        }
        for &byte in &buf[..read] {
            out.push(byte);
            if byte == b'\n' {
                if is_result_line(&String::from_utf8_lossy(&line)) {
                    out.push(b'\n');
                }
                line.clear();
            } else {
                line.push(byte);
            }
        }
        to.write_all(&out)?;
        to.flush()?;
        out.clear();
    }
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
        .context("Couldn't run cargo.")?;
    if !output.status.success() {
        bail!("cargo couldn't build the tests, see the errors above.");
    }
    let names = parse_test_list(&String::from_utf8_lossy(&output.stdout));
    if !names.iter().any(|name| name == full_name) {
        let available: Vec<&str> = names
            .iter()
            .map(|name| name.rsplit("::").next().unwrap_or(name))
            .collect();
        bail!(
            "No test \"{test}\" in {file}.rs. Tests available: {}",
            available.join(", ")
        );
    }
    Ok(())
}

/// Whether a `runique` entry of Cargo.toml (inline or `[…dependencies.runique]`
/// table) enables `test-utils`.
fn runique_has_test_utils(cargo_toml: &str) -> bool {
    let mut in_deps = false;
    let mut in_runique_table = false;
    let mut open_braces = 0i32;
    let mut entry = String::new();

    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && open_braces == 0 {
            in_deps = trimmed.ends_with("dependencies]");
            in_runique_table = trimmed.ends_with("dependencies.runique]");
            continue;
        }
        let starts_inline = in_deps
            && trimmed
                .strip_prefix("runique")
                .is_some_and(|rest| rest.trim_start().starts_with('='));
        if in_runique_table || starts_inline || open_braces > 0 {
            entry.push_str(trimmed);
            entry.push('\n');
            open_braces += count(trimmed, '{') - count(trimmed, '}');
        }
    }
    entry.contains("\"test-utils\"")
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
        .with_context(|| format!("Can't read {}", dir.display()))?
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
        assert!(runique_has_test_utils(toml));
    }

    #[test]
    fn test_utils_found_in_table_form() {
        let toml = "[dev-dependencies.runique]\nversion = \"2.3.0\"\nfeatures = [\"test-utils\"]\n";
        assert!(runique_has_test_utils(toml));
    }

    #[test]
    fn test_utils_found_in_multiline_inline_table() {
        let toml = "[dev-dependencies]\nrunique = {\n  path = \"../runique\",\n  features = [\"test-utils\"],\n}\n";
        assert!(runique_has_test_utils(toml));
    }

    #[test]
    fn test_utils_missing() {
        let toml =
            "[dependencies]\nrunique = { version = \"2.3.0\", features = [\"orm\", \"sqlite\"] }\n";
        assert!(!runique_has_test_utils(toml));
    }

    #[test]
    fn test_utils_on_another_crate_is_ignored() {
        let toml = "[dev-dependencies]\nother = { version = \"1\", features = [\"test-utils\"] }\n";
        assert!(!runique_has_test_utils(toml));
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
        forward_spaced(cargo.as_bytes(), &mut shown).unwrap();
        assert_eq!(
            String::from_utf8(shown).unwrap(),
            "running 2 tests\ntest a ... \n     1. x\nok\n\ntest b ... ok\n\n\ntest result: ok.\n"
        );
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
