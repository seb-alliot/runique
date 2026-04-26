document.getElementById('field-type-filter').addEventListener('change', function () {
    const selected = this.value;
    document.querySelectorAll('.field-section').forEach(function (section) {
        section.style.display = (!selected || section.dataset.section === selected) ? '' : 'none';
    });
    if (selected) {
        const target = document.getElementById(selected.toLowerCase());
        if (target) {
            target.open = true;
            target.scrollIntoView({ behavior: 'smooth' });
        }
    }
});

document.querySelectorAll('.field-nav a').forEach(function (link) {
    link.addEventListener('click', function (e) {
        e.preventDefault();
        const id = this.getAttribute('href').slice(1);
        const target = document.getElementById(id);
        if (target) {
            target.open = !target.open;
            if (target.open) target.scrollIntoView({ behavior: 'smooth' });
        }
    });
});
