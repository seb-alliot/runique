//! CLI for creating an admin superuser with a choice of hashing algorithm.
use crate::auth::user::{Column, Entity};
use crate::utils::{
    aliases::ADb,
    password::{BaseHash, Manual},
    trad::{t, tf},
};
use anyhow::Result;
use dialoguer::{Input, Password, Select, theme::ColorfulTheme};
use runique_dsl::ast::PkType;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QuerySelect,
    sea_query::{Alias, Expr, ExprTrait, Query},
};
use std::io::Write;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum AlgoChoice {
    Argon2,
    Bcrypt,
    Scrypt,
    Custom(String),
}

impl AlgoChoice {
    fn label(&self) -> String {
        match self {
            Self::Argon2 => "Argon2".to_string(),
            Self::Bcrypt => "Bcrypt".to_string(),
            Self::Scrypt => "Scrypt".to_string(),
            Self::Custom(path) => format!("Custom ({})", path),
        }
    }
}

#[derive(Debug, Default)]
struct WizardState {
    algorithm: Option<AlgoChoice>,
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

#[derive(Debug, PartialEq)]
enum Step {
    Algorithm,
    Username,
    Email,
    Password,
    Review,
    Done,
}

enum ReviewAction {
    Confirm,
    ChangeAlgo,
    Back,
}

// ─── Steps ────────────────────────────────────────────────────────────────────

fn step_algorithm() -> Option<AlgoChoice> {
    let items = vec![
        t(runique_i18n::keys::superuser::label::ALGO_ARGON2).to_string(),
        t(runique_i18n::keys::superuser::label::ALGO_BCRYPT).to_string(),
        t(runique_i18n::keys::superuser::label::ALGO_SCRYPT).to_string(),
        t(runique_i18n::keys::superuser::label::ALGO_CUSTOM).to_string(),
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(t(runique_i18n::keys::superuser::label::ALGO_PROMPT))
        .items(&items)
        .default(0)
        .interact_opt()
        .ok()??;

    match selection {
        0 => Some(AlgoChoice::Argon2),
        1 => Some(AlgoChoice::Bcrypt),
        2 => Some(AlgoChoice::Scrypt),
        3 => {
            let path: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(t(runique_i18n::keys::superuser::label::PROVIDER_PATH))
                .interact_text()
                .ok()?;
            if path.trim().is_empty() {
                println!("{}", t(runique_i18n::keys::superuser::error::INVALID_PATH));
                None
            } else {
                Some(AlgoChoice::Custom(path))
            }
        }
        _ => None,
    }
}

async fn step_username(db: &ADb) -> Option<String> {
    loop {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(t(runique_i18n::keys::superuser::label::USERNAME_PROMPT))
            .interact_text()
            .ok()?;

        let Some(input) = clean_username(&input) else {
            println!(
                "{}",
                t(runique_i18n::keys::superuser::error::USERNAME_EMPTY)
            );
            continue;
        };

        if taken(db, Column::Username, &input).await {
            println!(
                "{}",
                t(runique_i18n::keys::superuser::error::USERNAME_TAKEN)
            );
            continue;
        }
        return Some(input);
    }
}

async fn step_email(db: &ADb) -> Option<String> {
    loop {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(t(runique_i18n::keys::superuser::label::EMAIL_PROMPT))
            .interact_text()
            .ok()?;

        let Some(input) = clean_email(&input) else {
            println!("{}", t(runique_i18n::keys::superuser::error::EMAIL_INVALID));
            continue;
        };

        if taken(db, Column::Email, &input).await {
            println!("{}", t(runique_i18n::keys::superuser::error::EMAIL_TAKEN));
            continue;
        }
        return Some(input);
    }
}

/// The username as stored: trimmed, `None` when nothing is left.
fn clean_username(input: &str) -> Option<String> {
    let input = input.trim();
    (!input.is_empty()).then(|| input.to_string())
}

/// The email as stored: trimmed and lowercased, `None` unless it is an email
/// by the same rule as the forms' `EmailField` (`validator`).
fn clean_email(input: &str) -> Option<String> {
    use validator::ValidateEmail;
    let input = input.trim().to_lowercase();
    input.validate_email().then_some(input)
}

fn step_password() -> Option<String> {
    loop {
        let pass1 = Password::with_theme(&ColorfulTheme::default())
            .with_prompt(t(runique_i18n::keys::superuser::label::PASSWORD_PROMPT))
            .interact()
            .ok()?;

        if let Some(key) = superuser_password_weakness(&pass1) {
            println!("{}", t(key));
            continue;
        }

        let pass2 = Password::with_theme(&ColorfulTheme::default())
            .with_prompt(t(runique_i18n::keys::superuser::label::CONFIRM_PROMPT))
            .interact()
            .ok()?;

        if pass1 != pass2 {
            println!(
                "{}",
                t(runique_i18n::keys::superuser::error::PASSWORD_MISMATCH)
            );
            continue;
        }

        return Some(pass1);
    }
}

const SUPERUSER_PASSWORD_MIN_CHARS: usize = 12;

/// The translation key of what the superuser's password lacks, `None` when it
/// is strong enough. Length is counted in characters, not bytes: `len()` let
/// four 3-byte characters pass a 12 minimum.
fn superuser_password_weakness(password: &str) -> Option<&'static str> {
    if password.chars().count() < SUPERUSER_PASSWORD_MIN_CHARS {
        return Some(runique_i18n::keys::superuser::error::PASSWORD_TOO_SHORT);
    }
    let has_lower = password.chars().any(char::is_lowercase);
    let has_upper = password.chars().any(char::is_uppercase);
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| !c.is_alphanumeric() && !c.is_whitespace());
    if has_lower && has_upper && has_digit && has_special {
        None
    } else {
        Some(runique_i18n::keys::superuser::error::PASSWORD_WEAK)
    }
}

