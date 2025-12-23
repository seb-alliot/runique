use crate::formulaire::field::RustiField;
use std::collections::HashMap;
use serde_json::Value;
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
use axum::{
    extract::{FromRequest, Form},
    http::Request,
    body::Body,
    response::{IntoResponse, Response},
};

// --- 1. La Structure de Base ---

#[derive(Serialize, Deserialize, Clone)]

pub struct Forms {
    #[serde(default)] // Ajoute ceci ici
    pub errors: HashMap<String, String>,

    #[serde(default)] // Et ajoute ceci ici
    pub cleaned_data: HashMap<String, Value>,
}

impl Forms {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
            cleaned_data: HashMap::new(),
        }
    }

    /// Traite un champ et stocke la valeur nettoyée ou l'erreur
    pub fn field<F: RustiField>(
        &mut self,
        name: &str,
        field: &F,
        raw_value: &str
    ) -> Option<F::Output>
    where F::Output: Serialize + Clone
    {
        match field.process(raw_value) {
            Ok(value) => {
                if let Ok(json_val) = serde_json::to_value(value.clone()) {
                    self.cleaned_data.insert(name.to_string(), json_val);
                }
                Some(value)
            },
            Err(e) => {
                self.errors.insert(name.to_string(), e);
                None
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn is_not_valid(&self) -> bool {
        !self.is_valid()
    }

    pub fn get_value<T: DeserializeOwned + 'static + Clone + Send + Sync>(&self, field_name: &str) -> Option<T> {
        self.cleaned_data.get(field_name).and_then(|value| {
            serde_json::from_value(value.clone()).ok()
        })
    }
}

// --- 2. Le Trait (Le Contrat du Framework) ---

pub trait FormulaireTrait: Send {
    fn new() -> Self;
    /// Cette méthode fait le lien entre les données brutes et la validation
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool;
}


pub struct AxumForm<T>(pub T);

impl<S, T> FromRequest<S> for AxumForm<T>
where
    S: Send + Sync,
    T: FormulaireTrait + Send + 'static,
{
    type Rejection = Response;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        // On utilise l'extracteur standard d'Axum.
        // Comme le middleware a recréé le corps avec Body::from(bytes), Form peut le relire.
        let Form(payload) = Form::<HashMap<String, String>>::from_request(req, state)
            .await
            .map_err(|e| e.into_response())?;

        let mut form_instance = T::new();
        form_instance.validate(&payload); //

        Ok(AxumForm(form_instance))
    }
}
