/**
 * Runique CSRF Manager - v2.1
 */
if (!window.rustiCsrfInitialized) {
    window.rustiCsrfInitialized = true;

    // Récupération intelligente : cherche d'abord dans l'input injecté par l'extracteur
    window.getCsrfToken = function() {
        const input = document.querySelector('input[name="csrf_token"]');
        if (input && input.value) return input.value;

        const meta = document.querySelector('meta[name="csrf-token"]');
        if (meta && meta.content) return meta.content;

        return window._rusti_csrf_token || null;
    };

    const { fetch: originalFetch } = window;
    window.fetch = async (input, init = {}) => {
        let headers = new Headers(init.headers || {});
        const method = (init.method || 'GET').toUpperCase();

        if (['POST', 'PUT', 'PATCH', 'DELETE'].includes(method)) {
            const token = window.getCsrfToken();
            if (token) {
                headers.set('X-CSRF-Token', token); // Force l'envoi du header attendu par csrf.rs
            }
        }

        const response = await originalFetch(input, { ...init, headers });

        // Rotation automatique : capture le token de la ligne 92 de csrf.rs
        const newToken = response.headers.get('X-CSRF-Token');
        if (newToken) {
            window._rusti_csrf_token = newToken;
            document.querySelectorAll('input[name="csrf_token"]').forEach(el => el.value = newToken);
        }

        return response;
    };
}