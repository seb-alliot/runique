/**
 * admin-actions.js
 * Gestion des confirmations admin - Compatible CSP
 * Usage: <script src="admin-actions.js" nonce="TON_NONCE"></script>
 */

(function() {
    'use strict';

    function AdminActions(options) {
        options = options || {};
        this.msgDelete = options.deleteMessage || 'Vraiment supprimer ?';
        this.msgDefault = options.defaultMessage || 'Confirmer ?';
    }

    AdminActions.prototype.init = function() {
        var self = this;
        
        // Bind les suppressions
        document.querySelectorAll('[data-confirm-delete]').forEach(function(btn) {
            if (btn._adminBound) return;
            btn._adminBound = true;
            
            var msg = btn.getAttribute('data-confirm-delete') || self.msgDelete;
            
            btn.addEventListener('click', function(e) {
                if (!confirm(msg)) {
                    e.preventDefault();
                    e.stopPropagation();
                    return false;
                }
            });
        });

        // Bind les confirmations génériques
        document.querySelectorAll('[data-confirm]').forEach(function(el) {
            if (el._adminBound) return;
            el._adminBound = true;
            
            var msg = el.getAttribute('data-confirm') || self.msgDefault;
            
            el.addEventListener('click', function(e) {
                if (!confirm(msg)) {
                    e.preventDefault();
                    e.stopPropagation();
                    return false;
                }
            });
        });
    };

    AdminActions.prototype.refresh = function() {
        this.init();
    };

    // Expose globalement
    window.AdminActions = AdminActions;

    // Auto-init si data-admin-auto-init présent (fonctionne même si DOM déjà prêt)
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