# UML — Modules transverses (errors, flash, mailer, password, i18n)

Modules utilitaires sans flux de données propre, regroupés ici pour la complétude.

## errors — `RuniqueError` / `ErrorContext`

[`errors/error.rs`](../../../runique/src/errors/error.rs)

```mermaid
classDiagram
    class ErrorContext {
        +u16 status_code
        +ErrorType error_type
        +String timestamp / title / message
        +Option~String~ debug_repr / details
        +Option~TemplateInfo~ template_info
        +Option~RequestInfo~ request_info
        +Vec~StackFrame~ stack_trace
        +EnvironmentInfo environment
    }
    class ErrorType {
        <<enum>> Template / NotFound / Internal / Database / Validation / BadRequest
    }
    class RuniqueError {
        <<enum>>
        Build / Internal / Forbidden / NotFound
        Validation(String) / Database(String)
        Template « message + Option~usize~ line »
        +template(&tera::Error) Self
    }
    class StackFrame {
        +usize level
        +String message
        +Option~String~ debug_repr / location
    }
    class TemplateInfo { +name +source +line_number +available_templates }
    class RequestInfo { +method +path +query +headers }
    class EnvironmentInfo { +debug_mode +rust_version « build.rs » +runique_version }
    ErrorContext *-- ErrorType
    ErrorContext *-- "*" StackFrame
    ErrorContext *-- "0..1" TemplateInfo
    ErrorContext *-- "0..1" RequestInfo
    ErrorContext *-- EnvironmentInfo
```

`RuniqueError::Template` lit sa ligne dans le span de l'erreur Tera (`tera_error_line`),
plus par regex sur le texte. `rust_version` est relevé à la compilation (`build.rs`), pas
par `rustc --version` à l'exécution. La version de l'application vient de
`app_package(base_dir)` (lecture du `Cargo.toml` du projet, page de debug seulement) et
arrive dans le contexte (`app_name`, `app_version`).

`debug_repr`/`stack_trace`/`request_info` ne doivent être exposés qu'en `debug_mode`
(le rendu HTML d'erreur masque ces champs en prod — à vérifier côté template d'erreur).

## flash — messages éphémères

[`flash/flash_struct.rs`](../../../runique/src/flash/flash_struct.rs),
[`flash/flash_manager.rs`](../../../runique/src/flash/flash_manager.rs)

```mermaid
classDiagram
    class Message {
        +Session session
        +push(FlashMessage) [insert traced]
        +success/error/info/warning(msg)
        +get_all() Messages [read+remove]
    }
    class FlashMessage {
        +String content
        +MessageLevel level
    }
    class MessageLevel {
        <<enum>> Success / Error / Info / Warning
        +as_css_class() &str
    }
    Message ..> FlashMessage
    FlashMessage *-- MessageLevel
```

Écritures session **tracées** (`.trace`) ; lectures = défaut vide bénin. Audit clean.

## mailer — `Email` / `MailerConfig`

[`utils/mailer/mod.rs`](../../../runique/src/utils/mailer/mod.rs)

```mermaid
classDiagram
    class MailerConfig {
        +MailerBackend backend
        +String host / username / password / from
        +u16 port
        +bool starttls
        +from_env() Option~Self~
    }
    class Email {
        +template(tera, tpl, ctx) Result
        +html(body) / text(body)
        +send() async Result
    }
    class MailerBackend { <<enum>> Smtp / Console }
    MailerConfig *-- MailerBackend
    Email ..> MailerConfig : MAILER_CONFIG global
```

## password — hash (argon2/bcrypt/scrypt)

[`utils/password/mod.rs`](../../../runique/src/utils/password/mod.rs)

```mermaid
classDiagram
    class PasswordService {
        +hash(password) Result~String~
        +verify(password, hash) bool
    }
    class Manual {
        <<algorithme>>
        +hash(password, algo) Result
        +verify(password, hash) bool
    }
    class PasswordConfig {
        <<enum>>
        Delegated(External)
        Auto(AutoConfig)
        Manual(Manual)
        Custom(Box~dyn PasswordHandler~)
    }
    PasswordService --> PasswordConfig
    note for PasswordService "dummy_hash pour timing-constant (anti user-enumeration)"
```

Configuration globale : `password_init(config)` (`OnceLock`, le premier appel gagne,
`build()` l'appelle avec `config.password`) puis `password_get()` partout — la
réinitialisation intégrée comprise. Le hachage automatique (`Auto`) ne touche qu'une
valeur venue de la requête : elle est hachée même si elle a la forme d'un hash ; seule
une valeur posée par le code peut en être un. À la vérification, un hash demandant un
coût au-delà des plafonds est refusé avant tout calcul : argon2 `m ≤ 64 Mio, t ≤ 8,
p ≤ 4`, scrypt `log_n ≤ 17, r ≤ 8, p ≤ 4`, bcrypt `cost ≤ 14`.

## i18n — `Lang`

[`utils/trad/switch_lang.rs`](../../../runique/src/utils/trad/switch_lang.rs)

```mermaid
classDiagram
    class Lang {
        <<enum 9 langues>>
        +code() &str
        +current_lang() / set_lang()
    }
    note for Lang "t(key) / tf(key, args) — résolveur imbriqué (clés a.b.c)"
```

## Anomalies / flux suspects

### 🟡 TR1 — Exposition conditionnelle des détails d'erreur
`ErrorContext` porte `debug_repr`, `stack_trace`, `request_info` (headers inclus). À
**garantir** que le template d'erreur ne les rend qu'en `debug_mode` — une fuite en prod
exposerait stack traces + en-têtes de requête. À vérifier dans le template `errors/*.html`.

### 🟡 TR2 — `MailerConfig.password` en clair dans les logs — ✅ CORRIGÉ
**Corrigé (2.1.21).** `MailerConfig` dérivait `Debug` → `{:?}` imprimait le mot de passe SMTP en
clair. Impl `Debug` manuelle masquant `password: "***"`. (Le secret reste en `String` en mémoire,
inévitable pour l'auth SMTP, mais n'est plus exposable via un log.)
