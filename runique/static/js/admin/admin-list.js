/**
 * admin-list.js
 * Comportements de la vue liste admin :
 *   - init AdminActions (confirmations suppression)
 *   - bouton burger filtre (mobile)
 *   - repli/dépli de la sidebar filtres (desktop)
 *   - repli/dépli individuel des groupes de filtres
 *   - re-init automatique après swap HTMX
 */

function initAdminList() {
    'use strict';

    // ── AdminActions ──
    if (window.AdminActions) {
        new window.AdminActions().init();
    }

    // ── Valeurs de carte dépliables (clic, valeurs longues tronquées) ──
    document.querySelectorAll('.admin-list__cards').forEach(function (grid) {
        grid.addEventListener('click', function (e) {
            const field = e.target.closest('.admin-list__field-value');
            if (field) field.classList.toggle('expanded');
        });
    });

    const sidebar = document.getElementById('filterSidebar');
    if (!sidebar) return;

    const toggle     = document.getElementById('filterToggle');
    const chevron    = toggle.querySelector('svg');
    const layout     = sidebar.closest('.admin-list__layout');
    const FILTER_KEY = 'runique_admin_filter_collapsed';
    // Same breakpoint as components.css (filter panel docked from 64em).
    const isMobile   = !window.matchMedia('(min-width: 64em)').matches;

    // ── Bouton burger filtre (mobile) + overlay ──
    const mobileBtn = document.getElementById('mobileFilterToggle');
    const overlay   = document.getElementById('filterOverlay');

    const openFilterPanel = () => {
        sidebar.classList.add('mobile-open');
        if (overlay) overlay.classList.add('active');
        if (mobileBtn) mobileBtn.setAttribute('aria-expanded', 'true');
    };

    const closeFilterPanel = () => {
        sidebar.classList.remove('mobile-open');
        if (overlay) overlay.classList.remove('active');
        if (mobileBtn) mobileBtn.setAttribute('aria-expanded', 'false');
    };

    if (mobileBtn) {
        mobileBtn.addEventListener('click', function () {
            sidebar.classList.contains('mobile-open') ? closeFilterPanel() : openFilterPanel();
        });
    }

    if (overlay) {
        overlay.addEventListener('click', closeFilterPanel);
    }

    // ── Sidebar filtres globale (desktop uniquement) ──
    if (!isMobile && sessionStorage.getItem(FILTER_KEY) === '1') {
        layout.classList.add('filter-collapsed');
        chevron.style.transform = 'rotate(180deg)';
    }

    toggle.addEventListener('click', function () {
        if (isMobile) { closeFilterPanel(); return; }
        const isCollapsed = layout.classList.toggle('filter-collapsed');
        chevron.style.transform = isCollapsed ? 'rotate(180deg)' : '';
        sessionStorage.setItem(FILTER_KEY, isCollapsed ? '1' : '0');
    });

    // ── Groupes de filtres individuels ──
    const RESOURCE = sidebar.dataset.resource;

    document.querySelectorAll('.admin-filter__group').forEach(function (group) {
        const col     = group.dataset.filterCol;
        const btn     = group.querySelector('.admin-filter__group-title');
        const body    = group.querySelector('.admin-filter__group-body');
        const grpChev = btn.querySelector('.admin-filter__chevron');
        const KEY     = 'runique_fg_' + RESOURCE + '_' + col;
        const stored  = sessionStorage.getItem(KEY);

        // Replié par défaut sauf préférence explicite (sessionStorage)
        const shouldCollapse = stored !== '1';
        if (shouldCollapse) {
            body.style.display = 'none';
            grpChev.style.transform = 'rotate(-90deg)';
            btn.setAttribute('aria-expanded', 'false');
        }

        btn.addEventListener('click', function () {
            const open = body.style.display === 'none';
            body.style.display = open ? '' : 'none';
            grpChev.style.transform = open ? '' : 'rotate(-90deg)';
            btn.setAttribute('aria-expanded', open ? 'true' : 'false');
            sessionStorage.setItem(KEY, open ? '1' : '0');
        });
    });
}

window.initAdminList = initAdminList;

// Init au chargement initial
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initAdminList);
} else {
    initAdminList();
}

// Re-init après chaque swap HTMX sur #list-content
document.body.addEventListener('htmx:afterSwap', function (e) {
    if (e.target.id === 'list-content') {
        initAdminList();
    }
});
