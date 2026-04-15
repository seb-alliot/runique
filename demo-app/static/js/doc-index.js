(function () {
    document.querySelectorAll('.doc-index-section-title').forEach(function (btn) {
        const key     = 'runique_doc_' + btn.dataset.section;
        const grid    = btn.nextElementSibling;
        const chevron = btn.querySelector('.doc-section-chevron');

        if (!grid) return;

        const stored  = localStorage.getItem(key);
        if (stored !== '1') {
            grid.style.display = 'none';
            if (chevron) chevron.style.transform = 'rotate(-90deg)';
            btn.setAttribute('aria-expanded', 'false');
        }
        btn.addEventListener('click', function () {
            const open = grid.style.display === 'none';
            grid.style.display = open ? '' : 'none';
            if (chevron) chevron.style.transform = open ? '' : 'rotate(-90deg)';
            btn.setAttribute('aria-expanded', open ? 'true' : 'false');
            localStorage.setItem(key, open ? '1' : '0');
        });
    });
})();