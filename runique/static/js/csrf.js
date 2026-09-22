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

    // Refresh the token right before a native form submission, not on a timer.
    // A tab left open past the anonymous session's inactivity window (5 min
    // default, see MiddlewareStaging::anonymous_session_duration) carries a
    // stale embedded token — submitting as-is fails CSRF with no recovery
    // but re-typing everything. A lightweight GET just before submit reuses
    // the rotation above (any response carries a fresh X-CSRF-Token header,
    // csrf.rs) and updates every csrf_token input on the page, including
    // this form's. Works even if the session fully expired: the server just
    // issues a new one, and the real submission that follows is consistent
    // with it. `form.submit()` (not `.requestSubmit()`) is used deliberately
    // to bypass this same listener on the second, real submission.
    document.addEventListener('submit', async (e) => {
        const form = e.target;
        if (!(form instanceof HTMLFormElement)) return;
        if (!form.querySelector('input[name="csrf_token"]')) return;

        e.preventDefault();
        try {
            await fetch(window.location.href);
        } catch (_) {
            // Network unavailable — fall through and submit with whatever
            // token is already on the page rather than blocking the user.
        }
        form.submit();
    });
}