# Documentation CSP de Runique

## Content Security Policy dans le Framework Runique

### Vue d'ensemble

Runique fournit une implémentation complète de Content Security Policy (CSP) qui aide à protéger vos applications web contre les attaques de type Cross-Site Scripting (XSS), le clickjacking et d'autres injections de code malveillant.

### Configuration

#### Structure CspConfig

```rust
pub struct CspConfig {
    pub default_src: Vec<String>,
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub img_src: Vec<String>,
    pub font_src: Vec<String>,
    pub connect_src: Vec<String>,
    pub frame_ancestors: Vec<String>,
    pub base_uri: Vec<String>,
    pub form_action: Vec<String>,
    pub use_nonce: bool,
}
```

**Explications des champs:**
- `default_src`: Directive par défaut pour toutes les ressources
- `script_src`: Sources autorisées pour les scripts JavaScript
- `style_src`: Sources autorisées pour les feuilles de style
- `img_src`: Sources autorisées pour les images
- `font_src`: Sources autorisées pour les polices
- `connect_src`: Sources autorisées pour les connexions (fetch, WebSocket, etc.)
- `frame_ancestors`: Contrôle quelles pages peuvent embarquer cette page dans un iframe
- `base_uri`: URLs autorisées pour la balise `<base>`
- `form_action`: URLs autorisées pour les soumissions de formulaires
- `use_nonce`: Active la génération automatique de nonces cryptographiques

#### Configurations prédéfinies

Runique fournit trois configurations CSP prêtes à l'emploi :

**1. CspConfig::default() - Pour le développement**
```rust
let csp = CspConfig::default();
// Équivalent à :
CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
    style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
    img_src: vec!["'self'".to_string(), "data:".to_string()],
    font_src: vec!["'self'".to_string()],
    connect_src: vec!["'self'".to_string()],
    frame_ancestors: vec!["'none'".to_string()],
    base_uri: vec!["'self'".to_string()],
    form_action: vec!["'self'".to_string()],
    use_nonce: false,
}
```

**2. CspConfig::strict() - Pour la production**
```rust
let csp = CspConfig::strict();
// Équivalent à :
CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string()],
    img_src: vec!["'self'".to_string()],
    font_src: vec!["'self'".to_string()],
    connect_src: vec!["'self'".to_string()],
    frame_ancestors: vec!["'none'".to_string()],
    base_uri: vec!["'self'".to_string()],
    form_action: vec!["'self'".to_string()],
    use_nonce: true, // ✅ Nonces activés en production
}
```

**3. CspConfig::permissive() - Pour les tests**
```rust
let csp = CspConfig::permissive();
// Équivalent à :
CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec![
        "'self'".to_string(),
        "'unsafe-inline'".to_string(),
        "'unsafe-eval'".to_string(),
    ],
    style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
    img_src: vec![
        "'self'".to_string(),
        "data:".to_string(),
        "https:".to_string(),
    ],
    font_src: vec!["'self'".to_string(), "data:".to_string()],
    connect_src: vec!["'self'".to_string()],
    frame_ancestors: vec!["'self'".to_string()],
    base_uri: vec!["'self'".to_string()],
    form_action: vec!["'self'".to_string()],
    use_nonce: false,
}
```

#### Configuration personnalisée

```rust
let csp = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec![
        "'self'".to_string(),
        "https://cdn.example.com".to_string(),
    ],
    style_src: vec![
        "'self'".to_string(),
        "https://cdn.example.com".to_string(),
    ],
    img_src: vec![
        "'self'".to_string(),
        "data:".to_string(),
        "https://images.example.com".to_string(),
    ],
    font_src: vec![
        "'self'".to_string(),
        "https://fonts.gstatic.com".to_string(),
    ],
    connect_src: vec!["'self'".to_string(), "https://api.example.com".to_string()],
    frame_ancestors: vec!["'none'".to_string()],
    base_uri: vec!["'self'".to_string()],
    form_action: vec!["'self'".to_string(), "https://example.com".to_string()],
    use_nonce: true,
};
```

### Utilisation des nonces

Lorsque `use_nonce: true`, Runique génère automatiquement des nonces cryptographiques pour les scripts et styles inline.

**Dans vos templates :**
```html
<script {{ csp }}>
    console.log('Protégé par nonce CSP');
</script>

<style {{ csp }}>
    .protected { color: blue; }
</style>
```

**Le tag {{ csp }} est automatiquement remplacé par :**
```html
nonce="abc123xyz..."
```

**Important:** Le tag `{{ csp }}` ne génère le nonce que si `use_nonce: true`. Sinon, il reste vide.

### Intégration middleware

#### Méthode 1 : CSP seule

```rust
RuniqueApp::new(settings).await?
    .routes(routes)
    .with_csp(CspConfig::strict())
    .run()
    .await?;
```

#### Méthode 2 : CSP + tous les headers de sécurité

