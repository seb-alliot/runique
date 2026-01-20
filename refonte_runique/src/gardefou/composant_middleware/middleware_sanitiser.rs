use crate::moteur_engine::engine_struct::RuniqueEngine;
use axum::{extract::State, extract::Request, middleware::Next, response::Response};
use std::sync::Arc;

use http_body_util::BodyExt;
use serde_json::Value;

use crate::formulaire::utils::sanitizer;

/// Middleware de sanitisation automatique des formulaires
///
/// Sanitise automatiquement tous les champs String des formulaires
/// si `settings.sanitize_inputs` est activé.
pub async fn sanitize_middleware(
    State(engine): State<Arc<RuniqueEngine>>,
    mut request: Request,
    next: Next,
) -> Response {
    // On vérifie si la sanitisation est activée dans la config globale
    if !engine.config.security.sanitize_inputs {
        return next.run(request).await;
    }

    // Récupérer le Content-Type
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if content_type.contains("multipart/form-data") {
        // Pour l'instant, on ne gère pas multipart/form-data, on sépare le traitement
        // Gestion des uploads de fichiers à part
        return next.run(request).await;
    }
    // Sanitiser selon le type de contenu
    if content_type.contains("application/x-www-form-urlencoded") {
        // Formulaire HTML classique
        request = sanitize_form_urlencoded(request).await;
    } else if content_type.contains("application/json") {
        // JSON (API)
        request = sanitize_json(request).await;
    }

    // multipart/form-data pourrait être ajouté plus tard

    next.run(request).await
}

/// Sanitise un formulaire URL-encoded
async fn sanitize_form_urlencoded(request: Request) -> Request {
    let (parts, body) = request.into_parts();

    // Extraire le body
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => return Request::from_parts(parts, Body::empty()),
    };

    // Convertir en string
    let form_data = match std::str::from_utf8(&bytes) {
        Ok(s) => s,
        Err(_) => return Request::from_parts(parts, Body::from(bytes)),
    };

    // Sanitiser chaque champ
    let sanitized = sanitize_urlencoded_string(form_data);

    // Recréer la requête avec les données propres
    let new_body = Body::from(sanitized);
    Request::from_parts(parts, new_body)
}

/// Sanitise une chaîne URL-encoded
fn sanitize_urlencoded_string(data: &str) -> String {
    data.split('&')
        .map(|pair| {
            if let Some((key, value)) = pair.split_once('=') {
                // On vérifie si la CLÉ est sensible avant de toucher à la valeur
                if sanitizer::is_sensitive_field(key) {
                    return pair.to_string();
                }

                let decoded = urlencoding::decode(value).unwrap_or_default();
                let sanitized = sanitizer::auto_sanitize(&decoded);
                let encoded = urlencoding::encode(&sanitized);

                format!("{}={}", key, encoded)
            } else {
                pair.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("&")
}

/// Sanitise un body JSON
async fn sanitize_json(request: Request) -> Request {
    let (parts, body) = request.into_parts();

    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => return Request::from_parts(parts, Body::empty()),
    };

    // Parser le JSON
    let mut json: Value = match serde_json::from_slice(&bytes) {
        Ok(j) => j,
        Err(_) => return Request::from_parts(parts, Body::from(bytes)),
    };

    // Sanitiser récursivement
    sanitize_json_value("", &mut json);

    // Resérialiser
    let sanitized_bytes = match serde_json::to_vec(&json) {
        Ok(b) => b,
        Err(_) => bytes.to_vec(),
    };

    let new_body = Body::from(sanitized_bytes);
    Request::from_parts(parts, new_body)
}

/// Sanitise récursivement toutes les strings dans un JSON
fn sanitize_json_value(key: &str, value: &mut Value) {
    match value {
        Value::String(s) => {
            if !sanitizer::is_sensitive_field(key) {
                *s = sanitizer::auto_sanitize(s);
            }
        }
        Value::Object(map) => {
            for (k, v) in map {
                sanitize_json_value(k, v);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                sanitize_json_value("", item);
            }
        }
        _ => {}
    }
}
