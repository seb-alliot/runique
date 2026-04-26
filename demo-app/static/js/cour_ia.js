hljs.highlightAll();

// ── Sidebar mobile ────────────────────────────────────────────────────────────
(function () {
    const btn     = document.getElementById('doc-hamburger-btn');
    const sidebar = document.getElementById('doc-sidebar');
    const overlay = document.getElementById('doc-sidebar-overlay');
    if (!btn) return;

    const openSidebar  = () => { sidebar.classList.add('open');    overlay.classList.add('open'); };
    const closeSidebar = () => { sidebar.classList.remove('open'); overlay.classList.remove('open'); };

    btn.addEventListener('click', () => {
        sidebar.classList.contains('open') ? closeSidebar() : openSidebar();
    });
    overlay.addEventListener('click', closeSidebar);
    sidebar.querySelectorAll('a').forEach(a => a.addEventListener('click', closeSidebar));
})();

// ── Panel IA ──────────────────────────────────────────────────────────────────
(function () {
    let csrfToken = null;
    const coursSlug  = window.COUR_SLUG;
    const endpoint   = '/cours/' + coursSlug + '/exercice';

    // Récupère le token CSRF depuis le header de réponse de la page courante
    fetch(window.location.href, { method: 'HEAD' })
        .then(res => { csrfToken = res.headers.get('x-csrf-token'); })
        .catch(() => {});

    const docContent       = document.getElementById('doc-content');
    const iaPanel          = document.getElementById('ia-panel');
    const iaPanelHeader    = document.getElementById('ia-panel-header');
    const iaMessages       = document.getElementById('ia-messages');
    const iaInput          = document.getElementById('ia-input');
    const iaSendBtn        = document.getElementById('ia-send-btn');
    const iaOpenBtn        = document.getElementById('ia-open-btn');
    const iaOpenBtnSidebar = document.getElementById('ia-open-btn-sidebar');
    const iaCloseBtn       = document.getElementById('ia-close-btn');
    const iaMinimizeBtn    = document.getElementById('ia-minimize-btn');

    let attempt   = 0;
    let isLoading = false;

    // ── Ouvrir / fermer / réduire ─────────────────────────────────────────────
    const openPanel = () => {
        docContent.classList.add('ia-active');
        iaPanel.classList.add('ia-panel--visible');
        iaInput.focus();
    };

    const closePanel = () => {
        docContent.classList.remove('ia-active');
        iaPanel.classList.remove('ia-panel--visible');
        iaPanel.classList.remove('ia-panel--minimized');
    };

    const minimizePanel = () => iaPanel.classList.add('ia-panel--minimized');

    const restorePanel = () => {
        iaPanel.classList.remove('ia-panel--minimized');
        iaInput.focus();
    };

    if (iaOpenBtn)        iaOpenBtn.addEventListener('click', openPanel);
    if (iaOpenBtnSidebar) iaOpenBtnSidebar.addEventListener('click', openPanel);
    if (iaCloseBtn)       iaCloseBtn.addEventListener('click', closePanel);
    if (iaMinimizeBtn)    iaMinimizeBtn.addEventListener('click', minimizePanel);

    iaPanelHeader.addEventListener('click', e => {
        if (iaPanel.classList.contains('ia-panel--minimized') &&
            e.target !== iaCloseBtn && e.target !== iaMinimizeBtn) {
            restorePanel();
        }
    });

    // ── Messages ──────────────────────────────────────────────────────────────
    const addMessage = (text, type) => {
        const div = document.createElement('div');
        div.className = 'ia-message ia-message--' + type;
        div.innerHTML = text;
        iaMessages.appendChild(div);
        iaMessages.scrollTop = iaMessages.scrollHeight;
    };

    const setLoading = state => {
        isLoading = state;
        iaSendBtn.disabled = state;
        iaSendBtn.textContent = state ? '...' : 'Envoyer';
    };

    // ── Envoi ─────────────────────────────────────────────────────────────────
    const sendMessage = () => {
        if (isLoading) return;
        const message = iaInput.value.trim();
        if (!message) return;

        addMessage(message, 'user');
        iaInput.value = '';
        setLoading(true);

        fetch(endpoint, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-CSRF-Token': csrfToken,
            },
            body: JSON.stringify({ message }),
        })
        .then(res => res.json())
        .then(data => {
            setLoading(false);
            attempt = data.attempt;

            switch (data.status) {
                case 'fixed_reply':
                case 'exercise':
                case 'correction':
                    addMessage(data.response, 'ia');
                    break;
                case 'correct':
                    addMessage('✓ Correct !', 'success');
                    addMessage('<a href="#" class="ia-link" id="ia-retry">Nouvel exercice</a>', 'action');
                    document.getElementById('ia-retry').addEventListener('click', e => {
                        e.preventDefault();
                        resetSession();
                    });
                    break;
                case 'incorrect': {
                    let msg = '✗ Incorrect.';
                    if (attempt >= 3) {
                        msg += ' <strong>3 tentatives échouées.</strong>';
                        addMessage(msg, 'error');
                        addMessage(
                            '<button class="ia-action-btn" id="ia-ask-correction">Voir la correction</button>' +
                            '<button class="ia-action-btn ia-action-btn--secondary" id="ia-retry-loop">Réessayer</button>',
                            'action'
                        );
                        document.getElementById('ia-ask-correction').addEventListener('click', askCorrection);
                        document.getElementById('ia-retry-loop').addEventListener('click', resetSession);
                    } else {
                        msg += ' Tentative ' + attempt + '/3.';
                        addMessage(msg, 'error');
                    }
                    break;
                }
            }
        })
        .catch(() => {
            setLoading(false);
            addMessage('Erreur de connexion. Veuillez réessayer.', 'system');
        });
    };

    const askCorrection = () => {
        addMessage('Correction demandée.', 'user');
        setLoading(true);
        fetch(endpoint, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-CSRF-Token': csrfToken,
            },
            body: JSON.stringify({ message: 'correction' }),
        })
        .then(res => res.json())
        .then(data => { setLoading(false); addMessage(data.response, 'ia'); })
        .catch(() => setLoading(false));
    };

    const resetSession = () => {
        attempt = 0;
        iaMessages.innerHTML =
            '<div class="ia-message ia-message--system">Dites <strong>bonjour</strong> pour commencer un nouvel exercice.</div>';
    };

    iaSendBtn.addEventListener('click', sendMessage);
    iaInput.addEventListener('keydown', e => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            sendMessage();
        }
    });
})();
