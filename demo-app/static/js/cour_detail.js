hljs.highlightAll();

(function () {
    const btn     = document.getElementById('doc-hamburger-btn');
    const sidebar = document.getElementById('doc-sidebar');
    const overlay = document.getElementById('doc-sidebar-overlay');
    if (!btn) return;

    function openSidebar()  { sidebar.classList.add('open');    overlay.classList.add('open'); }
    function closeSidebar() { sidebar.classList.remove('open'); overlay.classList.remove('open'); }

    btn.addEventListener('click', function () {
        sidebar.classList.contains('open') ? closeSidebar() : openSidebar();
    });
    overlay.addEventListener('click', closeSidebar);

    sidebar.querySelectorAll('a').forEach(function (a) {
        a.addEventListener('click', closeSidebar);
    });
})();
