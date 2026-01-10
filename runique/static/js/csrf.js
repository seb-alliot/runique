if (!window.rustiCsrfInitialized) {
    window.rustiCsrfInitialized = true;

    function getCurrentToken() {
        return window._rusti_csrf_token || null;
    }

    function updateTokenInDom(newToken) {
        if (!newToken) return;

        document.querySelectorAll('input.runique-csrf-field').forEach(el => el.value = newToken);

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

        const contentType = response.headers.get('Content-Type') || '';
        if (contentType.includes('text/html')) {
            const newToken = response.headers.get('X-CSRF-Token');
            if (newToken) updateTokenInDom(newToken);
        }

        return response;
    };
}
