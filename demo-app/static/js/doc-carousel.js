(function () {
    if (window.innerWidth > 600) return;

    document.querySelectorAll('.doc-sections-grid').forEach(function (grid) {
        const originals = Array.from(grid.children);
        const n = originals.length;
        if (n < 2) return;

        // Prepend clones (permet le scroll vers la gauche)
        for (let i = n - 1; i >= 0; i--) {
            const clone = originals[i].cloneNode(true);
            clone.setAttribute('aria-hidden', 'true');
            grid.insertBefore(clone, grid.firstChild);
        }

        // Append clones (permet le scroll vers la droite)
        originals.forEach(function (card) {
            const clone = card.cloneNode(true);
            clone.setAttribute('aria-hidden', 'true');
            grid.appendChild(clone);
        });

        // Positionner au début des originaux (après les clones prepend)
        requestAnimationFrame(function () {
            const setWidth = grid.scrollWidth / 3;
            grid.scrollLeft = setWidth;

            let locked = false;

            grid.addEventListener('scroll', function () {
                if (locked) return;
                const sl = grid.scrollLeft;

                if (sl < setWidth) {
                    // On est dans les clones prepend → sauter vers les originaux
                    locked = true;
                    grid.style.scrollSnapType = 'none';
                    grid.scrollLeft = sl + setWidth;
                    requestAnimationFrame(function () {
                        grid.style.scrollSnapType = '';
                        locked = false;
                    });
                } else if (sl >= setWidth * 2) {
                    // On est dans les clones append → sauter vers les originaux
                    locked = true;
                    grid.style.scrollSnapType = 'none';
                    grid.scrollLeft = sl - setWidth;
                    requestAnimationFrame(function () {
                        grid.style.scrollSnapType = '';
                        locked = false;
                    });
                }
            });
        });
    });
})();
