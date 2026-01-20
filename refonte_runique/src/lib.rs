// src/lib.rs

// Déclaration des dossiers (modules)
pub mod config_runique;
pub mod data_base_runique;
pub mod formulaire;

pub mod gardefou;
pub mod macro_runique;
pub mod moteur_engine;
pub mod request_context;
pub mod runique_body;


// Déclaration des fichiers racine

// Re-exports pour simplifier la vie de l'utilisateur dans main.rs
pub use config_runique::config_struct::RuniqueConfig;
pub use moteur_engine::engine_struct::RuniqueEngine;