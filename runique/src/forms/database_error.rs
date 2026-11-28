//! What a database error becomes on a form: a sentence of ours on the field
//! or on the form, the engine's text staying in the log.
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::translation::tf;
use crate::{forms::form::Forms, translation::t};

/// What a driver error means for the form. The sentence is ours; the
/// engine's text stays in the log. Only built when an engine is compiled:
/// without one there is no SQLx error to class.
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
enum Violation {
    Unique,
    ForeignKey,
    NotNull,
    Check,
    Other,
}

/// The class SQLx reports, or [`Violation::Other`] when the error carries no
/// driver error.
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
fn classify_driver(err: &sea_orm::DbErr) -> Violation {
    use sea_orm::sqlx::error::ErrorKind;
    let Some(db) = driver_database(err) else {
        return Violation::Other;
    };
    match db.kind() {
        ErrorKind::UniqueViolation => Violation::Unique,
        ErrorKind::ForeignKeyViolation => Violation::ForeignKey,
        ErrorKind::NotNullViolation => Violation::NotNull,
        ErrorKind::CheckViolation | ErrorKind::ExclusionViolation => Violation::Check,
        _ => Violation::Other,
    }
}

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
fn driver_database(err: &sea_orm::DbErr) -> Option<&dyn sea_orm::sqlx::error::DatabaseError> {
    let runtime = match err {
        sea_orm::DbErr::Exec(err) | sea_orm::DbErr::Query(err) | sea_orm::DbErr::Conn(err) => err,
        _ => return None,
    };
    let sea_orm::RuntimeErr::SqlxError(err) = runtime else {
        return None;
    };
    err.as_database_error()
}

/// The column a unique violation names, from the constraint SQLx reports or
/// from the driver's message. Never the whole message.
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
fn unique_field(err: &sea_orm::DbErr) -> Option<String> {
    let db = driver_database(err)?;
    if let Some(constraint) = db.constraint()
        && let Some(field) = Forms::parse_constraint_name(constraint)
    {
        return Some(field);
    }
    Forms::extract_field_name(db.message()).or_else(|| Forms::extract_field_name(&err.to_string()))
}

