/**
 * admin-kebab.js
 * Gestion des kebab menus (⋮) — partagé sur toutes les pages admin.
 * Dépend de admin-actions.js (AdminActions).
 */

(function () {
    'use strict';

    const closeAllMenus = () => {
        document.querySelectorAll('.row-menu-dropdown.open').forEach(d => d.classList.remove('open'));
        document.querySelectorAll('.row-menu-trigger.active').forEach(t => t.classList.remove('active'));
    };

    document.addEventListener('click', function (e) {
        const trigger = e.target.closest('.row-menu-trigger');
        if (trigger) {
            const dropdown = trigger.nextElementSibling;
            const isOpen = dropdown.classList.contains('open');
            closeAllMenus();
            if (!isOpen) {
                dropdown.classList.add('open');
                trigger.classList.add('active');
            }
            e.stopPropagation();
            return;
        }
        if (!e.target.closest('.row-menu-dropdown')) closeAllMenus();
    });

    // Fermer avec Echap
    document.addEventListener('keydown', function (e) {
        if (e.key === 'Escape') closeAllMenus();
    });

    window.AdminKebab = { closeAllMenus };
})();
