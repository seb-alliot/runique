(function () {
    const wide = window.matchMedia('(min-width: 64rem)');

    // On a phone, tables become cards: each cell carries its column's name.
    function cards(root) {
        root.querySelectorAll('.doc-block-content table').forEach(table => {
            if (table.classList.contains('compare-cards')) return;
            const names = [...table.querySelectorAll('thead th')].map(th => th.textContent);
            if (!names.length) return;
            table.querySelectorAll('tbody tr').forEach(row => {
                [...row.cells].forEach((cell, i) => { cell.dataset.label = names[i] || ''; });
            });
            table.classList.add('compare-cards');
        });
    }

    // The table of contents stays open on a wide screen; on a phone it folds
    // back once a link is followed.
    function toc(root) {
        const toc = root.querySelector('.compare-toc');
        if (!toc) return;
        toc.open = wide.matches;
        toc.addEventListener('click', event => {
            if (!wide.matches && event.target.closest('a')) toc.open = false;
        });
    }

    function compareInit(root) {
        cards(root);
        toc(root);
    }

    wide.addEventListener('change', () => {
        const toc = document.querySelector('.compare-toc');
        if (toc) toc.open = wide.matches;
    });

    const top = document.querySelector('.compare-top');
    if (top) {
        const update = () => top.classList.toggle('is-hidden', window.scrollY < 600);
        window.addEventListener('scroll', update, { passive: true });
        update();
    }

    const page = document.querySelector('.compare-page');
    if (page) compareInit(page);

    // A comparison opened through htmx: the page swapped in gets its code
    // highlighting, copy buttons, table cards and table of contents. Right
    // after the swap, not at settle (~20 ms later): the table of contents
    // would show folded, then open.
    document.body.addEventListener('htmx:afterSwap', () => {
        const root = document.querySelector('.compare-page');
        if (!root) return;
        if (window.hljs) root.querySelectorAll('pre code:not(.hljs)').forEach(el => window.hljs.highlightElement(el));
        if (window.docCopyInit) window.docCopyInit(root);
        compareInit(root);
    });
})();
