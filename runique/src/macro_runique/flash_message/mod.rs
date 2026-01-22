//! Macros pour simplifier l'envoi de messages flash
//!
//! # Utilisation
//!
//! Ces macros prennent en premier paramètre la variable `Message`
//! et ensuite le ou les messages à envoyer.
//!
//! # Exemples
//!
//! ```rust
//! use runique::prelude::*;
//!
//! async fn create_user(mut message: Message) -> Response {
//!     // Un seul message
//!     success!(message => "Utilisateur créé avec succès");
//!
//!     // Plusieurs messages
//!     success!(message => "Utilisateur créé", "Email envoyé", "Bienvenue !");
//!
//!     // Messages mixtes
//!     success!(message => "Opération réussie");
//!     info!(message => "Vérifiez votre email");
//!
//!     Redirect::to("/users").into_response()
//! }
//! ```

#[macro_export]
macro_rules! success {
    ($msg:expr => $content:expr) => {
        $msg.success($content).await.unwrap()
    };
    ($msg:expr => $first:expr, $($rest:expr),+ $(,)?) => {
        $msg.success($first).await.unwrap();
        $(
            $msg.success($rest).await.unwrap();
        )+
    };
}

#[macro_export]
macro_rules! error {
    ($msg:expr => $content:expr) => {
        $msg.error($content).await.unwrap()
    };
    ($msg:expr => $first:expr, $($rest:expr),+ $(,)?) => {
        $msg.error($first).await.unwrap();
        $(
            $msg.error($rest).await.unwrap();
        )+
    };
}

#[macro_export]
macro_rules! info {
    ($msg:expr => $content:expr) => {
        $msg.info($content).await.unwrap()
    };
    ($msg:expr => $first:expr, $($rest:expr),+ $(,)?) => {
        $msg.info($first).await.unwrap();
        $(
            $msg.info($rest).await.unwrap();
        )+
    };
}

#[macro_export]
macro_rules! warning {
    ($msg:expr => $content:expr) => {
        $msg.warning($content).await.unwrap()
    };
    ($msg:expr => $first:expr, $($rest:expr),+ $(,)?) => {
        $msg.warning($first).await.unwrap();
        $(
            $msg.warning($rest).await.unwrap();
        )+
    };
}

#[macro_export]
macro_rules! flash_now {
    ($msg_type:ident => $content:expr) => {
        {
            let mut template = vec![];
            template.push($crate::gardefou::composant_middleware::flash_message::FlashMessage::$msg_type($content));
            template
        }
    };
    ($msg_type:ident => $first:expr, $($rest:expr),+ $(,)?) => {
        {
            let mut template = vec![$crate::gardefou::composant_middleware::flash_message::FlashMessage::$msg_type($first)];
            $(
                template.push($crate::gardefou::composant_middleware::flash_message::FlashMessage::$msg_type($rest));
            )+
            template
        }
    };
}

#[cfg(test)]
mod tests {
    use tokio;

    struct MockMessage {
        pub template: Vec<String>,
    }

    impl MockMessage {
        fn new() -> Self {
            Self { template: vec![] }
        }

        async fn success(&mut self, content: &str) -> Result<(), ()> {
            self.template.push(format!("success: {}", content));
            Ok(())
        }

        async fn error(&mut self, content: &str) -> Result<(), ()> {
            self.template.push(format!("error: {}", content));
            Ok(())
        }

        async fn info(&mut self, content: &str) -> Result<(), ()> {
            self.template.push(format!("info: {}", content));
            Ok(())
        }

        async fn warning(&mut self, content: &str) -> Result<(), ()> {
            self.template.push(format!("warning: {}", content));
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_success_macro() {
        let mut msg = MockMessage::new();
        success!(msg => "Test message 1", "Test message 2");
        assert_eq!(
            msg.template,
            vec!["success: Test message 1", "success: Test message 2"]
        );
    }

    #[tokio::test]
    async fn test_error_macro() {
        let mut msg = MockMessage::new();
        error!(msg => "Erreur 1", "Erreur 2");
        assert_eq!(msg.template, vec!["error: Erreur 1", "error: Erreur 2"]);
    }

    #[tokio::test]
    async fn test_info_macro() {
        let mut msg = MockMessage::new();
        info!(msg => "Info 1", "Info 2");
        assert_eq!(msg.template, vec!["info: Info 1", "info: Info 2"]);
    }

    #[tokio::test]
    async fn test_warning_macro() {
        let mut msg = MockMessage::new();
        warning!(msg => "Warning 1", "Warning 2");
        assert_eq!(
            msg.template,
            vec!["warning: Warning 1", "warning: Warning 2"]
        );
    }

    #[tokio::test]
    async fn test_flash_now_macro() {
        let messages = flash_now!(success => "Immediate success", "Another success");
        assert_eq!(messages.len(), 2);
    }
}
