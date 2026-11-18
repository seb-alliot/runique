// Audience measurement consent. Nothing is loaded from Google before the
// visitor accepts; the choice is kept six months in local storage, then asked
// again. "Refuse" also removes the cookies a past agreement left.
(function () {
    const banner = document.getElementById('consent');
    if (!banner) return;

    const KEY = 'consent';
    const SIX_MONTHS = 1000 * 60 * 60 * 24 * 182;

    function stored() {
        try {
            const saved = JSON.parse(localStorage.getItem(KEY));
            const fresh = saved && Date.now() - saved.date < SIX_MONTHS;
            return fresh && (saved.choice === 'granted' || saved.choice === 'denied') ? saved.choice : null;
        } catch (_) {
            return null;
        }
    }

    function store(choice) {
        try {
            localStorage.setItem(KEY, JSON.stringify({ choice: choice, date: Date.now() }));
        } catch (_) {
            // Storage refused: the choice holds for this page only.
        }
    }

    let loaded = false;
    function load() {
        const id = banner.dataset.gtm;
        if (loaded || !id) return;
        loaded = true;
        window.dataLayer = window.dataLayer || [];
        window.dataLayer.push({ 'gtm.start': new Date().getTime(), event: 'gtm.js' });
        const script = document.createElement('script');
        script.async = true;
        script.src = 'https://www.googletagmanager.com/gtm.js?id=' + encodeURIComponent(id);
        document.head.appendChild(script);
    }

    function forget() {
        const host = location.hostname;
        const domains = ['', '; domain=' + host, '; domain=.' + host.split('.').slice(-2).join('.')];
        document.cookie.split(';').forEach(function (cookie) {
            const name = cookie.split('=')[0].trim();
            if (name.indexOf('_ga') !== 0 && name !== '_gid') return;
            domains.forEach(function (domain) {
                document.cookie = name + '=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/' + domain;
            });
        });
    }

    banner.querySelectorAll('[data-consent]').forEach(function (button) {
        button.addEventListener('click', function () {
            const choice = button.dataset.consent;
            const was = stored();
            store(choice);
            banner.hidden = true;
            if (choice === 'granted') {
                load();
            } else {
                forget();
                // A script already running keeps measuring: only a reload stops it.
                if (was === 'granted' && loaded) location.reload();
            }
        });
    });

    document.querySelectorAll('[data-consent-open]').forEach(function (opener) {
        opener.addEventListener('click', function () {
            banner.hidden = false;
            banner.querySelector('[data-consent]').focus();
        });
    });

    const choice = stored();
    if (choice === 'granted') {
        load();
    } else if (choice === null) {
        banner.hidden = false;
    }
})();
