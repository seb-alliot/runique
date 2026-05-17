/**
 * admin-actions.js
 * Gestion des confirmations admin - Compatible CSP
 */

(function() {
    'use strict';

    function confirm_then(msg, callback) {
        if (typeof window.runique_confirm === 'function') {
            window.runique_confirm(msg, { okLabel: 'Supprimer', cancelLabel: 'Annuler' })
                .then(function (ok) { if (ok) callback(); });
        } else if (window.confirm(msg)) {
            callback();
        }
    }

    function AdminActions(options) {
        options = options || {};
        this.msgDelete  = options.deleteMessage  || 'Vraiment supprimer ?';
        this.msgDefault = options.defaultMessage || 'Confirmer ?';
    }

    AdminActions.prototype.init = function() {
        var self = this;

        document.querySelectorAll('[data-confirm-delete]').forEach(function(btn) {
            if (btn._adminBound) return;
            btn._adminBound = true;

            const msg = btn.getAttribute('data-confirm-delete') || self.msgDelete;

            btn.addEventListener('click', function(e) {
                e.preventDefault();
                e.stopPropagation();
                var form = btn.closest('form');
                confirm_then(msg, function () {
                    if (form) form.submit();
                });
            });
        });

        document.querySelectorAll('[data-confirm]').forEach(function(el) {
            if (el._adminBound) return;
            el._adminBound = true;

            const msg = el.getAttribute('data-confirm') || self.msgDefault;

            el.addEventListener('click', function(e) {
                e.preventDefault();
                e.stopPropagation();
                const form = el.closest('form');
                const href = el.href || null;
                confirm_then(msg, function () {
                    if (form) form.submit();
                    else if (href) window.location.href = href;
                });
            });
        });
    };

    AdminActions.prototype.refresh = function() {
        this.init();
    };

    window.AdminActions = AdminActions;

    function tryAutoInit() {
        if (document.querySelector('[data-admin-auto-init]')) {
            new AdminActions().init();
        }
    }
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', tryAutoInit);
    } else {
        tryAutoInit();
    }
})();
