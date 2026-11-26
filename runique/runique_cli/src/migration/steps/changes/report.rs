//! Reporting the checks to the developer: list every finding, then stop
//! (destructive changes, unless `--force`; name lengths, always), and warn
//! about the type changes `--force` let through.
use crate::migration::*;
use anyhow::Result;
use runique_i18n::t;

/// Prints `messages` under the translated `header_key`, then bails with the translated
/// `bail_key` — the shared "list every violation, then stop" shape behind both
/// `check_destructive` (forceable) and `check_identifier_lengths` (never forceable, always
/// passes `force: false`). An empty `messages` or `force: true` is a silent no-op.
pub(crate) fn report_and_bail_if_any(
    messages: &[String],
    header_key: &str,
    bail_key: &str,
    force: bool,
) -> Result<()> {
    if messages.is_empty() || force {
        return Ok(());
    }

    eprintln!("\n{}", t(header_key));
    for msg in messages {
        eprintln!("{}", msg);
    }
    anyhow::bail!("{}", t(bail_key));
}

pub(crate) fn check_destructive(all_changes: &[Changes], force: bool) -> Result<()> {
    // One guard for the whole run: the tables to drop first, then the rest.
    let blocking = collect_destructive_messages(all_changes);
    report_and_bail_if_any(
        &blocking,
        runique_i18n::keys::makemigrations::error::DESTRUCTIVE_DETECTED,
        runique_i18n::keys::makemigrations::error::DESTRUCTIVE_REQUIRE_FORCE,
        force,
    )
}

/// Says so loudly, in yellow on a terminal (crossterm: ANSI, or the Windows
/// console API on a terminal without it), plain when piped or with `NO_COLOR`.
pub(crate) fn warn_unmigrated_type_changes(changes: &[Changes]) {
    use crossterm::style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor};
    use std::io::IsTerminal;

    let lines = unmigrated_type_changes(changes);
    if lines.is_empty() {
        return;
    }
    let text = format!(
        "\n{}\n{}\n{}\n",
        t(runique_i18n::keys::makemigrations::error::TYPE_CHANGE_NOT_MIGRATED),
        lines.join("\n"),
        t(runique_i18n::keys::makemigrations::error::TYPE_CHANGE_WRITE_BY_HAND)
    );
    let mut err = std::io::stderr();
    let colored = err.is_terminal() && std::env::var_os("NO_COLOR").is_none();
    let printed = colored
        && crossterm::execute!(
            err,
            SetForegroundColor(Color::Yellow),
            SetAttribute(Attribute::Bold),
            Print(&text),
            SetAttribute(Attribute::Reset),
            ResetColor
        )
        .is_ok();
    if !printed {
        eprint!("{text}");
    }
}

pub(crate) fn check_identifier_lengths(all_changes: &[Changes]) -> Result<()> {
    let long_ids = collect_long_identifier_messages(all_changes);
    report_and_bail_if_any(
        &long_ids,
        runique_i18n::keys::makemigrations::error::LONG_IDENTIFIER_DETECTED,
        runique_i18n::keys::makemigrations::error::LONG_IDENTIFIER_RENAME_REQUIRED,
        false,
    )
}