fn step_review(state: &WizardState) -> ReviewAction {
    let algo = state.algorithm.as_ref().unwrap();
    let username = state.username.as_deref().unwrap();
    let email = state.email.as_deref().unwrap();

    println!("\n──────────────────────────────────");
    println!(
        "{}",
        tf(
            runique_i18n::keys::superuser::info::REVIEW_ALGO,
            &[&algo.label()]
        )
    );
    println!(
        "{}",
        tf(
            runique_i18n::keys::superuser::info::REVIEW_USERNAME,
            &[&username]
        )
    );
    println!(
        "{}",
        tf(runique_i18n::keys::superuser::info::REVIEW_EMAIL, &[&email])
    );
    println!(
        "{}",
        t(runique_i18n::keys::superuser::info::REVIEW_PASSWORD)
    );
    println!("──────────────────────────────────");

    let items = vec![
        t(runique_i18n::keys::superuser::label::CONFIRM_CREATE).to_string(),
        t(runique_i18n::keys::superuser::label::CHANGE_ALGO).to_string(),
        t(runique_i18n::keys::superuser::label::MODIFY_PASSWORD).to_string(),
    ];

    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt(t(runique_i18n::keys::superuser::info::REVIEW_ACTION))
        .items(&items)
        .default(0)
        .interact()
    {
        Ok(0) => ReviewAction::Confirm,
        Ok(1) => ReviewAction::ChangeAlgo,
        Ok(2) => ReviewAction::Back,
        _ => ReviewAction::Back,
    }
}

// ─── Hashing ──────────────────────────────────────────────────────────────────

fn hash_password(password: &str, algo: &AlgoChoice) -> Result<String, String> {
    let hasher = BaseHash::new();
    match algo {
        AlgoChoice::Argon2 => hasher.hash(password, &Manual::Argon2),
        AlgoChoice::Bcrypt => hasher.hash(password, &Manual::Bcrypt),
        AlgoChoice::Scrypt => hasher.hash(password, &Manual::Scrypt),
        AlgoChoice::Custom(path) => hash_via_provider(password, path),
    }
}

