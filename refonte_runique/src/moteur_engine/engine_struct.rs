use std::sync::Arc;
use std::collections::HashMap;
use std::sync::RwLock;
use tera::Tera;
use crate::config_runique::config_struct::RuniqueConfig;
use crate::gardefou::middleware_struct::GardeFou;
use crate::data_base_runique::DatabaseConfig;

pub struct RuniqueEngine {
    pub config: RuniqueConfig,
    pub tera: Arc<Tera>,
    pub db: DatabaseConfig,
    pub garde: GardeFou,
    pub url_registry: Arc<RwLock<HashMap<String, String>>>,
}

impl RuniqueEngine {
    pub fn new(tera: Tera, garde_fou: GardeFou) -> Self {
        Self {
            tera: Arc::new(tera),
            garde: garde_fou,
            db: DatabaseConfig::from_env().unwrap().build(),
            config: RuniqueConfig::from_env(),
            url_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
