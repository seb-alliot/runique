(function () {
    if (!navigator.clipboard) return;

    const isFr = document.documentElement.lang === 'fr';
    const copy = isFr ? 'Copier' : 'Copy';
    const done = isFr ? 'Copié' : 'Copied';

    document.querySelectorAll('.doc-block-content pre').forEach(function (pre) {
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
})();
