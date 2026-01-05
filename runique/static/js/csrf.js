if (!window.rustiCsrfInitialized) {
    window.rustiCsrfInitialized = true;

    function getCurrentToken() {
        // Priorité au token global
        return window._rusti_csrf_token || null;
    }

    function updateTokenInDom(newToken) {
        if (!newToken) return;

        // Mettre à jour tous les champs hidden existants
        document.querySelectorAll('input.rusti-csrf-field').forEach(el => el.value = newToken);

        // Mettre à jour le token global
        window._rusti_csrf_token = newToken;
    }

    const { fetch: originalFetch } = window;

    window.fetch = async (input, init = {}) => {
        init = init || {};
        init.headers = init.headers || {};

        const method = (init.method || 'GET').toUpperCase();

        if (['POST', 'PUT', 'PATCH', 'DELETE'].includes(method)) {
            const token = getCurrentToken();
            if (token) {
                init.headers['X-CSRF-Token'] = token;
            }
        }

        const response = await originalFetch(input, init);

        // Mettre à jour le token si réponse HTML
        const contentType = response.headers.get('Content-Type') || '';
        if (contentType.includes('text/html')) {
            const newToken = response.headers.get('X-CSRF-Token');
            if (newToken) updateTokenInDom(newToken);
        }

        return response;
    };
}
