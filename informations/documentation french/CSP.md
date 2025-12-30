# Documentation CSP de Rusti

## Content Security Policy dans le Framework Rusti

### Vue d'ensemble

Rusti fournit une implémentation complète de Content Security Policy (CSP) qui aide à protéger vos applications web contre les attaques de type Cross-Site Scripting (XSS), le clickjacking et d'autres injections de code malveillant. Le middleware CSP vous permet de contrôler quelles ressources le navigateur est autorisé à charger pour votre application.

### Qu'est-ce que Content Security Policy ?

Content Security Policy est un standard de sécurité qui aide à prévenir différents types d'attaques en déclarant quelles ressources dynamiques sont autorisées à se charger. Il fonctionne en définissant une liste blanche de sources de confiance pour le contenu comme les scripts, les styles, les images et autres ressources.

### Fonctionnalités principales

- **Configuration par directives** : Contrôle granulaire sur différents types de ressources
- **Génération de nonces** : Génération automatique de nonces cryptographiques pour les scripts et styles inline
- **Mode rapport seul** : Testez les politiques CSP sans bloquer les ressources
- **Rapport de violations** : Configurez des endpoints pour recevoir les rapports de violation CSP
- **Intégration middleware** : Intégration transparente avec le système de middleware de Rusti

### Configuration

#### Configuration de base

Ajoutez le middleware CSP à la pile de middlewares de votre application :

```rust
use rusti::middleware::csp::{CspMiddleware, CspConfig, CspDirective};

let csp_config = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'self'".to_string()]),
        CspDirective::ScriptSrc(vec!["'self'".to_string(), "'unsafe-inline'".to_string()]),
        CspDirective::StyleSrc(vec!["'self'".to_string(), "'unsafe-inline'".to_string()]),
    ],
    report_only: false,
    report_uri: None,
};

let csp_middleware = CspMiddleware::new(csp_config);
```

#### Directives disponibles

Rusti supporte toutes les directives CSP standard :

- **default-src** : Fallback pour tous les types de ressources
- **script-src** : Contrôle les sources JavaScript
- **style-src** : Contrôle les sources CSS
- **img-src** : Contrôle les sources d'images
- **font-src** : Contrôle les sources de polices
- **connect-src** : Contrôle les connexions fetch, XHR, WebSocket
- **media-src** : Contrôle les sources audio et vidéo
- **object-src** : Contrôle les éléments `<object>`, `<embed>`, et `<applet>`
- **frame-src** : Contrôle les sources de frames
- **worker-src** : Contrôle les sources Worker, SharedWorker et ServiceWorker
- **manifest-src** : Contrôle les sources de manifest d'application
- **base-uri** : Contrôle l'URL de base du document
- **form-action** : Contrôle les cibles de soumission de formulaires
- **frame-ancestors** : Contrôle quelles sources peuvent intégrer la page

#### Utilisation des nonces

Les nonces (number used once) sont des jetons cryptographiques qui permettent des scripts ou styles inline spécifiques tout en bloquant les autres :

```rust
let csp_config = CspConfig {
    directives: vec![
        CspDirective::ScriptSrc(vec![
            "'self'".to_string(),
            "'nonce-{nonce}'".to_string(),
        ]),
    ],
    report_only: false,
    report_uri: None,
};
```

Dans vos templates, accédez au nonce :

```html
<script nonce="{{ csp_nonce }}">
    // Votre script inline ici
    console.log('Protégé par un nonce CSP');
</script>
```

### Mode rapport seul

Testez votre politique CSP sans bloquer les ressources :

```rust
let csp_config = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'self'".to_string()]),
    ],
    report_only: true,  // Active le mode rapport seul
    report_uri: Some("/csp-violation-report".to_string()),
};
```

En mode rapport seul, les violations sont signalées mais les ressources ne sont pas bloquées. C'est utile pour tester les politiques avant leur application.

### Rapport de violations

Configurez un endpoint pour recevoir les rapports de violation CSP :

```rust
let csp_config = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'self'".to_string()]),
    ],
    report_only: false,
    report_uri: Some("/csp-report".to_string()),
};
```

Créez une vue pour gérer les rapports de violation :

```rust
use rusti::http::{Request, Response};

pub fn csp_report_handler(request: Request) -> Response {
    // Parse et log le rapport de violation
    let report = request.json::<CspViolationReport>().unwrap();

    eprintln!("Violation CSP : {:?}", report);

    Response::new()
        .status(204)
}
```

### Patterns courants

#### CSP stricte pour une sécurité maximale

```rust
let strict_csp = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'none'".to_string()]),
        CspDirective::ScriptSrc(vec!["'self'".to_string(), "'nonce-{nonce}'".to_string()]),
        CspDirective::StyleSrc(vec!["'self'".to_string(), "'nonce-{nonce}'".to_string()]),
        CspDirective::ImgSrc(vec!["'self'".to_string(), "data:".to_string()]),
        CspDirective::FontSrc(vec!["'self'".to_string()]),
        CspDirective::ConnectSrc(vec!["'self'".to_string()]),
        CspDirective::BaseUri(vec!["'self'".to_string()]),
        CspDirective::FormAction(vec!["'self'".to_string()]),
        CspDirective::FrameAncestors(vec!["'none'".to_string()]),
    ],
    report_only: false,
    report_uri: Some("/csp-report".to_string()),
};
```