```rust
RuniqueApp::new(settings).await?
    .routes(routes)
    .with_security_headers(CspConfig::strict())
    .run()
    .await?;
```

Headers inclus avec `with_security_headers()` :
- Content-Security-Policy
- X-Content-Type-Options: nosniff
- X-Frame-Options: DENY
- X-XSS-Protection: 1; mode=block
- Referrer-Policy: strict-origin-when-cross-origin
- Permissions-Policy: geolocation=(), microphone=(), camera=()

#### Méthode 3 : Mode report-only (pour tester)

```rust
RuniqueApp::new(settings).await?
    .routes(routes)
    .with_csp_report_only(CspConfig::strict())
    .run()
    .await?;
```

En mode report-only, les violations sont signalées mais les ressources ne sont pas bloquées.

### Patterns courants

#### Développement local

```rust
let csp = CspConfig::permissive();

RuniqueApp::new(settings).await?
    .with_csp(csp)
    .run()
    .await?;
```

#### Production sécurisée

```rust
let csp = CspConfig::strict();

RuniqueApp::new(settings).await?
    .with_security_headers(csp)
    .run()
    .await?;
```

#### Avec CDN externe

```rust
let csp = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec![
        "'self'".to_string(),
        "https://cdn.jsdelivr.net".to_string(),
    ],
    style_src: vec![
        "'self'".to_string(),
        "https://cdn.jsdelivr.net".to_string(),
    ],
    font_src: vec![
        "'self'".to_string(),
        "https://fonts.gstatic.com".to_string(),
    ],
    img_src: vec![
        "'self'".to_string(),
        "data:".to_string(),
        "https:".to_string(),
    ],
    use_nonce: true,
    ..CspConfig::default()
};
```

### Migration progressive

**Étape 1 : Tester en report-only**
```rust
.with_csp_report_only(CspConfig::strict())
```

**Étape 2 : Analyser les violations**
Consultez la console du navigateur pour identifier les ressources bloquées.

**Étape 3 : Ajuster la politique**
Ajoutez les domaines nécessaires aux directives appropriées.

**Étape 4 : Activer en mode enforcement**
```rust
.with_csp(CspConfig::strict())
```

**Étape 5 : Surveiller**
Continuez à surveiller les violations après activation.

### Débogage des problèmes CSP

#### Scripts inline bloqués

**Problème :** `Refused to execute inline script`

**Solutions :**
1. Utiliser des nonces : `use_nonce: true`
2. Déplacer les scripts dans des fichiers externes
3. Ajouter `'unsafe-inline'` (non recommandé en production)

#### Ressources CDN bloquées

**Problème :** `Refused to load the script 'https://cdn.example.com/script.js'`

**Solution :** Ajouter le domaine CDN
```rust
script_src: vec!["'self'".to_string(), "https://cdn.example.com".to_string()],
```

#### eval() bloqué

**Problème :** `Refused to evaluate a string as JavaScript`

**Solutions :**
1. Refactoriser le code pour éviter eval
2. Ajouter `'unsafe-eval'` uniquement si absolument nécessaire

### Bonnes pratiques

1. **Commencez en report-only** : Testez votre CSP avant de l'appliquer
2. **Utilisez des nonces en production** : `use_nonce: true` avec `CspConfig::strict()`
3. **Évitez 'unsafe-eval'** : Affaiblit considérablement la protection
4. **Soyez spécifique** : Listez les domaines exacts plutôt que des wildcards
5. **Surveillez les violations** : Configurez le monitoring
6. **Mises à jour régulières** : Révisez votre CSP lors des évolutions

### Considérations de sécurité

- CSP est une mesure de défense en profondeur, pas une solution complète
- Validez toujours les entrées côté serveur
- Gardez les dépendances à jour
- Utilisez HTTPS pour toutes les ressources
- Combinez CSP avec d'autres headers de sécurité

### Référence API

```rust
impl CspConfig {
    // Configurations prédéfinies
    pub fn default() -> Self
    pub fn strict() -> Self
    pub fn permissive() -> Self

    // Méthode interne
    fn to_header_value(&self, nonce: Option<&str>) -> String
}

// Middleware functions
pub async fn csp_middleware(...)
pub async fn security_headers_middleware(...)
pub async fn csp_report_only_middleware(...)
```

### Pour aller plus loin

- [MDN Content Security Policy](https://developer.mozilla.org/fr/docs/Web/HTTP/CSP)
- [Spécification CSP Level 3](https://www.w3.org/TR/CSP3/)
- [Google CSP Evaluator](https://csp-evaluator.withgoogle.com/)

---

Cette documentation fait partie du framework web Runique. Pour plus d'informations, consultez la documentation complète.

Version: 1.0 (Corrigée - 2 Janvier 2026)
Dernière mise à jour: Janvier 2026
Licence: MIT

*Documentation created with ❤️ by Claude for Itsuki*
