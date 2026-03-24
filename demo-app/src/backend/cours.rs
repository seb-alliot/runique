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

#[derive(serde::Serialize)]
struct GroqMessage {
    role: String,
    content: String,
}

#[derive(serde::Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<GroqMessage>,
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

async fn groq_call(system_prompt: &str, user_message: &str) -> Result<String, String> {
    let api_key = std::env::var("GROQ_API_KEY")
        .map_err(|_| "GROQ_API_KEY manquant dans .env".to_string())?;

    let client = reqwest::Client::new();

    let body = GroqRequest {
        model: GROQ_MODEL.to_string(),
        messages: vec![
            GroqMessage { role: "system".to_string(), content: system_prompt.to_string() },
            GroqMessage { role: "user".to_string(),   content: user_message.to_string() },
        ],
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
        .ok_or_else(|| "Réponse vide de l'API".to_string())
}

pub async fn cours_index(request: &mut Request) -> AppResult<Response> {
    crate::backend::inject_auth(request).await;
    let db = request.engine.db.clone();

    let cours = cour::Entity::find()
        .filter(cour::Column::Lang.eq("fr"))
        .order_by_asc(cour::Column::Ordre)
        .all(&*db)
        .await
        .unwrap_or_default();

    let niveaux = vec!["debutant", "intermediaire", "avance", "specifique"];

    context_update!(request => {
        "title"   => "Cours Rust",
        "cours"   => &cours,
        "niveaux" => &niveaux,
    });

    request.render("cours/cours_index.html")
}

pub async fn cours_detail(slug: &str, request: &mut Request) -> AppResult<Response> {
    crate::backend::inject_auth(request).await;
    let db = request.engine.db.clone();

    let Some(cour) = cour::Entity::find()
        .filter(cour::Column::Slug.eq(slug))
        .one(&*db)
        .await
        .unwrap_or(None)
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let chapitres = chapitre::Entity::find()
        .filter(chapitre::Column::CourId.eq(cour.id))
        .order_by_asc(chapitre::Column::SortOrder)
        .all(&*db)
        .await
        .unwrap_or_default();

    let chapitre_ids: Vec<i32> = chapitres.iter().map(|c| c.id).collect();

    let blocs = cour_block::Entity::find()
        .filter(cour_block::Column::ChapitreId.is_in(chapitre_ids))
        .order_by_asc(cour_block::Column::SortOrder)
        .all(&*db)
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
    if message.len() > MAX_INPUT_LEN {
        let body = ExerciceResponse {
            response: "Je suis uniquement disponible pour évaluer ta réponse à l'exercice en cours.".to_string(),
            attempt: 0,
            status: "fixed_reply".to_string(),
        };
        return Ok(Json(body).into_response());
    }

    // ── Chargement DB ───────────────────────────────────────────
    let db = request.engine.db.clone();

    let Some(cour) = cour::Entity::find()
        .filter(cour::Column::Slug.eq(slug))
        .one(&*db)
        .await
        .unwrap_or(None)
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let Some(cour_ia) = cour_ia::Entity::find()
        .filter(cour_ia::Column::CourId.eq(cour.id))
        .one(&*db)
        .await
        .unwrap_or(None)
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let Some(contrainte) = contrainte_ia::Entity::find()
        .filter(contrainte_ia::Column::Id.eq(cour_ia.contrainte_id))
        .one(&*db)
        .await
        .unwrap_or(None)
    else {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
    };

    // ── État session ────────────────────────────────────────────
    let session_exercise_key = format!("ia_exercise_{}", slug);
    let session_attempt_key  = format!("ia_attempt_{}", slug);

    let stored_exercise: Option<String> = request.session
        .get::<String>(&session_exercise_key)
        .await
        .ok()
        .flatten();

    let attempt: i32 = request.session
        .get::<i32>(&session_attempt_key)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);

    // ── Assemblage du prompt ────────────────────────────────────
    let system_prompt = &contrainte.contrainte_ia;
    let context = format!(
        "{}\n\n---\nContenu du cours :\n{}\n\nContraintes spécifiques :\n{}",
        system_prompt,
        cour_ia.context,
        cour_ia.contraintes,
    );

    let user_message = match &stored_exercise {
        None => {
            // Premier message — initialisation
            "L'utilisateur souhaite commencer un exercice.".to_string()
        }
        Some(exercise) => {
            if attempt >= 3 {
                // Demande de correction après 3 échecs
                format!(
                    "L'utilisateur demande la correction après 3 échecs. \
                     Fournis uniquement la réponse à l'exercice posé : {}",
                    exercise
                )
            } else {
                // Tentative de réponse
                format!(
                    "L'utilisateur a soumis la réponse suivante à l'exercice : \
                     [{}] \
                     Évalue uniquement si cette réponse est correcte par rapport à l'exercice posé : {}",
                    message,
                    exercise
                )
            }
        }
    };

    // ── Appel API Groq ──────────────────────────────────────────
    let ai_response = match groq_call(&context, &user_message).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("groq_call error: {e}");
            return Ok(Json(ExerciceResponse {
                response: "Service temporairement indisponible. Veuillez réessayer.".to_string(),
                attempt,
                status: "fixed_reply".to_string(),
            }).into_response());
        }
    };

    // ── Mise à jour session ─────────────────────────────────────
    let (status, new_attempt) = if stored_exercise.is_none() {
        // Stocker l'exercice généré et reset attempt
        request.session.insert(&session_exercise_key, &ai_response).await.ok();
        request.session.insert(&session_attempt_key, 1_i32).await.ok();
        ("exercise".to_string(), 1)
    } else if attempt >= 3 {
        // Correction fournie — reset session
        request.session.remove::<String>(&session_exercise_key).await.ok();
        request.session.remove::<i32>(&session_attempt_key).await.ok();
        ("correction".to_string(), 0)
    } else {
        // Incrémenter tentative
        let new_attempt = attempt + 1;
        request.session.insert(&session_attempt_key, new_attempt).await.ok();
        // TODO : détecter correct/incorrect depuis ai_response
        ("incorrect".to_string(), new_attempt)
    };

    let body = ExerciceResponse {
        response: ai_response,
        attempt: new_attempt,
        status,
    };

    Ok(Json(body).into_response())
}
