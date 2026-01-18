use crate::settings::Settings;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::Tera;

#[derive(Clone)]
pub struct AppState {
    pub tera: Arc<Tera>,
    pub settings: Arc<Settings>,
    pub db: DatabaseConnection,
    pub url_registry: Arc<RwLock<HashMap<String, String>>>,
}

impl AppState {
    pub fn new(
        tera: Arc<Tera>,
        settings: Arc<Settings>,
        db: DatabaseConnection,
        url_registry: Arc<RwLock<HashMap<String, String>>>,
    ) -> Self {
        Self {
            tera,
            settings,
            db,
            url_registry,
        }
    }
}

// Extraction de Tera
impl axum::extract::FromRef<AppState> for Arc<Tera> {
    fn from_ref(state: &AppState) -> Self {
        state.tera.clone()
    }
}

// Extraction des Settings (nécessaire pour ExtractForm)
impl axum::extract::FromRef<AppState> for Arc<Settings> {
    fn from_ref(state: &AppState) -> Self {
        state.settings.clone()
    }
}

// Extraction de la connexion BDD (pour tes vues)
impl axum::extract::FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}
