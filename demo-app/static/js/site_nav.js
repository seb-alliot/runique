// ── Onglets, groupe par groupe ───────────────────────────────
(function () {
    document.querySelectorAll('.hero-code, [data-tabs]').forEach(group => {
        const tabs   = group.querySelectorAll('.hero-tab');
        const panels = group.querySelectorAll('.hero-tab-panel');
        if (!tabs.length) return;

        const show = target => {
            tabs.forEach(t => t.classList.toggle('active', t.dataset.tab === target));
            panels.forEach(p => p.classList.toggle('active', p.dataset.panel === target));
        };
        const inUrl = group.dataset.tabs === 'hash';

        tabs.forEach(tab => {
            tab.addEventListener('click', () => {
                show(tab.dataset.tab);
                if (inUrl) history.replaceState(null, '', '#' + tab.dataset.tab);
            });
        });

        if (group.dataset.tabs) group.classList.add('is-ready');
        const fromUrl = () => {
            const wanted = location.hash.slice(1);
            if (!wanted) return;
            const target = document.getElementById(wanted);
            const panel = target && target.closest('.hero-tab-panel');
            if (!panel || !group.contains(panel)) return;
            show(panel.dataset.panel);
            target.scrollIntoView();
        };
        if (inUrl) {
            fromUrl();
            window.addEventListener('hashchange', fromUrl);
        }
    });
})();

// ── Navbar mobile ────────────────────────────────────────────
(function () {
    const toggle    = document.getElementById('site-nav-toggle');
    const panel     = document.getElementById('site-nav-panel');
    const overlay   = document.getElementById('site-nav-overlay');
    const closeBtn  = document.getElementById('site-nav-panel-close');
    if (!toggle) return;

    const openPanel  = () => { panel.classList.add('open'); overlay.classList.add('open'); };
    const closePanel = () => { panel.classList.remove('open'); overlay.classList.remove('open'); };

    toggle.addEventListener('click', () => panel.classList.contains('open') ? closePanel() : openPanel());
    overlay.addEventListener('click', closePanel);
    closeBtn.addEventListener('click', closePanel);
    panel.querySelectorAll('a').forEach(a => a.addEventListener('click', closePanel));
    document.addEventListener('keydown', e => { if (e.key === 'Escape') closePanel(); });
})();
