//! What the start does before anything else: the traces, then the project's
//! translations and its time zone.
use crate::{config::RuniqueConfig, errors::build::BuildError, log::log_init};
use tracing_appender::non_blocking::WorkerGuard;

/// Turns the traces on, with the channels `.env` asks for (DEBUG, RUNIQUE_ENV)
/// under those set by hand. First of all: `TemplateLoader::init()` and the
/// middleware staging already trace.
pub(super) fn start_logging(config: &mut RuniqueConfig) -> Vec<WorkerGuard> {
    config.log = config.log.clone().with_env_levels(config.dev, config.debug);
    log_init(config.log.clone());
    config.log.init_subscriber()
}

/// A zone nobody knows would leave every date as stored, without a word.
/// An empty one is no zone asked for: the site stays in UTC.
pub(super) fn check_timezone(timezone: &str) -> Result<(), BuildError> {
    if !timezone.is_empty() && timezone.parse::<chrono_tz::Tz>().is_err() {
        return Err(BuildError::validation(crate::translation::tf(
            runique_i18n::keys::build::error::TIMEZONE_UNKNOWN,
            &[timezone],
        )));
    }
    Ok(())
}

/// Reads the project's `locales/` folder (`LOCALES_DIR`), then takes the
/// language of the `.env` unless the code chose one. A folder that can't be
/// read as translations stops the start; what it leaves untranslated or
/// misspells is only said.
pub(super) fn load_project_locales(wanted: Option<&str>) -> Result<(), BuildError> {
    use crate::translation::{Lang, keys::build, load_project, project_dir, tf};
    use runique_i18n::{set_default_lang, set_lang};

    let report = load_project(&project_dir())
        .map_err(|e| BuildError::validation(tf(build::error::LOCALES_INVALID, &[e])))?;
    // After the folder: `LANG=nl` names a language only once it is loaded.
    // The file's own `LANG`, never the machine's: a server's locale isn't
    // the language of the site it serves.
    match wanted {
        Some(code) => {
            let Some(lang) = Lang::find(code) else {
                let loaded: Vec<&str> = Lang::available().iter().map(Lang::code).collect();
                return Err(BuildError::validation(tf(
                    build::error::LANG_UNKNOWN,
                    &[code.to_string(), loaded.join(", ")],
                )));
            };
            set_lang(lang);
        }
        None => {
            if let Some(lang) = Lang::from_dotenv() {
                set_default_lang(lang);
            }
        }
    }
    for language in &report.languages {
        if !language.missing.is_empty() {
            let total: usize = language.missing.iter().map(|(_, count)| count).sum();
            let subjects: Vec<String> = language
                .missing
                .iter()
                .map(|(subject, count)| format!("{subject}: {count}"))
                .collect();
            tracing::warn!(
                "{}",
                tf(
                    build::log::LOCALES_MISSING,
                    &[
                        language.code.clone(),
                        total.to_string(),
                        subjects.join(", ")
                    ]
                )
            );
        }
        if !language.unknown.is_empty() {
            tracing::warn!(
                "{}",
                tf(
                    build::log::LOCALES_UNKNOWN,
                    &[language.code.clone(), language.unknown.join(", ")]
                )
            );
        }
        if !language.mismatched.is_empty() {
            tracing::warn!(
                "{}",
                tf(
                    build::log::LOCALES_PLACEHOLDERS,
                    &[language.code.clone(), language.mismatched.join(", ")]
                )
            );
        }
    }
    Ok(())
}
