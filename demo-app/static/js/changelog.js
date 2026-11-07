(function () {
    const select = document.querySelector('.changelog-picker select');
    if (select) select.addEventListener('change', () => select.form.submit());
})();
