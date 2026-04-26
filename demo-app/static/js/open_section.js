// static/js/field_nav.js
document.addEventListener('DOMContentLoaded', () => {
    const hash = window.location.hash.substring(1);
    if (hash) {
        const section = document.getElementById(hash);
        if (section) {
            const details = section.querySelector('details');
            if (details) details.setAttribute('open', '');
            section.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }
    }

    document.querySelectorAll('.field-nav a').forEach(link => {
        link.addEventListener('click', e => {
            e.preventDefault(); // empêche le scroll automatique
            const targetId = link.getAttribute('href').substring(1);
            const section = document.getElementById(targetId);

            if (section) {
                const details = section.querySelector('details');
                if (details && !details.hasAttribute('open')) {
                    details.setAttribute('open', '');
                }
                section.scrollIntoView({ behavior: 'smooth', block: 'start' });
                history.replaceState(null, '', `#${targetId}`); // optionnel : update URL
            }
        });
    });
});