pub mod config_runique;
pub mod moteur_engine;
pub mod runique_body;
pub mod gardefou;
pub mod runiqueapp;
pub mod formulaire;
pub mod macro_runique;
pub mod data_base_runique;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialisation de la config (à adapter selon tes fichiers .toml/.env)
    // Ici on crée une instance manuelle pour le test
    let config = RuniqueConfig::default(); // Assure-toi que Default est implémenté

    // 2. Connexion DB (SeaORM)
    let db_url = "sqlite::memory:";
    let db = Database::connect(db_url).await?;

    // 3. Définition des routes
    let router = Router::new()
        .route("/", get(home_view));

    // 4. Construction de l'application Runique
    let app = RuniqueAppBuilder::new(config, db)
        .with_router(router)
        .build()
        .await?;

    // 5. Lancement
    app.run().await?;

    Ok(())
}