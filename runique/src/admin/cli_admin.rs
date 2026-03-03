use crate::middleware::auth::user::{ActiveModel, BuiltinUserEntity, UserEntity};
use crate::utils::password::{BaseHash, Manual};
use anyhow::Result;
use crossterm::{
    event::{read, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use sea_orm::{ActiveModelTrait, Set};
use std::io::{stdout, Write};

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

// ─── Input helpers ────────────────────────────────────────────────────────────

/// Lit une ligne de texte en mode raw.
/// Retourne None sur ESC (retour), quitte sur Ctrl+C, Some(valeur) sur Entrée.
fn read_line_esc(prompt: &str) -> Option<String> {
    print!("{}", prompt);
    let _ = stdout().flush();

    enable_raw_mode().ok();
    let mut buf = String::new();

    let result = loop {
        if let Ok(Event::Key(key)) = read() {
            match key.code {
                KeyCode::Enter => break Some(buf.clone()),
                KeyCode::Esc => break None,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = disable_raw_mode();
                    println!("\nAnnulé.");
                    std::process::exit(0);
                }
                KeyCode::Backspace => {
                    if !buf.is_empty() {
                        buf.pop();
                        print!("\x08 \x08");
                        let _ = stdout().flush();
                    }
                }
                KeyCode::Char(c) => {
                    buf.push(c);
                    print!("{}", c);
                    let _ = stdout().flush();
                }
                _ => {}
            }
        }
    };

    let _ = disable_raw_mode();
    println!();
    result
}

/// Même comportement que read_line_esc mais masque la saisie avec '•'.
fn read_password_esc(prompt: &str) -> Option<String> {
    print!("{}", prompt);
    let _ = stdout().flush();

    enable_raw_mode().ok();
    let mut buf = String::new();

    let result = loop {
        if let Ok(Event::Key(key)) = read() {
            match key.code {
                KeyCode::Enter => break Some(buf.clone()),
                KeyCode::Esc => break None,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = disable_raw_mode();
                    println!("\nAnnulé.");
                    std::process::exit(0);
                }
                KeyCode::Backspace => {
                    if !buf.is_empty() {
                        buf.pop();
                        print!("\x08 \x08");
                        let _ = stdout().flush();
                    }
                }
                KeyCode::Char(c) => {
                    buf.push(c);
                    print!("•");
                    let _ = stdout().flush();
                }
                _ => {}
            }
        }
    };

    let _ = disable_raw_mode();
    println!();
    result
}

// ─── Steps ────────────────────────────────────────────────────────────────────

/// Retourne None sur ESC. L'appelant gère la navigation.
fn step_algorithm() -> Option<AlgoChoice> {
    loop {
        println!("\n[1/5] Algorithme de hachage :");
        println!("  1) Argon2  (recommandé)");
        println!("  2) Bcrypt");
        println!("  3) Scrypt");
        println!("  4) Custom provider");
        println!("      ESC = retour  •  Ctrl+C = quitter");

        let input = read_line_esc("Choix [1-4] (défaut: 1) : ")?;

        match input.trim() {
            "" | "1" => return Some(AlgoChoice::Argon2),
            "2" => return Some(AlgoChoice::Bcrypt),
            "3" => return Some(AlgoChoice::Scrypt),
            "4" => loop {
                println!("  ESC = retour au menu  •  Ctrl+C = quitter");
                match read_line_esc("  Chemin du provider : ") {
                    None => break,
                    Some(p) => {
                        let p = p.trim().to_string();
                        if p.is_empty() {
                            println!("  Chemin invalide.");
                        } else {
                            return Some(AlgoChoice::Custom(p));
                        }
                    }
                }
            },
            _ => println!("  Choix invalide — entrez 1, 2, 3 ou 4."),
        }
    }
}

async fn step_username(db: &sea_orm::DatabaseConnection) -> Option<String> {
    loop {
        println!("\n  ESC = étape précédente  •  Ctrl+C = quitter");
        let input = read_line_esc("[2/5] Username : ")?;
        let input = input.trim().to_string();

        if input.is_empty() {
            println!("  Le nom d'utilisateur ne peut pas être vide.");
            continue;
        }
        if BuiltinUserEntity::find_by_username(db, &input)
            .await
            .is_some()
        {
            println!("  Un utilisateur avec ce nom existe déjà.");
            continue;
        }
        return Some(input);
    }
}

