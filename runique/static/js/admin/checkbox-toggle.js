/**
 * checkbox-toggle.js
 * Bouton "Tout cocher / Tout décocher" pour les BooleanField dans les forms admin.
 * Compatible CSP — aucun inline handler.
 *
 * Injecte automatiquement un bouton dans .form-actions si le formulaire contient
 * au moins un .boolean-field input[type="checkbox"].
 */

(function () {
    'use strict';

    function CheckboxToggle() {}

    CheckboxToggle.prototype.init = function () {
        var forms = document.querySelectorAll('form');
        forms.forEach(function (form) {
            var booleans = form.querySelectorAll('.boolean-field input[type="checkbox"]');
            if (booleans.length === 0) return;

            var actions = form.querySelector('.form-actions');
            if (!actions) return;

            var btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'btn btn-secondary';
            btn.setAttribute('data-checkbox-toggle', '');
            btn.textContent = 'Tout cocher';

            var allChecked = false;

            btn.addEventListener('click', function () {
                allChecked = !allChecked;
                booleans.forEach(function (cb) {
                    cb.checked = allChecked;
                });
                btn.textContent = allChecked ? 'Tout décocher' : 'Tout cocher';
            });

            // Insère le bouton avant le premier bouton existant
            actions.insertBefore(btn, actions.firstChild);
        });
    };

    window.CheckboxToggle = CheckboxToggle;

    function tryAutoInit() {
        if (document.querySelector('[data-admin-auto-init]')) {
            new CheckboxToggle().init();
        }
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', tryAutoInit);
    } else {
        tryAutoInit();
    }
})();
