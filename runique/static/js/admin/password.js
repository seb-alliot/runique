// password-placeholder-module.js
export function initPasswordFields() {
    document.querySelectorAll('input[type="password"]').forEach(el => {
        el.placeholder = '••••••••';
        el.disabled = true;
    });
}

// Auto-init si DOM prêt
if (document.readyState !== 'loading') {
    initPasswordFields();
} else {
    document.addEventListener('DOMContentLoaded', initPasswordFields);
}