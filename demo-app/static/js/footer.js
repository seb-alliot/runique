(function () {
    var btn = document.getElementById('footer-burger-btn');
    var nav = document.getElementById('footer-links');
    if (!btn) return;
    btn.addEventListener('click', function () {
        var open = nav.classList.toggle('open');
        btn.querySelector('.footer-burger-icon').textContent = open ? '✕' : '☰';
    });
})();
