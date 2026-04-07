/**
 * choice-filter.js
 * Barre de filtre alphabétique + compteur pour CheckboxField et RadioField admin.
 *
 * - Filtre par première lettre (navbar dynamique — seules les lettres existantes)
 * - Visibilité uniquement : l'état coché n'est jamais modifié
 * - Compteur "N sélectionnés" mis à jour en temps réel
 * - Compatible CSP — aucun inline handler
 */

(function () {
    'use strict';

    const FIELDSET_SELECTORS = ['.checkbox-fieldset', '.radio-fieldset'];
    const OPTION_SELECTORS   = { '.checkbox-fieldset': '.checkbox-option', '.radio-fieldset': '.radio-option' };

    function getLabelText(option) {
        const span = option.querySelector('.checkbox-label, .radio-label');
        return span ? span.textContent.trim() : '';
    }

    function buildFilter(fieldset, options, legend) {
        const isCheckbox = fieldset.classList.contains('checkbox-fieldset');

        // ── Compteur (checkboxes uniquement) ─────────────────────────────────
        if (isCheckbox) {
            const counter = document.createElement('span');
            counter.className = 'choice-filter-counter';
            legend.appendChild(counter);

            const updateCounter = () => {
                const checked = fieldset.querySelectorAll('input[type="checkbox"]:checked').length;
                counter.textContent = checked > 0 ? '— ' + checked + ' sélectionné' + (checked > 1 ? 's' : '') : '';
            };
            updateCounter();
            fieldset.addEventListener('change', updateCounter);
        }

        // ── Collecte des premières lettres existantes ─────────────────────────
        const letters = [];
        options.forEach(opt => {
            const first = getLabelText(opt).charAt(0).toUpperCase();
            if (first && letters.indexOf(first) === -1) {
                letters.push(first);
            }
        });
        letters.sort();

        if (letters.length <= 1) return;

        // ── Barre alphabétique ────────────────────────────────────────────────
        const bar = document.createElement('div');
        bar.className = 'choice-filter-bar';

        let activeBtn = null;

        const applyFilter = letter => {
            options.forEach(opt => {
                const first = getLabelText(opt).charAt(0).toUpperCase();
                opt.style.display = (!letter || first === letter) ? '' : 'none';
            });
        };

        // Bouton "Tout"
        const btnAll = document.createElement('button');
        btnAll.type = 'button';
        btnAll.textContent = 'Tout';
        btnAll.className = 'btn btn-xs btn-secondary choice-filter-btn choice-filter-active';
        activeBtn = btnAll;

        btnAll.addEventListener('click', () => {
            applyFilter(null);
            if (activeBtn) activeBtn.classList.remove('choice-filter-active');
            btnAll.classList.add('choice-filter-active');
            activeBtn = btnAll;
        });
        bar.appendChild(btnAll);

        letters.forEach(letter => {
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.textContent = letter;
            btn.className = 'btn btn-xs btn-secondary choice-filter-btn';

            btn.addEventListener('click', () => {
                applyFilter(letter);
                if (activeBtn) activeBtn.classList.remove('choice-filter-active');
                btn.classList.add('choice-filter-active');
                activeBtn = btn;
            });
            bar.appendChild(btn);
        });

        // Insère la barre avant la première option
        const optSel = OPTION_SELECTORS[isCheckbox ? '.checkbox-fieldset' : '.radio-fieldset'];
        const firstOption = fieldset.querySelector(optSel);
        if (firstOption) {
            fieldset.insertBefore(bar, firstOption);
        } else {
            fieldset.appendChild(bar);
        }
    }

    class ChoiceFilter {
        init() {
            FIELDSET_SELECTORS.forEach(sel => {
                document.querySelectorAll(sel).forEach(fieldset => {
                    const legend = fieldset.querySelector('legend');
                    if (!legend) return;

                    const optSel = OPTION_SELECTORS[sel];
                    const options = Array.from(fieldset.querySelectorAll(optSel));
                    if (options.length < 2) return;

                    buildFilter(fieldset, options, legend);
                });
            });
        }
    }

    window.ChoiceFilter = ChoiceFilter;

    function tryAutoInit() {
        if (document.querySelector('[data-admin-auto-init]')) {
            new ChoiceFilter().init();
        }
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', tryAutoInit);
    } else {
        tryAutoInit();
    }
})();