async fn step_email(db: &sea_orm::DatabaseConnection) -> Option<String> {
    loop {
        println!("\n  ESC = étape précédente  •  Ctrl+C = quitter");
        let input = read_line_esc("[3/5] Email : ")?;
        let input = input.trim().to_lowercase();

        if input.is_empty() || !input.contains('@') {
            println!("  Email invalide.");
            continue;
        }
        if BuiltinUserEntity::find_by_email(db, &input).await.is_some() {
            println!("  Un utilisateur avec cet email existe déjà.");
            continue;
        }
        return Some(input);
    }
}

/// ESC sur [4/5] = None (retour email).
/// ESC sur [5/5] = boucle interne (ressaisie du mot de passe).
fn step_password() -> Option<String> {
    loop {
        println!("\n  ESC = étape précédente  •  Ctrl + C = quitter");
        let pass1 = read_password_esc("[4/5] Mot de passe : ")?;

        if pass1.len() < 10 {
            println!("  Le mot de passe doit faire au moins 10 caractères.");
            continue;
        }

        println!("  ESC = ressaisir le mot de passe  •  Ctrl + C = quitter");
        let pass2 = match read_password_esc("[5/5] Confirmer le mot de passe : ") {
            None => {
                println!("  Ressaisie du mot de passe.");
                continue;
            }
            Some(p) => p,
        };

        if pass1 != pass2 {
            println!("  Les mots de passe ne correspondent pas. Réessayez.");
            continue;
        }

        return Some(pass1);
    }
}

fn step_review(state: &WizardState) -> ReviewAction {
    let algo = state.algorithm.as_ref().unwrap();
    let username = state.username.as_deref().unwrap();
    let email = state.email.as_deref().unwrap();

    loop {
        println!("\n──────────────────────────────────");
        println!("  Algorithme  : {}", algo.label());
        println!("  Username    : {}", username);
        println!("  Email       : {}", email);
        println!("  Mot de passe: ••••••••");
        println!("──────────────────────────────────");
        println!("  [Entrée] Confirmer  [a] Changer l'algo  [ESC] Retour  [Ctrl+C] Annuler");

        match read_line_esc("") {
            None => return ReviewAction::Back,
            Some(s) => match s.trim().to_lowercase().as_str() {
                "" => return ReviewAction::Confirm,
                "a" => return ReviewAction::ChangeAlgo,
                _ => println!("  Entrée non reconnue."),
            },
        }
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
        .map_err(|e| {
            format!(
                "Impossible de lancer le provider '{}' : {}",
                provider_path, e
            )
        })?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(password.as_bytes())
            .map_err(|e| format!("Erreur écriture stdin : {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Erreur attente provider : {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Le provider a retourné une erreur (code {:?})",
            output.status.code()
        ));
    }

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("Sortie provider invalide (UTF-8) : {}", e))
}

// ─── Entry point ──────────────────────────────────────────────────────────────

pub async fn create_superuser() -> Result<()> {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL doit être défini dans .env");
    let db = sea_orm::Database::connect(&database_url).await?;

    println!("=== Créer un superutilisateur ===  [Ctrl+C pour quitter]");

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
                    // première étape : ESC ignoré — on reste
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

    let algo = state.algorithm.as_ref().unwrap();
    let password = state.password.as_ref().unwrap();
    let hashed =
        hash_password(password, algo).map_err(|e| anyhow::anyhow!("Erreur de hachage : {}", e))?;

    let new_user = ActiveModel {
        username: Set(state.username.unwrap()),
        email: Set(state.email.unwrap()),
        password: Set(hashed),
        is_active: Set(true),
        is_staff: Set(true),
        is_superuser: Set(true),
        ..Default::default()
    };

    new_user.insert(&db).await?;
    println!("\nSuperuser créé avec succès !");

    Ok(())
}
