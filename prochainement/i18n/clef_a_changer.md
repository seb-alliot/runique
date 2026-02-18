# RUNIQUE I18N - COMPLET
# Format: key|fr|en|file|type

# === FORMS VALIDATION ===
forms.required|Ce champ est obligatoire|This field is required|forms/fields/text.rs|validation
forms.too_short|Trop court (min {})|Too short (min {})|forms/fields/text.rs|validation
forms.too_long|Trop long (max {})|Too long (max {})|forms/fields/text.rs|validation
forms.email_invalid|Format d'adresse email invalide|Invalid email address format|forms/fields/text.rs|validation
forms.url_invalid|Veuillez entrer une URL valide|Please enter a valid URL|forms/fields/text.rs|validation
forms.hash_error|Erreur de hachage : {}|Hashing error: {}|forms/fields/text.rs|validation
forms.number_invalid|Nombre invalide|Invalid number|forms/fields/number.rs|validation
forms.number_required|Requis|Required|forms/fields/number.rs|validation
forms.integer_required|Doit être un entier|Must be an integer|forms/fields/number.rs|validation
forms.decimal_required|Doit être au format décimal|Must be decimal format|forms/fields/number.rs|validation
forms.min_value|Min {} requis|Min {} required|forms/fields/number.rs|validation
forms.max_value|Max {} dépassé|Max {} exceeded|forms/fields/number.rs|validation
forms.precision_max|Précision max {} chiffres après virgule|Max {} decimal places|forms/fields/number.rs|validation
forms.precision_min|Précision min {} chiffres après virgule|Min {} decimal places|forms/fields/number.rs|validation
forms.color_invalid|Couleur invalide (format: #RRGGBB ou #RGB)|Invalid color (format: #RRGGBB or #RGB)|forms/fields/special.rs|validation
forms.color_no_hash|La couleur doit commencer par #|Color must start with #|forms/fields/special.rs|validation
forms.color_bad_hex|Contient des caractères non hexadécimaux|Contains non-hexadecimal characters|forms/fields/special.rs|validation
forms.slug_invalid|Slug invalide|Invalid slug|forms/fields/special.rs|validation
forms.slug_no_dash|Le slug ne doit pas commencer ou finir par un tiret|Slug must not start or end with dash|forms/fields/special.rs|validation
forms.uuid_invalid|Format UUID invalide (attendu: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)|Invalid UUID format (expected: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)|forms/fields/special.rs|validation
forms.json_invalid|JSON invalide : {}|Invalid JSON: {}|forms/fields/special.rs|validation
forms.ipv4_only|Seules les adresses IPv4 sont acceptées|Only IPv4 addresses accepted|forms/fields/special.rs|validation
forms.ipv6_only|Seules les adresses IPv6 sont acceptées|Only IPv6 addresses accepted|forms/fields/special.rs|validation
forms.ip_invalid|Adresse IP invalide|Invalid IP address|forms/fields/special.rs|validation
forms.date_invalid|Format de date invalide|Invalid date format|forms/fields/datetime.rs|validation
forms.date_too_old|Trop ancien|Too old|forms/fields/datetime.rs|validation
forms.date_too_far|Trop loin|Too far|forms/fields/datetime.rs|validation
forms.time_invalid|Format de temps invalide|Invalid time format|forms/fields/datetime.rs|validation
forms.file_required|Veuillez sélectionner au moins un fichier|Please select at least one file|forms/fields/file.rs|validation
forms.file_extension_blocked|non autorisé|not allowed|forms/fields/file.rs|validation
forms.finalize_error|Erreur lors de la finalisation du champ '{}': {}|Error finalizing field '{}': {}|forms/manager.rs|validation
forms.template_missing|Template manquant: {}|Missing template: {}|forms/manager.rs|render
forms.render_js_error|Erreur rendu JS: {}|JS render error: {}|forms/manager.rs|render
forms.tera_not_configured|Tera non configuré|Tera not configured|forms/manager.rs|render
forms.validation_overflow|Stack overflow détecté : récursion infinie dans la validation|Stack overflow detected: infinite recursion in validation|forms/manager.rs|validation

# === CSRF ===
csrf.missing|Token CSRF manquant|CSRF token missing|forms/fields/hidden.rs|security
csrf.invalid|Token CSRF invalide|Invalid CSRF token|forms/fields/hidden.rs|security
csrf.invalid_or_missing|Token CSRF invalide ou manquant|Invalid or missing CSRF token|forms/prisme/csrf_gate.rs|security
csrf.forbidden|Invalid CSRF token|Invalid CSRF token|middleware/security/csrf.rs|security
csrf.session_write_error|Session write error|Session write error|middleware/security/csrf.rs|security

# === ERRORS RUNTIME ===
error.build|Erreur de build: {}|Build error: {}|errors/error.rs|log
error.internal|Erreur interne|Internal error|errors/error.rs|log
error.forbidden|Accès interdit|Access forbidden|errors/error.rs|log
error.not_found|Ressource introuvable|Resource not found|errors/error.rs|log
error.validation|Erreur de validation: {}|Validation error: {}|errors/error.rs|log
error.database|Erreur base de données: {}|Database error: {}|errors/error.rs|log
error.io|Erreur IO: {}|IO error: {}|errors/error.rs|log
error.template|Erreur template: {}|Template error: {}|errors/error.rs|log
error.custom|Erreur custom: {}|Custom error: {}|errors/error.rs|log
error.title.not_found|Page non trouvée|Page Not Found|errors/error.rs|title
error.title.forbidden|Accès interdit|Access Forbidden|errors/error.rs|title
error.title.validation|Erreur de validation|Validation Error|errors/error.rs|title
error.title.database|Erreur base de données|Database Error|errors/error.rs|title
error.title.template|Erreur de rendu de template|Template Rendering Error|errors/error.rs|title
error.title.internal|Erreur serveur interne|Internal Server Error|errors/error.rs|title
error.path_not_found|The requested path '{}' was not found|The requested path '{}' was not found|errors/error.rs|message
error.internal_occurred|Une erreur interne est survenue|An internal error occurred|errors/error.rs|message
error.context_occurred|AppError occurred|AppError occurred|context/template.rs|log

# === BUILD ERRORS ===
build.validation_failed|Build validation failed: {}|Build validation failed: {}|app/error_build.rs|error
build.check_failed|Build failed with {} check error(s)|Build failed with {} check error(s)|app/error_build.rs|error
build.template_failed|Template loading failed: {}|Template loading failed: {}|app/error_build.rs|error
build.db_missing|Database connection required when 'orm' feature is enabled|Database connection required when 'orm' feature is enabled|app/error_build.rs|error
build.component_not_ready|Component '{}' is not ready|Component '{}' is not ready|app/error_build.rs|error

# === MIDDLEWARE ===
middleware.host_invalid|Hôte invalide: '{}' (Hosts autorisés: {})|Invalid host: '{}' (Allowed: {})|middleware/security/allowed_hosts.rs|security

# === ADMIN ===
admin.session_error|Erreur lors de l'ouverture de session.|Session opening error.|admin/router/admin_router.rs|error
admin.invalid_credentials|Identifiants incorrects ou droits insuffisants.|Invalid credentials or insufficient rights.|admin/router/admin_router.rs|error
admin.logout_success|Déconnexion réussie.|Logout successful.|admin/router/admin_router.rs|success
admin.no_auth_handler|Aucun handler d'authentification configuré. Appelez .auth(MyAuth) sur AdminConfig.|No auth handler configured. Call .auth(MyAuth) on AdminConfig.|admin/router/admin_router.rs|error
admin.insufficient_rights|Droits insuffisants pour accéder à l'administration|Insufficient rights to access admin|admin/middleware/admin_middleware.rs|error
admin.login_title|Connexion — {}|Login — {}|templates/admin/composant/admin_login.html|ui
admin.login_subtitle|Connectez-vous pour accéder au panneau d'administration|Sign in to access the admin panel|templates/admin/composant/admin_login.html|ui
admin.username|Nom d'utilisateur|Username|templates/admin/composant/admin_login.html|ui
admin.password|Mot de passe|Password|templates/admin/composant/admin_login.html|ui
admin.login_button|Se connecter|Sign in|templates/admin/composant/admin_login.html|ui

# === HTML FALLBACKS ===
html.404_title|404|404|middleware/errors/error.rs|title
html.404_text|Page non trouvée|Page not found|middleware/errors/error.rs|text
html.500_title|500|500|middleware/errors/error.rs|title
html.500_text|Erreur serveur interne|Internal server error|middleware/errors/error.rs|text
html.500_notice|Nos équipes ont été notifiées et travaillent sur le problème.|Our teams have been notified and are working on it.|middleware/errors/error.rs|text
html.back_home|Retour à l'accueil|Back to home|middleware/errors/error.rs|link
html.critical_error_title|Erreur critique|Critical Error|middleware/errors/error.rs|title
html.critical_error_text|Le système de gestion d'erreurs a lui-même rencontré une erreur.|The error handling system itself encountered an error.|middleware/errors/error.rs|text
html.critical_error_contact|Cette situation ne devrait jamais se produire. Veuillez contacter l'administrateur système.|This should never happen. Please contact the system administrator.|middleware/errors/error.rs|text

# === DEBUG TEMPLATES ===
debug.stack_trace_title|Trace d'erreur détaillée|Detailed error trace|templates/errors/corps-error/stack-trace-error.html|title
debug.stack_trace_tip|L'erreur la plus haute (niveau 0) est généralement la cause racine. Les erreurs suivantes montrent comment l'erreur s'est propagée dans le code.|The highest error (level 0) is usually the root cause. Following errors show how the error propagated through the code.|templates/errors/corps-error/stack-trace-error.html|tip

# === FLASH EXAMPLES ===
flash.welcome|Bienvenue {}, votre compte est créé !|Welcome {}, your account has been created!|demo-app/src/views.rs|success
flash.fix_errors|Veuillez corriger les erreurs|Please correct the errors|demo-app/src/views.rs|error
flash.success_demo|Ceci est un message de succès.|This is a success message.|demo-app/src/views.rs|success
flash.info_demo|Ceci est un message d'information.|This is an info message.|demo-app/src/views.rs|info
flash.warning_demo|Ceci est un message d'avertissement.|This is a warning message.|demo-app/src/views.rs|warning
flash.error_demo|Ceci est un message d'erreur.|This is an error message.|demo-app/src/views.rs|error

# === TRACING LOGS ===
log.db_connecting|Connecting to {} database...|Connecting to {} database...|db/config.rs|info
log.db_connected|Database connected successfully ({})|Database connected successfully ({})|db/config.rs|info
log.db_failed|Database connection failed|Database connection failed|db/config.rs|error
log.db_engine|Engine: {}|Engine: {}|db/config.rs|error
log.template_render_error|Template rendering error|Template rendering error|context/template.rs|error
log.error_handler|Critical error occurred|Critical error occurred|middleware/errors/error.rs|error
log.error_handled|Handled error|Handled error|middleware/errors/error.rs|info
log.files_count|add files JS to form|add files JS to form|forms/manager.rs|debug
log.js_file_skip|Skipping JS file|Skipping JS file|forms/manager.rs|warn
log.already_initialized|Logger déjà initialisé|Logger already initialized|utils/init_error/init.rs|eprintln

# === CLI MESSAGES ===
cli.creating_project|Créating projet '{}'...|Creating project '{}'...|bin/runique.rs|println
cli.admin_detected|Admin detected → starting the daemon|Admin detected → starting the daemon|bin/runique.rs|println
cli.add_admin_hint|Add .with_admin(...) in your builder to enable the AdminPanel.|Add .with_admin(...) in your builder to enable the AdminPanel.|bin/runique.rs|println
cli.daemon_error|[Daemon] Erreur: {}|[Daemon] Error: {}|bin/runique.rs|eprintln
cli.cargo_run_failed|Le serveur applicatif n'a pas démarré correctement (cargo run)|Application server failed to start correctly (cargo run)|bin/runique.rs|error
cli.file_not_found|Fichier non trouvé: {}\nAssurez-vous d'être à la racine de votre projet Runique.|File not found: {}\nMake sure you're at the root of your Runique project.|bin/runique.rs|error
cli.admin_not_found|Fichier admin non trouvé: {}\nCréez src/admin.rs avec le macro admin!{{}}.|Admin file not found: {}\nCreate src/admin.rs with admin!{{}} macro.|bin/runique.rs|error
cli.name_empty|The project name cannot be empty|The project name cannot be empty|bin/runique.rs|error
cli.name_invalid|The project name must contain only letters, numbers, _ or -|The project name must contain only letters, numbers, _ or -|bin/runique.rs|error
cli.name_dash|The project name cannot start with -|The project name cannot start with -|bin/runique.rs|error
cli.folder_exists|The folder '{}' already exists|The folder '{}' already exists|bin/runique.rs|error
cli.cargo_run_expect|Échec du lancement de cargo run|Failed to launch cargo run|bin/runique.rs|expect

# === DAEMON ===
daemon.modification_detected|Modification detected → regeneration...|Modification detected → regeneration...|admin/daemon/watcher.rs|println
daemon.unable_read|Unable to read: {}|Unable to read: {}|admin/daemon/watcher.rs|eprintln
daemon.parse_error|Parsing error: {}|Parsing error: {}|admin/daemon/watcher.rs|eprintln
daemon.no_resource|No resource in admin!{{}} — nothing to generate|No resource in admin!{{}} — nothing to generate|admin/daemon/watcher.rs|println
daemon.operational|Daemon operational → src/admin/generated.rs|Daemon operational → src/admin/generated.rs|admin/daemon/watcher.rs|println
daemon.generation_error|Generation error: {}|Generation error: {}|admin/daemon/watcher.rs|eprintln
daemon.watcher_error|Watcher error: {}|Watcher error: {}|admin/daemon/watcher.rs|eprintln
daemon.unable_create|Unable to create watcher: {}|Unable to create watcher: {}|admin/daemon/watcher.rs|error
daemon.unable_watch|Unable to watch {}: {}|Unable to watch {}: {}|admin/daemon/watcher.rs|error

# === DERIVE MACRO PANICS ===
macro.named_fields_only|Structs à champs nommés uniquement|Named fields structs only|derive_form/src/helpers.rs|panic
macro.structs_only|Fonctionne uniquement sur des structs|Works only on structs|derive_form/src/helpers.rs|panic
macro.forms_field_required|Struct '{}' doit avoir un champ 'Forms'|Struct '{}' must have a 'Forms' field|derive_form/src/helpers.rs|panic
macro.field_no_name|Champ sans nom|Field without name|derive_form/src/helpers.rs|expect
macro.model_form_named_only|DeriveModelForm : structs avec champs nommés uniquement|DeriveModelForm: structs with named fields only|derive_form/src/models.rs|panic
macro.model_form_structs_only|DeriveModelForm : structs uniquement|DeriveModelForm: structs only|derive_form/src/models.rs|panic

# === PARSER ERRORS ===
parser.string_expected|Expected string literal, found: {}|Expected string literal, found: {}|admin/daemon/parser.rs|error
parser.string_eof|Expected string literal, end of file|Expected string literal, end of file|admin/daemon/parser.rs|error
parser.array_expected|Expected [...] for permissions, found: {}|Expected [...] for permissions, found: {}|admin/daemon/parser.rs|error
parser.array_eof|Expected [...] for permissions, end of file|Expected [...] for permissions, end of file|admin/daemon/parser.rs|error
parser.role_required|At least one role required in permissions: [...]|At least one role required in permissions: [...]|admin/daemon/parser.rs|error
parser.punct_expected|Expected '{}', found: {}|Expected '{}', found: {}|admin/daemon/parser.rs|error
parser.punct_eof|Expected '{}', end of file|Expected '{}', end of file|admin/daemon/parser.rs|error