#### CSP souple pour le développement

```rust
let dev_csp = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'self'".to_string()]),
        CspDirective::ScriptSrc(vec![
            "'self'".to_string(),
            "'unsafe-inline'".to_string(),
            "'unsafe-eval'".to_string(),
        ]),
        CspDirective::StyleSrc(vec![
            "'self'".to_string(),
            "'unsafe-inline'".to_string(),
        ]),
    ],
    report_only: false,
    report_uri: None,
};
```

#### CDN et ressources tierces

```rust
let csp_with_cdn = CspConfig {
    directives: vec![
        CspDirective::DefaultSrc(vec!["'self'".to_string()]),
        CspDirective::ScriptSrc(vec![
            "'self'".to_string(),
            "https://cdn.jsdelivr.net".to_string(),
            "'nonce-{nonce}'".to_string(),
        ]),
        CspDirective::StyleSrc(vec![
            "'self'".to_string(),
            "https://cdn.jsdelivr.net".to_string(),
            "'nonce-{nonce}'".to_string(),
        ]),
        CspDirective::FontSrc(vec![
            "'self'".to_string(),
            "https://fonts.gstatic.com".to_string(),
        ]),
        CspDirective::ImgSrc(vec![
            "'self'".to_string(),
            "data:".to_string(),
            "https:".to_string(),
        ]),
    ],
    report_only: false,
    report_uri: None,
};
```

### Bonnes pratiques

1. **Commencez en mode rapport seul** : Testez votre configuration CSP avant de l'appliquer
2. **Utilisez des nonces pour les scripts inline** : Préférez les nonces à `unsafe-inline`
3. **Évitez `unsafe-eval`** : Cela affaiblit considérablement votre CSP
4. **Soyez spécifique** : Utilisez des domaines spécifiques plutôt que des wildcards quand c'est possible
5. **Surveillez les violations** : Configurez le rapport de violations pour détecter les problèmes
6. **Mises à jour régulières** : Révisez et mettez à jour votre CSP au fur et à mesure de l'évolution de votre application
7. **Testez minutieusement** : Vérifiez toutes les pages et fonctionnalités après l'implémentation de CSP

### Débogage des problèmes CSP

Lorsque des ressources sont bloquées par CSP, les navigateurs enregistrent des informations détaillées dans la console :

```
Refused to load the script 'https://example.com/script.js' because it violates
the following Content Security Policy directive: "script-src 'self'".
```

Problèmes courants :

- **Scripts inline bloqués** : Ajoutez des nonces ou déplacez les scripts dans des fichiers externes
- **Ressources CDN bloquées** : Ajoutez le domaine CDN aux directives appropriées
- **eval() bloqué** : Refactorisez le code pour éviter eval ou ajoutez `unsafe-eval` (non recommandé)
- **Styles inline bloqués** : Ajoutez des nonces ou déplacez les styles dans des fichiers externes

### Stratégie de migration

1. **Auditez les ressources actuelles** : Identifiez tous les scripts, styles et ressources externes
2. **Déployez en mode rapport seul** : Utilisez le mode rapport seul pour identifier les violations
3. **Ajustez la politique** : Mettez à jour la CSP en fonction des rapports de violation
4. **Application progressive** : Commencez avec des directives strictes et assouplissez au besoin
5. **Surveillez** : Continuez à surveiller les violations après l'application

### Considérations de sécurité

- CSP est une mesure de défense en profondeur, pas une solution de sécurité complète
- Validez et nettoyez toujours les entrées utilisateur côté serveur
- Gardez les dépendances à jour
- Utilisez HTTPS pour toutes les ressources
- Combinez CSP avec d'autres en-têtes de sécurité (HSTS, X-Frame-Options, etc.)

### Référence API

#### CspConfig

```rust
pub struct CspConfig {
    pub directives: Vec<CspDirective>,
    pub report_only: bool,
    pub report_uri: Option<String>,
}
```

#### CspDirective

```rust
pub enum CspDirective {
    DefaultSrc(Vec<String>),
    ScriptSrc(Vec<String>),
    StyleSrc(Vec<String>),
    ImgSrc(Vec<String>),
    FontSrc(Vec<String>),
    ConnectSrc(Vec<String>),
    MediaSrc(Vec<String>),
    ObjectSrc(Vec<String>),
    FrameSrc(Vec<String>),
    WorkerSrc(Vec<String>),
    ManifestSrc(Vec<String>),
    BaseUri(Vec<String>),
    FormAction(Vec<String>),
    FrameAncestors(Vec<String>),
}
```

#### CspMiddleware

```rust
impl CspMiddleware {
    pub fn new(config: CspConfig) -> Self
    pub fn generate_nonce() -> String
}
```

### Pour aller plus loin

- [MDN Content Security Policy](https://developer.mozilla.org/fr/docs/Web/HTTP/CSP)
- [Spécification CSP Level 3](https://www.w3.org/TR/CSP3/)
- [Google CSP Evaluator](https://csp-evaluator.withgoogle.com/)
- [Référence Content Security Policy](https://content-security-policy.com/)

---

*Cette documentation fait partie du framework web Rusti. Pour plus d'informations, consultez la documentation de Rusti.*