fn hash_via_provider(password: &str, provider_path: &str) -> Result<String, String> {
    use std::path::Path;
    use std::process::{Command, Stdio};

    // Verify that the path is an existing and executable file — no shell injection
    let path = Path::new(provider_path);
    if !path.exists() {
        return Err(format!("Provider not found: '{}'", provider_path));
    }
    if !path.is_file() {
        return Err(format!("Provider is not a file: '{}'", provider_path));
    }
    if path.is_symlink() {
        return Err(format!(
            "Provider path is a symbolic link, which is not allowed: '{}'",
            provider_path
        ));
    }

    use std::sync::mpsc;
    use std::thread;
    const PROVIDER_TIMEOUT_SECS: u64 = 10;

    let mut child = Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to launch provider '{}': {}", provider_path, e))?;

    // A provider that exits before reading stdin makes this write fail with
    // BrokenPipe depending on scheduling: its exit status is the real answer,
    // so the write error is only reported once the status is known.
    let write_result = match child.stdin.as_mut() {
        Some(stdin) => stdin.write_all(password.as_bytes()),
        None => Ok(()),
    };
    // Explicitly close stdin to signal EOF to the child process
    drop(child.stdin.take());

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });

    let output = rx
        .recv_timeout(std::time::Duration::from_secs(PROVIDER_TIMEOUT_SECS))
        .map_err(|_| {
            format!(
                "Provider '{}' timed out after {} seconds",
                provider_path, PROVIDER_TIMEOUT_SECS
            )
        })?
        .map_err(|e| format!("Error waiting for provider: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Provider returned an error (exit code {:?})",
            output.status.code()
        ));
    }
    write_result.map_err(|e| format!("Error writing to stdin: {}", e))?;

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("Invalid provider output (UTF-8): {}", e))
}

// ─── Entry point ──────────────────────────────────────────────────────────────

pub async fn create_superuser() -> Result<()> {
    dotenvy::dotenv().ok();

    // Before any question: the key of the accounts table follows a feature
    // of the project (`big-pk`, `pk-uuid`), not one of this build.
    let pk = crate::migration::cli_migration::project_pk::from_cargo()?;

    let config = superuser_db_config(|key| std::env::var(key).ok())?;
    let db: ADb = ADb::from_connection(config.connect().await?);

    println!("{}", t(runique_i18n::keys::superuser::label::TITLE));

    let mut state = WizardState::default();
    let mut step = Step::Algorithm;
    let mut from_review = false;

    loop {
        match step {
            Step::Algorithm => match step_algorithm() {
                None => {
                    if from_review {
                        from_review = false;
                        step = Step::Review;
                    }
                }
                Some(algo) => {
                    state.algorithm = Some(algo);
                    if from_review {
                        from_review = false;
                        step = Step::Review;
                    } else {
                        step = Step::Username;
                    }
                }
            },

            Step::Username => match step_username(&db).await {
                None => step = Step::Algorithm,
                Some(u) => {
                    state.username = Some(u);
                    step = Step::Email;
                }
            },

            Step::Email => match step_email(&db).await {
                None => step = Step::Username,
                Some(e) => {
                    state.email = Some(e);
                    step = Step::Password;
                }
            },

            Step::Password => match step_password() {
                None => step = Step::Email,
                Some(p) => {
                    state.password = Some(p);
                    step = Step::Review;
                }
            },

            Step::Review => match step_review(&state) {
                ReviewAction::Confirm => step = Step::Done,
                ReviewAction::ChangeAlgo => {
                    from_review = true;
                    step = Step::Algorithm;
                }
                ReviewAction::Back => step = Step::Password,
            },

            Step::Done => break,
        }
    }

    // ─── Create superuser ─────────────────────────────────────────────────────
    let algo = state.algorithm.as_ref().unwrap();
    let password = state.password.as_ref().unwrap();
    let username = state.username.unwrap();
    let email = state.email.unwrap();

    let id = insert_superuser(&db, pk, &username, &email, password, algo).await?;

    println!("\n{}", t(runique_i18n::keys::superuser::info::SUCCESS));
    println!(
        "{}",
        tf(runique_i18n::keys::superuser::info::ID_LINE, &[&id])
    );
    println!(
        "{}",
        tf(
            runique_i18n::keys::superuser::info::USERNAME_LINE,
            &[&username]
        )
    );
    println!(
        "{}",
        tf(runique_i18n::keys::superuser::info::EMAIL_LINE, &[&email])
    );

    Ok(())
}