impl Forms {
    /// Translates a database error into a user-facing form error.
    ///
    /// A `DbErr::Custom` is a sentence already chosen by the caller (`written`
    /// shows those as they are): it is stored unchanged. A driver error is
    /// classed from SQLx. A unique violation lands on the named field when
    /// the message or the constraint gives the column; every other class is
    /// one global sentence. The engine text is logged, never shown. Without
    /// a compiled engine there is nothing to class, so the sentence is the
    /// generic one.
    pub fn database_error(&mut self, db_err: &sea_orm::DbErr) {
        if let sea_orm::DbErr::Custom(message) = db_err {
            self.errors.push(message.clone());
            return;
        }
        tracing::warn!(error = %db_err, "form database error");
        #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
        match classify_driver(db_err) {
            Violation::Unique => self.push_unique(db_err),
            Violation::ForeignKey => self
                .errors
                .push(t(runique_i18n::keys::forms::error::DB_FOREIGN_KEY).into_owned()),
            Violation::NotNull => self
                .errors
                .push(t(runique_i18n::keys::forms::error::DB_NOT_NULL).into_owned()),
            Violation::Check => self
                .errors
                .push(t(runique_i18n::keys::forms::error::DB_CHECK).into_owned()),
            Violation::Other => self
                .errors
                .push(t(runique_i18n::keys::forms::error::DB).into_owned()),
        }
        #[cfg(not(any(feature = "sqlite", feature = "postgres", feature = "mysql")))]
        self.errors
            .push(t(runique_i18n::keys::forms::error::DB).into_owned());
    }

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
    fn push_unique(&mut self, db_err: &sea_orm::DbErr) {
        let Some(field) = unique_field(db_err) else {
            self.errors
                .push(t(runique_i18n::keys::forms::error::UNIQUE_CONSTRAINT_VIOLATED).into_owned());
            return;
        };
        if let Some(form_field) = self.fields.get_mut(&field) {
            let friendly_name = field.replace("_", " ");
            form_field.set_error(tf(
                runique_i18n::keys::forms::error::UNIQUE_FIELD_TAKEN,
                &[&friendly_name],
            ));
        } else {
            self.errors.push(tf(
                runique_i18n::keys::forms::error::UNIQUE_VALUE_TAKEN,
                &[&field],
            ));
        }
    }

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
    fn extract_field_name(err_msg: &str) -> Option<String> {
        use crate::forms::constants::{CONSTRAINT_REGEX, FAILED_REGEX, FOR_KEY_REGEX, KEY_REGEX};

        if let Some(cap) = CONSTRAINT_REGEX.captures(err_msg).ok()? {
            let constraint = cap.get(1)?.as_str();
            // Only return early if parse succeeds; _pkey/_fkey fall through to other regexes.
            if let Some(field) = Self::parse_constraint_name(constraint) {
                return Some(field);
            }
        }

        if let Some(cap) = KEY_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        if let Some(cap) = FAILED_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        if let Some(cap) = FOR_KEY_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        None
    }

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
    fn parse_constraint_name(constraint: &str) -> Option<String> {
        // PK and FK constraints are not field-level unique violations.
        if constraint.ends_with("_pkey") || constraint.ends_with("_fkey") {
            return None;
        }
        // PostgreSQL convention: tablename_fieldname_key
        // The field is the second-to-last segment (parts[-2] before "_key").
        // Taking parts[1..len-1] incorrectly includes the table name when it contains underscores.
        // Taking just parts[len-2] gives the last field segment — correct for single-word fields,
        // and a best-effort fallback for multi-word ones (handled upstream by KEY_REGEX / FAILED_REGEX).
        let parts: Vec<&str> = constraint.split('_').collect();
        if parts.len() >= 3 {
            return Some(parts[parts.len() - 2].to_string());
        }
        None
    }
}

#[cfg(test)]
mod database_error_tests {
    use super::Forms;
    use crate::{forms::fields::TextField, translation::t};
    use sea_orm::DbErr;

    fn form_with(name: &str) -> Forms {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text(name));
        form
    }

    #[test]
    fn a_custom_sentence_is_kept_whole() {
        let mut form = form_with("email");
        let sentence = r#"duplicate key value violates unique constraint "users_email_key""#;
        form.database_error(&DbErr::Custom(sentence.into()));
        assert!(form.fields["email"].error().is_none());
        assert_eq!(form.errors, vec![sentence.to_string()]);
    }

    #[test]
    fn an_error_without_a_driver_code_is_a_generic_sentence() {
        let mut form = form_with("email");
        form.database_error(&DbErr::RecordNotFound("eihwaz_users".into()));
        assert!(form.fields["email"].error().is_none());
        assert_eq!(
            form.errors,
            vec![t(runique_i18n::keys::forms::error::DB).into_owned()]
        );
        assert!(!form.errors[0].contains("eihwaz"));
    }
}

#[cfg(all(test, any(feature = "sqlite", feature = "postgres", feature = "mysql")))]
mod driver_error_tests {
    use super::Forms;
    use crate::{
        forms::fields::TextField,
        translation::{t, tf},
    };
    use sea_orm::{
        DbErr, RuntimeErr,
        sqlx::error::{DatabaseError, ErrorKind},
    };
    use std::sync::Arc;

    #[derive(Debug)]
    struct FakeDb {
        kind: ErrorKind,
        message: &'static str,
        constraint: Option<&'static str>,
    }

