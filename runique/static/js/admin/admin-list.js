/**
 * admin-list.js
 * Comportements de la vue liste admin :
 *   - init AdminActions (confirmations suppression)
 *   - bouton burger filtre (mobile)
 *   - repli/dépli de la sidebar filtres (desktop)
 *   - repli/dépli individuel des groupes de filtres
 */

(function () {
    'use strict';

    // ── AdminActions ──
    if (window.AdminActions) {
        new window.AdminActions().init();
    }

    const sidebar = document.getElementById('filterSidebar');
    if (!sidebar) return;

    const toggle     = document.getElementById('filterToggle');
    const chevron    = toggle.querySelector('svg');
    const layout     = sidebar.closest('.admin-list-layout');
    const FILTER_KEY = 'runique_admin_filter_collapsed';
    const isMobile   = window.innerWidth <= 768;

    // ── Cellules dépliables ──
    document.querySelectorAll('.admin-table tbody').forEach(function (tbody) {
        tbody.addEventListener('click', function (e) {
            const cell = e.target.closest('td.td-data');
            if (cell) cell.classList.toggle('expanded');
        });
    });

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
    if (!isMobile && localStorage.getItem(FILTER_KEY) === '1') {
        layout.classList.add('filter-collapsed');
        chevron.style.transform = 'rotate(180deg)';
    }

    toggle.addEventListener('click', function () {
        if (isMobile) { closeFilterPanel(); return; }
        const isCollapsed = layout.classList.toggle('filter-collapsed');
        chevron.style.transform = isCollapsed ? 'rotate(180deg)' : '';
        localStorage.setItem(FILTER_KEY, isCollapsed ? '1' : '0');
    });

    // ── Groupes de filtres individuels ──
    const RESOURCE = sidebar.dataset.resource;

    document.querySelectorAll('.filter-group').forEach(function (group) {
        const col     = group.dataset.filterCol;
        const btn     = group.querySelector('.filter-group-title');
        const body    = group.querySelector('.filter-group-body');
        const grpChev = btn.querySelector('.filter-group-chevron');
        const KEY     = 'runique_fg_' + RESOURCE + '_' + col;
        const stored  = localStorage.getItem(KEY);

        // Mobile : replié par défaut sauf préférence explicite ; desktop : ouvert par défaut
        const shouldCollapse = isMobile ? stored !== '1' : stored === '0';
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
            localStorage.setItem(KEY, open ? '1' : '0');
        });
    });
})();