/// The same keys as the app (`DATABASE_URL`, or `DB_ENGINE` + `DB_*`), so a
/// project that boots also reaches its database from the wizard. One
/// connection: the wizard runs a handful of queries, one at a time.
fn superuser_db_config(get: impl Fn(&str) -> Option<String>) -> Result<crate::db::DatabaseConfig> {
    Ok(crate::db::DatabaseConfig::from_lookup(get)
        .map_err(anyhow::Error::msg)?
        .pool_size(1, 1)
        .build())
}

/// Whether an account already holds `value` in `column`. Only that column is
/// read: the key of the account is never decoded, so the answer doesn't
/// depend on the type it has in this project.
async fn taken(db: &ADb, column: Column, value: &str) -> bool {
    Entity::find()
        .select_only()
        .column(column)
        .filter(column.eq(value))
        .into_tuple::<String>()
        .one(db)
        .await
        .ok()
        .flatten()
        .is_some()
}

/// Hashes the password and inserts the account: active, staff, superuser.
/// Returns its key, as text.
///
/// The row is written column by column, never through the account entity:
/// that one is compiled with the key type of this build, and the command
/// serves projects of the three types. `pk` is the project's.
async fn insert_superuser(
    db: &ADb,
    pk: PkType,
    username: &str,
    email: &str,
    password: &str,
    algo: &AlgoChoice,
) -> Result<String> {
    let hashed =
        hash_password(password, algo).map_err(|e| anyhow::anyhow!("Hashing error: {}", e))?;
    let now = chrono::Utc::now().naive_utc();

    let mut columns = vec![
        Column::Username,
        Column::Email,
        Column::Password,
        Column::IsActive,
        Column::IsSuperuser,
        Column::CreatedAt,
        Column::UpdatedAt,
        // Created from the command line by whoever runs the server: active
        // straight away, so activated at once.
        Column::ActivatedAt,
    ];
    let mut values: Vec<Expr> = vec![
        username.into(),
        email.into(),
        hashed.into(),
        true.into(),
        true.into(),
        now.into(),
        now.into(),
        now.into(),
    ];
    // A UUID key comes from the application, never from the database.
    let key = (pk == PkType::Uuid).then(uuid::Uuid::now_v7);
    if let Some(key) = key {
        columns.push(Column::Id);
        values.push(key.into());
    }

    let insert = Query::insert()
        .into_table(Entity)
        .columns(columns)
        .values(values)?
        .to_owned();
    db.execute(&insert).await?;

    match key {
        Some(key) => Ok(key.to_string()),
        None => stored_key(db, username).await,
    }
}

/// The key the database gave the account of `username`, read as text: the
/// same query whether the column is an `integer` or a `bigint`.
async fn stored_key(db: &ADb, username: &str) -> Result<String> {
    let text = Alias::new(crate::db::sql_dialect::text_cast_type(db));
    Entity::find()
        .select_only()
        .column_as(Expr::col((Entity, Column::Id)).cast_as(text), "id")
        .filter(Column::Username.eq(username))
        .into_tuple::<String>()
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("the account was inserted but could not be read back"))
}

/// Written from cargo-mutants survivors (2026-10-02): the hashing half of the
/// wizard, which runs without a terminal.
#[cfg(test)]
mod hashing_tests {
    use super::*;

    #[test]
    fn labels_name_the_algorithm() {
        assert_eq!(AlgoChoice::Argon2.label(), "Argon2");
        assert_eq!(AlgoChoice::Bcrypt.label(), "Bcrypt");
        assert_eq!(AlgoChoice::Scrypt.label(), "Scrypt");
        assert_eq!(
            AlgoChoice::Custom("/bin/h".into()).label(),
            "Custom (/bin/h)"
        );
    }

