// cours-ia.js
hljs.highlightAll();

(function () {
    // ── Sidebar mobile ────────────────────────────────
    var btn     = document.getElementById('doc-hamburger-btn');
    var sidebar = document.getElementById('doc-sidebar');
    var overlay = document.getElementById('doc-sidebar-overlay');
    if (!btn) return;

    function openSidebar()  { sidebar.classList.add('open');    overlay.classList.add('open'); }
    function closeSidebar() { sidebar.classList.remove('open'); overlay.classList.remove('open'); }

    btn.addEventListener('click', function () {
        sidebar.classList.contains('open') ? closeSidebar() : openSidebar();
    });
    overlay.addEventListener('click', closeSidebar);
    sidebar.querySelectorAll('a').forEach(function (a) {
        a.addEventListener('click', closeSidebar);
    });
})();

(function () {
    // ── Variables dynamiques (à injecter depuis le template si nécessaire) ──
    var CSRF_TOKEN  = window.COUR_CSRF_TOKEN;  // définir via template : <script>window.COUR_CSRF_TOKEN="{{ csrf_token }}"</script>
    var COURS_SLUG  = window.COUR_SLUG;        // idem : <script>window.COUR_SLUG="{{ cour.slug }}"</script>
    var ENDPOINT    = '/cours/' + COURS_SLUG + '/exercice';

    var docContent  = document.getElementById('doc-content');
    var iaPanel     = document.getElementById('ia-panel');
    var iaMessages  = document.getElementById('ia-messages');
    var iaInput     = document.getElementById('ia-input');
    var iaSendBtn   = document.getElementById('ia-send-btn');
    var iaOpenBtn   = document.getElementById('ia-open-btn');
    var iaOpenBtnSidebar = document.getElementById('ia-open-btn-sidebar');
    var iaCloseBtn  = document.getElementById('ia-close-btn');

    var attempt     = 0;
    var isLoading   = false;

    // ── Ouvrir / fermer le panel ────────────────────────
    function openPanel() {
        docContent.classList.add('ia-active');
        iaPanel.classList.add('ia-panel--visible');
        iaInput.focus();
    }

    function closePanel() {
        docContent.classList.remove('ia-active');
        iaPanel.classList.remove('ia-panel--visible');
    }

    if (iaOpenBtn)         iaOpenBtn.addEventListener('click', openPanel);
    if (iaOpenBtnSidebar)  iaOpenBtnSidebar.addEventListener('click', openPanel);
    if (iaCloseBtn)        iaCloseBtn.addEventListener('click', closePanel);

    // ── Gestion messages ───────────────────────────────
    function addMessage(text, type) {
        var div = document.createElement('div');
        div.className = 'ia-message ia-message--' + type;
        div.innerHTML = text;
        iaMessages.appendChild(div);
        iaMessages.scrollTop = iaMessages.scrollHeight;
    }

    function setLoading(state) {
        isLoading = state;
        iaSendBtn.disabled = state;
        iaSendBtn.textContent = state ? '...' : 'Envoyer';
    }

    // ── Envoi message ───────────────────────────────────
    function sendMessage() {
        if (isLoading) return;

        var message = iaInput.value.trim();
        if (!message) return;

        addMessage(message, 'user');
        iaInput.value = '';
        setLoading(true);

        fetch(ENDPOINT, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-CSRF-Token': CSRF_TOKEN,
            },
            body: JSON.stringify({ message: message }),
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
                case 'incorrect':
                    var msg = '✗ Incorrect.';
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
        })
        .catch(() => {
            setLoading(false);
            addMessage('Erreur de connexion. Veuillez réessayer.', 'system');
        });
    }

    function askCorrection() {
        addMessage('Correction demandée.', 'user');
        setLoading(true);
        fetch(ENDPOINT, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-CSRF-Token': CSRF_TOKEN,
            },
            body: JSON.stringify({ message: 'correction' }),
        })
        .then(res => res.json())
        .then(data => {
            setLoading(false);
            addMessage(data.response, 'ia');
        })
        .catch(() => { setLoading(false); });
    }

    function resetSession() {
        attempt = 0;
        iaMessages.innerHTML =
            '<div class="ia-message ia-message--system">Dites <strong>bonjour</strong> pour commencer un nouvel exercice.</div>';
    }

    // ── Événements ──────────────────────────────────────
    iaSendBtn.addEventListener('click', sendMessage);
    iaInput.addEventListener('keydown', function (e) {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            sendMessage();
        }
    });
})();