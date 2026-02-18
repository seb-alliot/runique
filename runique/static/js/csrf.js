/**
 * Runique CSRF Manager - v2.1
 */
if (!window.rustiCsrfInitialized) {
    window.rustiCsrfInitialized = true;

    // Smart retrieval: first looks in the input injected by the extractor
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
                headers.set('X-CSRF-Token', token); // Forces sending the header expected by csrf.rs
            }
        }

        const response = await originalFetch(input, { ...init, headers });

        // Automatic rotation: captures the token from line 92 of csrf.rs
        const newToken = response.headers.get('X-CSRF-Token');
        if (newToken) {
            window._rusti_csrf_token = newToken;
            document.querySelectorAll('input[name="csrf_token"]').forEach(el => el.value = newToken);
        }

        return response;
    };
}