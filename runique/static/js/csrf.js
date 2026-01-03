if (!window.rustiCsrfInitialized) {
    window.rustiCsrfInitialized = true;
    const { fetch: originalFetch } = window;
    window.fetch = async (...args) => {
        const response = await originalFetch(...args);
        const newToken = response.headers.get('X-CSRF-Token');
        if (newToken) {
            document.querySelectorAll('.rusti-csrf-field').forEach(el => el.value = newToken);
            window._rusti_csrf_token = newToken;
        }
        return response;
    };
}