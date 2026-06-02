(function () {
    var btn = document.getElementById('doc-hamburger-btn');
    var sidebar = document.getElementById('doc-sidebar');
    var overlay = document.getElementById('doc-sidebar-overlay');
    if (!btn) return;

    function openSidebar() {
        sidebar.classList.add('open');
        overlay.classList.add('open');
    }
    function closeSidebar() {
        sidebar.classList.remove('open');
        overlay.classList.remove('open');
    }

    btn.addEventListener('click', function () {
        sidebar.classList.contains('open') ? closeSidebar() : openSidebar();
    });
    overlay.addEventListener('click', closeSidebar);
    document.addEventListener('keydown', function (e) { if (e.key === 'Escape') closeSidebar(); });
    window.addEventListener('resize', function () { if (window.innerWidth > 1024) closeSidebar(); });

    sidebar.querySelectorAll('a').forEach(function (a) {
        a.addEventListener('click', closeSidebar);
    });
})();
