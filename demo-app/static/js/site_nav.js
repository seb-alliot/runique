// ── Hero tabs ────────────────────────────────────────────────
(function () {
    const tabs   = document.querySelectorAll('.hero-tab');
    const panels = document.querySelectorAll('.hero-tab-panel');
    if (!tabs.length) return;

    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const target = tab.dataset.tab;
            tabs.forEach(t => t.classList.toggle('active', t === tab));
            panels.forEach(p => p.classList.toggle('active', p.dataset.panel === target));
        });
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