    #[test]
    fn argon2_gives_a_verifiable_hash() {
        let hash = hash_password("s3cret-pass", &AlgoChoice::Argon2).unwrap();
        assert!(hash.starts_with("$argon2"), "{hash}");
        assert!(BaseHash::new().verify("s3cret-pass", &hash));
    }

    #[cfg(unix)]
    fn script(dir: &std::path::Path, name: &str, body: &str) -> String {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[cfg(unix)]
    #[test]
    fn a_provider_gets_the_password_on_stdin_and_its_answer_is_the_hash() {
        let dir = std::env::temp_dir().join(format!("rq_provider_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        let echo = script(&dir, "echo.sh", "read pw; printf '  hashed:%s\\n' \"$pw\"");
        assert_eq!(hash_via_provider("pa ss", &echo).unwrap(), "hashed:pa ss");
        let failing = script(&dir, "fail.sh", "exit 3");
        assert!(
            hash_via_provider("x", &failing)
                .unwrap_err()
                .contains("exit code")
        );

        assert!(
            hash_via_provider("x", &dir.join("none").to_string_lossy())
                .unwrap_err()
                .contains("not found")
        );
        assert!(
            hash_via_provider("x", &dir.to_string_lossy())
                .unwrap_err()
                .contains("not a file")
        );
        let link = dir.join("link.sh");
        std::os::unix::fs::symlink(&echo, &link).unwrap();
        assert!(
            hash_via_provider("x", &link.to_string_lossy())
                .unwrap_err()
                .contains("symbolic link")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod db_config_tests {
    use super::superuser_db_config;
    use std::collections::HashMap;

    fn lookup(vars: &[(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        let vars: HashMap<&str, &str> = vars.iter().copied().collect();
        move |key| vars.get(key).map(|v| v.to_string())
    }

    #[test]
    fn the_url_is_rebuilt_from_the_separate_keys() {
        let config = superuser_db_config(lookup(&[
            ("DB_ENGINE", "postgres"),
            ("DB_USER", "bob"),
            ("DB_PASSWORD", "secret"),
            ("DB_HOST", "127.0.0.1"),
            ("DB_PORT", "5433"),
            ("DB_NAME", "app"),
        ]))
        .unwrap();
        assert_eq!(config.url, "postgres://bob:secret@127.0.0.1:5433/app");
        assert_eq!((config.min_connections, config.max_connections), (1, 1));
    }

    #[test]
    fn database_url_is_taken_as_is() {
        let config = superuser_db_config(lookup(&[
            ("DATABASE_URL", "sqlite://direct.db"),
            ("DB_ENGINE", "postgres"),
        ]))
        .unwrap();
        assert_eq!(config.url, "sqlite://direct.db");
    }

    #[test]
    fn a_missing_key_is_an_error_not_a_panic() {
        let err = superuser_db_config(lookup(&[
            ("DB_ENGINE", "postgres"),
            ("DB_USER", "bob"),
            ("DB_PASSWORD", "secret"),
        ]))
        .unwrap_err();
        assert!(err.to_string().contains("DB_NAME"), "{err}");
    }
}

#[cfg(test)]
mod password_rule_tests {
    use super::superuser_password_weakness as weakness;

    #[test]
    fn a_long_mixed_password_passes() {
        assert_eq!(weakness("Campanile-2026"), None);
        assert_eq!(
            weakness("Aa1€aaaaaaaa"),
            None,
            "any non-alphanumeric symbol counts"
        );
    }

    #[test]
    fn under_twelve_characters_is_too_short() {
        assert_eq!(
            weakness("Aa1!aaaaaaa"),
            Some(runique_i18n::keys::superuser::error::PASSWORD_TOO_SHORT)
        );
        // 4 characters, 12 bytes: counted in characters
        assert_eq!(
            weakness("日本語!"),
            Some(runique_i18n::keys::superuser::error::PASSWORD_TOO_SHORT)
        );
    }

    #[test]
    fn each_missing_class_is_refused() {
        let weak = Some(runique_i18n::keys::superuser::error::PASSWORD_WEAK);
        assert_eq!(weakness("aa1!aaaaaaaa"), weak, "no uppercase");
        assert_eq!(weakness("AA1!AAAAAAAA"), weak, "no lowercase");
        assert_eq!(weakness("Aa!aaaaaaaaa"), weak, "no digit");
        assert_eq!(weakness("Aa1aaaaaaaaa"), weak, "no special character");
        assert_eq!(
            weakness("Aa1 aaaaaaaa"),
            weak,
            "a space isn't a special character"
        );
    }
}

/// The superuser the wizard writes, on the table the framework migration
/// creates (its `UNIQUE` and `CHECK` included): extracted from the prompts so
/// it runs without a terminal.
#[cfg(all(test, feature = "sqlite"))]
mod superuser_tests {
    use super::*;
    use crate::auth::migrations_table::EihwazUsersMigration;
    use crate::auth::user::BuiltinUserEntity;
    use sea_orm::{ConnectionTrait, EntityTrait, PaginatorTrait};
    use sea_orm_migration::{MigrationTrait, SchemaManager};

    async fn users_db() -> ADb {
        let conn = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        EihwazUsersMigration
            .up(&SchemaManager::new(&conn))
            .await
            .unwrap();
        ADb::from_connection(conn)
    }

    async fn count(db: &ADb) -> u64 {
        crate::auth::user::Entity::find().count(db).await.unwrap()
    }

    /// The key type of the table these tests create: the one of this build.
    fn built_pk() -> PkType {
        if cfg!(feature = "pk-uuid") {
            PkType::Uuid
        } else if cfg!(feature = "big-pk") {
            PkType::I64
        } else {
            PkType::I32
        }
    }

    /// `insert_superuser` for the table of this build.
    async fn insert(
        db: &ADb,
        username: &str,
        email: &str,
        password: &str,
        algo: &AlgoChoice,
    ) -> Result<String> {
        insert_superuser(db, built_pk(), username, email, password, algo).await
    }

    #[test]
    fn username_is_trimmed_and_never_empty() {
        assert_eq!(clean_username("  bob  ").as_deref(), Some("bob"));
        assert_eq!(clean_username("b").as_deref(), Some("b"));
        assert_eq!(clean_username("   "), None);
        assert_eq!(clean_username(""), None);
    }

    #[test]
    fn email_is_trimmed_lowercased_and_needs_an_at() {
        assert_eq!(
            clean_email("  Bob@Example.COM ").as_deref(),
            Some("bob@example.com")
        );
        assert_eq!(clean_email("a@b.fr").as_deref(), Some("a@b.fr"));
        assert_eq!(clean_email("bob.example.com"), None);
        assert_eq!(clean_email("   "), None);
        // An `@` alone was enough before: the forms' rule now applies.
        assert_eq!(clean_email("@"), None);
        assert_eq!(clean_email("a@"), None);
        assert_eq!(clean_email("a b@example.com"), None);
    }

    #[tokio::test]
    async fn the_superuser_is_active_staff_superuser_and_activated() {
        let db = users_db().await;
        let key = insert(
            &db,
            "root",
            "root@example.com",
            "Campanile-2026",
            &AlgoChoice::Argon2,
        )
        .await
        .unwrap();

        let stored = BuiltinUserEntity::find_by_username(&db, "root")
            .await
            .unwrap();
        assert!(stored.is_active && stored.is_superuser);
        assert!(stored.activated_at.is_some(), "the CHECK requires it");
        assert!(stored.created_at.is_some() && stored.updated_at.is_some());
        assert_eq!(
            (stored.username.as_str(), stored.email.as_str()),
            ("root", "root@example.com")
        );
        // The key the command shows is the one the account has.
        assert_eq!(key, stored.id.to_string());
        assert_ne!(
            stored.password, "Campanile-2026",
            "never the plain password"
        );
        assert!(BaseHash::new().verify("Campanile-2026", &stored.password));
        assert!(!BaseHash::new().verify("Campanile-2027", &stored.password));
    }

    #[tokio::test]
    async fn a_taken_username_or_email_is_refused_by_the_database() {
        let db = users_db().await;
        let argon = AlgoChoice::Argon2;
        insert(&db, "root", "root@example.com", "Campanile-2026", &argon)
            .await
            .unwrap();
        assert!(
            insert(&db, "root", "other@example.com", "Campanile-2026", &argon)
                .await
                .is_err()
        );
        assert!(
            insert(&db, "other", "root@example.com", "Campanile-2026", &argon)
                .await
                .is_err()
        );
        assert_eq!(count(&db).await, 1);
        insert(&db, "other", "other@example.com", "Campanile-2026", &argon)
            .await
            .unwrap();
        assert_eq!(count(&db).await, 2);
    }

    /// Username and email go through bound parameters: SQL in them is stored
    /// as text, never run, and a lookup only finds the exact value.
    #[tokio::test]
    async fn sql_in_the_username_or_email_is_only_text() {
        let db = users_db().await;
        let username = "root'); DROP TABLE eihwaz_users; --";
        let email = "x' OR '1'='1@example.com";
        insert(&db, username, email, "Campanile-2026", &AlgoChoice::Argon2)
            .await
            .unwrap();

        db.execute_unprepared("SELECT 1 FROM eihwaz_users")
            .await
            .expect("the table is still there");
        let found = BuiltinUserEntity::find_by_username(&db, username)
            .await
            .unwrap();
        assert_eq!(
            (found.username.as_str(), found.email.as_str()),
            (username, email)
        );
        assert!(
            BuiltinUserEntity::find_by_username(&db, "' OR '1'='1")
                .await
                .is_none()
        );
        assert!(
            BuiltinUserEntity::find_by_email(&db, "' OR '1'='1")
                .await
                .is_none()
        );
        assert!(
            BuiltinUserEntity::find_by_username(&db, "root")
                .await
                .is_none()
        );
    }

    // A name or an email already there is seen without reading the account's
    // key; one that isn't, or SQL in its place, is not.
    #[tokio::test]
    async fn a_taken_name_or_email_is_seen_and_only_the_exact_one() {
        let db = users_db().await;
        insert(
            &db,
            "root",
            "root@example.com",
            "Campanile-2026",
            &AlgoChoice::Argon2,
        )
        .await
        .unwrap();

        assert!(taken(&db, Column::Username, "root").await);
        assert!(taken(&db, Column::Email, "root@example.com").await);
        assert!(!taken(&db, Column::Username, "roo").await);
        assert!(!taken(&db, Column::Username, "root@example.com").await);
        assert!(!taken(&db, Column::Email, "root").await);
        assert!(!taken(&db, Column::Username, "' OR '1'='1").await);
    }

    // The key read back is the stored one, and nothing is read for a name
    // that has no account.
    #[tokio::test]
    async fn the_key_is_read_back_for_the_right_account() {
        let db = users_db().await;
        let argon = AlgoChoice::Argon2;
        let first = insert(&db, "root", "root@example.com", "Campanile-2026", &argon)
            .await
            .unwrap();
        let second = insert(&db, "other", "other@example.com", "Campanile-2026", &argon)
            .await
            .unwrap();

        assert_ne!(first, second);
        assert_eq!(stored_key(&db, "root").await.unwrap(), first);
        assert_eq!(stored_key(&db, "other").await.unwrap(), second);
        assert!(stored_key(&db, "nobody").await.is_err());
    }

    /// A provider that fails stops everything: no account without a hash.
    #[cfg(unix)]
    #[tokio::test]
    async fn no_account_when_hashing_fails() {
        let db = users_db().await;
        let missing = AlgoChoice::Custom("/nonexistent/runique-provider".into());
        let err = insert(&db, "root", "root@example.com", "Campanile-2026", &missing)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Hashing error"), "{err}");
        assert!(
            !err.to_string().contains("Campanile-2026"),
            "the password is never shown"
        );
        assert_eq!(count(&db).await, 0);
    }
}
