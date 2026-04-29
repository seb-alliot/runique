use runique::utils::mailer::{Email, MailerBackend, MailerConfig, dispatch_email, mailer_init};

fn init_console_mailer() {
    mailer_init(MailerConfig {
        backend: MailerBackend::Console,
        from: "test@runique.rs".to_string(),
        host: String::new(),
        port: 0,
        username: String::new(),
        password: String::new(),
        starttls: false,
    });
}

#[tokio::test]
async fn test_dispatch_email_console() {
    init_console_mailer();
    let result = dispatch_email("user@example.com", "Sujet test", "Bonjour depuis le test").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_email_builder_text_console() {
    init_console_mailer();
    let result = Email::new()
        .to("user@example.com")
        .subject("Builder test")
        .text("Corps texte")
        .send()
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_email_builder_html_console() {
    init_console_mailer();
    let result = Email::new()
        .to("user@example.com")
        .subject("HTML test")
        .html("<h1>Bonjour</h1>")
        .send()
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_email_sans_contenu_renvoie_erreur() {
    init_console_mailer();
    let result = Email::new()
        .to("user@example.com")
        .subject("Vide")
        .send()
        .await;
    assert!(result.is_err());
}
