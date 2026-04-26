const btn = document.getElementById('theme-toggle');
const html = document.documentElement;
const saved = localStorage.getItem('theme') || 'dark';
html.setAttribute('data-theme', saved);
btn.textContent = saved === 'dark' ? '🌙' : '☀️';

// ── Drag sur appui prolongé ───────────────────────────────────────────────────

const LONG_PRESS_MS = 500;

let longPressTimer = null;
let dragging = false;
let startX = 0, startY = 0;
let originRight = 0, originBottom = 0;

function getRight()  { return parseFloat(btn.style.right)  || 20; }
function getBottom() { return parseFloat(btn.style.bottom) || 32; }

btn.addEventListener('pointerdown', (e) => {
    startX = e.clientX;
    startY = e.clientY;
    originRight  = getRight();
    originBottom = getBottom();

    longPressTimer = setTimeout(() => {
        dragging = true;
        btn.setPointerCapture(e.pointerId);
        btn.style.cursor = 'grabbing';
        btn.style.transition = 'none';
    }, LONG_PRESS_MS);
});

btn.addEventListener('pointermove', (e) => {
    if (!dragging) return;

    const dx = e.clientX - startX;
    const dy = e.clientY - startY;

    const newRight  = Math.max(8, originRight  - dx);
    const newBottom = Math.max(8, originBottom - dy);

    btn.style.right  = newRight  + 'px';
    btn.style.bottom = newBottom + 'px';
});

btn.addEventListener('pointerup', (e) => {
    clearTimeout(longPressTimer);

    if (dragging) {
        dragging = false;
        btn.style.cursor = '';
        btn.style.transition = '';

        // Sauvegarde la position
        localStorage.setItem('theme-btn-right',  btn.style.right);
        localStorage.setItem('theme-btn-bottom', btn.style.bottom);
        return; // pas de toggle après un drag
    }

    // Clic normal → toggle thème
    const next = html.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
    html.setAttribute('data-theme', next);
    localStorage.setItem('theme', next);
    btn.textContent = next === 'dark' ? '🌙' : '☀️';
});

btn.addEventListener('pointercancel', () => {
    clearTimeout(longPressTimer);
    dragging = false;
    btn.style.cursor = '';
    btn.style.transition = '';
});

// Restaure la position sauvegardée
const savedRight  = localStorage.getItem('theme-btn-right');
const savedBottom = localStorage.getItem('theme-btn-bottom');
if (savedRight)  btn.style.right  = savedRight;
if (savedBottom) btn.style.bottom = savedBottom;
