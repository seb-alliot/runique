use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "runique")]
#[command(about = "CLI pour créer des projets Runique", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Créer un nouveau projet Runique
    New { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name } => create_new_project(&name)?,
    }
    Ok(())
}

fn create_new_project(name: &str) -> Result<()> {
    println!("🦀 Création du projet '{}'...", name);
    let project_dir = Path::new(name);
    // src/url.rs
    let url_rs = r#"// src/url.rs
use crate::views;
use runique::{post, urlpatterns, view, Router};

pub fn routes() -> Router {
    urlpatterns! {

        // Vos routes ici
        // Exemple :
        view!("index", crate::views::index),
        }
    }
"#;

    // src/main.rs
    let main_rs = r#"// src/main.rs
use runique::prelude::*;

mod forms;
mod models;
mod url;
mod views;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "change_your_secret_key")
        .build();

    settings.validate_allowed_hosts();

    RuniqueApp::new(settings)
        .routes(url::routes())
        .with_database(db)
        .with_static_files()?
        .with_allowed_hosts(
            env::var("ALLOWED_HOSTS")
            .ok()
            .map(|s| s.split(',').map(|h| h.to_string()).collect()),
        )
        .with_sanitize_text_inputs(false)
        .with_security_headers(CspConfig::strict())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
    }
"#;

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
runique = {{ version = "1.0.3", features = ["sqlite"] }}
serde = "1.0"
"#,
        name
    );
    // .env
    let env_file = r#"# src/.env
# Server Configuration
IP_SERVER=127.0.0.1
PORT=3000

DEBUG=true
# Database Configuration (SQLite par défaut)

# Secret key for csrf management
SECRETE_KEY=your_secret_key_here

# A completer pour toute bdd autre que SQLite
DB_ENGINE=sqlite
DB_USER=username
DB_PASSWORD=password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=database_name

DATABASE_URL=postgres://postgres:password@localhost:5432/database_name
# Allowed hosts for production
ALLOWED_HOSTS=exemple.com,www.exemple.com,.api.exemple.com,localhost,127.0.0.1
"#;

    // .gitignore
    let gitignore = r#"# .gitignore
/target/
*.db
*.sqlite
.env
"#;
    let formulaire =
r#"// src/forms.rs
// Your form example here
// use runique::prelude::*;
// use runique::prelude::*;

// #[derive(Deserialize)]
// pub struct UsernameForm {
//     pub form: Forms,
// }

// // Implémenter Serialize manuellement
// impl Serialize for UsernameForm {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut state = serializer.serialize_struct("UsernameForm", 1)?;
//         state.serialize_field("form_html", &self.form)?;
//         state.end()
//     }
// }

// impl RuniqueForm for UsernameForm {
//     fn register_fields(form: &mut Forms) {
//         form.register_field("username", "Nom d'utilisateur", &CharField::new());
//     }

//     fn validate_fields(form: &mut Forms, raw_data: &HashMap<String, String>) {
//         form.require("username", &CharField::new(), raw_data);
//     }

//     fn from_form(form: Forms) -> Self {
//         Self { form }
//     }

//     fn get_form(&self) -> &Forms {
//         &self.form
//     }

//     fn get_form_mut(&mut self) -> &mut Forms {
//         &mut self.form
//     }
// }
"#;
    let user_exemple = r#"// src/models/users.rs
// for exemple purposes only
// use runique::prelude::*;
// use runique::impl_objects;
// use runique::sea_orm;
// use runique::sea_orm::entity::prelude::*;
// use runique::serde::{Deserialize, Serialize};
// use runique::DeriveModelForm;

// #[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveModelForm, Serialize, Deserialize)]
// #[sea_orm(table_name = "users")]
// pub struct Model {
//     #[sea_orm(primary_key)]
//     pub id: i32,
//     pub username: String,
//     pub email: String,
//     pub password: String,
//     pub age: i32,
//     pub created_at: DateTime,
// }

// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
// pub enum Relation {}

// impl ActiveModelBehavior for ActiveModel {}

// impl_objects!(Entity);
"#;
    let mod_rs_content = r#"// src/models/mod.rs
pub mod users;
"#;
    let view_rs_content = r#"// src/views.rs
// pub mod users;
use runique::prelude::*;
pub async fn index(template: Template) -> Response {
    let ctx = context! {
        "title", "Bienvenue dans Runique";
    };
    template.render("index.html", &ctx)
}
"#;

    let htmtl_example = r#"<!-- templates/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }}</title>
</head>
<body>
    <h1>{{ title }}</h1>
    <p>Ceci est une page d'exemple pour votre projet Runique.</p>
</body>
</html>
"#;
    // Créer la structure de dossiers
    fs::create_dir_all(project_dir)?;
    fs::create_dir_all(project_dir.join("src/models"))?;
    fs::create_dir_all(project_dir.join("src/static/css"))?;
    fs::create_dir_all(project_dir.join("src/static/js"))?;
    fs::create_dir_all(project_dir.join("src/static/images"))?;
    fs::create_dir_all(project_dir.join("src/media"))?;
    fs::create_dir_all(project_dir.join("templates"))?;

    // Cargo.toml
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // src/main.rs
    fs::write(project_dir.join("src/main.rs"), main_rs)?;

    // src/forms.rs
    fs::write(project_dir.join("src/forms.rs"), formulaire)?;

    // src/url.rs
    fs::write(project_dir.join("src/url.rs"), url_rs)?;

    // src/views.rs
    fs::write(project_dir.join("src/views.rs"), view_rs_content)?;

    // src/models/mod.rs
    fs::write(
        project_dir.join("src/models/mod.rs"),mod_rs_content)?;
    fs::write(project_dir.join("src/models/users.rs"), user_exemple)?;

    // templates/index.html
    fs::write(project_dir.join("templates/index.html"), htmtl_example)?;

    // .env
    fs::write(project_dir.join(".env"), env_file)?;

    // .gitignore
    fs::write(project_dir.join(".gitignore"), gitignore)?;

    println!("   Projet '{}' créé avec succès !", name);
    println!("   Structure créée :");
    println!("   {}/", name);
    println!("   ├── src/");
    println!("   │   ├── models/");
    println!("   │   ├── static/");
    println!("   │   │   ├── css/");
    println!("   │   │   ├── js/");
    println!("   │   │   └── images/");
    println!("   │   ├── media/");
    println!("   │   ├── forms.rs");
    println!("   │   ├── main.rs");
    println!("   │   ├── url.rs");
    println!("   │   └── views.rs");
    println!("   ├── templates/");
    println!("   ├── .env");
    println!("   ├── .gitignore");
    println!("   └── Cargo.toml");
    println!("\n     Pour démarrer :");
    println!("   cd {}", name);
    println!("   cargo run");

    Ok(())
}
