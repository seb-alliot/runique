// Loaded without `defer`: data-theme must be set before the first paint,
// or a light page flashes dark. Stored choice > system preference.
(function () {
    try {
        var t = localStorage.getItem('runique-admin-theme');
        if (t !== 'light' && t !== 'dark') {
            t = window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
        }
        document.documentElement.setAttribute('data-theme', t);
    } catch (e) {
        document.documentElement.setAttribute('data-theme', 'dark');
    }
})();
