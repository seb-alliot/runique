(function () {
    if (!navigator.clipboard) return;

    const isFr = document.documentElement.lang === 'fr';
    const copy = isFr ? 'Copier' : 'Copy';
    const done = isFr ? 'Copié' : 'Copied';

    // A copy button on each code block of `root` that has none yet: run again
    // on content swapped in by htmx.
    function docCopyInit(root) {
        root.querySelectorAll('.doc-block-content pre').forEach(function (pre) {
            if (pre.parentNode.classList.contains('doc-code-wrap')) return;
            const wrap = document.createElement('div');
            wrap.className = 'doc-code-wrap';
            pre.parentNode.insertBefore(wrap, pre);
            wrap.appendChild(pre);

            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'doc-copy-btn';
            btn.textContent = copy;
            btn.addEventListener('click', function () {
                navigator.clipboard.writeText(pre.innerText).then(function () {
                    btn.textContent = done;
                    setTimeout(function () { btn.textContent = copy; }, 1500);
                });
            });
            wrap.appendChild(btn);
        });
    }

    window.docCopyInit = docCopyInit;
    docCopyInit(document);
})();