    impl std::fmt::Display for FakeDb {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.message)
        }
    }

    impl std::error::Error for FakeDb {}

    impl DatabaseError for FakeDb {
        fn message(&self) -> &str {
            self.message
        }

        fn constraint(&self) -> Option<&str> {
            self.constraint
        }

        fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
            self
        }

        fn kind(&self) -> ErrorKind {
            match &self.kind {
                ErrorKind::UniqueViolation => ErrorKind::UniqueViolation,
                ErrorKind::ForeignKeyViolation => ErrorKind::ForeignKeyViolation,
                ErrorKind::NotNullViolation => ErrorKind::NotNullViolation,
                ErrorKind::CheckViolation => ErrorKind::CheckViolation,
                ErrorKind::ExclusionViolation => ErrorKind::ExclusionViolation,
                _ => ErrorKind::Other,
            }
        }
    }

    fn driver(kind: ErrorKind, message: &'static str, constraint: Option<&'static str>) -> DbErr {
        DbErr::Exec(RuntimeErr::SqlxError(Arc::new(
            sea_orm::sqlx::Error::Database(Box::new(FakeDb {
                kind,
                message,
                constraint,
            })),
        )))
    }

    fn form_with(name: &str) -> Forms {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text(name));
        form
    }

    fn hides(shown: &str, engine: &str) {
        assert!(!shown.contains(engine), "{shown}");
        assert!(!shown.contains("Execution Error"), "{shown}");
    }

    #[test]
    fn a_unique_violation_lands_on_the_named_field_for_each_engine() {
        let cases = [
            (
                r#"duplicate key value violates unique constraint "users_email_key""#,
                None,
            ),
            ("UNIQUE constraint failed: users.email", None),
            ("Duplicate entry 'a@b.c' for key 'users.email'", None),
        ];
        for (message, constraint) in cases {
            let mut form = form_with("email");
            form.database_error(&driver(ErrorKind::UniqueViolation, message, constraint));
            let shown = form.fields["email"].error().unwrap();
            assert_eq!(
                shown,
                &tf(
                    runique_i18n::keys::forms::error::UNIQUE_FIELD_TAKEN,
                    &["email"]
                )
            );
            assert!(form.errors.is_empty(), "{message}");
            hides(shown, "a@b.c");
            hides(shown, "duplicate");
            hides(shown, "UNIQUE");
        }
    }

    #[test]
    fn the_field_is_the_segment_before_key() {
        let mut form = form_with("title");
        form.field(&TextField::text("item"));
        form.database_error(&driver(
            ErrorKind::UniqueViolation,
            r#"duplicate key value violates unique constraint "site_page_menu_item_title_key""#,
            Some("site_page_menu_item_title_key"),
        ));
        assert!(form.fields["title"].error().is_some());
        assert!(form.fields["item"].error().is_none());
    }

    #[test]
    fn a_unique_violation_without_a_column_is_global() {
        let mut form = form_with("email");
        form.database_error(&driver(
            ErrorKind::UniqueViolation,
            "unique violation",
            None,
        ));
        assert!(form.fields["email"].error().is_none());
        assert_eq!(
            form.errors,
            vec![t(runique_i18n::keys::forms::error::UNIQUE_CONSTRAINT_VIOLATED).into_owned()]
        );
        hides(&form.errors[0], "unique violation");
    }

    #[test]
    fn each_other_class_is_one_sentence_and_hides_the_engine_text() {
        let secret = "table eihwaz_users column password value 'x'";
        let cases = [
            (
                ErrorKind::ForeignKeyViolation,
                runique_i18n::keys::forms::error::DB_FOREIGN_KEY,
            ),
            (
                ErrorKind::NotNullViolation,
                runique_i18n::keys::forms::error::DB_NOT_NULL,
            ),
            (
                ErrorKind::CheckViolation,
                runique_i18n::keys::forms::error::DB_CHECK,
            ),
            (
                ErrorKind::ExclusionViolation,
                runique_i18n::keys::forms::error::DB_CHECK,
            ),
            (ErrorKind::Other, runique_i18n::keys::forms::error::DB),
        ];
        for (kind, key) in cases {
            let mut form = form_with("email");
            form.database_error(&driver(kind, secret, None));
            assert!(form.fields["email"].error().is_none(), "{key}");
            assert_eq!(form.errors, vec![t(key).into_owned()], "{key}");
            hides(&form.errors[0], "eihwaz_users");
            hides(&form.errors[0], "password");
        }
    }
}
