(function () {
    'use strict';

    var overlay  = null;
    var titleEl  = null;
    var msgEl    = null;
    var okBtn    = null;
    var cancelBtn = null;
    var _resolve = null;

    function init() {
        overlay   = document.getElementById('rqModalOverlay');
        titleEl   = document.getElementById('rqModalTitle');
        msgEl     = document.getElementById('rqModalMsg');
        okBtn     = document.getElementById('rqModalOk');
        cancelBtn = document.getElementById('rqModalCancel');

        if (!overlay) return;

        okBtn.addEventListener('click', function () { close(true); });
        cancelBtn.addEventListener('click', function () { close(false); });
        overlay.addEventListener('click', function (e) {
            if (e.target === overlay) close(false);
        });
        document.addEventListener('keydown', function (e) {
            if (e.key === 'Escape' && overlay.classList.contains('open')) close(false);
        });
    }

    function close(result) {
        overlay.classList.remove('open');
        if (_resolve) { _resolve(result); _resolve = null; }
    }

    window.runique_confirm = function (msg, opts) {
        opts = opts || {};
        return new Promise(function (resolve) {
            if (!overlay) { resolve(window.confirm(msg)); return; }
            titleEl.textContent  = opts.title      || '';
            msgEl.textContent    = msg;
            okBtn.textContent    = opts.okLabel     || 'Confirmer';
            cancelBtn.textContent = opts.cancelLabel || 'Annuler';
            _resolve = resolve;
            overlay.classList.add('open');
            cancelBtn.focus();
        });
    };

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
