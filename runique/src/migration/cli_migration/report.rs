//! Reporting the checks to the developer: list every finding, then stop
//! (destructive changes, unless `--force`; name lengths, always), and warn
//! about the type changes `--force` let through.
use crate::migration::*;
use crate::utils::trad::t;
use anyhow::Result;

/// Prints `messages` under the translated `header_key`, then bails with the translated
/// `bail_key` — the shared "list every violation, then stop" shape behind both
/// `check_destructive` (forceable) and `check_identifier_lengths` (never forceable, always
/// passes `force: false`). An empty `messages` or `force: true` is a silent no-op.
pub(super) fn report_and_bail_if_any(
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

pub(super) fn check_destructive(all_changes: &[Changes], force: bool) -> Result<()> {
    let blocking = collect_destructive_messages(all_changes);
    report_and_bail_if_any(
        &blocking,
        "makemigrations.destructive_detected",
        "makemigrations.destructive_require_force",
        force,
    )
}

/// Says so loudly, in yellow on a terminal (crossterm: ANSI, or the Windows
/// console API on a terminal without it), plain when piped or with `NO_COLOR`.
pub(super) fn warn_unmigrated_type_changes(changes: &[Changes]) {
    use crossterm::style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor};
    use std::io::IsTerminal;

    let lines = unmigrated_type_changes(changes);
    if lines.is_empty() {
        return;
    }
    let text = format!(
        "\n{}\n{}\n{}\n",
        t("makemigrations.type_change_not_migrated"),
        lines.join("\n"),
        t("makemigrations.type_change_write_by_hand")
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

pub(super) fn check_identifier_lengths(all_changes: &[Changes]) -> Result<()> {
    let long_ids = collect_long_identifier_messages(all_changes);
    report_and_bail_if_any(
        &long_ids,
        "makemigrations.long_identifier_detected",
        "makemigrations.long_identifier_rename_required",
        false,
    )
}
