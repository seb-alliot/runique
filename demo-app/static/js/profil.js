(function () {
    // Account deletion: one last confirmation in a modal before the form is
    // posted. Without this script the form posts straight away.
    const form = document.getElementById('delete_account_form');
    const dialog = document.getElementById('delete_account_dialog');
    if (!form || !dialog || typeof dialog.showModal !== 'function') return;

    let confirmed = false;

    form.addEventListener('submit', event => {
        if (confirmed) return;
        // The browser checks the required password first; the modal only
        // opens on a form that could be sent.
        event.preventDefault();
        dialog.showModal();
    });

    dialog.querySelector('[data-dialog-cancel]').addEventListener('click', () => dialog.close());

    dialog.querySelector('[data-dialog-confirm]').addEventListener('click', () => {
        confirmed = true;
        dialog.close();
        form.requestSubmit();
    });
})();
