/**
 * admin-kebab.js
 * Gestion des kebab menus (⋮) — partagé sur toutes les pages admin.
 * Dépend de admin-actions.js (AdminActions).
 */

(function () {
    'use strict';

    const closeAllMenus = () => {
        document.querySelectorAll('.row-menu-dropdown.open').forEach(d => {
            d.classList.remove('open');
            d.removeAttribute('style');
        });
        document.querySelectorAll('.row-menu-trigger.active').forEach(t => t.classList.remove('active'));
    };

    const positionDropdown = (trigger, dropdown) => {
        const rect = trigger.getBoundingClientRect();
        const dropW = 160;
        const dropH = 120;
        dropdown.style.position = 'fixed';
        dropdown.style.zIndex = '9999';
        dropdown.style.right = 'auto';

        // Horizontal
        let left = rect.right - dropW;
        if (left < 8) left = 8;
        if (left + dropW > window.innerWidth - 8) left = window.innerWidth - dropW - 8;
        dropdown.style.left = left + 'px';

        // Vertical : ouvre en haut si pas assez de place en bas
        const spaceBelow = window.innerHeight - rect.bottom;
        if (spaceBelow < dropH + 8) {
            dropdown.style.top = 'auto';
            dropdown.style.bottom = (window.innerHeight - rect.top + 4) + 'px';
        } else {
            dropdown.style.bottom = 'auto';
            dropdown.style.top = (rect.bottom + 4) + 'px';
        }
    };

    let lastOpenTime = 0;

    document.addEventListener('click', function (e) {
        const trigger = e.target.closest('.row-menu-trigger');
        if (trigger) {
            const dropdown = trigger.nextElementSibling;
            const isOpen = dropdown.classList.contains('open');
            closeAllMenus();
            if (!isOpen) {
                positionDropdown(trigger, dropdown);
                dropdown.classList.add('open');
                trigger.classList.add('active');
                lastOpenTime = Date.now();
            }
            e.stopPropagation();
            return;
        }
        if (!e.target.closest('.row-menu-dropdown')) closeAllMenus();
    });

    // Sur mobile, un tap génère un micro-scroll — on ignore les scrolls dans les 300ms suivant l'ouverture
    window.addEventListener('scroll', () => {
        if (Date.now() - lastOpenTime > 300) closeAllMenus();
    }, true);
    window.addEventListener('resize', closeAllMenus);

    // Fermer avec Echap
    document.addEventListener('keydown', function (e) {
        if (e.key === 'Escape') closeAllMenus();
    });

    window.AdminKebab = { closeAllMenus };
})();
