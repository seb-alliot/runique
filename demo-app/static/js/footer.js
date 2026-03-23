(function () {
    const btn      = document.getElementById('footer-burger-btn');
    const nav      = document.getElementById('footer-links');
    const overlay  = document.getElementById('nav-overlay');
    const closeBtn = document.getElementById('footer-links-close');
    if (!btn) return;

    const openNav = () => {
        nav.classList.add('open');
        overlay?.classList.add('active');
    };

    const closeNav = () => {
        nav.classList.remove('open');
        overlay?.classList.remove('active');
    };

    btn.addEventListener('click', () => nav.classList.contains('open') ? closeNav() : openNav());
    overlay?.addEventListener('click', closeNav);
    closeBtn?.addEventListener('click', closeNav);
    document.addEventListener('keydown', e => { if (e.key === 'Escape') closeNav(); });
})();
