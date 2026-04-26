/**
 * checkbox-toggle.js
 * Bouton "Tout cocher / Tout décocher" pour les BooleanField dans les forms admin.
 * Bouton "Tout sélectionner / Tout désélectionner" par CheckboxField (checkbox-fieldset).
 * Groupe les BooleanField dans un .boolean-group visuel.
 * Compatible CSP — aucun inline handler.
 */

(function () {
    'use strict';

    class CheckboxToggle {
        // Wrappe les .boolean-field consécutifs dans un .boolean-group
        groupBooleans() {
            document.querySelectorAll('.form-grid').forEach(function (grid) {
                const booleans = Array.from(grid.querySelectorAll(':scope > .boolean-field'));
                if (booleans.length === 0) return;

                const group = document.createElement('div');
                group.className = 'boolean-group';

                booleans[0].parentNode.insertBefore(group, booleans[0]);
                booleans.forEach(function (el) { group.appendChild(el); });
            });
        }

        // Bouton global "Tout cocher" pour les BooleanField (.boolean-field)
        init() {
            document.querySelectorAll('form').forEach(function (form) {
                const booleans = form.querySelectorAll('.boolean-field input[type="checkbox"]');
                if (booleans.length === 0) return;

                const actions = form.querySelector('.form-actions');
                if (!actions) return;

                const btn = document.createElement('button');
                btn.type = 'button';
                btn.className = 'btn btn-secondary';
                btn.setAttribute('data-checkbox-toggle', '');
                btn.textContent = 'Tout cocher';

                let allChecked = false;

                btn.addEventListener('click', function () {
                    allChecked = !allChecked;
                    booleans.forEach(function (cb) {
                        cb.checked = allChecked;
                    });
                    btn.textContent = allChecked ? 'Tout décocher' : 'Tout cocher';
                });

                actions.insertBefore(btn, actions.firstChild);
            });
        }

        // Bouton "Tout sélectionner" par .checkbox-fieldset (CheckboxField multi-options)
        initFieldsets() {
            document.querySelectorAll('.checkbox-fieldset').forEach(function (fieldset) {
                const checkboxes = fieldset.querySelectorAll('input[type="checkbox"]');
                if (checkboxes.length === 0) return;

                const btn = document.createElement('button');
                btn.type = 'button';
                btn.className = 'btn-select-all';
                btn.setAttribute('data-checkbox-all-toggle', '');
                btn.textContent = 'Tout sélectionner';

                let allChecked = false;

                btn.addEventListener('click', function () {
                    allChecked = !allChecked;
                    checkboxes.forEach(function (cb) {
                        cb.checked = allChecked;
                    });
                    btn.textContent = allChecked ? 'Tout désélectionner' : 'Tout sélectionner';
                });

                const legend = fieldset.querySelector('legend');
                if (legend) {
                    legend.after(btn);
                } else {
                    fieldset.insertBefore(btn, fieldset.firstChild);
                }
            });
        }
    }

    window.CheckboxToggle = CheckboxToggle;

    const tryAutoInit = () => {
        if (document.querySelector('[data-admin-auto-init]')) {
            const toggle = new CheckboxToggle();
            toggle.groupBooleans();
            toggle.init();
            toggle.initFieldsets();
        }
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', tryAutoInit);
    } else {
        tryAutoInit();
    }
})();
