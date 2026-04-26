//! Macros to simplify sending flash messages
//!
//! # Usage
//!
//! These macros take the `Message` variable as the first parameter
//! followed by one or more messages to send.
//!
//! # Examples
//!
//! ```rust
//! use runique::prelude::*;
//! use runique::{info, success};
//!
//! async fn create_user(mut message: Message) -> Response {
//!     // Single message
//!     success!(message => "User created successfully");
//!
//!     // Multiple messages
//!     success!(message => "User created", "Email sent", "Welcome!");
//!
//!     // Mixed messages
//!     success!(message => "Operation successful");
//!     info!(message => "Please check your email");
//!
//!     Redirect::to("/users").into_response()
//! }
//! ```
#![doc = include_str!("../../../doc-tests/flash_message/flash_usage.md")]

#[macro_export]
macro_rules! success {
    ($msg:expr => $content:expr) => {
        let _ = $msg.success($content).await;
    };
    ($msg:expr => $first:expr, $($rest:expr),+ $(,)?) => {
        let _ = $msg.success($first).await;
        $(
            let _ = $msg.success($rest).await;
        )+
    };
}

#[macro_export]
macro_rules! error {
    ($msg:expr => $content:expr) => {
        let _ = $msg.error($content).await;
    };
    ($msg:expr => $first:expr, $($rest:expr),+ $(,)?) => {
        let _ = $msg.error($first).await;
        $(
            let _ = $msg.error($rest).await;
        )+
    };
}

#[macro_export]
macro_rules! info {
    ($msg:expr => $content:expr) => {
        let _ = $msg.info($content).await;
    };
    ($msg:expr => $first:expr, $($rest:expr),+ $(,)?) => {
        let _ = $msg.info($first).await;
        $(
            let _ = $msg.info($rest).await;
        )+
    };
}

#[macro_export]
macro_rules! warning {
    ($msg:expr => $content:expr) => {
        let _ = $msg.warning($content).await;
    };
    ($msg:expr => $first:expr, $($rest:expr),+ $(,)?) => {
        let _ = $msg.warning($first).await;
        $(
            let _ = $msg.warning($rest).await;
        )+
    };
}

#[macro_export]
macro_rules! flash_now {
    ($msg_type:ident => $content:expr) => {
        {
            let template = vec![$crate::flash::FlashMessage::$msg_type($content)];
            template
        }
    };
    ($msg_type:ident => $first:expr, $($rest:expr),+ $(,)?) => {
        {
            let mut template = vec![$crate::flash::FlashMessage::$msg_type($first)];
            $(
                template.push($crate::flash::FlashMessage::$msg_type($rest));
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
        error!(msg => "Error 1", "Error 2");
        assert_eq!(msg.template, vec!["error: Error 1", "error: Error 2"]);
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
