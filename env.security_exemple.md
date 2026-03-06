# Exemple de .env.security pour Runique
# -------------------------------------
# Ce fichier sert de modèle pour sécuriser et configurer votre application Runique.
# Copiez-le en .env et adaptez les valeurs à votre environnement.

# --- Sécurité fondamentale ---
RUNIQUE_ALLOWED_HOSTS=localhost,127.0.0.1,mon-domaine.com
DEBUG=false
SECRET_KEY=change_this_secret_key

# --- Réseau ---
IP_SERVER=127.0.0.1
PORT=3000

# --- Base de données ---
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=motdepasse
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
DATABASE_URL=postgres://postgres:motdepasse@localhost:5432/runique

# --- Sécurité avancée ---
SANITIZE_INPUTS=true
STRICT_CSP=true
RATE_LIMITING=true
ENFORCE_HTTPS=true

# --- Content Security Policy (CSP) ---
# Personnalisez chaque directive CSP si besoin (sinon laissez vide pour les valeurs par défaut)
RUNIQUE_POLICY_CSP_DEFAULT='self',cdn.example.com
RUNIQUE_POLICY_CSP_SCRIPTS='self',cdn.js.com
RUNIQUE_POLICY_CSP_STYLES='self',cdn.css.com
RUNIQUE_POLICY_CSP_IMAGES='self',data:
RUNIQUE_POLICY_CSP_FONTS='self',fonts.gstatic.com
RUNIQUE_POLICY_CSP_STRICT_NONCE=true

# --- Fichiers statiques et templates ---
BASE_DIR=.
STATIC_RUNIQUE_PATH=static/
STATIC_RUNIQUE_URL=/runique/static
MEDIA_RUNIQUE_PATH=media/
MEDIA_RUNIQUE_URL=/runique/media
TEMPLATES_RUNIQUE=templates/
TEMPLATES_DIR=templates/
STATICFILES_DIRS=static
MEDIA_ROOT=media
STATIC_URL=/static
MEDIA_URL=/media
STATICFILES=default_storage

# --- Divers / Avancé ---
DEFAULT_AUTO_FIELD=BigAutoField

#
# Pour chaque variable, consulte la documentation ou le code source pour les valeurs par défaut et les usages précis.
#