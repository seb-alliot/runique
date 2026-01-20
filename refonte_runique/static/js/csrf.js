/**
 * Runique CSRF Manager avec Logs de suivi
 */
if (!window.rustiCsrfInitialized) {
    window.rustiCsrfInitialized = true;
    console.log("[CSRF JS] Initialisation du gestionnaire...");

    function initializeToken() {
        const metaToken = document.querySelector('meta[name="csrf-token"]');
        if (metaToken && metaToken.getAttribute('content') && metaToken.getAttribute('content') !== "...") {
            console.log("[CSRF JS] Token trouvé dans la balise <meta>");
            return metaToken.getAttribute('content');
        }

        const filledField = document.querySelector('input.runique-csrf-field[value]:not([value=""])');
        if (filledField) {
            console.log("[CSRF JS] Token trouvé dans un champ input");
            return filledField.value;
        }

        console.warn("[CSRF JS] Aucun token trouvé au chargement !");
        return window._rusti_csrf_token || null;
    }

    window._rusti_csrf_token = initializeToken();

    window.updateTokenInDom = function(newToken) {
        if (!newToken) return;
        console.log("[CSRF JS] Mise à jour du token dans le DOM");
        window._rusti_csrf_token = newToken;

        document.querySelectorAll('input.runique-csrf-field').forEach(el => {
            el.value = newToken;
        });

        const meta = document.querySelector('meta[name="csrf-token"]');
        if (meta) meta.setAttribute('content', newToken);
    };

    const { fetch: originalFetch } = window;

    window.fetch = async (input, init = {}) => {
        init = init || {};
        init.headers = init.headers || {};

        const method = (init.method || 'GET').toUpperCase();

        if (['POST', 'PUT', 'PATCH', 'DELETE'].includes(method)) {
            const token = window._rusti_csrf_token;
            if (token) {
                console.log(`[CSRF JS] Injection du header pour ${method}`);
                init.headers['X-CSRF-Token'] = token;
            } else {
                console.error(`[CSRF JS] Tentative de ${method} sans token disponible !`);
            }
        }

        const response = await originalFetch(input, init);

        const newToken = response.headers.get('X-CSRF-Token');
        if (newToken) {
            console.log("[CSRF JS] Rotation du token reçue du serveur");
            window.updateTokenInDom(newToken);
        }

        return response;
    };

    if (window._rusti_csrf_token) {
        window.updateTokenInDom(window._rusti_csrf_token);
    }
}