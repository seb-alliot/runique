const sidebar  = document.getElementById('adminSidebar');
const toggle   = document.getElementById('sidebarToggle');
const overlay  = document.getElementById('sidebarOverlay');
const burger   = document.getElementById('mobileBurger');
const chevron  = toggle.querySelector('svg');

const COLLAPSED_KEY = 'runique_admin_sidebar_collapsed';

// Restaure l'état sauvegardé
if (localStorage.getItem(COLLAPSED_KEY) === '1') {
    sidebar.classList.add('collapsed');
    chevron.style.transform = 'rotate(180deg)';
}

// Toggle desktop
toggle.addEventListener('click', () => {
    const isCollapsed = sidebar.classList.toggle('collapsed');
    chevron.style.transform = isCollapsed ? 'rotate(180deg)' : '';
    localStorage.setItem(COLLAPSED_KEY, isCollapsed ? '1' : '0');
});

// Burger / overlay mobile
function isMobile() { return window.innerWidth <= 900; }

function updateBurgerVisibility() {
    burger.style.display = isMobile() ? 'flex' : 'none';
}

updateBurgerVisibility();
window.addEventListener('resize', updateBurgerVisibility);

burger.addEventListener('click', () => {
    sidebar.classList.add('mobile-open');
    overlay.classList.add('active');
});

overlay.addEventListener('click', () => {
    sidebar.classList.remove('mobile-open');
    overlay.classList.remove('active');
});
