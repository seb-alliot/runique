/**
 * Runique Color Picker — sync between the color input and its text display.
 * Triggered by data-color-picker attributes, safe for CSP (no inline JS).
 */
(function () {
    function initColorPicker(container) {
        var picker = container.querySelector('[data-color-input]');
        var text = container.querySelector('[data-color-text]');
        if (!picker || !text) return;

        picker.addEventListener('input', function () {
            text.value = picker.value.toUpperCase();
        });
    }

    function initAll() {
        document.querySelectorAll('[data-color-picker]').forEach(initColorPicker);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initAll);
    } else {
        initAll();
    }
})();
