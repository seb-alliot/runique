const btn = document.getElementById('theme-toggle');
const html = document.documentElement;

function applyTheme(theme) {
    html.setAttribute('data-theme', theme);
    btn.textContent = theme === 'dark' ? btn.dataset.light : btn.dataset.dark;
}

applyTheme(localStorage.getItem('theme') || 'dark');

btn.addEventListener('click', () => {
    const next = html.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
    applyTheme(next);
    localStorage.setItem('theme', next);
});
