(function () {
    document.querySelectorAll('.compare-page .doc-block-content table').forEach(table => {
        const names = [...table.querySelectorAll('thead th')].map(th => th.textContent);
        if (!names.length) return;
        table.querySelectorAll('tbody tr').forEach(row => {
            [...row.cells].forEach((cell, i) => { cell.dataset.label = names[i] || ''; });
        });
        table.classList.add('compare-cards');
    });

    const toc = document.querySelector('.compare-toc');
    if (toc) {
        const wide = window.matchMedia('(min-width: 64rem)');
        const sync = () => { toc.open = wide.matches; };
        wide.addEventListener('change', sync);
        sync();
        toc.addEventListener('click', event => {
            if (!wide.matches && event.target.closest('a')) toc.open = false;
        });
    }

    const top = document.querySelector('.compare-top');
    if (top) {
        const update = () => top.classList.toggle('is-hidden', window.scrollY < 600);
        window.addEventListener('scroll', update, { passive: true });
        update();
    }
})();
