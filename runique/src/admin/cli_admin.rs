// src/bin/createsuperuser.rs
use crate::forms::fields::text::TextField;
use crate::forms::base::FormField;
use crate::middleware::auth::builtin_user::{ActiveModel, BuiltinUserEntity, UserEntity};
use anyhow::Result;
use rpassword::read_password;
use sea_orm::{ActiveModelTrait, Set};

pub async fn create_superuser() -> Result<()> {
    dotenvy::dotenv().ok();
    // Connexion à la base de données
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL doit être défini dans .env");
    let db = sea_orm::Database::connect(&database_url).await?;
    println!("Création d'un superuser Runique");

    // Nom d'utilisateur
    let username = loop {
        println!("Nom d'utilisateur :");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            println!("Le nom d'utilisateur ne peut pas être vide.");
            continue;
        }
        // Vérifie existence
        if BuiltinUserEntity::find_by_username(&db, input)
            .await
            .is_some()
        {
            println!("Un utilisateur avec ce nom existe déjà.");
            continue;
        }
        break input.to_string();
    };

    // Email
    let email = loop {
        println!("Email :");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        if input.is_empty() || !input.contains("@") {
            println!("Email invalide.");
            continue;
        }
        if BuiltinUserEntity::find_by_email(&db, &input)
            .await
            .is_some()
        {
            println!("Un utilisateur avec cet email existe déjà.");
            continue;
        }
        break input;
    };

    // Mot de passe
    let password = loop {
        println!("Mot de passe :");
        let pass1 = read_password()?;
        println!("Confirmer le mot de passe :");
        let pass2 = read_password()?;

        if pass1 != pass2 {
            println!("Les mots de passe ne correspondent pas. Réessayez.");
            continue;
        }
        if pass1.len() < 10 {
            println!("Le mot de passe doit faire au moins 10 caractères.");
            continue;
        }
        break pass1;
    };

    // Hash du mot de passe via le système de validation de TextField (DRY DRY DRY)
    let mut password_field = TextField::password("Mot de passe")
        .required()
        .min_length(10, "Le mot de passe doit faire au moins 10 caractères.");

    password_field.base.value = password.clone();

    if !password_field.validate() {
        println!("{}", password_field.base.error.as_deref().unwrap_or("Mot de passe invalide"));
        return Ok(());
    }

    if let Err(e) = password_field.finalize() {
        println!("Erreur lors de la validation du mot de passe : {}", e);
        return Ok(());
    }
    let hashed_password = password_field.base.value.clone();
        // Création du superuser
        let new_user = ActiveModel {
            username: Set(username),
            email: Set(email),
            password: Set(hashed_password),
            is_active: Set(true),
            is_staff: Set(true),
            is_superuser: Set(true),
            ..Default::default()
        };

    new_user.insert(&db).await?;

    println!(" Superuser créé avec succès !");

    Ok(())
}
