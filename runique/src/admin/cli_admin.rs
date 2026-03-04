use crate::middleware::auth::user::{ActiveModel, BuiltinUserEntity, UserEntity};
use crate::utils::password::{BaseHash, Manual};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
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
        "Argon2 (recommended)",
        "Bcrypt",
        "Scrypt",
        "Custom provider",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("[1/5] Hashing algorithm")
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
                .with_prompt("Provider path")
                .interact_text()
                .ok()?;
            if path.trim().is_empty() {
                println!("Invalid path.");
                None
            } else {
                Some(AlgoChoice::Custom(path))
            }
        }
        _ => None,
    }
}

async fn step_username(db: &DatabaseConnection) -> Option<String> {
    loop {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("[2/5] Username")
            .interact_text()
            .ok()?;

        let input = input.trim().to_string();
        if input.is_empty() {
            println!("Username cannot be empty.");
            continue;
        }

        if BuiltinUserEntity::find_by_username(db, &input)
            .await
            .is_some()
        {
            println!("A user with this name already exists.");
            continue;
        }
        return Some(input);
    }
}

async fn step_email(db: &DatabaseConnection) -> Option<String> {
    loop {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("[3/5] Email")
            .interact_text()
            .ok()?;

        let input = input.trim().to_lowercase();
        if input.is_empty() || !input.contains('@') {
            println!("Invalid email.");
            continue;
        }

        if BuiltinUserEntity::find_by_email(db, &input).await.is_some() {
            println!("A user with this email already exists.");
            continue;
        }
        return Some(input);
    }
}

fn step_password() -> Option<String> {
    loop {
        let pass1 = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("[4/5] Password")
            .interact()
            .ok()?;

        if pass1.len() < 10 {
            println!("Password must be at least 10 characters.");
            continue;
        }

        let pass2 = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("[5/5] Confirm password")
            .interact()
            .ok()?;

        if pass1 != pass2 {
            println!("Passwords do not match. Please try again.");
            continue;
        }

        return Some(pass1);
    }
}

fn step_review(state: &WizardState) -> ReviewAction {
    let algo = state.algorithm.as_ref().unwrap();
    let username = state.username.as_deref().unwrap();
    let email = state.email.as_deref().unwrap();

    println!("\n──────────────────────────────────");
    println!("  Algorithm   : {}", algo.label());
    println!("  Username    : {}", username);
    println!("  Email       : {}", email);
    println!("  Password    : ••••••••");
    println!("──────────────────────────────────");

    let items = vec!["Confirm and create", "Change algorithm", "Modify password"];

    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Action")
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
    use std::process::{Command, Stdio};

    let mut child = Command::new(provider_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to launch provider '{}': {}", provider_path, e))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(password.as_bytes())
            .map_err(|e| format!("Error writing to stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Error waiting for provider: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Provider returned an error (exit code {:?})",
            output.status.code()
        ));
    }

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("Invalid provider output (UTF-8): {}", e))
}

// ─── Entry point ──────────────────────────────────────────────────────────────

pub async fn create_superuser() -> Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be defined in .env");
    let db = sea_orm::Database::connect(&database_url).await?;

    println!("=== Create Superuser ===");

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
    let hashed =
        hash_password(password, algo).map_err(|e| anyhow::anyhow!("Hashing error: {}", e))?;

    let username = state.username.unwrap();
    let email = state.email.unwrap();

    let new_user = ActiveModel {
        username: Set(username.clone()),
        email: Set(email.clone()),
        password: Set(hashed),
        is_active: Set(true),
        is_staff: Set(true),
        is_superuser: Set(true),
        created_at: Set(Some(chrono::Utc::now().naive_utc())),
        updated_at: Set(Some(chrono::Utc::now().naive_utc())),
        ..Default::default()
    };

    let inserted = new_user.insert(&db).await?;

    println!("\n✓ Superuser created successfully!");
    println!("  ID       : {}", inserted.id);
    println!("  Username : {}", username);
    println!("  Email    : {}", email);

    Ok(())
}
