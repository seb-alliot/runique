use crate::aliases::Messages;
use crate::constante::FLASH_KEY;
use crate::flash::flash_struct::FlashMessage;
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use tower_sessions::Session;

#[derive(Clone, Debug)]
pub struct Message {
    pub session: Session,
}

impl<S> FromRequestParts<S> for Message
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let session = parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Self { session })
    }
}

impl Message {
    async fn push(&self, msg: FlashMessage) {
        let mut messages = self
            .session
            .get::<Messages>(FLASH_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or_default();

        messages.push(msg);
        let _ = self.session.insert(FLASH_KEY, messages).await;
    }

    pub async fn success(&self, msg: impl Into<String>) {
        self.push(FlashMessage::success(msg)).await
    }
    pub async fn error(&self, msg: impl Into<String>) {
        self.push(FlashMessage::error(msg)).await
    }
    pub async fn info(&self, msg: impl Into<String>) {
        self.push(FlashMessage::info(msg)).await
    }
    pub async fn warning(&self, msg: impl Into<String>) {
        self.push(FlashMessage::warning(msg)).await
    }
    pub async fn get_all(&self) -> Messages {
        let messages = self
            .session
            .get::<Messages>(FLASH_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or_default();

        // Supprime après lecture pour effet “flash”
        let _ = self.session.remove::<Messages>(FLASH_KEY).await;
        messages
    }
}
