use crate::entities::{chapitre, contrainte_ia, cour, cour_block, cour_ia};
use runique::prelude::*;

#[derive(serde::Deserialize)]
pub struct ExerciceInput {
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct ExerciceResponse {
    pub response: String,
    pub attempt: i32,
    pub status: String,
}

const MAX_INPUT_LEN: usize = 500;
const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/chat/completions";
const GROQ_MODEL: &str = "llama-3.3-70b-versatile";
const GROQ_MAX_TOKENS: u32 = 512;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct ConversationMessage {
    role: String,
    content: String,
}

#[derive(serde::Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<ConversationMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(serde::Deserialize)]
struct GroqResponse {
    choices: Vec<GroqChoice>,
}

#[derive(serde::Deserialize)]
struct GroqChoice {
    message: GroqChoiceMessage,
}

#[derive(serde::Deserialize)]
struct GroqChoiceMessage {
    content: String,
}

async fn groq_call(system_prompt: &str, history: &[ConversationMessage]) -> Result<String, String> {
    let api_key =
        std::env::var("GROQ_API_KEY").map_err(|_| "GROQ_API_KEY missing from .env".to_string())?;

    let client = reqwest::Client::new();

    let mut messages = vec![ConversationMessage {
        role: "system".to_string(),
        content: system_prompt.to_string(),
    }];
    messages.extend_from_slice(history);

    let body = GroqRequest {
        model: GROQ_MODEL.to_string(),
        messages,
        max_tokens: GROQ_MAX_TOKENS,
        temperature: 0.3,
    };

    let res = client
        .post(GROQ_API_URL)
        .bearer_auth(&api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Groq API error: {}", res.status()));
    }

    let groq_res: GroqResponse = res.json().await.map_err(|e| e.to_string())?;

    groq_res
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| "Empty API response".to_string())
}

pub async fn cours_index(request: &mut Request) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    let db = request.engine.db.clone();

    let cours = search!(cour::Entity => Lang eq "fr", asc Ordre)
        .all(&db)
        .await
        .unwrap_or_default();

    let niveaux = vec!["debutant", "intermediaire", "avance", "specifique"];

    context_update!(request => {
        "title"   => "Rust Courses",
        "cours"   => &cours,
        "niveaux" => &niveaux,
    });

    request.render("cours/cours_index.html")
}

pub async fn cours_detail(slug: &str, request: &mut Request) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    let db = request.engine.db.clone();

    let Some(cour) = search!(cour::Entity => Slug eq slug)
        .first(&db)
        .await
        .unwrap_or(None)
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let chapitres = search!(chapitre::Entity => CourId eq cour.id, asc SortOrder)
        .all(&db)
        .await
        .unwrap_or_default();

    let chapitre_ids: Vec<i32> = chapitres.iter().map(|c| c.id).collect();

    let blocs = search!(cour_block::Entity => ChapitreId in (chapitre_ids), asc SortOrder)
        .all(&db)
        .await
        .unwrap_or_default();

    context_update!(request => {
        "title"     => &cour.title,
        "cour"      => &cour,
        "chapitres" => &chapitres,
        "blocs"     => &blocs,
    });

    request.render("cours/cours_detail.html")
}

pub async fn cours_exercice(
    slug: &str,
    input: ExerciceInput,
    request: &mut Request,
) -> AppResult<Response> {
    // ── Validation input ────────────────────────────────────────
    let message = input.message.trim().to_string();
    if message.is_empty() || message.len() > MAX_INPUT_LEN {
        let body = ExerciceResponse {
            response: "I am only available to evaluate your response to the current exercise."
                .to_string(),
            attempt: 0,
            status: "fixed_reply".to_string(),
        };
        return Ok(Json(body).into_response());
    }

    // ── Chargement DB ───────────────────────────────────────────
    let db = request.engine.db.clone();

    let Some(cour) = search!(cour::Entity => Slug eq slug)
        .first(&db)
        .await
        .unwrap_or(None)
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let Some(cour_ia) = search!(cour_ia::Entity => CourId eq cour.id)
        .first(&db)
        .await
        .unwrap_or(None)
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let Some(contrainte) = search!(contrainte_ia::Entity => Id eq cour_ia.contrainte_id)
        .first(&db)
        .await
        .unwrap_or(None)
    else {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
    };

    // ── Session ─────────────────────────────────────────────────
    let history_key = format!("ia_history_{slug}");
    let attempt_key = format!("ia_attempt_{slug}");

    let mut history: Vec<ConversationMessage> = request
        .session
        .get::<Vec<ConversationMessage>>(&history_key)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    let attempt: i32 = request
        .session
        .get::<i32>(&attempt_key)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);

    // ── Assemblage du prompt système ────────────────────────────
    let system_prompt = format!(
        "{}\n\n---\nContenu du cours :\n{}\n\nContraintes spécifiques :\n{}",
        contrainte.contrainte_ia, cour_ia.context, cour_ia.contraintes,
    );

    // ── Ajout du message utilisateur à l'historique ─────────────
    // Le message est encapsulé pour éviter les injections de prompt.
    let safe_content = if history.is_empty() {
        "The user wants to start an exercise.".to_string()
    } else {
        format!("User response: [{message}]")
    };

    history.push(ConversationMessage {
        role: "user".to_string(),
        content: safe_content,
    });

    // ── Appel API Groq avec historique complet ──────────────────
    let ai_response = match groq_call(&system_prompt, &history).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("groq_call error: {e}");
            return Ok(Json(ExerciceResponse {
                response: "Service temporarily unavailable. Please try again.".to_string(),
                attempt,
                status: "fixed_reply".to_string(),
            })
            .into_response());
        }
    };

    // ── Ajout réponse IA à l'historique ─────────────────────────
    history.push(ConversationMessage {
        role: "assistant".to_string(),
        content: ai_response.clone(),
    });

    // ── Détection du statut ─────────────────────────────────────
    let trimmed = ai_response.trim().to_lowercase();
    let (status, new_attempt, reset) =
        if trimmed.starts_with("correct") && !trimmed.starts_with("incorrect") {
            ("correct".to_string(), 0, true)
        } else if trimmed.starts_with("incorrect") {
            let new_attempt = attempt + 1;
            ("incorrect".to_string(), new_attempt, false)
        } else if attempt >= 3 {
            // Correction fournie après 3 échecs
            ("correction".to_string(), 0, true)
        } else {
            // Phase setup : sélection de type ou génération d'exercice
            ("exercise".to_string(), attempt, false)
        };

    // ── Mise à jour session ─────────────────────────────────────
    if reset {
        request
            .session
            .remove::<Vec<ConversationMessage>>(&history_key)
            .await
            .ok();
        request.session.remove::<i32>(&attempt_key).await.ok();
    } else {
        request.session.insert(&history_key, &history).await.ok();
        request.session.insert(&attempt_key, new_attempt).await.ok();
    }

    Ok(Json(ExerciceResponse {
        response: ai_response,
        attempt: new_attempt,
        status,
    })
    .into_response())
}
